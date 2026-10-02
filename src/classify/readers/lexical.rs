use super::{Attempt, Context};
use crate::{
    IssueCategory,
    domain::{
        electronic::{self},
        numeric::{self},
    },
    model::Value,
};
pub(super) fn read(ctx: &Context<'_>, index: usize) -> Option<Attempt> {
    let tokens = ctx.tokens;
    let token = tokens[index];
    let source = token.text;
    if source == "&" {
        return Some((Ok(Value::Symbol("ve".to_owned())), index));
    }
    if source.starts_with('#') {
        return Some((
            electronic::hashtag(source)
                .map(Value::Symbol)
                .ok_or(IssueCategory::Unsupported),
            index,
        ));
    }
    if !tokens
        .get(index + 1)
        .is_some_and(|next| numeric::label(source) && next.text.chars().any(|c| c.is_ascii_digit()))
        && let Some(reading) = numeric::lexical_reading(source)
    {
        return Some((
            reading.map(|(entry, case)| Value::Lexical(entry, case)),
            index,
        ));
    }

    None
}
