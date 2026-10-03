use super::{Attempt, Context, whitespace_between};
use crate::{
    IssueCategory,
    domain::{
        electronic::{self},
        lexicon::Around,
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
    if let Some(body) = source.strip_prefix('#') {
        return Some((
            electronic::hashtag(source)
                .map(|_| Value::Hashtag(body.to_owned()))
                .ok_or(IssueCategory::Unsupported),
            index,
        ));
    }
    // The words around an abbreviation decide whether it is one (`Atatürk Bul.`, but not the
    // `Bul.` that ends a sentence) and how it is said: `123. Sok.` is a numbered street, said
    // without the possessive of a named one, and `Sipariş No:` a compound, with it.
    let around = Around {
        previous: index
            .checked_sub(1)
            .map(|previous| tokens[previous])
            .filter(|previous| whitespace_between(ctx.text, previous.range.end, token.range.start))
            .map(|previous| previous.text),
        next: tokens
            .get(index + 1)
            .filter(|next| whitespace_between(ctx.text, token.range.end, next.range.start))
            .map(|next| next.text),
    };
    if !tokens
        .get(index + 1)
        .is_some_and(|next| numeric::label(source) && next.text.chars().any(|c| c.is_ascii_digit()))
        && let Some(reading) = numeric::lexical_reading(source, around)
    {
        return Some((
            reading.map(|(entry, case)| Value::Lexical(entry, case)),
            index,
        ));
    }

    None
}
