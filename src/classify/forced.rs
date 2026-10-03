use super::readers;
use crate::{
    HintKind, SegmentKind,
    domain::{
        electronic,
        identifiers::Iban,
        lexicon::{Currency, lookup_key, numbered_place},
        literal::{self, Literal},
        numeric::{Numeric, Quantity, split_suffix},
    },
    model::{Clock, Date, Value},
    morphology::{Harmony, Style, Suffix, Word, WordEnd},
    verbalize::literal_text,
};

/// What a guess may look at besides the span itself.
#[derive(Clone, Copy)]
struct Context<'a> {
    /// The span's apostrophe suffix was split off, or the span is one end of a range.
    suffixed: bool,
    /// The recognition text before the span.
    before: &'a str,
    /// The recognition text after the span.
    following: &'a str,
}

/// A forced reading of an unresolved span.
pub(crate) struct Forced {
    /// The `forced.` rule the reading is reported under.
    pub(crate) rule_id: &'static str,
    pub(crate) value: Value,
    /// Bytes right after the span that the reading also says, which are then not kept as
    /// written: the inch mark of `6.8"`, the seconds mark of a coordinate, or the period that
    /// makes `90+4.` an ordinal.
    pub(crate) said_after: usize,
}

type Guess = fn(&str, Context<'_>) -> Option<Value>;

/// Guesses tried before the literal reading, in order, and the rule id each is reported
/// under. Where an explicit hint of the same kind accepts the span, the guess reads it as the
/// hint does. Some shapes are read only here: a date the calendar lacks, a ratio, a fraction,
/// an IBAN whose checksum fails, money in English digits, the end of the day and a clock with
/// seconds. A bare domain is never guessed.
const ORDER: [(&str, Guess); 14] = [
    ("forced.digits", digits),
    ("forced.time", time),
    ("forced.date", date),
    ("forced.range", between),
    ("forced.range", range),
    ("forced.ratio", ratio),
    ("forced.fraction", fraction),
    ("forced.telephone", telephone),
    ("forced.iban", iban),
    ("forced.money", money),
    ("forced.cardinal", cardinal),
    ("forced.roman", roman),
    ("forced.ordinal", ordinal),
    ("forced.electronic", address),
];

/// Two exact numbers, clocks or dates around a dash, with a noun or a suffix, fit in this
/// many bytes. A longer span is no range and is not searched for one, which keeps a span
/// full of dashes from being rescanned at every dash.
const LONGEST_RANGE: usize = 96;

/// A span that starts with `0` and another digit is an identifier, never a cardinal.
fn zero_padded(text: &str) -> bool {
    matches!(text.as_bytes(), [b'0', digit, ..] if digit.is_ascii_digit())
}

fn digit_run(text: &str) -> bool {
    !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit())
}

/// Digits read a zero-padded digit run (`0532`, `007`) and a card number in four groups of
/// four (`4111 1111 1111 1111`).
fn digits(text: &str, _: Context<'_>) -> Option<Value> {
    let groups: Vec<&str> = text.split(' ').collect();
    let card = groups.len() == CARD_GROUPS
        && groups
            .iter()
            .all(|group| group.len() == 4 && digit_run(group))
        && card_number(&groups.concat());
    ((zero_padded(text) && digit_run(text)) || card)
        .then(|| readers::hint(text, HintKind::Digits))
        .flatten()
}

/// Groups of four digits written with single spaces that make a card number.
const CARD_GROUPS: usize = 4;

/// Whether sixteen digits are a card's: a network's first digits and a Luhn check digit that
/// holds, which a list of years or prices almost never has (`2020 2021 2022 2023`).
fn card_number(digits: &str) -> bool {
    let network = digits.starts_with(['3', '4', '5', '6'])
        || digits
            .get(..4)
            .and_then(|first| first.parse::<u16>().ok())
            .is_some_and(|first| (2221..=2720).contains(&first));
    let sum: u32 = digits
        .bytes()
        .rev()
        .enumerate()
        .map(|(at, byte)| {
            let digit = u32::from(byte - b'0');
            match (at % 2, digit * 2) {
                (1, doubled) if doubled > 9 => doubled - 9,
                (1, doubled) => doubled,
                _ => digit,
            }
        })
        .sum();
    digits.len() == 16
        && digits.bytes().all(|byte| byte.is_ascii_digit())
        && network
        && sum.is_multiple_of(10)
}

/// A group of a card number the engine resolved as a cardinal, said digit by digit: each of
/// `4111 1111 1111 1111` is dört bir bir bir. Only the groups next to it are looked at.
pub(crate) fn card_group(text: &str, start: usize, end: usize) -> Option<String> {
    let group = text.get(start..end)?;
    if group.len() != 4 || !digit_run(group) {
        return None;
    }
    let bytes = text.as_bytes();
    let four = |at: usize| {
        bytes
            .get(at..at + 4)
            .is_some_and(|digits| digits.iter().all(u8::is_ascii_digit))
    };
    let digit = |at: usize| bytes.get(at).is_some_and(u8::is_ascii_digit);
    let mut groups = 1;
    let mut left = start;
    while groups < CARD_GROUPS
        && left >= 5
        && bytes[left - 1] == b' '
        && four(left - 5)
        && !(left > 5 && digit(left - 6))
    {
        groups += 1;
        left -= 5;
    }
    let mut right = end;
    while groups < CARD_GROUPS
        && bytes.get(right) == Some(&b' ')
        && four(right + 1)
        && !digit(right + 5)
    {
        groups += 1;
        right += 5;
    }
    let number: String = text.get(left..right)?.split(' ').collect();
    (groups == CARD_GROUPS && card_number(&number)).then(|| crate::numerals::digits(group))
}

/// A colon clock always; a dotted one only after `saat`, with a suffix, with a zero before
/// its hour or with whole hours (`saat 14.30`, `15.30'da`, `09.30`, `22.00`), because
/// `macOS 10.15` and `3.12` are more often a version or a decimal. `24:00` is the end of the
/// day, and `3:45:30` a clock with seconds or a duration.
fn time(text: &str, context: Context<'_>) -> Option<Value> {
    if matches!(text, "24:00" | "24.00") {
        return Some(Value::Time(Clock::end_of_day(), false));
    }
    if let Some(seconds) = seconds(text, context.before).or_else(|| lap(text)) {
        return Some(seconds);
    }
    let clock = text.split(['\'', '’']).next().unwrap_or(text);
    let suffixed = context.suffixed || clock.len() < text.len();
    let dotted = clock.split_once('.').is_some_and(|(hour, minute)| {
        hour.len() == 2 && (hour.starts_with('0') || minute == "00" || after_saat(context.before))
    });
    (clock.contains(':') || suffixed || dotted)
        .then(|| readers::hint(text, HintKind::Time))
        .flatten()
}

/// Whether the word right before a span is `saat`.
fn after_saat(before: &str) -> bool {
    let previous = before
        .trim_end()
        .rsplit(char::is_whitespace)
        .next()
        .unwrap_or_default();
    lookup_key(previous) == "saat"
}

