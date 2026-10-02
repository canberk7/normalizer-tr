use crate::{
    NormalizeError, SourceRange, WorkControl, resources::Resources, source_map::SourceMap,
};

use super::context;

#[derive(Clone, Copy)]
pub(super) struct Token<'a> {
    pub(super) range: SourceRange,
    pub(super) text: &'a str,
}

pub(super) fn overlaps(a: SourceRange, b: SourceRange) -> bool {
    a.start < b.end && b.start < a.end
}

pub(super) fn whitespace_between(text: &str, start: usize, end: usize) -> bool {
    start < end && text[start..end].chars().all(char::is_whitespace)
}
pub(super) fn cue_whitespace(text: &str, start: usize, end: usize) -> bool {
    start == end || whitespace_between(text, start, end)
}

fn delimiter(ch: char) -> bool {
    matches!(
        ch,
        ';' | '!' | '?' | '(' | ')' | '{' | '}' | '[' | ']' | '«' | '»' | '"'
    )
}

fn trimmed_range(text: &str, offset: usize) -> Option<SourceRange> {
    let leading = text.trim_start_matches(|ch| delimiter(ch) || matches!(ch, ',' | '…'));
    let mut body = leading.trim_end_matches(|ch| delimiter(ch) || ch == '…');
    if body.ends_with(',') && !body.ends_with(",,") {
        body = &body[..body.len() - 1];
    }

    if body.ends_with('.')
        && !body.ends_with("..")
        && body != "Dr."
        && !body[..body.len() - 1].bytes().all(|b| b.is_ascii_digit())
    {
        body = &body[..body.len() - 1];
    }
    let start = offset + text.len() - leading.len();
    (!body.is_empty()).then_some(SourceRange::new(start, start + body.len()))
}

fn expression_range(text: &str, offset: usize) -> Option<SourceRange> {
    if text.starts_with('"') && text.contains('@') {
        let body = text.trim_end_matches([',', ';', '!']);
        return Some(SourceRange::new(offset, offset + body.len()));
    }
    let leading = text.trim_start_matches(['(', '[', '{', '«', '"']);
    if crate::domain::electronic::looks_like(leading) {
        let mut body = leading.trim_end_matches(['.', ',', ';', '!', '»', '"']);
        while body.ends_with(')') && body.matches(')').count() > body.matches('(').count() {
            body = &body[..body.len() - 1];
        }
        body = body.trim_end_matches([']', '}']);
        let start = offset + text.len() - leading.len();
        return (!body.is_empty()).then_some(SourceRange::new(start, start + body.len()));
    }
    let range = trimmed_range(text, offset)?;
    let raw = &text[range.start - offset..];
    let without = raw.trim_end_matches(|c| delimiter(c) || matches!(c, ',' | '…'));
    let base = without.split(['\'', '’']).next().unwrap_or(without);
    let roman = base.trim_end_matches('.');
    let retain = lexical_period(without)
        || (!roman.is_empty() && roman.bytes().all(|b| b"IVXLCDM".contains(&b)));
    if retain {
        Some(SourceRange::new(range.start, range.start + without.len()))
    } else {
        Some(range)
    }
}

fn lexical_period(text: &str) -> bool {
    crate::domain::lexicon::abbreviation(text.split(['\'', '’']).next().unwrap_or(text)).is_some()
}

fn numeric_parenthesis_compound(raw: &str) -> bool {
    let body = raw
        .strip_prefix('(')
        .and_then(|body| body.strip_suffix(')'))
        .unwrap_or(raw);
    body.contains(['(', ')'])
        && !body.chars().any(char::is_alphabetic)
        && body
            .split(|c: char| !c.is_ascii_digit())
            .filter(|run| !run.is_empty())
            .count()
            > 1
}

