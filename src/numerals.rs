use crate::morphology::{Spoken, Style, Word};

pub(crate) const MAGNITUDE_LIMIT: u64 = 1_000_000_000_000_000_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Sign {
    None,
    Plus,
    Minus,
}

#[derive(Clone, Debug)]
pub(crate) struct Amount {
    sign: Sign,
    major: u64,
    minor: u8,
}

impl Amount {
    pub(crate) fn parse(text: &str) -> Option<Self> {
        let number = Number::parse(text)?;
        Some(Self {
            sign: number.sign(),
            major: number.integer(),
            minor: number.minor()?,
        })
    }
    pub(crate) fn sign(&self) -> Sign {
        self.sign
    }
    pub(crate) fn major(&self) -> u64 {
        self.major
    }
    pub(crate) fn minor(&self) -> u8 {
        self.minor
    }
}

pub(crate) fn amount(amount: &Amount, major: &str, major_tail: Word, minor_tail: Word) -> Spoken {
    let text = format!(
        "{}{} {}",
        sign_text(amount.sign()),
        cardinal(amount.major()).into_text(),
        major
    );
    let mut spoken = Spoken::lexical(&text, major_tail);
    if amount.minor() != 0 {
        spoken.append_literal(&format!(
            " {}",
            cardinal(u64::from(amount.minor())).into_text()
        ));
        spoken.append_word(minor_tail);
    }
    spoken
}

#[derive(Clone, Debug)]
pub(crate) struct Number {
    sign: Sign,
    integer: u64,
    fraction: String,
    grouped: bool,
}

impl Number {
    pub(crate) fn parse(text: &str) -> Option<Self> {
        Self::parse_with_padding(text, false)
    }
    pub(crate) fn parse_cardinal_hint(text: &str) -> Option<Self> {
        Self::parse_with_padding(text, true)
    }
    fn parse_with_padding(text: &str, allow_padding: bool) -> Option<Self> {
        let (sign, body) = match text.as_bytes().first() {
            Some(b'-') => (Sign::Minus, &text[1..]),
            Some(b'+') => (Sign::Plus, &text[1..]),
            _ => (Sign::None, text),
        };
        let mut parts = body.split(',');
        let whole = parts.next()?;
        let fraction = parts.next();
        if parts.next().is_some()
            || fraction.is_some_and(|f| {
                !(1..=9).contains(&f.len()) || !f.bytes().all(|b| b.is_ascii_digit())
            })
        {
            return None;
        }
        let grouped = whole.contains('.');
        let mut integer = 0_u64;
        for (index, group) in whole.split('.').enumerate() {
            if group.is_empty()
                || !group.bytes().all(|b| b.is_ascii_digit())
                || (!allow_padding && index == 0 && group.len() > 1 && group.starts_with('0'))
                || (grouped && index == 0 && group.len() > 3)
                || (index > 0 && group.len() != 3)
            {
                return None;
            }
            for digit in group.bytes() {
                integer = integer
                    .checked_mul(10)?
                    .checked_add(u64::from(digit - b'0'))?;
                if integer >= MAGNITUDE_LIMIT {
                    return None;
                }
            }
        }
        Some(Self {
            sign,
            integer,
            fraction: fraction.unwrap_or("").to_owned(),
            grouped,
        })
    }
    pub(crate) fn integer(&self) -> u64 {
        self.integer
    }
    pub(crate) fn fraction(&self) -> &str {
        &self.fraction
    }
    pub(crate) fn sign(&self) -> Sign {
        self.sign
    }
    pub(crate) fn grouped(&self) -> bool {
        self.grouped
    }
    pub(crate) fn minor(&self) -> Option<u8> {
        match self.fraction.as_bytes() {
            [] => Some(0),
            [a] => Some((a - b'0') * 10),
            [a, b] => Some((a - b'0') * 10 + b - b'0'),
            _ => None,
        }
    }
}

use crate::morphology::{Harmony, WordEnd};