/// `H:MM:SS`: a clock with seconds after the word `saat`, with a two-digit hour or with zero
/// seconds (`saat 3:45:30`, `14:30:15`, `9:30:00`), and otherwise a duration, as a race or a
/// film is timed: `rekor 2:00:35` is iki saat otuz beş saniye.
fn seconds(text: &str, before: &str) -> Option<Value> {
    let mut parts = text.split(':');
    let (hour, minute, second) = (parts.next()?, parts.next()?, parts.next()?);
    let two_digits = |part: &str| part.len() == 2 && digit_run(part);
    if parts.next().is_some() || !two_digits(minute) || !two_digits(second) {
        return None;
    }
    let second: u8 = second.parse().ok().filter(|second| *second < 60)?;
    if hour.len() == 2 || second == 0 || after_saat(before) {
        // Zero seconds are not said: `14:30:00` is on dört otuz.
        let clock = Clock::parse(&format!("{hour}:{minute}"))?;
        return Some(if second == 0 {
            Value::Time(clock, false)
        } else {
            Value::Seconds(clock, second)
        });
    }
    let hour: u8 = (hour.len() == 1).then(|| hour.parse().ok()).flatten()?;
    let minute: u8 = minute.parse().ok().filter(|minute| *minute < 60)?;
    duration(hour, minute, second)
}

/// A race time with a fraction of a second: `1:23.456` is bir dakika yirmi üç virgül dört yüz
/// elli altı saniye.
fn lap(text: &str) -> Option<Value> {
    let (clock, fraction) = text.rsplit_once('.')?;
    if !(1..=3).contains(&fraction.len()) || !digit_run(fraction) {
        return None;
    }
    let parts: Vec<&str> = clock.split(':').collect();
    let (hour, minute, second) = match parts.as_slice() {
        [minute, second] => ("0", *minute, *second),
        [hour, minute, second] => (*hour, *minute, *second),
        _ => return None,
    };
    let number = |part: &str, most: u8| {
        (!part.is_empty() && part.len() <= 2 && digit_run(part))
            .then(|| part.parse::<u8>().ok())
            .flatten()
            .filter(|value| *value < most)
    };
    let hour = number(hour, 100)?;
    let minute = number(minute, if parts.len() == 3 { 60 } else { 100 })?;
    if second.len() != 2 {
        return None;
    }
    let second = number(second, 60)?;
    let mut reading: Option<Value> = None;
    for (amount, unit) in [(hour, "sa"), (minute, "dk")] {
        if amount == 0 {
            continue;
        }
        let part = Value::Quantity(Quantity::parse(&amount.to_string(), unit)?);
        reading = Some(match reading {
            Some(left) => Value::Joined(SegmentKind::Unit, Box::new(left), " ", Box::new(part)),
            None => part,
        });
    }
    let seconds = Value::Quantity(Quantity::parse(&format!("{second},{fraction}"), "sn")?);
    Some(match reading {
        Some(left) => Value::Joined(SegmentKind::Unit, Box::new(left), " ", Box::new(seconds)),
        None => seconds,
    })
}

/// A duration said in hours, minutes and seconds, leaving out the ones that are zero.
fn duration(hour: u8, minute: u8, second: u8) -> Option<Value> {
    let mut reading: Option<Value> = None;
    for (amount, unit) in [(hour, "sa"), (minute, "dk"), (second, "sn")] {
        if amount == 0 {
            continue;
        }
        let part = Value::Quantity(Quantity::parse(&amount.to_string(), unit)?);
        reading = Some(match reading {
            Some(left) => Value::Joined(SegmentKind::Unit, Box::new(left), " ", Box::new(part)),
            None => part,
        });
    }
    reading.or_else(|| Quantity::parse("0", "sn").map(Value::Quantity))
}

/// A dotted or ISO date as the hint reads it, then any day-month-year by its shape alone, so
/// that `29.02.1900` is still read as a date. The same holds for a date with slashes, one
/// written year first (`2026.05.19`) and one with a two-digit year (`01/04/26`).
fn date(text: &str, _: Context<'_>) -> Option<Value> {
    let read = |written: &str| {
        readers::hint(written, HintKind::Date)
            .or_else(|| Date::shaped(written).map(|date| Value::Date(date, false)))
    };
    let slashed = text.matches('/').count() == 2 && !text.contains('.');
    if slashed {
        read(&text.replace('/', "."))
    } else {
        read(text)
    }
    .or_else(|| read(&day_first(text)?))
}

/// A date respelled day first with a four-digit year: `2026.05.19` and `2026/05/19` are
/// 19.05.2026, and a two-digit year is in this century below 50, else in the last one
/// (`01/04/26` is 01.04.2026). A dotted date with a two-digit year needs two-digit day and
/// month, since `1.5.26` is more often a version.
fn day_first(text: &str) -> Option<String> {
    let separator = ['/', '.']
        .into_iter()
        .find(|separator| text.matches(*separator).count() == 2)?;
    let parts: Vec<&str> = text.split(separator).collect();
    let [first, second, third] = parts.as_slice() else {
        return None;
    };
    if !parts.iter().all(|part| digit_run(part)) || !(1..=2).contains(&second.len()) {
        return None;
    }
    match (first.len(), third.len()) {
        (4, 1..=2) => Some(format!("{third}.{second}.{first}")),
        (1..=2, 2) if separator == '/' || (first.len() == 2 && second.len() == 2) => {
            let year: u16 = third.parse().ok()?;
            let century = if year < 50 { 2000 } else { 1900 };
            Some(format!("{first}.{second}.{}", century + year))
        }
        _ => None,
    }
}

/// Two clocks or two dates around one dash, said with the dash: `09:00-17:30` is dokuz tire on
/// yedi otuz.
fn between(text: &str, _: Context<'_>) -> Option<Value> {
    if text.len() > LONGEST_RANGE {
        return None;
    }
    let end = Context {
        suffixed: true,
        before: "",
        following: "",
    };
    let mut found = None;
    for (at, dash) in text.match_indices(['-', '–']) {
        let (from, to) = (text[..at].trim_end(), text[at + dash.len()..].trim_start());
        let ends = [time as Guess, date]
            .into_iter()
            .find_map(|read| Some((read(from, end)?, read(to, end)?)));
        if ends.is_some() && found.is_some() {
            return None;
        }
        found = found.or(ends);
    }
    let (from, to) = found?;
    Some(Value::Joined(
        SegmentKind::Range,
        Box::new(from),
        " tire ",
        Box::new(to),
    ))
}

fn range(text: &str, _: Context<'_>) -> Option<Value> {
    (text.len() <= LONGEST_RANGE)
        .then(|| readers::hint(text, HintKind::Range))
        .flatten()
}

/// Two plain numbers around a colon that are no clock: `1:3` is bire üç.
fn ratio(text: &str, _: Context<'_>) -> Option<Value> {
    let (first, second) = text.split_once(':')?;
    let plain = |number: &str| digit_run(number) && !zero_padded(number);
    if !plain(first) || !plain(second) {
        return None;
    }
    let first = readers::hint(first, HintKind::Cardinal)?;
    let second = readers::hint(second, HintKind::Cardinal)?;
    Some(Value::Joined(
        SegmentKind::Cardinal,
        Box::new(Value::Suffixed(Box::new(first), Suffix::new("e"))),
        " ",
        Box::new(second),
    ))
}

