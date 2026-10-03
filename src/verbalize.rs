use crate::{
    SegmentKind,
    domain::{
        electronic,
        literal::{Literal, Part},
    },
    model::{Clock, Date, Value},
    morphology::{Inflection, Spoken, Style, Suffix},
    numerals,
};

const MONTHS: [&str; 12] = [
    "Ocak", "Şubat", "Mart", "Nisan", "Mayıs", "Haziran", "Temmuz", "Ağustos", "Eylül", "Ekim",
    "Kasım", "Aralık",
];

pub(crate) fn date_spoken(date: Date) -> Spoken {
    let mut year = numerals::cardinal(u64::from(date.year()));
    let prefix = format!(
        "{} {} ",
        numerals::cardinal(u64::from(date.day())).into_text(),
        MONTHS[usize::from(date.month() - 1)]
    );
    year.prefix(&prefix);
    year
}

/// A clock with seconds, every part said, a part below ten with its zero: `2:00:35` is iki
/// sıfır sıfır otuz beş.
pub(crate) fn seconds_spoken(time: Clock, second: u8) -> Spoken {
    let pair = |value: u8| match value {
        0 => "sıfır sıfır".to_owned(),
        1..=9 => format!("sıfır {}", numerals::cardinal(u64::from(value)).into_text()),
        _ => numerals::cardinal(u64::from(value)).into_text(),
    };
    let mut spoken = numerals::cardinal(u64::from(second));
    spoken.prefix(&format!(
        "{} {} {}",
        numerals::cardinal(u64::from(time.hour())).into_text(),
        pair(time.minute()),
        if second < 10 { "sıfır " } else { "" }
    ));
    spoken
}

/// A clock's hour and minute. In the spoken style a minute below ten keeps its zero: `12:05`
/// is on iki sıfır beş.
pub(crate) fn time_spoken(time: Clock, style: Style) -> Spoken {
    let hour = numerals::cardinal(u64::from(time.hour()));
    if time.minute() == 0 {
        hour
    } else {
        let mut minute = numerals::cardinal(u64::from(time.minute()));
        let zero = if style == Style::Spoken && time.minute() < 10 {
            "sıfır "
        } else {
            ""
        };
        minute.prefix(&format!("{} {zero}", hour.into_text()));
        minute
    }
}

/// One literal part, with its apostrophe suffixes attached to the spoken tail.
fn literal_part(part: &Part, style: Style) -> String {
    let (mut spoken, suffixes) = match part {
        Part::Number(value, suffixes) => (numerals::cardinal(*value), suffixes),
        Part::Name(word, suffixes) => (Spoken::from_words(&[*word]), suffixes),
        Part::Label(lexeme, suffixes) => {
            // A derivation is built on the everyday word in either style: `TL'lik` is liralık.
            let derived = suffixes.first().and_then(Suffix::family) == Some(Inflection::Derivation);
            let (output, target) = lexeme.said(if derived { Style::Spoken } else { style });
            (Spoken::lexical(output, target), suffixes)
        }
        Part::Letters(written) | Part::Gap(written) => return written.clone(),
    };
    for suffix in suffixes {
        spoken.attach(suffix);
    }
    spoken.into_text()
}

/// Reads a literal span part by part, with one space between neighbouring words.
pub(crate) fn literal_text(literal: &Literal, style: Style) -> String {
    let mut text = String::new();
    let mut after_word = false;
    for part in literal.parts() {
        let word = !matches!(part, Part::Gap(_));
        if word && after_word {
            text.push(' ');
        }
        text.push_str(&literal_part(part, style));
        after_word = word;
    }
    text
}

/// Spoken form of an unsuffixed reading that carries its tail word.
fn spoken(value: &Value, style: Style) -> Option<Spoken> {
    match value {
        Value::Numeric(number) | Value::Roman(number) => Some(number.render(style)),
        Value::Date(date, false) => Some(date_spoken(*date)),
        Value::Time(time, false) => Some(time_spoken(*time, style)),
        Value::Seconds(time, second) => Some(seconds_spoken(*time, *second)),
        Value::Quantity(quantity) => Some(quantity.render(style)),
        Value::Joined(_, left, joiner, right) => {
            let mut tail = spoken(right, style)?;
            tail.prefix(&format!("{}{joiner}", render(left, style).2));
            Some(tail)
        }
        _ => None,
    }
}