const DIGITS: [Word; 10] = [
    Word::new("sıfır", Harmony::BackFlat, WordEnd::Voiced),
    Word::new("bir", Harmony::FrontFlat, WordEnd::Voiced),
    Word::new("iki", Harmony::FrontFlat, WordEnd::Vowel),
    Word::new("üç", Harmony::FrontRound, WordEnd::Voiceless),
    Word::new("dört", Harmony::FrontRound, WordEnd::Softens),
    Word::new("beş", Harmony::FrontFlat, WordEnd::Voiceless),
    Word::new("altı", Harmony::BackFlat, WordEnd::Vowel),
    Word::new("yedi", Harmony::FrontFlat, WordEnd::Vowel),
    Word::new("sekiz", Harmony::FrontFlat, WordEnd::Voiced),
    Word::new("dokuz", Harmony::BackRound, WordEnd::Voiced),
];
const TENS: [Word; 9] = [
    Word::new("on", Harmony::BackRound, WordEnd::Voiced),
    Word::new("yirmi", Harmony::FrontFlat, WordEnd::Vowel),
    Word::new("otuz", Harmony::BackRound, WordEnd::Voiced),
    Word::new("kırk", Harmony::BackFlat, WordEnd::Voiceless),
    Word::new("elli", Harmony::FrontFlat, WordEnd::Vowel),
    Word::new("altmış", Harmony::BackFlat, WordEnd::Voiceless),
    Word::new("yetmiş", Harmony::FrontFlat, WordEnd::Voiceless),
    Word::new("seksen", Harmony::FrontFlat, WordEnd::Voiced),
    Word::new("doksan", Harmony::BackFlat, WordEnd::Voiced),
];
const HUNDRED: Word = Word::new("yüz", Harmony::FrontRound, WordEnd::Voiced);
const SCALES: [Word; 5] = [
    Word::new("bin", Harmony::FrontFlat, WordEnd::Voiced),
    Word::new("milyon", Harmony::BackRound, WordEnd::Voiced),
    Word::new("milyar", Harmony::BackFlat, WordEnd::Voiced),
    Word::new("trilyon", Harmony::BackRound, WordEnd::Voiced),
    Word::new("katrilyon", Harmony::BackRound, WordEnd::Voiced),
];

fn push_group(value: u16, words: &mut Vec<Word>) {
    if value >= 100 {
        if value / 100 > 1 {
            words.push(DIGITS[usize::from(value / 100)]);
        }
        words.push(HUNDRED);
    }
    let rest = value % 100;
    if rest >= 10 {
        words.push(TENS[usize::from(rest / 10 - 1)]);
    }
    if !rest.is_multiple_of(10) {
        words.push(DIGITS[usize::from(rest % 10)]);
    }
}

/// Only called with validated domain magnitudes or bounded calendar/clock fields.
pub(crate) fn cardinal(value: u64) -> Spoken {
    if value == 0 {
        return Spoken::from_words(&[DIGITS[0]]);
    }
    let mut groups = [0_u16; 6];
    let mut rest = value;
    for group in &mut groups {
        *group = (rest % 1000) as u16;
        rest /= 1000;
    }
    let mut words = Vec::with_capacity(24);
    for index in (0..groups.len()).rev() {
        let group = groups[index];
        if group == 0 {
            continue;
        }
        if index != 1 || group != 1 {
            push_group(group, &mut words);
        }
        if index > 0 {
            words.push(SCALES[index - 1]);
        }
    }
    Spoken::from_words(&words)
}

pub(crate) fn sign_text(sign: Sign) -> &'static str {
    match sign {
        Sign::None => "",
        Sign::Plus => "artı ",
        Sign::Minus => "eksi ",
    }
}

pub(crate) fn number(number: &Number) -> Spoken {
    number_as(number, Style::Exact)
}