/// A fraction, said with bölü: `1/2` is bir bölü iki, `2/3'ü` iki bölü üçü. Its numerator is
/// one digit below a denominator of at most 1000; a rating out of five, ten or a hundred may
/// have any numerator up to it (`4,5/5`, `10/10`, `85/100`). Other slashes are said slaş:
/// `24/7`, `7/24`, `60/65`, and a building and flat after an address word (`No: 3/5`).
fn fraction(text: &str, context: Context<'_>) -> Option<Value> {
    let (numerator, denominator) = text.split_once('/')?;
    let plain = |number: &str| digit_run(number) && !zero_padded(number);
    if !plain(denominator) || addressed(context.before) {
        return None;
    }
    let bottom: u16 = denominator.parse().ok()?;
    let small = numerator.len() == 1
        && plain(numerator)
        && numerator
            .parse::<u16>()
            .is_ok_and(|top| top >= 1 && top < bottom)
        && bottom <= 1000
        && (numerator, bottom) != ("7", 24);
    if !small && !(matches!(bottom, 5 | 10 | 100) && rated(numerator, bottom)) {
        return None;
    }
    let top = readers::hint(numerator, HintKind::Cardinal)
        .or_else(|| Numeric::automatic(numerator).ok().map(Value::Numeric))?;
    let bottom = readers::hint(denominator, HintKind::Cardinal)?;
    Some(Value::Joined(
        SegmentKind::Cardinal,
        Box::new(top),
        " bölü ",
        Box::new(bottom),
    ))
}

/// Whether a numerator scores out of `bottom`: a number, perhaps with a decimal comma, that
/// is no more than it.
fn rated(numerator: &str, bottom: u16) -> bool {
    let (whole, fraction) = numerator.split_once(',').unwrap_or((numerator, ""));
    let plain =
        digit_run(whole) && !zero_padded(whole) && (fraction.is_empty() || digit_run(fraction));
    plain
        && whole.parse::<u16>().is_ok_and(|whole| {
            whole < bottom || (whole == bottom && fraction.bytes().all(|digit| digit == b'0'))
        })
}

/// Whether the word before a span numbers an address, after which `3/5` is a building and a
/// flat: `No: 3/5`, `Daire 2/4`.
fn addressed(before: &str) -> bool {
    let word = before
        .trim_end()
        .rsplit(char::is_whitespace)
        .next()
        .unwrap_or_default()
        .trim_end_matches([':', '.']);
    matches!(
        lookup_key(word).as_str(),
        "no" | "nu" | "nr" | "numara" | "numarası" | "daire" | "d" | "kat" | "blok" | "bl" | "blk"
    )
}

fn telephone(text: &str, _: Context<'_>) -> Option<Value> {
    readers::hint(text, HintKind::Telephone)
        .or_else(|| readers::hint(&regrouped(text)?, HintKind::Telephone))
}

/// A Turkish number written in other groups or with dashes, in the groups the telephone
/// reader knows: `+90 532 123 4567` is +90 532 123 45 67, `0532-123-4567` is 0532 123 45 67.
fn regrouped(text: &str) -> Option<String> {
    let phone_like = text
        .chars()
        .all(|ch| ch.is_ascii_digit() || matches!(ch, '+' | ' ' | '-' | '(' | ')' | '.'));
    // Four dotted groups of at most 255 are an internet address: `255.255.255.0`.
    let address = text.split('.').count() == 4
        && text.split('.').all(|group| {
            (1..=3).contains(&group.len()) && group.parse::<u16>().is_ok_and(|n| n <= 255)
        });
    if !phone_like || address || !text.contains([' ', '-', '(', '.']) {
        return None;
    }
    let digits: String = text.chars().filter(char::is_ascii_digit).collect();
    let (prefix, national) = if text.starts_with('+') {
        ("+90 ", digits.strip_prefix("90")?)
    } else if let Some(national) = digits.strip_prefix('0') {
        ("0", national)
    } else {
        ("", digits.as_str())
    };
    if national.len() != 10 || !national.starts_with(['2', '3', '4', '5', '8']) {
        return None;
    }
    Some(format!(
        "{prefix}{} {} {} {}",
        &national[..3],
        &national[3..6],
        &national[6..8],
        &national[8..]
    ))
}

/// Money with its currency sign or code, also written in English digits: `$1,299.99` is bin
/// iki yüz doksan dokuz dolar doksan dokuz sent, `₺42.000` kırk iki bin lira.
fn money(text: &str, _: Context<'_>) -> Option<Value> {
    let (number, label) = if let Some(sign) = text.chars().next().filter(|ch| "₺$€£".contains(*ch))
    {
        (&text[sign.len_utf8()..], &text[..sign.len_utf8()])
    } else if let Some(sign) = text.chars().last().filter(|ch| "₺$€£".contains(*ch)) {
        (
            &text[..text.len() - sign.len_utf8()],
            &text[text.len() - sign.len_utf8()..],
        )
    } else {
        let at = text.find(|ch: char| ch.is_alphabetic())?;
        let (number, code) = (text[..at].trim_end(), &text[at..]);
        Currency::parse(code)?;
        (number, code)
    };
    Quantity::parse(&turkish_amount(number)?, label).map(Value::Quantity)
}

/// An amount in Turkish digits with thousand dots: English `1,299.99` is 1.299,99, and
/// Turkish digits are kept. One to two digits after a lone dot are an English fraction.
fn turkish_amount(number: &str) -> Option<String> {
    let english = number.contains(',') && number.rfind('.') > number.rfind(',')
        || number.matches(',').count() > 1
        || number.split_once('.').is_some_and(|(_, fraction)| {
            (1..=2).contains(&fraction.len()) && !number.contains(',')
        });
    let (integer, fraction) = if english {
        let (integer, fraction) = number.split_once('.').unwrap_or((number, ""));
        (integer.replace(',', ""), fraction)
    } else {
        let (integer, fraction) = number.split_once(',').unwrap_or((number, ""));
        (integer.replace('.', ""), fraction)
    };
    if !digit_run(&integer) || !(fraction.is_empty() || digit_run(fraction)) {
        return None;
    }
    let grouped = integer
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(|chunk| String::from_utf8_lossy(chunk).into_owned())
        .collect::<Vec<_>>()
        .join(".");
    Some(if fraction.is_empty() {
        grouped
    } else {
        format!("{grouped},{fraction}")
    })
}

/// An IBAN by its shape: the failed checksum stays an issue, but the span is read as one.
fn iban(text: &str, _: Context<'_>) -> Option<Value> {
    Iban::shaped(text).map(Value::Iban)
}

fn cardinal(text: &str, _: Context<'_>) -> Option<Value> {
    (!zero_padded(text))
        .then(|| readers::hint(text, HintKind::Cardinal))
        .flatten()
}

/// Roman is guessed for a numeral of I, V and X that has two letters or more, a period or a
/// suffix, and for any numeral of four letters or more. A lone `X`, sizes and abbreviations
/// such as `XL`, `CD` and `CV`, and initials such as `M.` are not numbers to guess; nor is
/// `V.` before a name, an initial far more often than a ruler's number (`V. Öztürk`), while
/// `I.` and `X.` there are numbers (`I. Dünya Savaşı`).
fn roman(text: &str, context: Context<'_>) -> Option<Value> {
    let numeral = text.split(['.', '\'', '’']).next().unwrap_or(text);
    let small = numeral
        .bytes()
        .all(|byte| matches!(byte, b'I' | b'V' | b'X'));
    let word = context.following.trim_start();
    let initial = text == "V."
        && !breaks_line(&context.following[..context.following.len() - word.len()])
        && word.starts_with(char::is_uppercase);
    (!initial
        && (numeral.len() >= 4 || (small && (numeral.len() >= 2 || numeral.len() < text.len()))))
    .then(|| readers::hint(text, HintKind::Roman))
    .flatten()
}

