use super::super::scan::{Token, whitespace_between};
use super::{Attempt, Context};
use crate::{
    IssueCategory,
    domain::{
        identifiers::{Iban, Telephone},
        lexicon,
        numeric::{self, NumericRange},
    },
    model::Value,
};
pub(super) fn read(ctx: &Context<'_>, index: usize) -> Option<Attempt> {
    let text = ctx.text;
    let tokens = ctx.tokens;
    let token = tokens[index];
    let source = token.text;
    let numeric_group_context = tokens
        .get(index + 1)
        .is_some_and(|next| next.text.len() == 4 && next.text.bytes().all(|b| b.is_ascii_digit()))
        && tokens.get(index + 2).is_some_and(|next| {
            next.text.len() == 4 && next.text.bytes().all(|b| b.is_ascii_digit())
        });
    let iban_like = (source.len() >= 4
        && source.as_bytes()[..2].iter().all(u8::is_ascii_alphabetic)
        && source.as_bytes()[2..].iter().all(u8::is_ascii_alphanumeric)
        && (((source.starts_with("TR") || source.starts_with("tr"))
            && (source.as_bytes()[2..].iter().all(u8::is_ascii_digit)
                || source.len() == 26
                || (source.len() == 4
                    && (source.as_bytes()[2..].iter().any(u8::is_ascii_digit)
                        || numeric_group_context))))
            || (source.len() == 4
                && source.as_bytes()[2..].iter().all(u8::is_ascii_digit)
                && numeric_group_context)))
        || (source == "TR"
            && tokens.get(index + 1).is_some_and(|next| {
                next.text.len() == 2 && next.text.bytes().all(|b| b.is_ascii_digit())
            }));
    if iban_like {
        let end = iban_end(text, tokens, index);
        let whole = &text[token.range.start..tokens[end].range.end];
        return Some((
            Iban::parse(whole)
                .map(Value::Iban)
                .ok_or(IssueCategory::ProtectedIdentifier),
            end,
        ));
    }
    let zero_group_signal = source == "0"
        && tokens.get(index + 1).is_some_and(|next| {
            next.text.len() == 3 && next.text.bytes().all(|b| b.is_ascii_digit())
        })
        && tokens.get(index + 2).is_some_and(|next| {
            next.text.len() >= 2 && next.text.bytes().all(|b| b.is_ascii_digit())
        });
    let telephone_cue =
        source.bytes().any(|b| b.is_ascii_digit()) && ctx.cue(index, &["telefon", "tel"]);
    let quantity_context = telephone_cue
        && tokens.get(index + 1).is_some_and(|next| {
            numeric::label(next.text)
                || ["kişi", "adet", "gün", "yaş"].contains(&lexicon::lookup_key(next.text).as_str())
        });
    if source.starts_with("+90")
        || (source.starts_with('0') && source != "0" && source.chars().any(|c| c.is_ascii_digit()))
        || zero_group_signal
        || (source == "0" && ctx.cue(index, &["telefon", "tel"]))
        || (telephone_cue && !quantity_context && source.bytes().all(|b| b.is_ascii_digit()))
        || (source.len() == 10
            && source.bytes().all(|b| b.is_ascii_digit())
            && ctx.cue(index, &["telefon", "tel"]))
    {
        let end = group_end(text, tokens, index);
        let whole = &text[token.range.start..tokens[end].range.end];
        let digits = whole.chars().filter(|c| c.is_ascii_digit()).count();
        let phone_cue = telephone_cue;
        let written_prefix = source.split(['-', '(']).next().unwrap_or(source);
        let national_shape = source.starts_with('0')
            && ((source.len() == 4 && source.bytes().all(|b| b.is_ascii_digit()))
                || (written_prefix.len() == 4
                    && written_prefix.bytes().all(|b| b.is_ascii_digit()))
                || (source.len() >= 10 && source.bytes().all(|b| b.is_ascii_digit()))
                || zero_group_signal
                || (source.starts_with("0-") && digits == 11)
                || phone_cue);
        let contextual_range = tokens
            .get(index + 1)
            .is_some_and(|next| NumericRange::parse(source, Some(next.text)).is_some());
        if !contextual_range
            && !quantity_context
            && ((source.starts_with("+90") && (end > index || digits == 12))
                || national_shape
                || phone_cue)
        {
            return Some((
                Telephone::parse(whole, phone_cue)
                    .map(Value::Telephone)
                    .ok_or(IssueCategory::InvalidExpression),
                end,
            ));
        }
    }

    None
}

fn number_group(token: &str) -> bool {
    !token.is_empty()
        && token
            .chars()
            .all(|ch| ch.is_ascii_digit() || matches!(ch, '+' | '-' | '(' | ')'))
}
fn group_end(text: &str, tokens: &[Token<'_>], index: usize) -> usize {
    let mut end = index;
    while let Some(next) = tokens.get(end + 1) {
        if !(number_group(next.text)
            || (next.text.len() <= 4
                && next.text.starts_with(|c: char| c.is_ascii_digit())
                && next.text.bytes().all(|b| b.is_ascii_alphanumeric())))
            || !text[tokens[end].range.end..next.range.start]
                .chars()
                .all(|c| c.is_whitespace() || matches!(c, '(' | ')' | '-'))
        {
            break;
        }
        end += 1;
    }
    end
}
fn iban_end(text: &str, tokens: &[Token<'_>], index: usize) -> usize {
    let mut end = index;
    let mut characters = tokens[index].text.len();
    while let Some(next) = tokens.get(end + 1) {
        if lexicon::abbreviation(next.text).is_some()
            || numeric::label(next.text)
            || (next.text.bytes().all(|b| b.is_ascii_digit())
                && tokens.get(end + 2).is_some_and(|label| {
                    numeric::label(label.text)
                        && whitespace_between(text, next.range.end, label.range.start)
                }))
        {
            break;
        }
        if !whitespace_between(text, tokens[end].range.end, next.range.start)
            || next.text.is_empty()
            || !next
                .text
                .bytes()
                .all(|b| b.is_ascii_digit() || b.is_ascii_uppercase())
            || (!next.text.bytes().any(|b| b.is_ascii_digit()) && next.text.len() > 4)
            || (characters >= 26 && !next.text.bytes().all(|b| b.is_ascii_digit()))
        {
            break;
        }
        characters += next.text.len();
        end += 1;
    }
    end
}
