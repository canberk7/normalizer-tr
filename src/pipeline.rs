use std::mem::size_of;

use unicode_segmentation::UnicodeSegmentation;

use crate::{
    AmbiguityPolicy, Issue, IssueCategory, LimitKind, MAX_HINTS, MAX_INPUT_BYTES, MAX_RESULT_BYTES,
    NormalizeError, NormalizeOptions, NormalizeResult, Segment, SegmentKind, SourceRange,
    WorkControl, classify, morphology::Style, resources::Resources, source_map::SourceMap,
    verbalize,
};

#[derive(Default)]
struct ResultBudget {
    used: usize,
}

impl ResultBudget {
    fn charge(&mut self, bytes: usize) -> Result<(), NormalizeError> {
        self.used = self
            .used
            .checked_add(bytes)
            .filter(|used| *used <= MAX_RESULT_BYTES)
            .ok_or(NormalizeError::LimitExceeded(LimitKind::Result))?;
        Ok(())
    }
    fn segment(&mut self, text_bytes: usize) -> Result<(), NormalizeError> {
        // Text is owned once per segment and once again by normalized_text.
        self.charge(size_of::<Segment>() + text_bytes * 2)
    }
    fn issue(&mut self) -> Result<(), NormalizeError> {
        self.charge(size_of::<Issue>())
    }
}

fn explanation(category: IssueCategory) -> &'static str {
    match category {
        IssueCategory::Ambiguous => "expression requires an explicit supported cue or hint",
        IssueCategory::InvalidExpression => "expression has an invalid value, grammar, or suffix",
        IssueCategory::ProtectedIdentifier => "structured identifier is preserved in full",
        IssueCategory::Unsupported => "expression is outside the bounded profile",
        IssueCategory::UnknownAbbreviation => "uppercase abbreviation is not approved",
    }
}

/// Forced segment of an unresolved span: the reading's own kind under its `forced.` rule id,
/// and how many bytes after the span it also says. It reads the same NFC slice an explicit
/// hint on the span would, and may look at the text around it.
fn forced(
    text: &str,
    range: SourceRange,
) -> Result<(SegmentKind, &'static str, String, usize), NormalizeError> {
    let reading = text
        .get(..range.start)
        .zip(text.get(range.start..range.end))
        .zip(text.get(range.end..))
        .and_then(|((before, span), following)| classify::forced::read(span, before, following))
        .ok_or(NormalizeError::Internal)?;
    let (kind, _, spoken) = verbalize::render(&reading.value, Style::Spoken);
    Ok((kind, reading.rule_id, spoken, reading.said_after))
}

/// The mark right after a forced span that its reading also says, as an empty spoken segment,
/// when the mark ends on a grapheme boundary before the next candidate starts.
fn said_mark(
    source: &SourceMap,
    span: SourceRange,
    said_after: usize,
    next: Option<SourceRange>,
) -> Option<Segment> {
    let end = span.end.checked_add(said_after)?;
    if said_after == 0 || next.is_some_and(|next| next.start < end) {
        return None;
    }
    let range = source.original(SourceRange::new(span.end, end)).ok()?;
    Some(Segment {
        range,
        kind: SegmentKind::Literal,
        text: String::new(),
        rule_id: "forced.spoken",
    })
}

fn verbatim(
    input: &str,
    start: usize,
    end: usize,
    segments: &mut Vec<Segment>,
    budget: &mut ResultBudget,
) -> Result<(), NormalizeError> {
    if start != end {
        budget.segment(end - start)?;
        segments.push(Segment {
            range: SourceRange::new(start, end),
            kind: SegmentKind::Verbatim,
            text: input[start..end].to_owned(),
            rule_id: "source.verbatim",
        });
    }
    Ok(())
}