/// Whether whitespace holds a line break.
fn breaks_line(gap: &str) -> bool {
    gap.contains([
        '\n', '\r', '\u{b}', '\u{c}', '\u{85}', '\u{2028}', '\u{2029}',
    ])
}

/// A period is an ordinal mark when a suffix follows it, or a word after whitespace: a
/// lowercase word always (`3. kez`), a capital on the same line after a number below 1000
/// (`1. Dünya Savaşı`), and a numbered street of any number (`1234. Sok.`). At the end of the
/// text, and before a capital on the next line or after a year, it may close a sentence and
/// is not guessed.
fn ordinal(text: &str, context: Context<'_>) -> Option<Value> {
    let word = context.following.trim_start();
    let gap = &context.following[..context.following.len() - word.len()];
    let small = text.len() <= "999.".len();
    // A comma after the period makes a list of ordinals: `1., 2. ve 3.`.
    let listed = context.following.starts_with([',', ';']);
    let next = word.split(char::is_whitespace).next().unwrap_or_default();
    let street = !gap.is_empty() && !breaks_line(gap) && numbered_place(next);
    let follows = listed
        || street
        || (!gap.is_empty()
            && word.starts_with(|ch: char| {
                ch.is_lowercase() || (ch.is_alphabetic() && small && !breaks_line(gap))
            }));
    (context.suffixed || follows)
        .then(|| readers::hint(text, HintKind::Ordinal))
        .flatten()
}

/// A whole address that only its suffix kept unresolved is read as the engine reads it.
/// A bare domain is never guessed.
fn address(text: &str, context: Context<'_>) -> Option<Value> {
    (context.suffixed && electronic::looks_like(text))
        .then(|| readers::hint(text, HintKind::Electronic))
        .flatten()
}

fn guess(text: &str, context: Context<'_>) -> Option<(&'static str, Value)> {
    ORDER
        .into_iter()
        .find_map(|(rule_id, read)| Some((rule_id, read(text, context)?)))
}

/// A guess at the span without its apostrophe suffix, which is then attached to that reading.
fn suffixed(text: &str, context: Context<'_>) -> Option<(&'static str, Value)> {
    let (base, suffix) = split_suffix(text)?;
    let suffix = suffix.filter(|letters| letters.chars().all(char::is_alphabetic))?;
    let context = Context {
        suffixed: true,
        ..context
    };
    let (rule_id, reading) = guess(base, context)?;
    Some((
        rule_id,
        Value::Suffixed(Box::new(reading), Suffix::new(suffix)),
    ))
}

/// Brackets, quotes and punctuation written around a span, kept around its reading.
const OPENING: &[char] = &['(', '[', '{', '«', '"', '“', '‘'];
const CLOSING: &[char] = &[')', ']', '}', '»', '"', '”', '’', ',', ';', ':', '!', '?'];

/// A guess at the span without the brackets, quotes and commas around it, which stay as
/// written: `09:00-18:00),` is dokuz tire on sekiz),.
fn framed(text: &str, context: Context<'_>) -> Option<(&'static str, Value)> {
    let inner = text.trim_start_matches(OPENING);
    let core = inner.trim_end_matches(CLOSING);
    if core.is_empty() || core.len() == text.len() {
        return None;
    }
    let (open, close) = (&text[..text.len() - inner.len()], &inner[core.len()..]);
    let after = format!("{close}{}", context.following);
    let context = Context {
        suffixed: false,
        before: context.before,
        following: &after,
    };
    let (rule_id, reading) = guess(core, context).or_else(|| suffixed(core, context))?;
    Some((
        rule_id,
        Value::Framed(open.to_owned(), Box::new(reading), close.to_owned()),
    ))
}

/// A period right after the span, before a lowercase word, marks its last number an
/// ordinal: `90+4. dakikada`.
fn ordinal_period(following: &str) -> bool {
    following.strip_prefix('.').is_some_and(|rest| {
        let word = rest.trim_start_matches([' ', '\t', '\u{a0}']);
        word.len() < rest.len() && word.starts_with(char::is_lowercase)
    })
}

/// Whether the next word is a unit or a currency, which makes a dotted number before it a
/// decimal: `3.5 dolar` is üç virgül beş dolar.
fn measured(following: &str) -> bool {
    let word: String = following
        .trim_start()
        .chars()
        .take_while(|ch| ch.is_alphabetic())
        .collect();
    let key = crate::domain::lexicon::lookup_key(&word);
    !word.is_empty()
        && (crate::domain::lexicon::unit(&word).is_some()
            || crate::domain::lexicon::unit(&key).is_some()
            || Currency::parse(&word).is_some()
            || ["dolar", "lira", "avro", "euro", "sterlin", "kuruş", "sent"]
                .iter()
                .any(|currency| key.starts_with(currency)))
}

/// `"` right after a decimal span is inches: `6.8" ekran` is altı virgül sekiz inç ekran.
fn inches(text: &str, following: &str) -> bool {
    following.starts_with(['"', '″'])
        && text.ends_with(|ch: char| ch.is_ascii_digit())
        && text.contains(['.', ','])
}

/// Whether nothing but a line break or the end of the text follows.
fn closes_text(following: &str) -> bool {
    let word = following.trim_start();
    word.is_empty() || breaks_line(&following[..following.len() - word.len()])
}

/// The forced reading of an unresolved span: a guess at the span as written, else at the span
/// without its suffix, else at the span without the brackets and punctuation around it, else
/// the literal reading. Literal rejects nothing, so this is always `Some`. A compass letter
/// right after a coordinate is its direction, compass points joined by a dash are a wind's,
/// and `"` right after a coordinate's minutes is seconds. A Roman ordinal at the end of the
/// text keeps its period, which closes the sentence too: `Elizabeth II.` is Elizabeth ikinci.
pub(crate) fn read(text: &str, before: &str, following: &str) -> Option<Forced> {
    if unspoken(text, before, following) {
        return Some(Forced {
            rule_id: "forced.literal",
            value: Value::Literal(Literal::empty()),
            said_after: 0,
        });
    }
    let pointed = direction(text, before.chars().next_back())
        .map(|direction| {
            let mut reading = Literal::word(direction);
            reading.open_with(" ");
            reading
        })
        .or_else(|| compass(text));
    if let Some(reading) = pointed {
        return Some(Forced {
            rule_id: "forced.literal",
            value: Value::Literal(reading),
            said_after: 0,
        });
    }
    let context = Context {
        suffixed: false,
        before,
        following,
    };
    let mut said_after = 0;
    let (rule_id, reading) = guess(text, context)
        .or_else(|| suffixed(text, context))
        .or_else(|| framed(text, context))
        .unwrap_or_else(|| {
            let mut literal = Literal::parse(text);
            if ordinal_period(following) && literal.ordinal_tail() {
                said_after = '.'.len_utf8();
            }
            if inches(text, following) || measured(following) {
                literal.decimal_point();
            }
            literal.multiplier(text, before, following);
            literal.rating(text, following);
            ("forced.literal", Value::Literal(literal))
        });
    let mark = following
        .chars()
        .next()
        .filter(|mark| matches!(mark, '"' | '″'));
    let seconds =
        text.contains('°') && text.ends_with(|ch: char| ch.is_ascii_digit()) && mark.is_some();
    let close = |reading: Value, word: &str| {
        Value::Framed(String::new(), Box::new(reading), word.to_owned())
    };
    let value = if inches(text, following) || seconds {
        said_after = mark.map_or(0, char::len_utf8);
        close(reading, if seconds { " saniye" } else { " inç" })
    } else if rule_id == "forced.roman" && text.ends_with('.') && closes_text(following) {
        close(reading, ".")
    } else {
        reading
    };
    Some(Forced {
        rule_id,
        value,
        said_after,
    })
}

