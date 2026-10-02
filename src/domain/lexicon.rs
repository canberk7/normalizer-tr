use crate::morphology::{Harmony, Word, WordEnd};

use Harmony::{BackFlat, BackRound, FrontFlat, FrontRound};
use WordEnd::{Possessive, SoftensP, Voiced, Voiceless, Vowel};

/// Turkish casing is confined to explicit contextual lookup keys, never source rewriting.
pub(crate) fn lookup_key(text: &str) -> String {
    text.chars()
        .flat_map(|ch| match ch {
            'I' => 'ı'.to_lowercase(),
            'İ' => 'i'.to_lowercase(),
            _ => ch.to_lowercase(),
        })
        .collect()
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Lexeme {
    pub(crate) output: &'static str,
    pub(crate) source: Word,
    pub(crate) target: Word,
}

impl Lexeme {
    const fn same(output: &'static str, harmony: Harmony, end: WordEnd) -> Self {
        let word = Word::new(output, harmony, end);
        Self {
            output,
            source: word,
            target: word,
        }
    }
    const fn distinct(output: &'static str, source: Word, target: Word) -> Self {
        Self {
            output,
            source,
            target,
        }
    }
}

const UNITS: &[(&str, Lexeme)] = &[
    ("kg", Lexeme::same("kilogram", BackFlat, Voiced)),
    ("g", Lexeme::same("gram", BackFlat, Voiced)),
    ("gr", Lexeme::same("gram", BackFlat, Voiced)),
    ("mg", Lexeme::same("miligram", BackFlat, Voiced)),
    ("µg", Lexeme::same("mikrogram", BackFlat, Voiced)),
    ("μg", Lexeme::same("mikrogram", BackFlat, Voiced)),
    ("km", Lexeme::same("kilometre", FrontFlat, Vowel)),
    ("m", Lexeme::same("metre", FrontFlat, Vowel)),
    ("cm", Lexeme::same("santimetre", FrontFlat, Vowel)),
    ("mm", Lexeme::same("milimetre", FrontFlat, Vowel)),
    ("L", Lexeme::same("litre", FrontFlat, Vowel)),
    ("lt", Lexeme::same("litre", FrontFlat, Vowel)),
    ("mL", Lexeme::same("mililitre", FrontFlat, Vowel)),
    ("ml", Lexeme::same("mililitre", FrontFlat, Vowel)),
    ("dk", Lexeme::same("dakika", BackFlat, Vowel)),
    ("sn", Lexeme::same("saniye", FrontFlat, Vowel)),
    ("sa", Lexeme::same("saat", FrontFlat, Voiceless)),
    ("m²", Lexeme::same("metrekare", FrontFlat, Vowel)),
    ("cm²", Lexeme::same("santimetrekare", FrontFlat, Vowel)),
    ("km²", Lexeme::same("kilometrekare", FrontFlat, Vowel)),
    ("m³", Lexeme::same("metreküp", FrontRound, SoftensP)),
];

pub(crate) fn unit(symbol: &str) -> Option<Lexeme> {
    UNITS
        .iter()
        .find(|(key, _)| *key == symbol)
        .map(|(_, value)| *value)
}

/// Detection of an unsupported spelling is not acceptance or source case rewriting.
pub(crate) fn unit_marker(symbol: &str) -> bool {
    UNITS
        .iter()
        .any(|(key, _)| key.eq_ignore_ascii_case(symbol))
}

pub(crate) fn rate(symbol: &str) -> Option<(Lexeme, Lexeme)> {
    match symbol {
        "km/sa" | "km/h" => Some((unit("sa")?, unit("km")?)),
        "m/s" => Some((unit("sn")?, unit("m")?)),
        _ => None,
    }
}

pub(crate) fn abbreviation(symbol: &str) -> Option<Lexeme> {
    Some(match symbol {
        "Dr." => Lexeme::same("doktor", BackRound, Voiced),
        "Prof." => Lexeme::same("profesör", FrontRound, Voiced),
        "vb." => Lexeme::distinct(
            "ve benzeri",
            Word::new("be", FrontFlat, Vowel),
            Word::new("benzeri", FrontFlat, Vowel),
        ),
        "TBMM" => Lexeme::distinct(
            "te be me me",
            Word::new("me", FrontFlat, Vowel),
            Word::new("me", FrontFlat, Vowel),
        ),
        "PTT" => Lexeme::distinct(
            "pe te te",
            Word::new("te", FrontFlat, Vowel),
            Word::new("te", FrontFlat, Vowel),
        ),
        "NATO" => Lexeme::same("nato", BackRound, Vowel),
        "IBAN" => Lexeme::same("iban", BackFlat, Voiced),
        "KDV" => Lexeme::distinct(
            "katma değer vergisi",
            Word::new("ve", FrontFlat, Vowel),
            Word::new("vergisi", FrontFlat, Possessive),
        ),
        _ => return None,
    })
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Currency {
    Try,
    Usd,
    Eur,
    Gbp,
}

impl Currency {
    pub(crate) fn parse(label: &str) -> Option<Self> {
        match label {
            "TL" | "TRY" | "₺" => Some(Self::Try),
            "USD" | "$" => Some(Self::Usd),
            "EUR" | "€" => Some(Self::Eur),
            "GBP" | "£" => Some(Self::Gbp),
            _ => None,
        }
    }
    pub(crate) fn lexeme(self, label: &str) -> Lexeme {
        match self {
            Self::Try => Lexeme::distinct(
                "Türk lirası",
                if label == "TRY" {
                    Word::new("ye", FrontFlat, Vowel)
                } else if label == "₺" {
                    Word::new("lira", BackFlat, Vowel)
                } else {
                    Word::new("le", FrontFlat, Vowel)
                },
                Word::new("lirası", BackFlat, Possessive),
            ),
            Self::Usd => Lexeme::distinct(
                "dolar",
                if label == "USD" {
                    Word::new("de", FrontFlat, Vowel)
                } else {
                    Word::new("dolar", BackFlat, Voiced)
                },
                Word::new("dolar", BackFlat, Voiced),
            ),
            Self::Eur => Lexeme::distinct(
                "avro",
                if label == "EUR" {
                    Word::new("re", FrontFlat, Vowel)
                } else {
                    Word::new("avro", BackRound, Vowel)
                },
                Word::new("avro", BackRound, Vowel),
            ),
            Self::Gbp => Lexeme::distinct(
                "sterlin",
                if label == "GBP" {
                    Word::new("pe", FrontFlat, Vowel)
                } else {
                    Word::new("sterlin", FrontFlat, Voiced)
                },
                Word::new("sterlin", FrontFlat, Voiced),
            ),
        }
    }
    pub(crate) fn minor(self) -> Lexeme {
        match self {
            Self::Try => Lexeme::same("kuruş", BackRound, Voiceless),
            Self::Usd | Self::Eur => Lexeme::same("sent", FrontFlat, Voiceless),
            Self::Gbp => Lexeme::same("peni", FrontFlat, Vowel),
        }
    }
}