/// Bidirectional controls, which can make text display in another order than it is read in.
fn bidi_control(ch: char) -> bool {
    matches!(ch, '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
}

/// The text without its bidirectional controls, and where each of its bytes is in the input.
struct Uncontrolled {
    text: String,
    origin: Vec<usize>,
}

impl Uncontrolled {
    fn new(input: &str) -> Self {
        let mut text = String::with_capacity(input.len());
        let mut origin = Vec::with_capacity(input.len());
        for (at, ch) in input.char_indices().filter(|(_, ch)| !bidi_control(*ch)) {
            origin.extend(at..at + ch.len_utf8());
            text.push(ch);
        }
        Self { text, origin }
    }
    /// Where a range of the text is in the input, without the controls around it.
    fn range(&self, range: SourceRange, input_len: usize) -> SourceRange {
        let start = self.origin.get(range.start).copied().unwrap_or(input_len);
        let end = range
            .end
            .checked_sub(1)
            .and_then(|last| self.origin.get(last))
            .map_or(start, |last| last + 1);
        SourceRange::new(start, end.max(start))
    }
    /// Where an input offset is in the text: one at a control is where the next kept byte is.
    fn offset(&self, at: usize) -> usize {
        self.origin.partition_point(|origin| *origin < at)
    }
}

/// A dropped mark takes a space next to it with it, so that no double space, and no space at
/// the start or the end of the text, is left where it stood: `lira | Garanti` is lira Garanti
/// and `→ Adım 1` adım bir. A stray mark's segment takes the space into its own range; an
/// issue whose reading says nothing (`# Başlık`) keeps the range of its issue, and the space
/// becomes an empty spoken segment of its own. One pass, in order.
fn tidy(segments: Vec<Segment>) -> Vec<Segment> {
    let leading = |segment: &Segment| {
        segment.kind == SegmentKind::Verbatim && segment.text.graphemes(true).next() == Some(" ")
    };
    let trailing = |segment: &Segment| {
        segment.kind == SegmentKind::Verbatim
            && segment.text.graphemes(true).next_back() == Some(" ")
    };
    let space = ' '.len_utf8();
    let mut tidied: Vec<Segment> = Vec::with_capacity(segments.len());
    let mut rest = segments.into_iter().peekable();
    while let Some(mut segment) = rest.next() {
        let silent = segment.text.is_empty() && segment.rule_id.starts_with("forced.");
        let stray = segment.rule_id == "forced.spoken";
        // Marks dropped one after another each take one space: `:D <3 ^^` leaves none.
        let spaced = tidied
            .iter()
            .rev()
            .find(|before| !before.text.is_empty())
            .is_none_or(|before| before.text.ends_with(char::is_whitespace));
        if silent && spaced && rest.peek().is_some_and(leading) {
            let start = segment.range.end;
            let end = start + space;
            if let Some(next) = rest.peek_mut()
                && next.text.len() > space
            {
                next.text.remove(0);
                next.range = SourceRange::new(end, next.range.end);
            } else {
                rest.next();
            }
            if stray {
                segment.range = SourceRange::new(segment.range.start, end);
                tidied.push(segment);
            } else {
                tidied.push(segment);
                tidied.push(unsaid(start, end));
            }
            continue;
        }
        if silent && rest.peek().is_none() && tidied.last().is_some_and(trailing) {
            let end = segment.range.start;
            let start = end - space;
            if let Some(before) = tidied.last_mut()
                && before.text.len() > space
            {
                before.text.pop();
                before.range = SourceRange::new(before.range.start, start);
            } else {
                tidied.pop();
            }
            if stray {
                segment.range = SourceRange::new(start, segment.range.end);
            } else {
                tidied.push(unsaid(start, end));
            }
        }
        tidied.push(segment);
    }
    tidied
}

/// An empty spoken segment for text that is not said: a space a dropped mark takes with it.
fn unsaid(start: usize, end: usize) -> Segment {
    Segment {
        range: SourceRange::new(start, end),
        kind: SegmentKind::Literal,
        text: String::new(),
        rule_id: "forced.spoken",
    }
}

/// Input text kept as written, each run of bidirectional controls in it an empty segment.
fn kept(input: &str, start: usize, end: usize, segments: &mut Vec<Segment>) {
    let mut run = start;
    let mut chars = input[start..end].char_indices().peekable();
    while let Some((_, ch)) = chars.next() {
        let controls = bidi_control(ch);
        let next = chars
            .peek()
            .map(|(at, next)| (start + at, bidi_control(*next)));
        if next.is_some_and(|(_, next)| next == controls) {
            continue;
        }
        let stop = next.map_or(end, |(at, _)| at);
        segments.push(Segment {
            range: SourceRange::new(run, stop),
            kind: if controls {
                SegmentKind::Literal
            } else {
                SegmentKind::Verbatim
            },
            text: if controls {
                String::new()
            } else {
                input[run..stop].to_owned()
            },
            rule_id: if controls {
                "forced.spoken"
            } else {
                "source.verbatim"
            },
        });
        run = stop;
    }
}

/// Forced reads text with bidirectional controls as if they were not there, since only the
/// order text is read in is spoken: the text without them is normalized, its result is mapped
/// back to the input, and each run of controls becomes an empty spoken segment. A control
/// whose removal would join two graphemes into one, as two halves of a flag, is still invalid
/// input.
fn without_bidi_controls(
    input: &str,
    options: &NormalizeOptions,
    control: &WorkControl,
    rules: &Resources,
) -> Result<NormalizeResult, NormalizeError> {
    if options.hints.len() > MAX_HINTS {
        return Err(NormalizeError::LimitExceeded(LimitKind::Hints));
    }
    let uncontrolled = Uncontrolled::new(input);
    let mut options = options.clone();
    for hint in &mut options.hints {
        let SourceRange { start, end } = hint.range;
        if start > end || !input.is_char_boundary(start) || !input.is_char_boundary(end) {
            return Err(NormalizeError::InvalidHint);
        }
        hint.range = SourceRange::new(uncontrolled.offset(start), uncontrolled.offset(end));
    }
    let read = normalize(&uncontrolled.text, &options, control, rules)?;
    let mut segments = Vec::with_capacity(read.segments.len() + 2);
    let mut cursor = 0;
    for segment in read.segments {
        control.check()?;
        let range = uncontrolled.range(segment.range, input.len());
        kept(input, cursor, range.start, &mut segments);
        if segment.kind == SegmentKind::Verbatim {
            kept(input, range.start, range.end, &mut segments);
        } else {
            segments.push(Segment { range, ..segment });
        }
        cursor = range.end;
    }
    kept(input, cursor, input.len(), &mut segments);
    let segments = tidy(segments);
    let issues: Vec<Issue> = read
        .issues
        .into_iter()
        .map(|issue| Issue {
            range: uncontrolled.range(issue.range, input.len()),
            ..issue
        })
        .collect();
    let mut boundaries: Vec<usize> = input.grapheme_indices(true).map(|(at, _)| at).collect();
    boundaries.push(input.len());
    let on_boundaries = segments.iter().all(|segment| {
        boundaries.binary_search(&segment.range.start).is_ok()
            && boundaries.binary_search(&segment.range.end).is_ok()
    });
    if !on_boundaries {
        return Err(NormalizeError::InvalidInput);
    }
    let used = segments
        .iter()
        .map(|segment| size_of::<Segment>().saturating_add(segment.text.len().saturating_mul(2)))
        .chain(issues.iter().map(|_| size_of::<Issue>()))
        .fold(size_of::<NormalizeResult>(), usize::saturating_add);
    if used > MAX_RESULT_BYTES {
        return Err(NormalizeError::LimitExceeded(LimitKind::Result));
    }
    control.check()?;
    Ok(NormalizeResult {
        normalized_text: segments
            .iter()
            .map(|segment| segment.text.as_str())
            .collect(),
        locale: read.locale,
        normalizer_id: read.normalizer_id,
        complete: read.complete,
        segments,
        issues,
    })
}

pub(crate) fn run(
    input: &str,
    options: &NormalizeOptions,
    control: &WorkControl,
    rules: &Resources,
) -> Result<NormalizeResult, NormalizeError> {
    control.check()?;
    if input.len() > MAX_INPUT_BYTES {
        return Err(NormalizeError::LimitExceeded(LimitKind::Input));
    }
    if options.ambiguity_policy == AmbiguityPolicy::Forced && input.contains(bidi_control) {
        return without_bidi_controls(input, options, control, rules);
    }
    normalize(input, options, control, rules)
}

fn normalize(
    input: &str,
    options: &NormalizeOptions,
    control: &WorkControl,
    rules: &Resources,
) -> Result<NormalizeResult, NormalizeError> {
    control.check()?;
    if input.len() > MAX_INPUT_BYTES {
        return Err(NormalizeError::LimitExceeded(LimitKind::Input));
    }
    if options.hints.len() > MAX_HINTS {
        return Err(NormalizeError::LimitExceeded(LimitKind::Hints));
    }
    if input.trim().is_empty()
        || input
            .chars()
            .any(|c| (c.is_control() && !matches!(c, '\n' | '\r' | '\t')) || bidi_control(c))
    {
        return Err(NormalizeError::InvalidInput);
    }
    let source = SourceMap::new(input, control)?;
    let hints = source.hints(&options.hints)?;
    // Forced speaks in the everyday style and also reads stray symbols and spelled letters.
    let forced_policy = options.ambiguity_policy == AmbiguityPolicy::Forced;
    let style = if forced_policy {
        Style::Spoken
    } else {
        Style::Exact
    };
    let candidates = classify::collect(&source, &hints, rules, control, forced_policy)?;
    let mut budget = ResultBudget::default();
    budget.charge(size_of::<NormalizeResult>())?;
    let mut segments = Vec::new();
    let mut issues = Vec::new();
    let mut cursor = 0;
    let mut candidates = candidates.into_iter().peekable();
    while let Some(candidate) = candidates.next() {
        control.check()?;
        let range = source
            .original(candidate.range)
            .map_err(|_| NormalizeError::Internal)?;
        if range.start < cursor || range.start >= range.end || range.end > input.len() {
            return Err(NormalizeError::Internal);
        }
        verbatim(input, cursor, range.start, &mut segments, &mut budget)?;
        let mut said_after = 0;
        let (kind, rule_id, text) = match candidate.reading {
            Ok(value) => {
                let (kind, rule_id, text) = verbalize::render(&value, style);
                // Forced says a long plain number the engine resolved as the identifier it is.
                let identifier = (forced_policy && kind == SegmentKind::Cardinal)
                    .then(|| {
                        source
                            .text()
                            .get(candidate.range.start..candidate.range.end)
                            .and_then(classify::forced::identifier)
                            .or_else(|| {
                                classify::forced::card_group(
                                    source.text(),
                                    candidate.range.start,
                                    candidate.range.end,
                                )
                            })
                    })
                    .flatten();
                (kind, rule_id, identifier.unwrap_or(text))
            }
            Err(category) => {
                budget.issue()?;
                issues.push(Issue {
                    range,
                    category,
                    explanation: explanation(category),
                });
                if forced_policy {
                    let (kind, rule_id, text, said) = forced(source.text(), candidate.range)?;
                    said_after = said;
                    (kind, rule_id, text)
                } else {
                    (
                        SegmentKind::Unresolved,
                        "source.unresolved",
                        input[range.start..range.end].to_owned(),
                    )
                }
            }
        };
        budget.segment(text.len())?;
        segments.push(Segment {
            range,
            kind,
            text,
            rule_id,
        });
        cursor = range.end;
        let next = candidates.peek().map(|next| next.range);
        if let Some(mark) = said_mark(&source, candidate.range, said_after, next) {
            budget.segment(0)?;
            cursor = mark.range.end;
            segments.push(mark);
        }
    }
    verbatim(input, cursor, input.len(), &mut segments, &mut budget)?;
    if forced_policy {
        segments = tidy(segments);
    }
    control.check()?;
    if options.ambiguity_policy == AmbiguityPolicy::Reject && !issues.is_empty() {
        return Err(NormalizeError::Unresolved(issues));
    }
    let length = segments.iter().map(|segment| segment.text.len()).sum();
    let mut normalized_text = String::with_capacity(length);
    for segment in &segments {
        control.check()?;
        normalized_text.push_str(&segment.text);
    }
    Ok(NormalizeResult {
        normalized_text,
        locale: "tr-TR",
        normalizer_id: crate::NORMALIZER_ID,
        complete: issues.is_empty(),
        segments,
        issues,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn result_budget_exact_boundary_and_overflow() {
        let mut budget = ResultBudget::default();
        assert_eq!(budget.charge(MAX_RESULT_BYTES), Ok(()));
        assert_eq!(
            budget.charge(1),
            Err(NormalizeError::LimitExceeded(LimitKind::Result))
        );
        let mut budget = ResultBudget::default();
        assert_eq!(
            budget.charge(usize::MAX),
            Err(NormalizeError::LimitExceeded(LimitKind::Result))
        );
    }
}
