use crate::{
    IssueCategory,
    model::{Clock, Date, Value},
    morphology::Inflection,
    verbalize::{date_spoken, time_spoken},
};

use super::{
    context::{DATE_CUES, TIME_CUES, cue_key, is_clock_word},
    scan::{Token, whitespace_between},
};
use crate::domain::numeric::split_suffix;

pub(super) fn date(text: &str, permitted: bool) -> Result<Value, IssueCategory> {
    let (base, suffix) = split_suffix(text).ok_or(IssueCategory::InvalidExpression)?;
    let date = Date::parse(base).ok_or(IssueCategory::InvalidExpression)?;
    if suffix.is_some() && !date.dotted() {
        return Err(IssueCategory::Unsupported);
    }
    if let Some(suffix) = suffix
        && date_spoken(date).source_suffix(Inflection::Locative) != suffix
    {
        return Err(IssueCategory::InvalidExpression);
    }
    if !permitted {
        return Err(IssueCategory::Ambiguous);
    }
    Ok(Value::Date(date, suffix.is_some()))
}

pub(super) fn time(text: &str, permitted: bool) -> Result<Value, IssueCategory> {
    let (base, suffix) = split_suffix(text).ok_or(IssueCategory::InvalidExpression)?;
    let time = Clock::parse(base).ok_or(IssueCategory::InvalidExpression)?;
    if let Some(suffix) = suffix
        && time_spoken(time).source_suffix(Inflection::Locative) != suffix
    {
        return Err(IssueCategory::InvalidExpression);
    }
    if !permitted {
        return Err(IssueCategory::Ambiguous);
    }
    Ok(Value::Time(time, suffix.is_some()))
}

fn cue(text: &str, tokens: &[Token<'_>], index: usize, allowed: &[&str]) -> bool {
    let Some(mut previous_index) = index.checked_sub(1) else {
        return false;
    };
    let adjacent = |start, end| start == end || whitespace_between(text, start, end);
    if !adjacent(tokens[previous_index].range.end, tokens[index].range.start) {
        return false;
    }
    if tokens[previous_index].text == ":" {
        let Some(word_index) = previous_index.checked_sub(1) else {
            return false;
        };
        if !whitespace_between(
            text,
            tokens[word_index].range.end,
            tokens[previous_index].range.start,
        ) || tokens[word_index].text.contains(':')
        {
            return false;
        }
        previous_index = word_index;
    }
    allowed.contains(&cue_key(tokens[previous_index].text).as_str())
}

fn frame(text: &str, tokens: &[Token<'_>], index: usize) -> bool {
    let (Some(date_token), Some(cue_token), Some(time_token)) = (
        tokens.get(index),
        tokens.get(index + 1),
        tokens.get(index + 2),
    ) else {
        return false;
    };
    matches!(date(date_token.text, true), Ok(Value::Date(date, true)) if date.dotted())
        && is_clock_word(cue_token.text)
        && matches!(time(time_token.text, true), Ok(Value::Time(_, true)))
        && whitespace_between(text, date_token.range.end, cue_token.range.start)
        && whitespace_between(text, cue_token.range.end, time_token.range.start)
}

pub(super) fn recognize(
    text: &str,
    tokens: &[Token<'_>],
    index: usize,
) -> Option<Result<Value, IssueCategory>> {
    let token = tokens[index].text;
    let (base, _) = split_suffix(token)?;
    let dots = base.bytes().filter(|b| *b == b'.').count();
    let hyphens = base.bytes().filter(|b| *b == b'-').count();
    if dots == 2 || (hyphens == 2 && base.len() >= 8) {
        if dots == 2
            && crate::numerals::Number::parse(base).is_some_and(|n| n.grouped())
            && !cue(text, tokens, index, DATE_CUES)
        {
            return None;
        }
        return Some(date(
            token,
            frame(text, tokens, index) || cue(text, tokens, index, DATE_CUES),
        ));
    }
    if base.contains(':') || (dots == 1 && cue(text, tokens, index, TIME_CUES)) {
        return Some(time(token, cue(text, tokens, index, TIME_CUES)));
    }
    None
}
