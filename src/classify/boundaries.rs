use super::scan::{Token, overlaps};
use crate::{Hint, HintKind, NormalizeError, SourceRange, domain::numeric};

/// The one source of cursor advancement, source claims and whole-hint ownership.
pub(super) struct Boundaries<'a> {
    pub(super) tokens: &'a [Token<'a>],
    phone_like: &'a [SourceRange],
}

pub(super) struct Claim {
    pub(super) range: SourceRange,
    pub(super) next: usize,
}

impl<'a> Boundaries<'a> {
    pub(super) fn new(tokens: &'a [Token<'a>], phone_like: &'a [SourceRange]) -> Self {
        Self { tokens, phone_like }
    }
    pub(super) fn claim(&self, start: usize, end: usize) -> Result<Claim, NormalizeError> {
        let first = self.tokens.get(start).ok_or(NormalizeError::Internal)?;
        let last = self
            .tokens
            .get(end)
            .filter(|_| end >= start)
            .ok_or(NormalizeError::Internal)?;
        Ok(Claim {
            range: SourceRange::new(first.range.start, last.range.end),
            next: end + 1,
        })
    }
    pub(super) fn hint_claim(
        &self,
        hint: Hint,
        detected: Option<&Claim>,
    ) -> Result<Claim, NormalizeError> {
        let range = hint.range;
        let start = self.tokens.partition_point(|t| t.range.end <= range.start);
        let end = self.tokens.partition_point(|t| t.range.start < range.end);
        let touched = &self.tokens[start..end];
        let wrap_allowed = matches!(hint.kind, HintKind::Digits | HintKind::Telephone);
        let phones = &self.phone_like[self.phone_like.partition_point(|p| p.end <= range.start)
            ..self.phone_like.partition_point(|p| p.start < range.end)];
        if touched.is_empty()
            || (!wrap_allowed
                && (touched.first().is_none_or(|t| t.range.start != range.start)
                    || touched.last().is_none_or(|t| t.range.end != range.end)))
            || touched
                .iter()
                .any(|t| t.range.start < range.start || t.range.end > range.end)
            || phones
                .iter()
                .any(|p| overlaps(*p, range) && (*p != range || !wrap_allowed))
        {
            return Err(NormalizeError::InvalidHint);
        }
        if detected.is_some_and(|span| span.range.start < range.start || span.range.end > range.end)
        {
            return Err(NormalizeError::InvalidHint);
        }
        Ok(Claim { range, next: end })
    }
    pub(super) fn phone_at(&self, start: usize) -> Option<SourceRange> {
        self.phone_like
            .binary_search_by_key(&start, |p| p.start)
            .ok()
            .map(|i| self.phone_like[i])
    }
    pub(super) fn next_at(&self, end: usize) -> usize {
        self.tokens.partition_point(|t| t.range.start < end)
    }
}

pub(super) fn overlaps_hint(hints: &[Hint], range: SourceRange) -> bool {
    hints
        .get(hints.partition_point(|h| h.range.end <= range.start))
        .is_some_and(|h| overlaps(h.range, range))
}

pub(super) fn quantity_tail(text: &str) -> bool {
    if text.starts_with(['\'', '’']) {
        return true;
    }
    let base = text.split(['\'', '’']).next().unwrap_or(text);
    numeric::label(base)
        || numeric::unsupported_label(base)
        || numeric::unsupported_label(&base.to_ascii_uppercase())
        || (base.len() == 3
            && base.bytes().all(|b| b.is_ascii_uppercase())
            && crate::domain::lexicon::abbreviation(base).is_none())
        || crate::domain::lexicon::unit_marker(base)
        || base == "%"
}