/// A reading with the apostrophe suffix that was split off it: harmonized to the reading's
/// tail word when it carries one, otherwise attached as written.
fn suffixed(reading: &Value, suffix: &Suffix, style: Style) -> (SegmentKind, &'static str, String) {
    let (kind, rule_id, text) = render(reading, style);
    let ordinal = kind == SegmentKind::Cardinal && suffix.family() == Some(Inflection::Ordinal);
    let text = match spoken(reading, style) {
        Some(mut spoken) => {
            spoken.attach(suffix);
            spoken.into_text()
        }
        None => text + suffix.letters(),
    };
    let kind = if ordinal { SegmentKind::Ordinal } else { kind };
    (kind, rule_id, text)
}

pub(crate) fn render(value: &Value, style: Style) -> (SegmentKind, &'static str, String) {
    match value {
        Value::Numeric(number) => (number.kind(), "number", number.render(style).into_text()),
        Value::Digits(text) => (SegmentKind::Digits, "digits.hint", numerals::digits(text)),
        Value::Date(date, locative) => {
            let mut spoken = date_spoken(*date);
            if *locative {
                spoken.inflect(Inflection::Locative);
            }
            (SegmentKind::Date, "date.gregorian", spoken.into_text())
        }
        Value::Time(time, locative) => {
            let mut spoken = time_spoken(*time, style);
            if *locative {
                spoken.inflect(Inflection::Locative);
            }
            (SegmentKind::Time, "time.digital", spoken.into_text())
        }
        Value::Percent(number, case) => {
            let mut spoken = numerals::number_as(number, style);
            if let Some(case) = case {
                spoken.inflect(*case);
            }
            spoken.prefix("yüzde ");
            (SegmentKind::Percent, "percent", spoken.into_text())
        }
        Value::Quantity(quantity) => (
            if quantity.is_money() {
                SegmentKind::Money
            } else {
                SegmentKind::Unit
            },
            "quantity",
            quantity.render(style).into_text(),
        ),
        Value::Lexical(entry, case) => {
            let (output, target) = entry.said(style);
            let mut spoken = Spoken::lexical(output, target);
            if let Some(case) = case {
                spoken.inflect(*case);
            }
            (
                SegmentKind::Abbreviation,
                "abbreviation",
                spoken.into_text(),
            )
        }
        Value::Range(range) => (SegmentKind::Range, "range.context", range.render(style)),
        Value::Telephone(phone) => (SegmentKind::Telephone, "telephone.tr", phone.render()),
        Value::Iban(iban) => (SegmentKind::Iban, "iban.tr.mod97", iban.render()),
        Value::Roman(number) => (
            SegmentKind::Roman,
            "roman.canonical",
            number.render(style).into_text(),
        ),
        Value::Electronic(address) => (
            SegmentKind::Electronic,
            "electronic.ascii",
            address.render(style),
        ),
        Value::Symbol(text) => (SegmentKind::Symbol, "symbol.prose", text.clone()),
        Value::Literal(literal) => (
            SegmentKind::Literal,
            "literal.hint",
            literal_text(literal, style),
        ),
        Value::Suffixed(reading, suffix) => suffixed(reading, suffix, style),
        Value::Joined(kind, left, joiner, right) => {
            let (_, rule_id, right) = render(right, style);
            (
                *kind,
                rule_id,
                format!("{}{joiner}{right}", render(left, style).2),
            )
        }
        Value::Stray(literal) => (
            SegmentKind::Literal,
            "forced.spoken",
            literal_text(literal, style),
        ),
        Value::Framed(open, reading, close) => {
            let (kind, rule_id, text) = render(reading, style);
            (kind, rule_id, format!("{open}{text}{close}"))
        }
        Value::Seconds(time, second) => (
            SegmentKind::Time,
            "time.digital",
            seconds_spoken(*time, *second).into_text(),
        ),
        Value::Hashtag(body) => (
            SegmentKind::Symbol,
            "symbol.prose",
            match style {
                Style::Exact => electronic::hashtag(&format!("#{body}")).unwrap_or_default(),
                Style::Spoken => electronic::spoken_hashtag(body),
            },
        ),
    }
}
