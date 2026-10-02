#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Inflection {
    Ordinal,
    Accusative,
    Dative,
    Locative,
    Ablative,
    Genitive,
    Derivation,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Harmony {
    BackFlat,
    FrontFlat,
    BackRound,
    FrontRound,
}

impl Harmony {
    fn high(self) -> &'static str {
        match self {
            Self::BackFlat => "ı",
            Self::FrontFlat => "i",
            Self::BackRound => "u",
            Self::FrontRound => "ü",
        }
    }
    fn low(self) -> &'static str {
        match self {
            Self::BackFlat | Self::BackRound => "a",
            Self::FrontFlat | Self::FrontRound => "e",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WordEnd {
    Vowel,
    Voiced,
    Voiceless,
    Softens,
    SoftensP,
    Possessive,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Word {
    pub(crate) text: &'static str,
    harmony: Harmony,
    end: WordEnd,
}

impl Word {
    pub(crate) const fn new(text: &'static str, harmony: Harmony, end: WordEnd) -> Self {
        Self { text, harmony, end }
    }
    pub(crate) fn source_suffix(self, inflection: Inflection) -> String {
        let high = self.harmony.high();
        let low = self.harmony.low();
        let vowel = matches!(self.end, WordEnd::Vowel | WordEnd::Possessive);
        let stop = match self.end {
            WordEnd::Voiceless | WordEnd::Softens | WordEnd::SoftensP => "t",
            _ => "d",
        };
        match inflection {
            Inflection::Accusative if self.end == WordEnd::Possessive => format!("n{high}"),
            Inflection::Dative if self.end == WordEnd::Possessive => format!("n{low}"),
            Inflection::Locative if self.end == WordEnd::Possessive => format!("nd{low}"),
            Inflection::Ablative if self.end == WordEnd::Possessive => format!("nd{low}n"),
            Inflection::Ordinal if vowel => format!("nc{high}"),
            Inflection::Ordinal => format!("{high}nc{high}"),
            Inflection::Accusative if vowel => format!("y{high}"),
            Inflection::Accusative => high.to_owned(),
            Inflection::Dative if vowel => format!("y{low}"),
            Inflection::Dative => low.to_owned(),
            Inflection::Locative => format!("{stop}{low}"),
            Inflection::Ablative => format!("{stop}{low}n"),
            Inflection::Genitive if vowel => format!("n{high}n"),
            Inflection::Genitive => format!("{high}n"),
            Inflection::Derivation => format!("l{high}k"),
        }
    }
}

/// Carries final spoken-word metadata; never recovers morphology by parsing text.
#[derive(Clone, Debug)]
pub(crate) struct Spoken {
    text: String,
    tail: Word,
}

impl Spoken {
    pub(crate) fn lexical(text: &str, tail: Word) -> Self {
        Self {
            text: text.to_owned(),
            tail,
        }
    }
    pub(crate) fn from_words(words: &[Word]) -> Self {
        // Callers always supply at least one validated numeral word.
        let mut spoken = Self {
            text: String::new(),
            tail: Word::new("sıfır", Harmony::BackFlat, WordEnd::Voiced),
        };
        for word in words {
            spoken.append_word(*word);
        }
        spoken
    }
    pub(crate) fn prefix(&mut self, prefix: &str) {
        self.text.insert_str(0, prefix);
    }
    pub(crate) fn append_literal(&mut self, text: &str) {
        self.text.push_str(text);
    }
    pub(crate) fn append_word(&mut self, word: Word) {
        if !self.text.is_empty() {
            self.text.push(' ');
        }
        self.text.push_str(word.text);
        self.tail = word;
    }
    pub(crate) fn source_suffix(&self, inflection: Inflection) -> String {
        self.tail.source_suffix(inflection)
    }
    pub(crate) fn inflect(&mut self, inflection: Inflection) {
        let suffix = self.source_suffix(inflection);
        let vowel_suffix = matches!(
            inflection,
            Inflection::Ordinal
                | Inflection::Accusative
                | Inflection::Dative
                | Inflection::Genitive
        );
        if self.tail.end == WordEnd::Softens && vowel_suffix {
            self.text.pop();
            self.text.push('d');
        }
        if self.tail.end == WordEnd::SoftensP && vowel_suffix {
            self.text.pop();
            self.text.push('b');
        }
        self.text.push_str(&suffix);
        match inflection {
            Inflection::Ordinal => self.tail.end = WordEnd::Vowel,
            Inflection::Derivation => self.tail.end = WordEnd::Voiceless,
            _ => {}
        }
    }
    pub(crate) fn into_text(self) -> String {
        self.text
    }
}

pub(crate) fn integer_inflection(spoken: &Spoken, suffix: &str) -> Option<Inflection> {
    [
        Inflection::Ordinal,
        Inflection::Accusative,
        Inflection::Dative,
        Inflection::Locative,
        Inflection::Ablative,
        Inflection::Genitive,
    ]
    .into_iter()
    .find(|inflection| spoken.source_suffix(*inflection) == suffix)
}

pub(crate) fn case_inflection(source: Word, suffix: &str) -> Option<Inflection> {
    [
        Inflection::Accusative,
        Inflection::Dative,
        Inflection::Locative,
        Inflection::Ablative,
        Inflection::Genitive,
    ]
    .into_iter()
    .find(|family| source.source_suffix(*family) == suffix)
}

pub(crate) fn spoken_case(spoken: &Spoken, suffix: &str) -> Option<Inflection> {
    [
        Inflection::Accusative,
        Inflection::Dative,
        Inflection::Locative,
        Inflection::Ablative,
        Inflection::Genitive,
    ]
    .into_iter()
    .find(|family| spoken.source_suffix(*family) == suffix)
}
