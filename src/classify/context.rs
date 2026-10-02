pub(super) const DATE_CUES: &[&str] = &["tarih", "tarihi"];
pub(super) const TIME_CUES: &[&str] = &["saat"];

/// Casing and optional colon removal apply only to reviewed lookup keys.
pub(super) fn cue_key(text: &str) -> String {
    crate::domain::lexicon::lookup_key(text.strip_suffix(':').unwrap_or(text))
}

pub(super) fn is_cue_word(text: &str) -> bool {
    let key = cue_key(text);
    DATE_CUES.contains(&key.as_str()) || TIME_CUES.contains(&key.as_str())
}

pub(super) fn is_clock_word(text: &str) -> bool {
    !text.contains(':') && TIME_CUES.contains(&cue_key(text).as_str())
}

pub(super) fn inline_prefix(text: &str) -> Option<usize> {
    let (word, body) = text.split_once(':')?;
    (is_cue_word(word) && body.starts_with(|c: char| c.is_ascii_digit())).then_some(word.len() + 1)
}
