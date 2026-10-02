use super::{Attempt, Context};
use crate::{
    IssueCategory,
    domain::electronic::{self, Electronic},
    model::Value,
};
pub(super) fn whole(ctx: &Context<'_>, index: usize) -> Option<Attempt> {
    let tokens = ctx.tokens;
    let token = tokens[index];
    let source = token.text;
    if electronic::looks_like(source) {
        return Some((
            Electronic::parse(source, false)
                .map(Value::Electronic)
                .ok_or(IssueCategory::Unsupported),
            index,
        ));
    }
    None
}

pub(super) fn contextual(ctx: &Context<'_>, index: usize) -> Option<Attempt> {
    let source = ctx.tokens[index].text;
    if source.contains('.') && ctx.cue(index, &["web", "site"]) {
        return Some((
            Electronic::parse(source, true)
                .map(Value::Electronic)
                .ok_or(IssueCategory::Unsupported),
            index,
        ));
    }
    if source.contains('.')
        && !source.chars().any(char::is_numeric)
        && source
            .split('.')
            .all(|part| !part.is_empty() && part.chars().all(|c| c.is_alphanumeric() || c == '-'))
    {
        return Some((
            Err(if Electronic::parse(source, true).is_some() {
                IssueCategory::Ambiguous
            } else {
                IssueCategory::Unsupported
            }),
            index,
        ));
    }

    None
}
