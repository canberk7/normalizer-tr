use std::mem::size_of;

use crate::{
    AmbiguityPolicy, Issue, IssueCategory, LimitKind, MAX_HINTS, MAX_INPUT_BYTES, MAX_RESULT_BYTES,
    NormalizeError, NormalizeOptions, NormalizeResult, Segment, SegmentKind, SourceRange,
    WorkControl, classify, resources::Resources, source_map::SourceMap, verbalize,
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
    if options.hints.len() > MAX_HINTS {
        return Err(NormalizeError::LimitExceeded(LimitKind::Hints));
    }
    if input.trim().is_empty()
        || input.chars().any(|c| {
            (c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
                || matches!(c, '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
        })
    {
        return Err(NormalizeError::InvalidInput);
    }
    let source = SourceMap::new(input, control)?;
    let hints = source.hints(&options.hints)?;
    let candidates = classify::collect(&source, &hints, rules, control)?;
    let mut budget = ResultBudget::default();
    budget.charge(size_of::<NormalizeResult>())?;
    let mut segments = Vec::new();
    let mut issues = Vec::new();
    let mut cursor = 0;
    for candidate in candidates {
        control.check()?;
        let range = source
            .original(candidate.range)
            .map_err(|_| NormalizeError::Internal)?;
        if range.start < cursor || range.start >= range.end || range.end > input.len() {
            return Err(NormalizeError::Internal);
        }
        verbatim(input, cursor, range.start, &mut segments, &mut budget)?;
        let (kind, rule_id, text) = match candidate.reading {
            Ok(value) => verbalize::render(&value),
            Err(category) => {
                budget.issue()?;
                issues.push(Issue {
                    range,
                    category,
                    explanation: explanation(category),
                });
                (
                    SegmentKind::Unresolved,
                    "source.unresolved",
                    input[range.start..range.end].to_owned(),
                )
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
    }
    verbatim(input, cursor, input.len(), &mut segments, &mut budget)?;
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