/// Whether a span or a stray word says nothing where it stands: a face, an arrow, a
/// decoration or a Markdown mark (`:D`, `->`, `***`, `# Başlık`), and a lone `*`, `<` or `>`
/// between words (`Kategori > Telefon`).
fn unspoken(word: &str, before: &str, following: &str) -> bool {
    literal::silent(word, before, following)
        || (matches!(word, "*" | "<" | ">") && !literal::operates(word, before, following))
}

/// A resolved plain number that is an identifier, said as the literal reading says one: a
/// mobile number of ten digits that starts with 5 in its groups, and any other run of seven
/// digits or more digit by digit.
pub(crate) fn identifier(text: &str) -> Option<String> {
    literal::identifier(text).then(|| literal_text(&Literal::parse(text), Style::Spoken))
}

/// The compass points in Turkish letters.
const COMPASS: &[(&str, Word)] = &[
    ("K", Word::new("kuzey", Harmony::FrontFlat, WordEnd::Voiced)),
    ("G", Word::new("güney", Harmony::FrontFlat, WordEnd::Voiced)),
    ("D", Word::new("doğu", Harmony::BackRound, WordEnd::Vowel)),
    ("B", Word::new("batı", Harmony::BackFlat, WordEnd::Vowel)),
    (
        "KD",
        Word::new("kuzeydoğu", Harmony::BackRound, WordEnd::Vowel),
    ),
    (
        "KB",
        Word::new("kuzeybatı", Harmony::BackFlat, WordEnd::Vowel),
    ),
    (
        "GD",
        Word::new("güneydoğu", Harmony::BackRound, WordEnd::Vowel),
    ),
    (
        "GB",
        Word::new("güneybatı", Harmony::BackFlat, WordEnd::Vowel),
    ),
];

fn compass_point(letters: &str) -> Option<Word> {
    COMPASS
        .iter()
        .find(|(written, _)| *written == letters)
        .map(|(_, word)| *word)
}

/// A compass letter right after the seconds of a coordinate, in Turkish or English letters:
/// `41°00'49"K` is kuzey.
fn direction(word: &str, previous: Option<char>) -> Option<Word> {
    if !matches!(previous, Some('"' | '″')) {
        return None;
    }
    compass_point(match word {
        "N" => "K",
        "S" => "G",
        "E" => "D",
        "W" => "B",
        _ if word.len() == 1 => word,
        _ => return None,
    })
}

/// Compass points joined by a dash, as a wind's direction is written: `K-KD` is kuzey
/// kuzeydoğu. One point has two letters, since `K-D` alone may be anything.
fn compass(text: &str) -> Option<Literal> {
    let points: Option<Vec<Word>> = text.split('-').map(compass_point).collect();
    let points = points.filter(|points| points.len() > 1)?;
    text.split('-')
        .any(|point| point.len() > 1)
        .then(|| Literal::words(&points))
}