/// Exact style reads a fraction digit by digit: `4,25` is dört virgül iki beş. Spoken style
/// reads it as a number after its leading zeros: dört virgül yirmi beş. More than three digits
/// after the zeros are read one by one in both.
pub(crate) fn number_as(number: &Number, style: Style) -> Spoken {
    let mut spoken = cardinal(number.integer);
    spoken.prefix(sign_text(number.sign));
    if number.fraction.is_empty() {
        return spoken;
    }
    spoken.append_literal(" virgül");
    let zeros = number.fraction.bytes().take_while(|b| *b == b'0').count();
    let (padding, rest) = number.fraction.split_at(zeros);
    let as_number = style == Style::Spoken && (1..=3).contains(&rest.len());
    let one_by_one = if as_number { padding } else { &number.fraction };
    for digit in one_by_one.bytes() {
        spoken.append_word(DIGITS[usize::from(digit - b'0')]);
    }
    if !as_number {
        return spoken;
    }
    let value = rest
        .bytes()
        .fold(0, |value, digit| value * 10 + u64::from(digit - b'0'));
    let mut tail = cardinal(value);
    tail.prefix(&format!("{} ", spoken.into_text()));
    tail
}

pub(crate) fn digits(text: &str) -> String {
    let mut result = String::new();
    for ch in text.chars() {
        let word = match ch {
            '+' => "artı",
            '0'..='9' => DIGITS[(ch as u8 - b'0') as usize].text,
            _ => continue,
        };
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(word);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_parser_retains_sign_scale_and_grouping() {
        let number = Number::parse("-1.234,050").unwrap();
        assert_eq!(number.sign(), Sign::Minus);
        assert_eq!(number.integer(), 1234);
        assert_eq!(number.fraction(), "050");
        assert!(number.grouped());
        assert_eq!(number.minor(), None);
        assert_eq!(Number::parse("-0,5").unwrap().minor(), Some(50));
        assert_eq!(Number::parse("+0,00").unwrap().sign(), Sign::Plus);
        assert_eq!(Number::parse_cardinal_hint("00042").unwrap().integer(), 42);
        assert_eq!(
            Number::parse_cardinal_hint("001.234").unwrap().integer(),
            1234
        );
        assert!(Number::parse_cardinal_hint("0001.234").is_none());
        assert_eq!(
            Number::parse("999999999999999999").unwrap().integer(),
            MAGNITUDE_LIMIT - 1
        );
    }

    #[test]
    fn a_fraction_is_read_digit_by_digit_or_as_a_number() {
        for (written, exact, spoken) in [
            ("4,25", "dört virgül iki beş", "dört virgül yirmi beş"),
            (
                "12,05",
                "on iki virgül sıfır beş",
                "on iki virgül sıfır beş",
            ),
            ("0,18", "sıfır virgül bir sekiz", "sıfır virgül on sekiz"),
            ("2,50", "iki virgül beş sıfır", "iki virgül elli"),
            (
                "1,250",
                "bir virgül iki beş sıfır",
                "bir virgül iki yüz elli",
            ),
            (
                "0,0025",
                "sıfır virgül sıfır sıfır iki beş",
                "sıfır virgül sıfır sıfır yirmi beş",
            ),
            ("3,00", "üç virgül sıfır sıfır", "üç virgül sıfır sıfır"),
            // Longer than anyone says as one number: still one by one.
            (
                "3,1415",
                "üç virgül bir dört bir beş",
                "üç virgül bir dört bir beş",
            ),
            ("-0,5", "eksi sıfır virgül beş", "eksi sıfır virgül beş"),
            ("7", "yedi", "yedi"),
        ] {
            let parsed = Number::parse(written).unwrap();
            assert_eq!(number_as(&parsed, Style::Exact).into_text(), exact);
            assert_eq!(number(&parsed).into_text(), exact);
            assert_eq!(number_as(&parsed, Style::Spoken).into_text(), spoken);
        }
    }

    #[test]
    fn exact_parser_rejects_invalid_whole_expressions() {
        for input in [
            "",
            "-",
            "+",
            "00",
            "01",
            "001.000",
            "1.23",
            "12.3456",
            "1..234",
            "1,2345678900",
            "1,",
            "1,2,3",
            "1e3",
            "1000000000000000000",
            "١٢٣",
        ] {
            assert!(Number::parse(input).is_none(), "{input}");
        }
    }
}
