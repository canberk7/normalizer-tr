mod boundaries;
mod context;
pub(crate) mod forced;
mod readers;
mod scan;
mod temporal;

use crate::{
    Hint, LimitKind, MAX_CANDIDATES, NormalizeError, SourceRange, WorkControl, model::Value,
    resources::Resources, source_map::SourceMap,
};
use boundaries::{Boundaries, Claim, overlaps_hint};
use readers::Context;

pub(crate) struct Candidate {
    pub(crate) range: SourceRange,
    pub(crate) reading: Result<Value, crate::IssueCategory>,
}

fn push(
    candidates: &mut Vec<Candidate>,
    claim: Claim,
    reading: Result<Value, crate::IssueCategory>,
) -> Result<(), NormalizeError> {
    if candidates.len() == MAX_CANDIDATES {
        return Err(NormalizeError::LimitExceeded(LimitKind::Candidates));
    }
    candidates.push(Candidate {
        range: claim.range,
        reading,
    });
    Ok(())
}

/// With `strays`, a token no reader claims is also a candidate when the Forced policy speaks
/// it: a named symbol or spelled letters. Such a candidate is resolved, never an issue.
pub(crate) fn collect(
    source: &SourceMap,
    hints: &[Hint],
    resources: &Resources,
    control: &WorkControl,
    strays: bool,
) -> Result<Vec<Candidate>, NormalizeError> {
    let text = source.text();
    let tokens = scan::tokens(source, resources, control)?;
    let phone_like = scan::phones(text, &tokens, resources, control)?;
    let bounds = Boundaries::new(&tokens, &phone_like);
    let ctx = Context {
        text,
        tokens: &tokens,
    };
    let mut candidates = Vec::new();
    let mut index = 0;
    let mut hint_index = 0;
    let mut role_until = 0;
    while index < tokens.len() {
        control.check()?;
        if let Some(hint) = hints
            .get(hint_index)
            .filter(|h| h.range.start <= tokens[index].range.start)
        {
            let detected = readers::read(&ctx, index, &bounds, index < role_until)
                .map(|(_, end)| bounds.claim(index, end))
                .transpose()?;
            let claim = bounds.hint_claim(*hint, detected.as_ref())?;
            let text = &text[claim.range.start..claim.range.end];
            let value = readers::hint(text, hint.kind).ok_or(NormalizeError::InvalidHint)?;
            if matches!(value, Value::Roman(_)) {
                role_until = ctx.roman_anchor_end(index).unwrap_or(role_until);
            }
            index = claim.next;
            push(&mut candidates, claim, Ok(value))?;
            hint_index += 1;
            continue;
        }
        if hints
            .get(hint_index)
            .is_some_and(|h| h.range.start < tokens[index].range.end)
        {
            return Err(NormalizeError::InvalidHint);
        }
        if let Some((reading, end)) = readers::read(&ctx, index, &bounds, index < role_until) {
            let claim = bounds.claim(index, end)?;
            if overlaps_hint(hints, claim.range) {
                return Err(NormalizeError::InvalidHint);
            }
            if matches!(&reading, Ok(Value::Roman(_))) {
                role_until = ctx.roman_anchor_end(index).unwrap_or(role_until);
            }
            index = claim.next;
            push(&mut candidates, claim, reading)?;
        } else {
            let before = &text[..tokens[index].range.start];
            let following = &text[tokens[index].range.end..];
            if strays && let Some(value) = forced::stray(tokens[index].text, before, following) {
                push(&mut candidates, bounds.claim(index, index)?, Ok(value))?;
            }
            index += 1;
        }
    }
    if hint_index != hints.len() {
        return Err(NormalizeError::InvalidHint);
    }
    Ok(candidates)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn candidate_limit_accepts_exact_boundary() {
        let mut candidates = Vec::new();
        for _ in 0..MAX_CANDIDATES {
            assert!(
                push(
                    &mut candidates,
                    Claim {
                        range: SourceRange::new(0, 1),
                        next: 1
                    },
                    Err(crate::IssueCategory::Ambiguous)
                )
                .is_ok()
            );
        }
        assert!(matches!(
            push(
                &mut candidates,
                Claim {
                    range: SourceRange::new(0, 1),
                    next: 1
                },
                Err(crate::IssueCategory::Ambiguous)
            ),
            Err(NormalizeError::LimitExceeded(LimitKind::Candidates))
        ));
    }
}
