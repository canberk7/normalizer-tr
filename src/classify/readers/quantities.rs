use super::super::scan::{Token, whitespace_between};
use super::{Attempt, Context};
use crate::domain::numeric::split_suffix;
use crate::{
    IssueCategory,
    domain::{
        lexicon,
        numeric::{self, NumericRange, Quantity},
    },
    model::Value,
};
pub(super) fn read(ctx: &Context<'_>, index: usize) -> Option<Attempt> {
    let text = ctx.text;
    let tokens = ctx.tokens;
    let token = tokens[index];
    let source = token.text;
    // Prefix/suffix symbols are whole quantities; no fragment fallback for malformed values.
    if let Some(ch) = source.chars().next().filter(|c| "₺$€£".contains(*c)) {
        if let Some(end) = quantity_math_end(text, tokens, index) {
            return Some((Err(IssueCategory::Unsupported), end));
        }
        if tokens.get(index + 1).is_some_and(|tail| {
            numeric::label(tail.text) && whitespace_between(text, token.range.end, tail.range.start)
        }) {
            return Some((Err(IssueCategory::InvalidExpression), index + 1));
        }
        let (number, case) = split_suffix(&source[ch.len_utf8()..]).unwrap_or(("", None));
        let label = format!("{ch}{}", case.map_or(String::new(), |s| format!("'{s}")));
        return Some((
            Quantity::parse(number, &label)
                .map(Value::Quantity)
                .ok_or(IssueCategory::InvalidExpression),
            index,
        ));
    }
    let (symbol_base, symbol_case) = split_suffix(source).unwrap_or((source, None));
    if let Some(ch) = symbol_base.chars().last().filter(|c| "₺$€£".contains(*c)) {
        if let Some(end) = quantity_math_end(text, tokens, index) {
            return Some((Err(IssueCategory::Unsupported), end));
        }
        if tokens.get(index + 1).is_some_and(|tail| {
            numeric::label(tail.text) && whitespace_between(text, token.range.end, tail.range.start)
        }) {
            return Some((Err(IssueCategory::InvalidExpression), index + 1));
        }
        let label = format!(
            "{ch}{}",
            symbol_case.map_or(String::new(), |case| format!("'{case}"))
        );
        return Some((
            Quantity::parse(&symbol_base[..symbol_base.len() - ch.len_utf8()], &label)
                .map(Value::Quantity)
                .ok_or(IssueCategory::InvalidExpression),
            index,
        ));
    }
    let next = tokens.get(index + 1)?;
    if !whitespace_between(text, token.range.end, next.range.start) {
        return None;
    }
    if source.chars().any(|c| c.is_ascii_digit())
        && source.contains(['-', '–'])
        && (lexicon::unit(next.text).is_some()
            || ["kişi", "adet", "gün", "yaş"].contains(&lexicon::lookup_key(next.text).as_str()))
    {
        let end = index + 1;
        if let Some(end) = quantity_math_end(text, tokens, end) {
            return Some((Err(IssueCategory::Unsupported), end));
        }
        return Some((
            NumericRange::parse(source, Some(next.text))
                .map(Value::Range)
                .ok_or(IssueCategory::InvalidExpression),
            end,
        ));
    }
    if numeric::label(next.text) && source.chars().any(|c| c.is_ascii_digit()) {
        if let Some(end) = quantity_math_end(text, tokens, index + 1) {
            return Some((Err(IssueCategory::Unsupported), end));
        }
        if tokens.get(index + 2).is_some_and(|tail| {
            numeric::label(tail.text) && whitespace_between(text, next.range.end, tail.range.start)
        }) {
            return Some((Err(IssueCategory::InvalidExpression), index + 2));
        }
        return Some((
            Quantity::parse(source, next.text)
                .map(Value::Quantity)
                .ok_or(IssueCategory::InvalidExpression),
            index + 1,
        ));
    }
    if source.chars().any(|c| c.is_ascii_digit())
        && lexicon::unit(&next.text.to_ascii_lowercase()).is_some()
    {
        return Some((Err(IssueCategory::Unsupported), index + 1));
    }
    if source.chars().any(|c| c.is_ascii_digit())
        && next.text.starts_with(char::is_alphabetic)
        && (next.text.contains(['/', '^', '²', '³'])
            || ["Μg", "μG", "µG", "ug", "oz", "cl", "dl", "ms"].contains(&next.text))
    {
        return Some((Err(IssueCategory::Unsupported), index + 1));
    }
    if numeric::label(source) && next.text.chars().any(|c| c.is_ascii_digit()) {
        return Some((
            Quantity::parse(next.text, source)
                .map(Value::Quantity)
                .ok_or(IssueCategory::InvalidExpression),
            index + 1,
        ));
    }

    None
}

fn quantity_math_end(text: &str, tokens: &[Token<'_>], mut end: usize) -> Option<usize> {
    let initial = end;
    while let (Some(operator), Some(number)) = (tokens.get(end + 1), tokens.get(end + 2)) {
        if !super::super::scan::math_operator(operator.text)
            || !number.text.chars().any(|c| c.is_ascii_digit())
            || !whitespace_between(text, tokens[end].range.end, operator.range.start)
            || !whitespace_between(text, operator.range.end, number.range.start)
        {
            break;
        }
        end += 2;
        if tokens.get(end + 1).is_some_and(|tail| {
            numeric::label(tail.text)
                && whitespace_between(text, tokens[end].range.end, tail.range.start)
        }) {
            end += 1;
        }
    }
    (end > initial).then_some(end)
}
