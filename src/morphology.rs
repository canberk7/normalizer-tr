use crate::domain::lexicon::lookup_key;

/// The register a reading is said in.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Style {
    /// The exact default: fraction digits one by one, formal currency names.
    Exact,
    /// Everyday speech, which the Forced policy uses: a fraction as a number, `TL` as lira.
    Spoken,
}

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
    /// A final `k` that becomes `ğ` before a vowel: blok, bloğu.
    SoftensK,
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
            WordEnd::Voiceless | WordEnd::Softens | WordEnd::SoftensP | WordEnd::SoftensK => "t",
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
        if vowel_suffix {
            self.soften();
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
    /// The last consonant of a word that softens before a vowel: `dört` is dördü.
    fn soften(&mut self) {
        let soft = match self.tail.end {
            WordEnd::Softens => 'd',
            WordEnd::SoftensP => 'b',
            WordEnd::SoftensK => 'ğ',
            _ => return,
        };
        self.text.pop();
        self.text.push(soft);
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

const HARMONIES: [Harmony; 4] = [
    Harmony::BackFlat,
    Harmony::FrontFlat,
    Harmony::BackRound,
    Harmony::FrontRound,
];
const WORD_ENDS: [WordEnd; 7] = [
    WordEnd::Vowel,
    WordEnd::Voiced,
    WordEnd::Voiceless,
    WordEnd::Softens,
    WordEnd::SoftensP,
    WordEnd::SoftensK,
    WordEnd::Possessive,
];

/// The inflection a written suffix spells for some word shape, whichever harmony it was given.
fn written_inflection(suffix: &str) -> Option<Inflection> {
    [
        Inflection::Ordinal,
        Inflection::Accusative,
        Inflection::Dative,
        Inflection::Locative,
        Inflection::Ablative,
        Inflection::Genitive,
        Inflection::Derivation,
    ]
    .into_iter()
    .find(|family| {
        HARMONIES.iter().any(|harmony| {
            WORD_ENDS
                .iter()
                .any(|end| Word::new("", *harmony, *end).source_suffix(*family) == suffix)
        })
    })
}

/// Letters written after an apostrophe, with their inflection when it is a known one.
#[derive(Clone, Debug)]
pub(crate) struct Suffix {
    letters: String,
    family: Option<Inflection>,
}

impl Suffix {
    /// A suffix is never a name, so its letters are taken in Turkish lowercase: `TBMM'DE` is
    /// said as `TBMM'de` is.
    pub(crate) fn new(letters: &str) -> Self {
        let letters = lookup_key(letters);
        Self {
            family: written_inflection(&letters),
            letters,
        }
    }
    pub(crate) fn letters(&self) -> &str {
        &self.letters
    }
    pub(crate) fn family(&self) -> Option<Inflection> {
        self.family
    }
}

/// The harmony a suffix vowel takes after this vowel.
pub(crate) fn vowel_harmony(letter: char) -> Option<Harmony> {
    Some(match letter {
        'a' | 'ı' => Harmony::BackFlat,
        'e' | 'i' => Harmony::FrontFlat,
        'o' | 'u' => Harmony::BackRound,
        'ö' | 'ü' => Harmony::FrontRound,
        _ => return None,
    })
}

fn high_vowel(letter: char) -> bool {
    matches!(letter, 'ı' | 'i' | 'u' | 'ü')
}

/// The first letter of the harmony's high or low vowel.
fn vowel(text: &str) -> char {
    text.chars().next().unwrap_or('i')
}

/// A suffix that makes a new word (`-lI`, `-lIk`, `-sIz`, `-CI`), before which a compound
/// drops its possessive ending: `KDV'li` is katma değer vergili. The instrumental `-lA` is no
/// such suffix: `KDV'yle` keeps it.
fn derives(letters: &str) -> bool {
    let mut chars = letters.chars();
    match (chars.next(), chars.next(), chars.next()) {
        (Some('l' | 'c' | 'ç'), Some(next), _) => high_vowel(next),
        (Some('s'), Some(next), Some('z')) => high_vowel(next),
        _ => false,
    }
}

/// The plural `-lAr` at the start of a suffix, and the letters after it.
fn plural(letters: &str) -> Option<(&str, &str)> {
    let rest = letters
        .strip_prefix("lar")
        .or_else(|| letters.strip_prefix("ler"))?;
    Some((&letters[..letters.len() - rest.len()], rest))
}

/// The letters after a plural that a compound's possessive, said again after the plural,
/// already holds: the vowel of `KDV'leri` and of `KDV'lerine`, but not the genitive of
/// `KDV'lerin`.
fn after_plural_possessive(rest: &str) -> &str {
    let mut chars = rest.char_indices();
    match (chars.next(), chars.next(), chars.next()) {
        (Some((_, high)), None, _) if high_vowel(high) => "",
        (Some((_, high)), Some((at, 'n')), Some(_)) if high_vowel(high) => &rest[at..],
        _ => rest,
    }
}

/// A suffix that takes the buffer `y` after a vowel, which writers often leave out: the past
/// `-DI` (not `-DIr`), `-ken`, the instrumental `-lA`, the conditional `-sA` and `-mIş`.
fn takes_buffer(chars: &[char]) -> bool {
    match chars {
        ['d' | 't', high, rest @ ..] => high_vowel(*high) && rest.first() != Some(&'r'),
        ['k', 'e', 'n', ..] | ['l', 'a' | 'e'] | ['s', 'a' | 'e', ..] => true,
        ['m', high, 'ş', ..] => high_vowel(*high),
        _ => false,
    }
}

/// The letters after a possessive `-sI` that a compound already says: `TL'sine` is Türk
/// lirasına, since `lirası` holds it.
fn after_possessive(letters: &str) -> Option<&str> {
    let mut chars = letters.char_indices();
    match (chars.next(), chars.next(), chars.next()) {
        (Some((_, 's')), Some((_, high)), Some((_, 'z'))) if high_vowel(high) => None,
        (Some((_, 's')), Some((_, high)), rest) if high_vowel(high) => {
            Some(rest.map_or("", |(at, _)| &letters[at..]))
        }
        _ => None,
    }
}

/// A suffix with no known inflection, fitted to the word it now follows: after a consonant a
/// buffer `y` or a possessive `s` is dropped and after a vowel a missing buffer `y` is added,
/// a first `d` or `c` is voiced or not like the word's last sound, and each vowel takes the
/// harmony of the vowel before it. The relative `ki` and the converb `ken` keep their vowel,
/// and after a compound's possessive a case ending takes its `n`. A suffix written for the
/// same sounds stays as it is.
fn fitted(letters: &str, tail: Word) -> String {
    let mut chars: Vec<char> = letters.chars().collect();
    let vowel_final = matches!(tail.end, WordEnd::Vowel | WordEnd::Possessive);
    if let [first, second, rest @ ..] = chars.as_slice() {
        let possessive = *first == 's' && high_vowel(*second) && rest.first() != Some(&'z');
        if !vowel_final && (*first == 'y' || possessive) {
            chars.remove(0);
        }
    }
    let voiceless = matches!(
        tail.end,
        WordEnd::Voiceless | WordEnd::Softens | WordEnd::SoftensP | WordEnd::SoftensK
    );
    if let Some(first) = chars.first_mut() {
        *first = match (*first, voiceless) {
            ('d' | 't', true) => 't',
            ('d' | 't', false) => 'd',
            ('c' | 'ç', true) => 'ç',
            ('c' | 'ç', false) => 'c',
            (other, _) => other,
        };
    }
    if vowel_final && takes_buffer(&chars) {
        chars.insert(0, 'y');
    }
    let fixed = match chars.as_slice() {
        [.., 'k', 'i' | 'ü'] => chars.len().checked_sub(1),
        [.., 'k', 'e', 'n'] => chars.len().checked_sub(2),
        _ => None,
    };
    let case = tail.end == WordEnd::Possessive && matches!(chars.as_slice(), ['d', 'a' | 'e', ..]);
    let mut harmony = tail.harmony;
    let fitted = chars.into_iter().enumerate().map(|(at, letter)| {
        let letter = match letter {
            _ if fixed == Some(at) => letter,
            'a' | 'e' => vowel(harmony.low()),
            letter if high_vowel(letter) => vowel(harmony.high()),
            letter => letter,
        };
        harmony = vowel_harmony(letter).unwrap_or(harmony);
        letter
    });
    case.then_some('n').into_iter().chain(fitted).collect()
}

/// The shape a fitted suffix leaves for the next one.
fn fitted_tail(fitted: &str, tail: Word) -> Word {
    let harmony = fitted
        .chars()
        .rev()
        .find_map(vowel_harmony)
        .unwrap_or(tail.harmony);
    let end = match fitted.chars().last() {
        None => return tail,
        Some(letter) if vowel_harmony(letter).is_some() => WordEnd::Vowel,
        Some('f' | 's' | 't' | 'k' | 'ç' | 'ş' | 'h' | 'p') => WordEnd::Voiceless,
        Some(_) => WordEnd::Voiced,
    };
    Word::new("", harmony, end)
}

impl Spoken {
    /// Attaches a written suffix: harmonized again to this tail when its inflection is known,
    /// otherwise fitted to it letter by letter. A compound says its possessive once, drops it
    /// before a suffix that derives a word, and says it after a plural: `KDV'ler` is katma
    /// değer vergileri, `KDV'lerde` katma değer vergilerinde.
    pub(crate) fn attach(&mut self, suffix: &Suffix) {
        if self.tail.end == WordEnd::Possessive {
            if let Some(rest) = after_possessive(&suffix.letters) {
                if !rest.is_empty() {
                    self.attach(&Suffix::new(rest));
                }
                return;
            }
            if let Some((plural, rest)) = plural(&suffix.letters) {
                self.drop_possessive();
                self.attach(&Suffix::new(plural));
                self.possessive_again();
                let rest = after_plural_possessive(rest);
                if !rest.is_empty() {
                    self.attach(&Suffix::new(rest));
                }
                return;
            }
            if derives(&suffix.letters) {
                self.drop_possessive();
            }
        }
        match suffix.family {
            Some(family) => self.inflect(family),
            None => {
                let fitted = fitted(&suffix.letters, self.tail);
                if fitted.starts_with(|letter| vowel_harmony(letter).is_some()) {
                    self.soften();
                }
                self.tail = fitted_tail(&fitted, self.tail);
                self.text.push_str(&fitted);
            }
        }
    }
    /// The possessive `-I` of a compound, said again after its plural: `vergiler` becomes
    /// `vergileri`, which takes a case ending as a possessive does.
    fn possessive_again(&mut self) {
        self.text.push(vowel(self.tail.harmony.high()));
        self.tail = Word::new("", self.tail.harmony, WordEnd::Possessive);
    }
    /// `lirası` and `vergisi` without their possessive `-sI`, which ends in a vowel.
    fn drop_possessive(&mut self) {
        let word = self.tail.text;
        let mut ending = word.char_indices().rev();
        if let (Some((_, high)), Some((at, 's'))) = (ending.next(), ending.next())
            && high_vowel(high)
            && self.text.ends_with(word)
        {
            self.text.truncate(self.text.len() - (word.len() - at));
            self.tail = Word::new(&word[..at], self.tail.harmony, WordEnd::Vowel);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_generated_allomorph_names_exactly_its_own_inflection() {
        for family in [
            Inflection::Ordinal,
            Inflection::Accusative,
            Inflection::Dative,
            Inflection::Locative,
            Inflection::Ablative,
            Inflection::Genitive,
            Inflection::Derivation,
        ] {
            for harmony in HARMONIES {
                for end in WORD_ENDS {
                    let suffix = Word::new("", harmony, end).source_suffix(family);
                    assert_eq!(Suffix::new(&suffix).family(), Some(family), "{suffix}");
                }
            }
        }
        for written in ["", "ünci", "lar", "üncünün", "x"] {
            assert_eq!(Suffix::new(written).family(), None, "{written}");
        }
        // Capitals are the same suffix.
        for (written, family) in [
            ("DE", Inflection::Locative),
            ("YI", Inflection::Accusative),
            ("İN", Inflection::Genitive),
            ("ÜNCÜ", Inflection::Ordinal),
        ] {
            let suffix = Suffix::new(written);
            assert_eq!(suffix.family(), Some(family), "{written}");
            assert_eq!(suffix.letters(), lookup_key(written), "{written}");
        }
    }

    #[test]
    fn a_written_suffix_is_harmonized_again_to_the_spoken_tail() {
        for (word, written, expected) in [
            (
                Word::new("otuz", Harmony::BackRound, WordEnd::Voiced),
                "de",
                "otuzda",
            ),
            (
                Word::new("beş", Harmony::FrontFlat, WordEnd::Voiceless),
                "de",
                "beşte",
            ),
            (
                Word::new("yüz", Harmony::FrontRound, WordEnd::Voiced),
                "da",
                "yüzde",
            ),
            (
                Word::new("dört", Harmony::FrontRound, WordEnd::Softens),
                "in",
                "dördün",
            ),
            // A suffix without a known inflection is fitted to the word letter by letter.
            (
                Word::new("doksan", Harmony::BackFlat, WordEnd::Voiced),
                "lar",
                "doksanlar",
            ),
        ] {
            let mut spoken = Spoken::from_words(&[word]);
            spoken.attach(&Suffix::new(written));
            assert_eq!(spoken.into_text(), expected);
        }
    }

    #[test]
    fn a_suffix_written_for_other_sounds_is_fitted_to_the_spoken_word() {
        let lira = Word::new("lira", Harmony::BackFlat, WordEnd::Vowel);
        let dolar = Word::new("dolar", Harmony::BackFlat, WordEnd::Voiced);
        let bes = Word::new("beş", Harmony::FrontFlat, WordEnd::Voiceless);
        let dort = Word::new("dört", Harmony::FrontRound, WordEnd::Softens);
        let otuz = Word::new("otuz", Harmony::BackRound, WordEnd::Voiced);
        for (word, written, expected) in [
            // `TL'li` was written for "te le"; the spoken word is lira.
            (lira, "li", "liralı"),
            (lira, "ler", "liralar"),
            (lira, "si", "lirası"),
            (lira, "deki", "liradaki"),
            // After a consonant the buffer `y` and the possessive `s` fall away.
            (dolar, "si", "doları"),
            (dolar, "yle", "dolarla"),
            (dolar, "siz", "dolarsız"),
            // The first `d` follows the voicing, and the relative `ki` keeps its vowel.
            (bes, "deki", "beşteki"),
            (otuz, "taki", "otuzdaki"),
            (dort, "ünkü", "dördünkü"),
            // A suffix written for the same sounds stays as it is.
            (otuz, "daki", "otuzdaki"),
            (bes, "er", "beşer"),
            (lira, "yken", "lirayken"),
            (dolar, "yken", "dolarken"),
            // After a vowel a missing buffer `y` is added; `-DIr` takes none.
            (lira, "di", "liraydı"),
            (lira, "ken", "lirayken"),
            (lira, "le", "lirayla"),
            (lira, "se", "liraysa"),
            (lira, "miş", "liraymış"),
            (lira, "dir", "liradır"),
            // A misspelled ordinal is fitted too.
            (
                Word::new("üç", Harmony::FrontRound, WordEnd::Voiceless),
                "ünci",
                "üçüncü",
            ),
        ] {
            let mut spoken = Spoken::from_words(&[word]);
            spoken.attach(&Suffix::new(written));
            assert_eq!(spoken.into_text(), expected, "{} + {written}", word.text);
        }
    }

    #[test]
    fn every_spelling_of_a_suffix_is_said_the_same_after_any_word() {
        // Each family in all its written shapes, checked once against a Turkish morphology
        // analyzer: the shape a writer chose must not change what is said.
        const FAMILIES: &[&[&str]] = &[
            &["li", "lı", "lu", "lü"],
            &["ler", "lar"],
            &["siz", "sız", "suz", "süz"],
            &["de", "da", "te", "ta"],
            &["den", "dan", "ten", "tan"],
            &["e", "a", "ye", "ya"],
            &["i", "ı", "u", "ü", "yi", "yı", "yu", "yü"],
            &["in", "ın", "un", "ün", "nin", "nın", "nun", "nün"],
            &["deki", "daki", "teki", "taki"],
            &["dir", "dır", "dur", "dür", "tir", "tır", "tur", "tür"],
            &[
                "ydi", "ydı", "ydu", "ydü", "di", "dı", "du", "dü", "ti", "tı", "tu", "tü",
            ],
            &["yken", "ken"],
            &["yle", "yla", "le", "la"],
            &["si", "sı", "su", "sü"],
            &["lik", "lık", "luk", "lük"],
            &["ci", "cı", "cu", "cü", "çi", "çı", "çu", "çü"],
            &["inci", "ıncı", "uncu", "üncü", "nci", "ncı", "ncu", "ncü"],
        ];
        let words = [
            ("iki", Word::new("iki", Harmony::FrontFlat, WordEnd::Vowel)),
            ("altı", Word::new("altı", Harmony::BackFlat, WordEnd::Vowel)),
            (
                "beş",
                Word::new("beş", Harmony::FrontFlat, WordEnd::Voiceless),
            ),
            (
                "dört",
                Word::new("dört", Harmony::FrontRound, WordEnd::Softens),
            ),
            (
                "otuz",
                Word::new("otuz", Harmony::BackRound, WordEnd::Voiced),
            ),
            (
                "yüz",
                Word::new("yüz", Harmony::FrontRound, WordEnd::Voiced),
            ),
            ("lira", Word::new("lira", Harmony::BackFlat, WordEnd::Vowel)),
            (
                "dolar",
                Word::new("dolar", Harmony::BackFlat, WordEnd::Voiced),
            ),
            (
                "blok",
                Word::new("blok", Harmony::BackRound, WordEnd::SoftensK),
            ),
            (
                "katma değer vergisi",
                Word::new("vergisi", Harmony::FrontFlat, WordEnd::Possessive),
            ),
        ];
        for (text, word) in words {
            for spellings in FAMILIES {
                let said: std::collections::BTreeSet<_> = spellings
                    .iter()
                    .map(|written| {
                        let mut spoken = Spoken::lexical(text, word);
                        spoken.attach(&Suffix::new(written));
                        spoken.into_text()
                    })
                    .collect();
                assert_eq!(said.len(), 1, "{text} + {spellings:?}: {said:?}");
            }
        }
        // A few of the forms the analyzer accepted.
        for (word, written, expected) in [
            (words[0].1, "di", "ikiydi"),
            (words[3].1, "dü", "dörttü"),
            (words[3].1, "ünkü", "dördünkü"),
            (words[4].1, "la", "otuzla"),
            (words[8].1, "u", "bloğu"),
            (words[9].1, "le", "vergisiyle"),
        ] {
            let mut spoken = Spoken::from_words(&[word]);
            spoken.attach(&Suffix::new(written));
            assert_eq!(spoken.into_text(), expected, "{} + {written}", word.text);
        }
    }

    #[test]
    fn a_compound_says_its_possessive_once_drops_it_to_derive_and_says_it_after_a_plural() {
        let vergisi = Word::new("vergisi", Harmony::FrontFlat, WordEnd::Possessive);
        let lirasi = Word::new("lirası", Harmony::BackFlat, WordEnd::Possessive);
        for (output, word, written, expected) in [
            ("katma değer vergisi", vergisi, "li", "katma değer vergili"),
            (
                "katma değer vergisi",
                vergisi,
                "siz",
                "katma değer vergisiz",
            ),
            (
                "katma değer vergisi",
                vergisi,
                "lik",
                "katma değer vergilik",
            ),
            ("katma değer vergisi", vergisi, "si", "katma değer vergisi"),
            (
                "katma değer vergisi",
                vergisi,
                "sine",
                "katma değer vergisine",
            ),
            (
                "katma değer vergisi",
                vergisi,
                "de",
                "katma değer vergisinde",
            ),
            (
                "katma değer vergisi",
                vergisi,
                "yle",
                "katma değer vergisiyle",
            ),
            (
                "katma değer vergisi",
                vergisi,
                "deki",
                "katma değer vergisindeki",
            ),
            (
                "katma değer vergisi",
                vergisi,
                "dir",
                "katma değer vergisidir",
            ),
            (
                "katma değer vergisi",
                vergisi,
                "le",
                "katma değer vergisiyle",
            ),
            // After a plural the possessive is said again, and a case ending follows it.
            (
                "katma değer vergisi",
                vergisi,
                "ler",
                "katma değer vergileri",
            ),
            (
                "katma değer vergisi",
                vergisi,
                "leri",
                "katma değer vergileri",
            ),
            (
                "katma değer vergisi",
                vergisi,
                "lerde",
                "katma değer vergilerinde",
            ),
            (
                "katma değer vergisi",
                vergisi,
                "lerinde",
                "katma değer vergilerinde",
            ),
            (
                "katma değer vergisi",
                vergisi,
                "lerin",
                "katma değer vergilerinin",
            ),
            (
                "katma değer vergisi",
                vergisi,
                "lere",
                "katma değer vergilerine",
            ),
            (
                "katma değer vergisi",
                vergisi,
                "lerle",
                "katma değer vergileriyle",
            ),
            ("Türk lirası", lirasi, "lar", "Türk liraları"),
            ("Türk lirası", lirasi, "li", "Türk liralı"),
            ("Türk lirası", lirasi, "sinden", "Türk lirasından"),
        ] {
            let mut spoken = Spoken::lexical(output, word);
            spoken.attach(&Suffix::new(written));
            assert_eq!(spoken.into_text(), expected, "{output} + {written}");
        }
    }
}