/// The literal reading of a word no reader claims, when it says more than the word's text:
/// a named symbol, letters spelled out by name, a compass letter after a coordinate or
/// compass points joined by a dash. It is spoken, but it is no issue. It is set apart from a
/// bracket written right after it: `f(x)` is fe (iks).
pub(crate) fn stray(word: &str, before: &str, following: &str) -> Option<Value> {
    if unspoken(word, before, following) {
        return Some(Value::Stray(Literal::empty()));
    }
    if let Some(direction) = direction(word, before.chars().next_back()) {
        let mut reading = Literal::word(direction);
        // Set apart from the seconds mark it follows, which the coordinate before it says.
        reading.open_with(" ");
        return Some(Value::Stray(reading));
    }
    if let Some(points) = compass(word) {
        return Some(Value::Stray(points));
    }
    // A unit right after a number that another span read: `8,25 mm`, `± 5,3 kg`.
    let after_number = before.ends_with(char::is_whitespace)
        && before.trim_end().ends_with(|ch: char| ch.is_ascii_digit());
    if after_number && let Some(unit) = Literal::unit_after_number(word) {
        return Some(Value::Stray(unit));
    }
    if !literal::respells(word) {
        return None;
    }
    let mut reading = Literal::parse(word);
    if literal_text(&reading, Style::Spoken) == word {
        return None;
    }
    reading.multiplier(word, before, following);
    if following.starts_with(['(', '[', '{']) {
        reading.close_with(" ");
    }
    Some(Value::Stray(reading))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::verbalize::render;

    fn forced_before(text: &str, following: &str) -> (&'static str, String) {
        let reading = read(text, "", following).unwrap();
        (reading.rule_id, render(&reading.value, Style::Spoken).2)
    }

    fn forced(text: &str) -> (&'static str, String) {
        forced_before(text, "")
    }

    #[test]
    fn the_first_guess_that_reads_a_span_wins() {
        for (text, rule_id, spoken) in [
            ("0532", "forced.digits", "sıfır beş üç iki"),
            ("007", "forced.digits", "sıfır sıfır yedi"),
            ("09:30'da", "forced.time", "dokuz otuzda"),
            ("09.30", "forced.time", "dokuz otuz"),
            ("12.05'te", "forced.time", "on iki sıfır beşte"),
            (
                "14.03.2026",
                "forced.date",
                "on dört Mart iki bin yirmi altı",
            ),
            ("01.02.2026", "forced.date", "bir Şubat iki bin yirmi altı"),
            (
                "2026-03-14",
                "forced.date",
                "on dört Mart iki bin yirmi altı",
            ),
            ("10-15", "forced.range", "on tire on beş"),
            (
                "532 123 45 67",
                "forced.telephone",
                "beş yüz otuz iki yüz yirmi üç kırk beş altmış yedi",
            ),
            ("1.234", "forced.cardinal", "bin iki yüz otuz dört"),
            ("1.234'ü", "forced.cardinal", "bin iki yüz otuz dördü"),
            ("IV", "forced.roman", "dört"),
            // At the end of the text the period of an ordinal closes the sentence too.
            ("IV.", "forced.roman", "dördüncü."),
            (
                "29.02.1900",
                "forced.date",
                "yirmi dokuz Şubat bin dokuz yüz",
            ),
            (
                "32.01.2026",
                "forced.literal",
                "otuz iki nokta sıfır bir nokta iki bin yirmi altı",
            ),
        ] {
            assert_eq!(forced(text), (rule_id, spoken.to_owned()), "{text}");
        }
    }

    #[test]
    fn a_zero_padded_run_is_digits_and_never_a_cardinal() {
        for (text, rule_id, spoken) in [
            ("00042", "forced.digits", "sıfır sıfır sıfır dört iki"),
            ("0532'yi", "forced.digits", "sıfır beş üç ikiyi"),
            (
                "001.234",
                "forced.literal",
                "sıfır sıfır bir nokta iki yüz otuz dört",
            ),
            // Not zero-padded: the same shapes are cardinals.
            ("532", "forced.cardinal", "beş yüz otuz iki"),
            ("0", "forced.cardinal", "sıfır"),
            // Zero-padded with separators: left to the guesses that read the shape.
            (
                "0850 22 33 44",
                "forced.literal",
                "sıfır sekiz beş sıfır yirmi iki otuz üç kırk dört",
            ),
        ] {
            assert_eq!(forced(text), (rule_id, spoken.to_owned()), "{text}");
        }
    }

    #[test]
    fn clocks_dates_and_their_ranges_are_read_in_the_shapes_people_write() {
        for (text, rule_id, spoken) in [
            // A dotted clock needs a suffix, a zero before its hour or whole hours; else it is
            // a version or a decimal.
            ("20.00", "forced.time", "yirmi"),
            ("9.30'da", "forced.time", "dokuz otuzda"),
            ("20.45", "forced.literal", "yirmi nokta kırk beş"),
            ("10.15", "forced.literal", "on nokta on beş"),
            ("12.05", "forced.literal", "on iki nokta sıfır beş"),
            ("3.12", "forced.literal", "üç nokta on iki"),
            ("3.5", "forced.literal", "üç nokta beş"),
            // The end of the day.
            ("24:00", "forced.time", "yirmi dört"),
            ("24:00'te", "forced.time", "yirmi dörtte"),
            ("24:01", "forced.literal", "yirmi dört iki nokta sıfır bir"),
            // A slash date is day, month, year.
            ("02/10/2026", "forced.date", "iki Ekim iki bin yirmi altı"),
            (
                "02/10/2026'da",
                "forced.date",
                "iki Ekim iki bin yirmi altıda",
            ),
            // A day the calendar lacks is still read by the shape of a date.
            (
                "31/04/2026",
                "forced.date",
                "otuz bir Nisan iki bin yirmi altı",
            ),
            ("2026-02-30", "forced.date", "otuz Şubat iki bin yirmi altı"),
            (
                "31.04.2026'da",
                "forced.date",
                "otuz bir Nisan iki bin yirmi altıda",
            ),
            (
                "13/13/2026",
                "forced.literal",
                "on üç slaş on üç slaş iki bin yirmi altı",
            ),
            // A minute below ten keeps its zero.
            ("09:05", "forced.time", "dokuz sıfır beş"),
            ("00:30", "forced.time", "sıfır otuz"),
            // Two clocks or two dates around a dash, said with the dash.
            ("09:00-17:30", "forced.range", "dokuz tire on yedi otuz"),
            ("09:00 - 17:30", "forced.range", "dokuz tire on yedi otuz"),
            ("9.30–10.30", "forced.range", "dokuz otuz tire on otuz"),
            ("09:00-17:30'a", "forced.range", "dokuz tire on yedi otuza"),
            (
                "01.02.2026-05.02.2026",
                "forced.range",
                "bir Şubat iki bin yirmi altı tire beş Şubat iki bin yirmi altı",
            ),
            (
                "2026-01-01-2026-01-05",
                "forced.range",
                "bir Ocak iki bin yirmi altı tire beş Ocak iki bin yirmi altı",
            ),
        ] {
            assert_eq!(forced(text), (rule_id, spoken.to_owned()), "{text}");
        }
    }

    #[test]
    fn a_clock_with_seconds_follows_saat_or_two_hour_digits_and_else_is_a_duration() {
        for (before, text, spoken) in [
            ("Saat ", "3:45:30'da", "üç kırk beş otuzda"),
            ("", "14:30:15", "on dört otuz on beş"),
            ("", "14:30:00'da", "on dört otuzda"),
            ("rekor ", "2:00:35", "iki saat otuz beş saniye"),
            ("", "1:23:45", "bir saat yirmi üç dakika kırk beş saniye"),
            ("", "0:45:30", "kırk beş dakika otuz saniye"),
            ("", "2:00:35'lik", "iki saat otuz beş saniyelik"),
            // Zero seconds are a clock's, which does not say them.
            ("Toplantı ", "9:30:00'da", "dokuz otuzda"),
        ] {
            let reading = read(text, before, "").unwrap();
            assert_eq!(
                (reading.rule_id, render(&reading.value, Style::Spoken).2),
                ("forced.time", spoken.to_owned()),
                "{before}{text}"
            );
        }
        // After `saat` a dotted clock needs nothing more.
        let reading = read("20.45", "saat ", "").unwrap();
        assert_eq!(
            (reading.rule_id, render(&reading.value, Style::Spoken).2),
            ("forced.time", "yirmi kırk beş".to_owned())
        );
        // An explicit time hint reads no seconds: the guess is the only reading of them.
        assert!(readers::hint("2:00:35", HintKind::Time).is_none());
    }

    #[test]
    fn a_fraction_is_said_with_bolu_and_other_slashes_are_slas() {
        for (text, spoken) in [
            ("1/2", "bir bölü iki"),
            ("3/4", "üç bölü dört"),
            ("9/10", "dokuz bölü on"),
            ("1/1000", "bir bölü bin"),
            // A suffix follows the denominator, after which it is written.
            ("2/3'ü", "iki bölü üçü"),
            ("1/2'si", "bir bölü ikisi"),
            ("3/4'lük", "üç bölü dörtlük"),
            // A rating out of five, ten or a hundred.
            ("4,5/5", "dört virgül beş bölü beş"),
            ("10/10", "on bölü on"),
            ("85/100", "seksen beş bölü yüz"),
            ("0/5", "sıfır bölü beş"),
        ] {
            assert_eq!(
                forced(text),
                ("forced.fraction", spoken.to_owned()),
                "{text}"
            );
        }
        for (text, spoken) in [
            ("7/24", "yedi slaş yirmi dört"),
            ("24/7", "yirmi dört slaş yedi"),
            ("60/65", "altmış slaş altmış beş"),
            ("1/1", "bir slaş bir"),
            ("1/2000", "bir slaş iki bin"),
        ] {
            assert_eq!(
                forced(text),
                ("forced.literal", spoken.to_owned()),
                "{text}"
            );
        }
        // After an address word, a building and a flat.
        let reading = read("3/5", "No: ", "").unwrap();
        assert_eq!(
            (reading.rule_id, render(&reading.value, Style::Spoken).2),
            ("forced.literal", "üç slaş beş".to_owned())
        );
    }

    #[test]
    fn a_race_time_a_numbered_street_and_a_card_number_are_read_as_people_say_them() {
        for (text, spoken) in [
            (
                "1:23.456",
                "bir dakika yirmi üç virgül dört yüz elli altı saniye",
            ),
            (
                "1:23.456'lık",
                "bir dakika yirmi üç virgül dört yüz elli altı saniyelik",
            ),
            ("0:59.9", "elli dokuz virgül dokuz saniye"),
            ("1:02:03.5", "bir saat iki dakika üç virgül beş saniye"),
        ] {
            assert_eq!(forced(text), ("forced.time", spoken.to_owned()), "{text}");
        }
        assert_eq!(
            forced_before("1234.", " Sok. No: 5"),
            ("forced.ordinal", "bin iki yüz otuz dördüncü".to_owned())
        );
        assert_eq!(
            forced("4111 1111 1111 1111").0,
            "forced.digits",
            "a card number is digits"
        );
        let card = "Kart 4111 1111 1111 1111";
        assert_eq!(card_group(card, 10, 14).as_deref(), Some("bir bir bir bir"));
        assert_eq!(card_group("2023 2024 2025 yılları", 5, 9), None);
        // A unit after a number another span read, and a rank.
        let unit = stray("mm", "8,25 ", ".").map(|value| render(&value, Style::Spoken).2);
        assert_eq!(unit.as_deref(), Some("milimetre"));
        assert!(stray("mm", "Ahmet ", "").is_none());
    }

    #[test]
    fn compass_points_joined_by_a_dash_are_a_wind() {
        for (text, spoken) in [
            ("K-KD", "kuzey kuzeydoğu"),
            ("GB-B", "güneybatı batı"),
            ("KD-D", "kuzeydoğu doğu"),
        ] {
            assert_eq!(
                forced(text),
                ("forced.literal", spoken.to_owned()),
                "{text}"
            );
        }
        // With one letter on each side it may be anything.
        assert_eq!(forced("K-D"), ("forced.literal", "Ke tire De".to_owned()));
    }

    #[test]
    fn a_reading_counts_the_mark_after_its_span_that_it_says() {
        for (text, following, said_after, spoken) in [
            ("6.8", "\" ekran", 1, "altı virgül sekiz inç"),
            ("6,8", "″", 3, "altı virgül sekiz inç"),
            (
                "41°00'49",
                "\"K",
                1,
                "kırk bir derece sıfır dakika kırk dokuz saniye",
            ),
            ("90+4", ". dakikada", 1, "doksan artı dördüncü"),
            // Not said: a period that may close a sentence, a quote after a whole number.
            ("90+4", ". Sonra", 0, "doksan artı dört"),
            ("55", "\" ekran", 0, "elli beş"),
        ] {
            let reading = read(text, "", following).unwrap();
            assert_eq!(
                (reading.said_after, render(&reading.value, Style::Spoken).2),
                (said_after, spoken.to_owned()),
                "{text}{following}"
            );
        }
    }

    #[test]
    fn a_span_too_long_for_a_range_is_not_searched_for_one() {
        // The longest ranges the readers accept are well inside the bound.
        let longest = "-999.999.999.999.999.999,999999999–-999.999.999.999.999.999,999999999 KİŞİ";
        assert!(longest.len() <= LONGEST_RANGE);
        assert_eq!(forced(longest).0, "forced.range");
        let dates = "01.02.2026 – 05.02.2026'da";
        assert!(dates.len() <= LONGEST_RANGE);
        assert_eq!(forced(dates).0, "forced.range");
        for dashed in ["1-".repeat(60) + "1", "09:00-".repeat(40) + "17:30"] {
            assert!(dashed.len() > LONGEST_RANGE);
            assert_eq!(forced(&dashed).0, "forced.literal");
        }
    }

    #[test]
    fn a_dash_between_numbers_is_said_and_a_ratio_is_no_clock() {
        for (text, rule_id, spoken) in [
            // Tire is right for a score and for a range alike.
            ("3-1", "forced.range", "üç tire bir"),
            ("2-0", "forced.range", "iki tire sıfır"),
            ("1-3", "forced.range", "bir tire üç"),
            ("10-3", "forced.range", "on tire üç"),
            ("1:3", "forced.ratio", "bire üç"),
            ("16:9", "forced.ratio", "on altıya dokuz"),
            ("2:1", "forced.ratio", "ikiye bir"),
            ("3:16", "forced.time", "üç on altı"),
            ("1:2:3", "forced.literal", "bir iki nokta iki iki nokta üç"),
        ] {
            assert_eq!(forced(text), (rule_id, spoken.to_owned()), "{text}");
        }
    }

    #[test]
    fn a_suffix_no_guess_reads_is_split_off_and_attached_to_the_reading() {
        for (text, rule_id, spoken) in [
            // Harmonized again to the reading's last word.
            ("09:30'de", "forced.time", "dokuz otuzda"),
            ("09:30'dan", "forced.time", "dokuz otuzdan"),
            ("11:00'e", "forced.time", "on bire"),
            (
                "14.03.2026'ya",
                "forced.date",
                "on dört Mart iki bin yirmi altıya",
            ),
            (
                "01.02.2026'dan",
                "forced.date",
                "bir Şubat iki bin yirmi altıdan",
            ),
            (
                "2026-03-14'te",
                "forced.date",
                "on dört Mart iki bin yirmi altıda",
            ),
            ("1.234'de", "forced.cardinal", "bin iki yüz otuz dörtte"),
            ("XIV'ün", "forced.roman", "on dördün"),
            // Letters that are no known inflection are attached as written.
            ("09:30'daki", "forced.time", "dokuz otuzdaki"),
            ("1.990'lar", "forced.cardinal", "bin dokuz yüz doksanlar"),
            // A reading without a tail word takes the suffix as written.
            ("10-15'e", "forced.range", "on tire on beşe"),
            ("3-1'lik", "forced.range", "üç tire birlik"),
            // No guess reads the rest either: the whole span is literal.
            (
                "24:01'de",
                "forced.literal",
                "yirmi dört iki nokta sıfır birde",
            ),
            ("5'6", "forced.literal", "beş altı"),
        ] {
            assert_eq!(forced(text), (rule_id, spoken.to_owned()), "{text}");
        }
    }

    #[test]
    fn roman_is_guessed_only_for_shapes_that_are_not_ordinary_text() {
        for (text, spoken) in [
            ("II", "iki"),
            ("XIV", "on dört"),
            ("I.", "birinci"),
            ("X.", "onuncu"),
            ("IV'üncü", "dördüncü"),
            ("II.'nin", "ikincinin"),
            ("IV.'nın", "dördüncünün"),
            ("X'uncu", "onuncu"),
            ("XLII", "kırk iki"),
            ("MMXXIV", "iki bin yirmi dört"),
            ("MMXXIV'te", "iki bin yirmi dörtte"),
        ] {
            assert_eq!(
                forced_before(text, " yüzyıl"),
                ("forced.roman", spoken.to_owned()),
                "{text}"
            );
        }
        // The period of an ordinal at the end of the text, or of its line, also closes it.
        for (text, following, spoken) in [
            ("II.", "", "ikinci."),
            ("XIV.", "\nSonra", "on dördüncü."),
            ("II.", " Dünya", "ikinci"),
            ("II.'nin", "", "ikincinin"),
        ] {
            assert_eq!(
                forced_before(text, following),
                ("forced.roman", spoken.to_owned()),
                "{text}{following:?}"
            );
        }
        // Not guessed: an abbreviation the lexicon knows is said as such, other consonants
        // by name, and Roman numeral letters stay as written.
        for (text, spoken) in [
            ("X", "İks"),
            ("V", "Ve"),
            ("I", "I"),
            ("C", "Ce"),
            ("X'te", "İks'te"),
            ("M.", "Me."),
            ("XL", "İks Le"),
            ("CD", "si di"),
            ("CV", "si vi"),
            ("CLI", "CLI"),
            ("CD'yi", "si diyi"),
            ("IIII", "IIII"),
        ] {
            assert_eq!(
                forced(text),
                ("forced.literal", spoken.to_owned()),
                "{text}"
            );
        }
    }

    #[test]
    fn a_period_is_an_ordinal_mark_unless_it_may_close_a_sentence() {
        for (text, following, spoken) in [
            ("21.", " yüzyılda çok şey değişti.", "yirmi birinci"),
            ("3.", " kez arıyorum", "üçüncü"),
            ("1.'nın", "", "birincinin"),
            ("4.'ya", " kadar", "dördüncüye"),
            ("2.'si", "", "ikincisi"),
            // A capital on the same line after a number below 1000.
            ("1.", " Dünya Savaşı", "birinci"),
            ("2.", "\u{a0}Abdülhamit", "ikinci"),
            ("999.", "\tYıl", "dokuz yüz doksan dokuzuncu"),
            // A lowercase word after any whitespace and after any number.
            ("16.", "\nyüzyılda", "on altıncı"),
            ("1923.", " yılında", "bin dokuz yüz yirmi üçüncü"),
        ] {
            assert_eq!(
                forced_before(text, following),
                ("forced.ordinal", spoken.to_owned()),
                "{text}{following}"
            );
        }
        for (text, following, spoken) in [
            ("25.", "", "yirmi beş."),
            ("25.", "sonra", "yirmi beş."),
            ("25.", "\n", "yirmi beş."),
            ("25.", "\nSonra gittik.", "yirmi beş."),
            ("25.", " (Sonra)", "yirmi beş."),
            ("1923.", " Sonra büyüdü.", "bin dokuz yüz yirmi üç."),
            ("1000.", " Kişi", "bin."),
        ] {
            assert_eq!(
                forced_before(text, following),
                ("forced.literal", spoken.to_owned()),
                "{text}{following}"
            );
        }
    }

    #[test]
    fn an_address_is_read_under_its_suffix_but_a_bare_domain_is_never_guessed() {
        for (text, spoken) in [
            ("info@ornek.com'a", "info et ornek nokta koma"),
            (
                "www.ornek.com'dan",
                "çift ve çift ve çift ve nokta ornek nokta komdan",
            ),
        ] {
            assert_eq!(
                forced(text),
                ("forced.electronic", spoken.to_owned()),
                "{text}"
            );
        }
        // Its `com` is still said as people say it.
        for (text, spoken) in [
            ("ornek.com", "ornek nokta kom"),
            ("ornek.com'a", "ornek nokta kom'a"),
            ("şirket.com.tr", "şirket nokta kom nokta te re"),
        ] {
            assert_eq!(
                forced(text),
                ("forced.literal", spoken.to_owned()),
                "{text}"
            );
        }
    }

    #[test]
    fn an_iban_with_a_failed_checksum_is_still_read_as_an_iban() {
        let spoken = "te re bir iki, sıfır sıfır sıfır altı, bir sıfır sıfır beş, bir dokuz yedi sekiz, altı dört beş yedi, sekiz dört bir üç, iki altı";
        for text in [
            "TR12 0006 1005 1978 6457 8413 26",
            "TR120006100519786457841326",
        ] {
            assert_eq!(forced(text), ("forced.iban", spoken.to_owned()), "{text}");
        }
        // Not the shape of one: the capitals are spelled, digit runs are read.
        assert_eq!(
            forced("TR12 0006 1005"),
            (
                "forced.literal",
                "Te Re on iki sıfır sıfır sıfır altı bin beş".to_owned()
            )
        );
    }

    #[test]
    fn marks_and_shapes_of_ordinary_text_are_not_misread() {
        // A face, an arrow, a decoration, a Markdown mark and a lone symbol between words say
        // nothing.
        for (word, before, following) in [
            (":D", "hahaha ", " göz"),
            ("<3", "seni seviyorum ", ""),
            ("->", "Adım 1 ", " Adım 2"),
            ("***", "", ""),
            ("#", "", " Başlık"),
            ("##", "metin\n", " Alt başlık"),
            ("*", "", " madde"),
            (">", "", " alıntı"),
            (">", "Kategori ", " Telefon"),
            ("*", "Tuş ", ""),
        ] {
            let value = stray(word, before, following).unwrap();
            assert_eq!(
                render(&value, Style::Spoken).2,
                "",
                "{before}{word}{following}"
            );
        }
        // Between two numbers or letters, or before a number it compares, it is said.
        for (word, before, following, spoken) in [
            ("*", "3 ", " 4", "çarpı"),
            (">", "x ", " 5", "büyüktür"),
            ("<", "", " 1 dk", "küçüktür"),
        ] {
            let value = stray(word, before, following).unwrap();
            assert_eq!(
                render(&value, Style::Spoken).2,
                spoken,
                "{before}{word}{following}"
            );
        }
        // A list of years or prices is no card number: a card's check digit holds.
        assert_ne!(forced("2020 2021 2022 2023").0, "forced.digits");
        assert_ne!(forced("1250 1500 1750 2000").0, "forced.digits");
        assert_eq!(forced("5555 5555 5555 4444").0, "forced.digits");
        assert_eq!(card_group("2020 2021 2022 2023", 0, 4), None);
        // An internet address is no phone number.
        assert_eq!(
            forced("255.255.255.0").1,
            "iki yüz elli beş nokta iki yüz elli beş nokta iki yüz elli beş nokta sıfır"
        );
        // `V.` before a name is an initial, while `I.` and `X.` there are numbers.
        assert_eq!(
            forced_before("V.", " Öztürk"),
            ("forced.literal", "Ve.".to_owned())
        );
        assert_eq!(
            forced_before("I.", " Dünya"),
            ("forced.roman", "birinci".to_owned())
        );
        // Stars after a count, before what they rate; emphasis around a word says nothing.
        assert_eq!(
            forced_before("5*", " otel"),
            ("forced.literal", "beş yıldızlı".to_owned())
        );
        assert_eq!(
            forced("**Önemli:**"),
            ("forced.literal", "Önemli:".to_owned())
        );
    }

    #[test]
    fn every_span_has_a_forced_reading() {
        for text in ["", " ", "①", "'", "'te", "a", "\u{301}", "1e3", "3 + 4 = 7"] {
            assert!(read(text, "", "").is_some(), "{text:?}");
        }
    }

    #[test]
    fn a_stray_word_is_spoken_only_when_its_reading_differs() {
        for (word, spoken) in [
            ("+", "artı"),
            ("C++", "Ce artı artı"),
            ("R&D", "Re ve De"),
            ("a>b", "a büyüktür be"),
            ("B", "Be"),
            ("D:", "De:"),
            ("B'ye", "Be'ye"),
            ("TK", "Te Ke"),
            ("TK'YI", "Te Ke'yı"),
            ("TL'li", "liralı"),
            ("ABC", "A Be Ce"),
            // Capitals no reader claims are said in lowercase, and lowercase consonants by
            // name.
            ("ASK", "ask"),
            ("SAAT", "saat"),
            ("vs", "ve se"),
        ] {
            let value = stray(word, "", "").unwrap();
            assert_eq!(
                render(&value, Style::Spoken),
                (SegmentKind::Literal, "forced.spoken", spoken.to_owned()),
                "{word}"
            );
        }
        for word in [
            "A", "o", "ve", "e-posta", "Ali'nin", "q\u{308}", "YouTube", "km", "hmm", "Pff",
        ] {
            assert!(stray(word, "", "").is_none(), "{word}");
        }
    }
}