pub(super) fn tokens<'a>(
    source: &'a SourceMap,
    rules: &Resources,
    control: &WorkControl,
) -> Result<Vec<Token<'a>>, NormalizeError> {
    let text = source.text();
    let mut tokens = Vec::new();
    let mut skip_until = 0;
    for matched in rules.tokens.find_iter(text) {
        control.check()?;
        if matched.start() < skip_until {
            continue;
        }
        let raw = matched.as_str();
        if raw.starts_with('"')
            && let Some(address) = rules
                .quoted_email
                .find_at(text, matched.start())
                .filter(|m| m.start() == matched.start())
        {
            let body = address.as_str().trim_end_matches([',', ';', '!']);
            append_token(
                &mut tokens,
                source,
                SourceRange::new(address.start(), address.start() + body.len()),
            );
            skip_until = address.end();
            continue;
        }
        if numeric_parenthesis_compound(raw) {
            append_token(
                &mut tokens,
                source,
                SourceRange::new(matched.start(), matched.end()),
            );
            continue;
        }
        let Some(range) = expression_range(raw, matched.start()) else {
            continue;
        };
        if crate::domain::electronic::looks_like(&text[range.start..range.end])
            || lexical_period(&text[range.start..range.end])
            || text[range.start..range.end]
                .trim_end_matches('.')
                .bytes()
                .all(|b| b"IVXLCDM".contains(&b))
        {
            append_token(&mut tokens, source, range);
            continue;
        }
        let body = &text[range.start..range.end];
        let prefix = context::inline_prefix(body).or_else(|| {
            (body.starts_with(':')
                && body[1..].starts_with(|c: char| c.is_ascii_digit())
                && tokens.last().is_some_and(|previous: &Token<'_>| {
                    !previous.text.contains(':')
                        && context::is_cue_word(previous.text)
                        && whitespace_between(text, previous.range.end, range.start)
                }))
            .then_some(1)
        });
        if let Some(prefix) = prefix {
            let body_start = range.start + prefix;
            let body_range = trimmed_range(&text[body_start..matched.end()], body_start)
                .ok_or(NormalizeError::Internal)?;
            append_token(
                &mut tokens,
                source,
                SourceRange::new(range.start, range.start + prefix),
            );
            append_token(&mut tokens, source, body_range);
            continue;
        }
        // Identifier punctuation (including URL queries) must not split the token.
        if identifier(&text[range.start..range.end]) {
            append_token(&mut tokens, source, range);
            continue;
        }
        let mut start = 0;
        for (offset, ch) in raw.char_indices().filter(|(_, ch)| delimiter(*ch)) {
            if let Some(range) = trimmed_range(&raw[start..offset], matched.start() + start) {
                append_token(&mut tokens, source, range);
            }
            start = offset + ch.len_utf8();
        }
        if let Some(range) = trimmed_range(&raw[start..], matched.start() + start) {
            append_token(&mut tokens, source, range);
        }
    }
    Ok(tokens)
}

fn append_token<'a>(tokens: &mut Vec<Token<'a>>, source: &'a SourceMap, range: SourceRange) {
    let range = source.cover(range);
    tokens.push(Token {
        range,
        text: &source.text()[range.start..range.end],
    });
}

pub(super) fn phones(
    text: &str,
    tokens: &[Token<'_>],
    rules: &Resources,
    control: &WorkControl,
) -> Result<Vec<SourceRange>, NormalizeError> {
    let mut phones = Vec::new();
    for matched in rules.phone_like.find_iter(text) {
        control.check()?;
        let range = SourceRange::new(matched.start(), matched.end());
        if tokens
            .binary_search_by_key(&range.start, |t| t.range.start)
            .is_ok()
            && tokens
                .binary_search_by_key(&range.end, |t| t.range.end)
                .is_ok()
        {
            phones.push(range);
        }
    }
    Ok(phones)
}

pub(super) fn spaced_compound(text: &str, tokens: &[Token<'_>], index: usize) -> Option<usize> {
    if !tokens[index].text.chars().any(|c| c.is_ascii_digit()) {
        return None;
    }
    let mut end = index;
    while let (Some(operator), Some(number)) = (tokens.get(end + 1), tokens.get(end + 2)) {
        if !math_operator(operator.text)
            || !number.text.chars().any(|c| c.is_ascii_digit())
            || !whitespace_between(text, tokens[end].range.end, operator.range.start)
            || !whitespace_between(text, operator.range.end, number.range.start)
        {
            break;
        }
        end += 2;
    }
    (end > index).then_some(end)
}

pub(super) fn math_operator(text: &str) -> bool {
    matches!(
        text,
        "/" | "-"
            | "–"
            | "—"
            | ":"
            | "x"
            | "×"
            | "^"
            | "+"
            | "="
            | "*"
            | "÷"
            | "<"
            | ">"
            | "≤"
            | "≥"
            | "≈"
            | "±"
    )
}

pub(super) fn identifier(text: &str) -> bool {
    if text.contains('@') || text.contains("://") || text.starts_with("www.") {
        return true;
    }
    let base = text.split(['\'', '’']).next().unwrap_or(text);
    let letters = base.chars().any(|c| c.is_alphabetic() && !c.is_numeric());
    let digits = base.chars().any(char::is_numeric);
    digits && (letters || base.contains('_'))
}

pub(super) fn unknown_abbreviation(text: &str) -> bool {
    let base = text.split(['\'', '’']).next().unwrap_or(text);
    let mut letters = base.chars().filter(|c| c.is_alphabetic());
    let Some(first) = letters.next() else {
        return false;
    };
    let Some(second) = letters.next() else {
        return false;
    };
    first.is_uppercase() && second.is_uppercase() && letters.all(char::is_uppercase)
}
