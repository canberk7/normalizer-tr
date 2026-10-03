use std::{borrow::Cow, ops::Range};

use unicode_segmentation::UnicodeSegmentation;

use super::lexicon::{self, Currency, Lexeme};
use crate::{
    morphology::{Harmony, Inflection, Suffix, Word, WordEnd},
    numerals::Number,
};

use Harmony::{BackFlat, BackRound, FrontFlat, FrontRound};
use WordEnd::{Softens, SoftensP, Voiced, Voiceless, Vowel};

/// Spoken name of every separator and symbol: the one table the literal reading draws on.
const NAMES: &[(char, Word)] = &[
    ('.', Word::new("nokta", BackFlat, Vowel)),
    (',', Word::new("virgül", FrontRound, Voiced)),
    (':', Word::new("iki nokta", BackFlat, Vowel)),
    ('/', Word::new("slaş", BackFlat, Voiceless)),
    ('-', Word::new("tire", FrontFlat, Vowel)),
    ('+', Word::new("artı", BackFlat, Vowel)),
    ('−', Word::new("eksi", FrontFlat, Vowel)),
    ('=', Word::new("eşittir", FrontFlat, Voiced)),
    ('×', Word::new("iks", FrontFlat, Voiceless)),
    ('÷', Word::new("bölü", FrontRound, Vowel)),
    ('*', Word::new("çarpı", BackFlat, Vowel)),
    ('%', Word::new("yüzde", FrontFlat, Vowel)),
    ('&', Word::new("ve", FrontFlat, Vowel)),
    ('@', Word::new("et", FrontFlat, Voiceless)),
    ('#', Word::new("heşteg", FrontFlat, Voiced)),
    ('<', Word::new("küçüktür", FrontRound, Voiced)),
    ('>', Word::new("büyüktür", FrontRound, Voiced)),
    ('°', Word::new("derece", FrontFlat, Vowel)),
    ('℃', Word::new("derece", FrontFlat, Vowel)),
    ('℉', FAHRENHEIT),
    ('_', Word::new("alt çizgi", FrontFlat, Vowel)),
    ('½', Word::new("yarım", BackFlat, Voiced)),
    ('¼', Word::new("çeyrek", FrontFlat, Voiceless)),
    ('¾', Word::new("üç çeyrek", FrontFlat, Voiceless)),
    ('⅓', Word::new("bir bölü üç", FrontRound, Voiceless)),
    ('⅔', Word::new("iki bölü üç", FrontRound, Voiceless)),
    ('⅕', Word::new("bir bölü beş", FrontFlat, Voiceless)),
    ('⅖', Word::new("iki bölü beş", FrontFlat, Voiceless)),
    ('⅗', Word::new("üç bölü beş", FrontFlat, Voiceless)),
    ('⅘', Word::new("dört bölü beş", FrontFlat, Voiceless)),
    ('⅙', Word::new("bir bölü altı", BackFlat, Vowel)),
    ('⅚', Word::new("beş bölü altı", BackFlat, Vowel)),
    ('⅐', Word::new("bir bölü yedi", FrontFlat, Vowel)),
    ('⅛', Word::new("bir bölü sekiz", FrontFlat, Voiced)),
    ('⅜', Word::new("üç bölü sekiz", FrontFlat, Voiced)),
    ('⅝', Word::new("beş bölü sekiz", FrontFlat, Voiced)),
    ('⅞', Word::new("yedi bölü sekiz", FrontFlat, Voiced)),
    ('⅑', Word::new("bir bölü dokuz", BackRound, Voiced)),
    ('⅒', Word::new("bir bölü on", BackRound, Voiced)),
    ('₺', Word::new("lira", BackFlat, Vowel)),
    ('$', Word::new("dolar", BackFlat, Voiced)),
    ('€', Word::new("avro", BackRound, Vowel)),
    ('£', Word::new("sterlin", FrontFlat, Voiced)),
    ('¥', Word::new("yen", FrontFlat, Voiced)),
    ('₽', Word::new("ruble", FrontFlat, Vowel)),
    ('₹', Word::new("rupi", FrontFlat, Vowel)),
    ('₩', Word::new("von", BackRound, Voiced)),
    ('₼', Word::new("manat", BackFlat, Voiceless)),
    ('₾', Word::new("lari", FrontFlat, Vowel)),
    ('₴', Word::new("grivna", BackFlat, Vowel)),
    ('₸', Word::new("tenge", FrontFlat, Vowel)),
    ('₿', Word::new("bitkoin", FrontFlat, Voiced)),
    ('¢', Word::new("sent", FrontFlat, Voiceless)),
    ('§', Word::new("paragraf", BackFlat, Voiceless)),
    ('№', Word::new("numara", BackFlat, Vowel)),
    ('★', Word::new("yıldız", BackFlat, Voiced)),
    ('☆', Word::new("yıldız", BackFlat, Voiced)),
    ('≈', Word::new("yaklaşık", BackFlat, Voiceless)),
    ('~', Word::new("yaklaşık", BackFlat, Voiceless)),
    ('≠', Word::new("eşit değildir", FrontFlat, Voiced)),
    ('≤', Word::new("küçük eşittir", FrontFlat, Voiced)),
    ('≥', Word::new("büyük eşittir", FrontFlat, Voiced)),
    ('±', Word::new("artı eksi", FrontFlat, Vowel)),
    ('∞', Word::new("sonsuz", BackRound, Voiced)),
    ('‰', Word::new("binde", FrontFlat, Vowel)),
    ('√', Word::new("karekök", FrontRound, Voiceless)),
    ('^', Word::new("üssü", FrontRound, Vowel)),
    ('π', Word::new("pi", FrontFlat, Vowel)),
    ('⭐', Word::new("yıldız", BackFlat, Voiced)),
    ('²', Word::new("kare", FrontFlat, Vowel)),
    ('³', Word::new("küp", FrontRound, SoftensP)),
    ('⁰', Word::new("üssü sıfır", BackFlat, Voiced)),
    ('¹', Word::new("üssü bir", FrontFlat, Voiced)),
    ('⁴', Word::new("üssü dört", FrontRound, Softens)),
    ('⁵', Word::new("üssü beş", FrontFlat, Voiceless)),
    ('⁶', Word::new("üssü altı", BackFlat, Vowel)),
    ('⁷', Word::new("üssü yedi", FrontFlat, Vowel)),
    ('⁸', Word::new("üssü sekiz", FrontFlat, Voiced)),
    ('⁹', Word::new("üssü dokuz", BackRound, Voiced)),
];

/// Said for a temperature in Fahrenheit, `°F` or `℉`; Celsius is the degree said alone.
const FAHRENHEIT: Word = Word::new("derece Fahrenhayt", BackFlat, Voiceless);
/// Said for `x` right after a count, by what follows it: `3x 1.250 TL` is üç kere, `2x Intel`
/// iki adet, `3x faydalı` üç kat.
const TIMES: Word = Word::new("kere", FrontFlat, Vowel);
const PIECES: Word = Word::new("adet", FrontFlat, Voiceless);
const FOLD: Word = Word::new("kat", BackFlat, Voiceless);
/// Said for the marks a number is written with: inches, minutes and seconds of a degree.
const INCH: Word = Word::new("inç", FrontFlat, Voiceless);
const MINUTE: Word = Word::new("dakika", BackFlat, Vowel);
const SECOND: Word = Word::new("saniye", FrontFlat, Vowel);
/// Said for `v` before a version number, and for `K` and `M` after a count.
const VERSION: Word = Word::new("versiyon", BackRound, Voiced);
const THOUSAND: Word = Word::new("bin", FrontFlat, Voiced);
const MILLION: Word = Word::new("milyon", BackRound, Voiced);
/// Said for `W`, `V` and `MP` right after a number: watts, volts and megapixels.
const WATT: Word = Word::new("vat", BackFlat, Voiceless);
const VOLT: Word = Word::new("volt", BackRound, Voiceless);
const MEGAPIXEL: Word = Word::new("megapiksel", FrontFlat, Voiced);
/// Said for `^` where it is no power, and for `!` and `?` among symbols only (`!@#`).
const CARET: Word = Word::new("şapka", BackFlat, Vowel);
const EXCLAMATION: Word = Word::new("ünlem", FrontFlat, Voiced);
const QUESTION: Word = Word::new("soru işareti", FrontFlat, Vowel);

/// Symbols with no name that the literal reading drops, also when one is a stray word: marks
/// of a list, an arrow, a check mark and the signs of a trademark.
const DROPPED: &[char] = &[
    '|', '¦', '\\', '`', '©', '®', '™', '•', '·', '◦', '▪', '►', '▶', '→', '←', '↑', '↓', '⇒', '✓',
    '✔',
];

/// Said for `#` before a number or a code, `*` and `#` on a phone's keypad, and a power
/// written in superscripts; `*` after a count is stars, `5* otel` beş yıldızlı otel.
const NUMBER_SIGN: Word = Word::new("numara", BackFlat, Vowel);
const STAR: Word = Word::new("yıldız", BackFlat, Voiced);
const STARRED: Word = Word::new("yıldızlı", BackFlat, Vowel);
const SQUARE: Word = Word::new("kare", FrontFlat, Vowel);
const POWER: Word = Word::new("üssü", FrontRound, Vowel);

/// Faces and arrows typed with letters and symbols, which say nothing: `:D` is no letter D and
/// `<3` no comparison.
const SILENT_WORDS: &[&str] = &[
    ":D", ":-D", ";D", ":P", ":-P", ":p", ":-p", ";P", ";p", ":O", ":-O", ":o", ":S", ":s", ":3",
    ":*", ":-*", ":/", ":-/", ":|", ":-|", ":$", "=D", "=P", "XD", "xD", "xd", "XDD", "<3", "<33",
    "<333", "</3", "^^", "^_^", "^.^", "-_-", "-.-", "o_O", "O_o", "o.O", "O.o", "T_T", ";_;",
    "->", "-->", "=>", "==>", "<-", "<--", "<==", "<->", "<=>",
];

/// Symbols written over and over as a line or around words, which say nothing: `***`, `###`,
/// `___`, `===`.
const DECORATIONS: &[char] = &['*', '#', '_', '=', '~', '^', '<', '>', '-', '+'];

/// Whether a word no reader claims says nothing: a face or an arrow (`:D`, `<3`, `->`), one
/// symbol written over and over (`***`), or the mark that starts a Markdown line before a
/// space: a list item or a quote (`* madde`, `> alıntı`), and a heading before a capital or a
/// number (`# Başlık`), since `# işaretinden sonra` speaks of the sign itself.
pub(crate) fn silent(word: &str, before: &str, following: &str) -> bool {
    let line_start = before
        .trim_end_matches([' ', '\t', '\u{a0}'])
        .chars()
        .next_back()
        .is_none_or(|ch| matches!(ch, '\n' | '\r' | '\u{2028}' | '\u{2029}'));
    let spaced = following.starts_with([' ', '\t']);
    let heading = (1..=6).contains(&word.len())
        && word.bytes().all(|byte| byte == b'#')
        && following
            .trim_start()
            .starts_with(|ch: char| ch.is_uppercase() || ch.is_ascii_digit());
    unspeakable(word) || (line_start && spaced && (matches!(word, "*" | ">") || heading))
}

/// The text without the marks of emphasis and of footnotes around its words, which say
/// nothing: `**kişi**den` is kişiden, `_vurgu_` vurgu, `kullanın.*` kullanın. A star between
/// two numbers or single letters stays (`3 * 4`, `a*b`), as does a count's (`5*`), and so
/// does an underscore inside a word (`a_b`). Symbols with no word are all named.
fn unemphasized(text: &str) -> Cow<'_, str> {
    if !text.contains(['*', '_']) || !text.chars().any(char::is_alphanumeric) {
        return Cow::Borrowed(text);
    }
    let chars: Vec<char> = text.chars().collect();
    let skipped = |ch: &char| matches!(ch, '*' | ' ' | '\t' | '\u{a0}');
    // A number, a bracket, or a letter with no other letter on its far side.
    let operand = |at: usize, beyond: Option<usize>| {
        let ch = chars[at];
        ch.is_ascii_digit()
            || matches!(ch, '(' | ')' | '[' | ']')
            || (ch.is_alphabetic() && !beyond.is_some_and(|far| chars[far].is_alphabetic()))
    };
    let mut kept = String::with_capacity(text.len());
    for (at, ch) in chars.iter().enumerate() {
        let keep = match ch {
            '*' => {
                let before = chars[..at].iter().rposition(|ch| !skipped(ch));
                let after = chars[at + 1..]
                    .iter()
                    .position(|ch| !skipped(ch))
                    .map(|offset| at + 1 + offset);
                let joins = before.is_some_and(|left| operand(left, left.checked_sub(1)))
                    && after.is_some_and(|right| {
                        operand(right, Some(right + 1).filter(|far| *far < chars.len()))
                    });
                let count = at > 0
                    && chars[at - 1].is_ascii_digit()
                    && !chars.get(at + 1).is_some_and(|next| next.is_alphanumeric());
                joins || count
            }
            '_' => {
                at > 0
                    && chars[at - 1].is_alphanumeric()
                    && chars.get(at + 1).is_some_and(|next| next.is_alphanumeric())
            }
            _ => true,
        };
        if keep {
            kept.push(*ch);
        }
    }
    Cow::Owned(kept)
}

/// A face, an arrow or one symbol written over and over, which says nothing wherever it
/// stands: `:D`, `->`, `***`.
fn unspeakable(word: &str) -> bool {
    let mut symbols = word.chars();
    SILENT_WORDS.contains(&word)
        || symbols
            .next()
            .filter(|first| DECORATIONS.contains(first))
            .is_some_and(|first| word.chars().count() > 1 && symbols.all(|ch| ch == first))
}

/// Whether a lone `*`, `<` or `>` stands between two things it can join, numbers or single
/// letters (`3 * 4`, `x > y`), or before a number it compares (`< 1 dk`). Between words it is
/// a mark of a menu path or a footnote and says nothing: `Kategori > Telefon`.
pub(crate) fn operates(symbol: &str, before: &str, following: &str) -> bool {
    let left = before.trim_end_matches([' ', '\t', '\u{a0}']);
    let right = following.trim_start_matches([' ', '\t', '\u{a0}']);
    let comparison = matches!(symbol, "<" | ">");
    // A letter is an operand when no other letter is written next to it.
    let lone = |letter: Option<char>, beyond: Option<char>| {
        letter.is_some_and(char::is_alphabetic) && !beyond.is_some_and(char::is_alphabetic)
    };
    let mut ending = left.chars().rev();
    let (last, before_last) = (ending.next(), ending.next());
    let mut starting = right.chars();
    let (first, second) = (starting.next(), starting.next());
    let operand_before = last.is_some_and(|ch| ch.is_ascii_digit() || matches!(ch, ')' | ']'))
        || lone(last, before_last);
    let operand_after = first.is_some_and(|ch| ch.is_ascii_digit() || matches!(ch, '(' | '['))
        || (comparison && first == Some('='))
        || lone(first, second);
    (operand_before && operand_after) || (comparison && first.is_some_and(|ch| ch.is_ascii_digit()))
}

/// The Greek letters, said by their Turkish names when one stands alone: `β-karoten` is beta
/// karoten.
const GREEK: &[(char, char, &str, Harmony, WordEnd)] = &[
    ('α', 'Α', "alfa", BackFlat, Vowel),
    ('β', 'Β', "beta", FrontFlat, Vowel),
    ('γ', 'Γ', "gama", BackFlat, Vowel),
    ('δ', 'Δ', "delta", BackFlat, Vowel),
    ('ε', 'Ε', "epsilon", BackRound, Voiced),
    ('ζ', 'Ζ', "zeta", BackFlat, Vowel),
    ('η', 'Η', "eta", BackFlat, Vowel),
    ('θ', 'Θ', "teta", BackFlat, Vowel),
    ('ι', 'Ι', "yota", BackFlat, Vowel),
    ('κ', 'Κ', "kapa", BackFlat, Vowel),
    ('λ', 'Λ', "lamda", BackFlat, Vowel),
    ('μ', 'Μ', "mü", FrontRound, Vowel),
    ('ν', 'Ν', "nü", FrontRound, Vowel),
    ('ξ', 'Ξ', "ksi", FrontFlat, Vowel),
    ('ο', 'Ο', "omikron", BackRound, Voiced),
    ('ρ', 'Ρ', "ro", BackRound, Vowel),
    ('σ', 'Σ', "sigma", BackFlat, Vowel),
    ('ς', 'Σ', "sigma", BackFlat, Vowel),
    ('τ', 'Τ', "tav", BackFlat, Voiced),
    ('υ', 'Υ', "üpsilon", BackRound, Voiced),
    ('φ', 'Φ', "fi", FrontFlat, Vowel),
    ('χ', 'Χ', "ki", FrontFlat, Vowel),
    ('ψ', 'Ψ', "psi", FrontFlat, Vowel),
    ('ω', 'Ω', "omega", BackFlat, Vowel),
];

fn greek_name(letter: char) -> Option<Word> {
    GREEK
        .iter()
        .find(|(lower, upper, ..)| *lower == letter || *upper == letter)
        .map(|(_, _, name, harmony, end)| Word::new(name, *harmony, *end))
}

/// The value of a superscript digit: `²³` is a power of twenty-three.
fn superscript(grapheme: &str) -> Option<u64> {
    let mut chars = grapheme.chars();
    let digit = match (chars.next()?, chars.next()) {
        ('⁰', None) => 0,
        ('¹', None) => 1,
        ('²', None) => 2,
        ('³', None) => 3,
        (ch @ '⁴'..='⁹', None) => u64::from(ch) - u64::from('⁴') + 4,
        _ => return None,
    };
    Some(digit)
}

/// `½` right after a digit run is the half of that number: `2½` is iki buçuk.
const AND_A_HALF: Word = Word::new("buçuk", BackRound, Voiceless);

/// Named only between two parts of the span; at its edges they are ordinary punctuation.
const SEPARATORS: [char; 5] = ['.', ',', ':', '/', '-'];

/// Turkish names of the consonants, lowercase and capital; a vowel is its own name.
const LETTERS: &[(char, &str, char, &str)] = &[
    ('b', "be", 'B', "Be"),
    ('c', "ce", 'C', "Ce"),
    ('ç', "çe", 'Ç', "Çe"),
    ('d', "de", 'D', "De"),
    ('f', "fe", 'F', "Fe"),
    ('g', "ge", 'G', "Ge"),
    ('ğ', "yumuşak ge", 'Ğ', "Yumuşak ge"),
    ('h', "he", 'H', "He"),
    ('j', "je", 'J', "Je"),
    ('k', "ke", 'K', "Ke"),
    ('l', "le", 'L', "Le"),
    ('m', "me", 'M', "Me"),
    ('n', "ne", 'N', "Ne"),
    ('p', "pe", 'P', "Pe"),
    ('q', "kü", 'Q', "Kü"),
    ('r', "re", 'R', "Re"),
    ('s', "se", 'S', "Se"),
    ('ş', "şe", 'Ş', "Şe"),
    ('t', "te", 'T', "Te"),
    ('v', "ve", 'V', "Ve"),
    ('w', "çift ve", 'W', "Çift ve"),
    ('x', "iks", 'X', "İks"),
    ('y', "ye", 'Y', "Ye"),
    ('z', "ze", 'Z', "Ze"),
];

fn spoken_name(symbol: char) -> Option<Word> {
    NAMES
        .iter()
        .find(|(key, _)| *key == symbol)
        .map(|(_, word)| *word)
}

fn letter_name(letter: char) -> Option<&'static str> {
    LETTERS.iter().find_map(|(lower, name, upper, capital)| {
        if *lower == letter {
            Some(*name)
        } else if *upper == letter {
            Some(*capital)
        } else {
            None
        }
    })
}

/// Sentence and bracketing punctuation, which stays as written at the edges of a span.
/// Symbols without a name, such as `|`, are not punctuation and are removed.
fn punctuation(ch: char) -> bool {
    matches!(
        ch,
        '!' | '?'
            | ';'
            | '"'
            | '('
            | ')'
            | '['
            | ']'
            | '{'
            | '}'
            | '‘'
            | '“'
            | '”'
            | '«'
            | '»'
            | '…'
            | '–'
            | '—'
    )
}

/// How the literal reading treats one grapheme: by its first character, unless it holds a
/// digit or a named symbol.
#[derive(Clone, Copy, Eq, PartialEq)]
enum Class {
    Digit,
    Letter,
    Space,
    Separator,
    Symbol,
    Apostrophe,
    Punctuation,
    Other,
}

/// The digit in a grapheme, a subscript one included: `H₂O` is He iki O.
fn ascii_digit(grapheme: &str) -> Option<char> {
    grapheme.chars().find_map(|ch| match ch {
        '0'..='9' => Some(ch),
        '₀'..='₉' => char::from_digit(u32::from(ch) - u32::from('₀'), 10),
        _ => None,
    })
}

/// The named symbol in a grapheme; a separator is named only by its place in the span.
fn symbol(grapheme: &str) -> Option<char> {
    grapheme
        .chars()
        .find(|ch| !SEPARATORS.contains(ch) && spoken_name(*ch).is_some())
}

fn class(grapheme: &str) -> Class {
    // A prepended mark must not carry a digit or a symbol into the words kept as written.
    if ascii_digit(grapheme).is_some() {
        return Class::Digit;
    }
    if symbol(grapheme).is_some() {
        return Class::Symbol;
    }
    match grapheme.chars().next() {
        Some(ch) if ch.is_alphabetic() => Class::Letter,
        Some(ch) if ch.is_whitespace() => Class::Space,
        Some(ch) if SEPARATORS.contains(&ch) => Class::Separator,
        // A backtick or an acute accent is often typed for an apostrophe: `Mehmet`in`.
        Some('\'' | '’' | '`' | '´') => Class::Apostrophe,
        Some(ch) if punctuation(ch) => Class::Punctuation,
        _ => Class::Other,
    }
}

/// Consonant pairs a Turkish word can end in. Three capitals that end in any other pair can
/// only be letters: `ABD`, `AKP`, `ATM`.
const CODAS: &[&str] = &[
    "lç", "lf", "lg", "lk", "lm", "lp", "ls", "lt", "lz", "mb", "mp", "ms", "nç", "nd", "ng", "nk",
    "ns", "nş", "nt", "nz", "rb", "rç", "rd", "rf", "rg", "rk", "rl", "rm", "rn", "rp", "rs", "rş",
    "rt", "rz", "sk", "st", "şk", "şt", "ft", "ks", "kt", "ps", "pt", "ht", "zm",
];

/// Consonant pairs a word can start with, in Turkish loanwords and in English ones written in
/// capitals. Capitals that start with any other pair can only be letters: `DNA`, `YTÜ`.
const ONSETS: &[&str] = &[
    "bl", "br", "dr", "fl", "fr", "gl", "gr", "kl", "kr", "pl", "pr", "ps", "sf", "sk", "sl", "sm",
    "sn", "sp", "st", "sv", "şp", "şt", "tr", "ts", "vl", "hr", "kv", "sh", "th", "ch", "ph", "wh",
    "sc", "sw", "tw", "wr", "kn", "gn", "cl", "cr", "dw",
];

/// Words of two letters a headline may write in capitals, the Turkish words of the dictionary
/// and the English words a Turkish text quotes most; other pairs of capitals are letters said
/// by name: `ID` is İ De, `BU` is bu, `VE` ve.
const PAIRS: &[&str] = &[
    "aç", "ad", "af", "ağ", "ah", "ak", "al", "am", "an", "ar", "as", "aş", "at", "av", "ay", "az",
    "be", "bi", "bu", "by", "da", "de", "do", "eh", "ek", "el", "em", "en", "er", "es", "eş", "et",
    "ev", "ey", "fa", "go", "ha", "he", "hu", "ıh", "iç", "if", "iğ", "il", "im", "in", "ip", "is",
    "iş", "it", "iz", "ki", "la", "me", "mi", "mı", "mu", "mü", "my", "na", "ne", "no", "nü", "od",
    "of", "oh", "ol", "on", "or", "ot", "oy", "öç", "öd", "öf", "öl", "ön", "öp", "ör", "öz", "pi",
    "re", "sa", "si", "so", "su", "şu", "ta", "to", "tu", "uç", "uf", "un", "up", "ur", "us", "ut",
    "uz", "üç", "üf", "ün", "üs", "ve", "we", "ya", "ye", "yo",
];

/// Roman numerals of two letters, which are not spelled.
const ROMAN_PAIRS: &[&str] = &["II", "IV", "VI", "IX", "XI", "LI", "CI", "DI"];

/// The word two capitals make, as it is said: `VE` is ve and `MI` mı, and an `I` that makes
/// no Turkish word is an English one, so `IN` is in.
fn pair(letters: &[char]) -> Option<String> {
    let [first, second] = letters else {
        return None;
    };
    if !first.is_uppercase() || !second.is_uppercase() {
        return None;
    }
    let turkish = lexicon::lookup_key(&String::from_iter(letters));
    let english: String = turkish.replace('ı', "i");
    [turkish, english]
        .into_iter()
        .find(|word| PAIRS.contains(&word.as_str()))
}

/// Sounds written with consonants only, which are said as written and never spelled: `Hmm`,
/// `Pff`, `Şşt`, `Zzz`, `Brr`, `Pst`. In capitals one has three letters or more (`HMM`), since
/// two are more often a code (`ZZ`).
fn interjection(letters: &[char]) -> bool {
    if letters.len() < 3 && letters.iter().all(|letter| letter.is_uppercase()) {
        return false;
    }
    let word = lexicon::lookup_key(&String::from_iter(letters));
    // The runs of one letter the word is written in, in order: `hmmm` is h then m.
    let mut runs: Vec<(char, usize)> = Vec::new();
    for letter in word.chars() {
        match runs.last_mut() {
            Some((last, count)) if *last == letter => *count += 1,
            _ => runs.push((letter, 1)),
        }
    }
    let shape: String = runs.iter().map(|(letter, _)| *letter).collect();
    let long = |at: usize| runs.get(at).is_some_and(|(_, count)| *count > 1);
    match shape.as_str() {
        "hm" | "pf" | "pst" | "pşt" | "hşt" | "mhm" => true,
        "m" | "z" | "ş" | "şt" => long(0),
        "br" | "gr" | "hr" => long(1),
        _ => false,
    }
}

/// Lowercase consonants that are no unit or sound, which are letters said by name: `vs` is ve
/// se. A unit (`km`), a sound (`hmm`) and an English word whose `y` is its vowel (`my`, `by`)
/// stay as written.
fn lowercase_letters(letters: &[char]) -> bool {
    letters.len() > 1
        && letters
            .iter()
            .all(|letter| letter.is_lowercase() && letter_name(*letter).is_some())
        && !letters.contains(&'y')
        && !interjection(letters)
        && lexicon::unit(&String::from_iter(letters)).is_none()
}

/// Whether letters are said one by one by name: a single consonant; consonants only, in
/// capitals (`TK`) or after one capital (`Hz`), unless they write a sound (`Hmm`); two capitals
/// that are no word (`ID`, `CO2`); three capitals whose last two letters no Turkish word ends
/// in (`ABD`); or three capitals whose first two letters no word starts with (`NGO`). A longer
/// word is a word even so (`DZEKO`, `ŞNORKEL`), a doubled last letter is no proof (`OFF`), and
/// any other word with a vowel is a word.
fn spelled_out(letters: &[char], before_digit: bool) -> bool {
    let named = |letter: &char| letter_name(*letter).is_some();
    let capitals = letters.iter().all(|letter| letter.is_uppercase());
    let key = |pair: &[char]| lexicon::lookup_key(&String::from_iter(pair));
    match letters {
        [] => false,
        [letter] => named(letter),
        [first, rest @ ..] if letters.iter().all(named) => {
            (capitals || (first.is_uppercase() && rest.iter().all(|letter| letter.is_lowercase())))
                && !interjection(letters)
                && pair(letters).is_none()
        }
        _ if formula(letters) => true,
        _ if !capitals => false,
        [first, second] => {
            let written = String::from_iter([*first, *second]);
            before_digit || (pair(letters).is_none() && !ROMAN_PAIRS.contains(&written.as_str()))
        }
        [first, second, _]
            if named(first) && named(second) && !ONSETS.contains(&key(&letters[..2]).as_str()) =>
        {
            true
        }
        [_, before, last] => {
            named(before)
                && named(last)
                && before != last
                && !CODAS.contains(&key(&letters[1..]).as_str())
        }
        _ => false,
    }
}

/// Chemical symbols written together, each a capital with at most one lowercase letter after
/// it: `NaCl` is Ne a Ce le.
fn formula(letters: &[char]) -> bool {
    letters
        .iter()
        .filter(|letter| letter.is_uppercase())
        .count()
        >= 2
        && letters.first().is_some_and(|letter| letter.is_uppercase())
        && letters.iter().any(|letter| letter.is_lowercase())
        && !letters
            .windows(2)
            .any(|pair| pair[0].is_lowercase() && pair[1].is_lowercase())
}

/// Capitals that are no numeral and are not spelled out are a word, said in lowercase:
/// `İSTANBUL'U` is istanbul'u. Letters a Roman numeral is written with stay as written.
fn capitals_word(letters: &[char]) -> bool {
    letters.len() > 1
        && letters.iter().all(|letter| letter.is_uppercase())
        && !letters.iter().all(|letter| "IVXLCDM".contains(*letter))
}

/// A part of a word the lexicon reads as a currency, an abbreviation or a unit: `TL` in
/// `Dolar/TL`, `GB` in `12GB/512GB`.
fn labelled(part: &str) -> bool {
    part == "tl"
        || Currency::parse(part).is_some()
        || lexicon::abbreviation(part).is_some()
        || lexicon::measure(part).is_some()
}

/// Whether the literal reading of an ordinary word says more than its text: the word holds a
/// named symbol, its letters before any apostrophe are spelled out or are one Greek letter,
/// it joins words with a slash (`Euro/dolar`), it is a slash or the lowercase `tl`, or it is
/// only symbols the reading drops (`|`).
pub(crate) fn respells(word: &str) -> bool {
    let stem = word.split(['\'', '’']).next().unwrap_or(word);
    // Each run of letters is spelled or not on its own, as the reading says it: `T.C` is two
    // letters, `Wi-Fi` two words.
    let runs: Vec<Vec<char>> = stem
        .split(|ch: char| !ch.is_alphabetic())
        .filter(|run| !run.is_empty())
        .map(|run| run.chars().collect())
        .collect();
    // Capitals are said in lowercase, and lowercase consonants spelled, as the reading says
    // them: `SAAT` is saat, `VE` ve, `vs` ve se.
    let spelled = runs.iter().any(|run| {
        spelled_out(run, false)
            || lowercase_letters(run)
            || capitals_word(run)
            || pair(run).is_some()
            || matches!(run.as_slice(), [letter] if greek_name(*letter).is_some())
    });
    let slashed = stem.split('/').count() > 1
        && stem
            .split('/')
            .all(|part| part.chars().next().is_some_and(char::is_alphabetic));
    symbol(word).is_some()
        || spelled
        || slashed
        || matches!(word, "/" | "tl")
        || word.chars().any(|ch| DROPPED.contains(&ch))
        || (word.contains('/') && word.split('/').any(labelled))
}

/// One piece of a literal span, in written order.
#[derive(Clone, Debug)]
pub(crate) enum Part {
    /// Digit run read as a cardinal, with the apostrophe suffixes written after it.
    Number(u64, Vec<Suffix>),
    /// Separator or symbol read by its name, with the apostrophe suffixes written after it.
    Name(Word, Vec<Suffix>),
    /// Unit, currency or abbreviation the lexicon reads, with the suffixes written after it.
    Label(Lexeme, Vec<Suffix>),
    /// Letters kept as written, or spelled out by name.
    Letters(String),
    /// Whitespace and edge punctuation kept as written.
    Gap(String),
}

/// A span read as written, left to right, without inventing meaning.
#[derive(Clone, Debug)]
pub(crate) struct Literal {
    parts: Vec<Part>,
}

impl Literal {
    /// Reads any text: there is no span the literal reading rejects. A slash alone is slaş, and
    /// a face, an arrow or a decoration says nothing.
    pub(crate) fn parse(text: &str) -> Self {
        if text == "/" {
            return Self::word(SLASH);
        }
        if unspeakable(text) {
            return Self::empty();
        }
        // Said as it is said, without its slash.
        if lexicon::lookup_key(text) == "ve/veya" {
            return Self {
                parts: vec![Part::Letters(text.replace('/', " "))],
            };
        }
        if let Some(keypad) = Self::keypad(text) {
            return keypad;
        }
        let text = unemphasized(text);
        let text = text.as_ref();
        if let Some(label) = Self::abbreviated(text) {
            return label;
        }
        let span = Span::new(text);
        let core = span.core();
        let mut reading = Reading {
            parts: Vec::new(),
            amount: None,
            degree: false,
        };
        let mut index = 0;
        while index < span.graphemes.len() {
            index = if core.contains(&index) {
                span.read(index, &mut reading)
            } else {
                span.edge(index, &core, &mut reading)
            };
        }
        // A span starts and ends with what is said: whitespace a dropped mark leaves at either
        // edge goes with it (`→ adım` is adım).
        let mut parts = reading.parts;
        if let Some(Part::Gap(gap)) = parts.first_mut() {
            *gap = gap.trim_start().to_owned();
        }
        if let Some(Part::Gap(gap)) = parts.last_mut() {
            *gap = gap.trim_end().to_owned();
        }
        parts.retain(|part| !matches!(part, Part::Gap(gap) if gap.is_empty()));
        let mut literal = Self { parts };
        let measured = literal.parts.iter().any(|part| match part {
            Part::Label(..) => true,
            Part::Name(word, _) => {
                matches!(word.text, "bin" | "milyon")
                    || NAMES.iter().any(|(sign, name)| {
                        currency_sign(&sign.to_string()) && name.text == word.text
                    })
            }
            _ => false,
        });
        if measured {
            literal.decimal_point();
        }
        literal
    }
    /// A number dialled on a phone's keypad, with `*` said yıldız and `#` kare: `*123#` is
    /// yıldız yüz yirmi üç kare.
    fn keypad(text: &str) -> Option<Self> {
        let keyed = (text.starts_with('*') || text.ends_with('#'))
            && text.chars().any(|ch| ch.is_ascii_digit())
            && text
                .chars()
                .all(|ch| ch.is_ascii_digit() || matches!(ch, '*' | '#'));
        if !keyed {
            return None;
        }
        let mut parts = Vec::new();
        let mut digits = String::new();
        for key in text.chars() {
            if key.is_ascii_digit() {
                digits.push(key);
                continue;
            }
            if !digits.is_empty() {
                push_number(&mut parts, &digits);
                digits.clear();
            }
            let name = if key == '*' { STAR } else { SQUARE };
            parts.push(Part::Name(name, Vec::new()));
        }
        if !digits.is_empty() {
            push_number(&mut parts, &digits);
        }
        Some(Self { parts })
    }
    /// A word the lexicon knows as a whole, hyphen or period and all, with the suffix written
    /// after its apostrophe: `Wi-Fi’ye` is vay faya.
    fn abbreviated(text: &str) -> Option<Self> {
        let (stem, suffix) = text
            .find(['\'', '’'])
            .map_or((text, ""), |at| text.split_at(at));
        let letters = suffix.trim_start_matches(['\'', '’']);
        if !stem.contains(['-', '.']) || !letters.chars().all(char::is_alphabetic) {
            return None;
        }
        let lexeme = lexicon::abbreviation(stem)?;
        let suffixes = if letters.is_empty() {
            Vec::new()
        } else {
            vec![Suffix::new(letters)]
        };
        Some(Self {
            parts: vec![Part::Label(lexeme, suffixes)],
        })
    }
    /// A reading that says nothing, for a word that is no speech: `:D`, `***`.
    pub(crate) fn empty() -> Self {
        Self { parts: Vec::new() }
    }
    /// Says the stars of a count as what they rate, before the word they rate: `5* otel` is beş
    /// yıldızlı otel. A star sign is a star given: `5⭐ verdim` is beş yıldız verdim.
    pub(crate) fn rating(&mut self, written: &str, following: &str) {
        let word = following.trim_start();
        if !written.ends_with('*')
            || word.len() == following.len()
            || !word.starts_with(char::is_alphabetic)
        {
            return;
        }
        if let Some(Part::Name(stars, suffixes)) = self.parts.last_mut()
            && stars.text == STAR.text
            && suffixes.is_empty()
        {
            *stars = STARRED;
        }
    }
    /// A unit written right after a number that another span reads: the `mm` of
    /// `159,9 x 76,7 x 8,25 mm` is milimetre.
    pub(crate) fn unit_after_number(word: &str) -> Option<Self> {
        let mut literal = Self::parse(&format!("0 {word}"));
        match literal.parts.as_slice() {
            [Part::Number(0, suffixes), Part::Gap(_), Part::Label(..)] if suffixes.is_empty() => {
                literal.parts.drain(..2);
                Some(literal)
            }
            _ => None,
        }
    }
    /// Says the one point between two numbers as a decimal comma, for a number with a unit or
    /// a currency: `9.58 sn` is dokuz virgül elli sekiz saniye. A version, with more points,
    /// keeps them.
    pub(crate) fn decimal_point(&mut self) {
        let points: Vec<usize> = (1..self.parts.len().saturating_sub(1))
            .filter(|at| {
                matches!(&self.parts[*at], Part::Name(word, _) if word.text == "nokta")
                    && matches!(self.parts[at - 1], Part::Number(..))
                    && matches!(self.parts[at + 1], Part::Number(..))
            })
            .collect();
        if let ([point], Some(comma)) = (points.as_slice(), spoken_name(',')) {
            self.parts[*point] = Part::Name(comma, Vec::new());
        }
    }
    /// One word said for the whole span.
    pub(crate) fn word(word: Word) -> Self {
        Self::words(&[word])
    }
    /// Words said for the whole span, in order.
    pub(crate) fn words(words: &[Word]) -> Self {
        Self {
            parts: words
                .iter()
                .map(|word| Part::Name(*word, Vec::new()))
                .collect(),
        }
    }
    /// Puts whitespace or punctuation before the reading.
    pub(crate) fn open_with(&mut self, gap: &str) {
        self.parts.insert(0, Part::Gap(gap.to_owned()));
    }
    /// Puts whitespace or punctuation after the reading.
    pub(crate) fn close_with(&mut self, gap: &str) {
        push_gap(&mut self.parts, gap);
    }
    /// Says the last number as an ordinal, when a period right after the span marks it so:
    /// `90+4. dakikada` is doksan artı dördüncü dakikada. Returns whether it did, and with it
    /// whether the period is said.
    pub(crate) fn ordinal_tail(&mut self) -> bool {
        if let Some(Part::Number(_, suffixes)) = self
            .parts
            .iter_mut()
            .rev()
            .find(|part| !matches!(part, Part::Gap(_)))
            && suffixes.is_empty()
        {
            suffixes.push(Suffix::new("inci"));
            return true;
        }
        false
    }
    /// Says an `x` written right after a count as what it multiplies, by the word or number
    /// after the span: kere before a number (`3x 1.250 TL`), adet before a name (`2x Intel`)
    /// and kat before another word (`3x faydalı`). After an operator, or with no word after
    /// it, the `x` is a variable (`y = 2x`) and stays iks.
    pub(crate) fn multiplier(&mut self, written: &str, before: &str, following: &str) {
        if !multiplied(written) {
            return;
        }
        let next = following.trim_start();
        let algebra = before
            .trim_end()
            .ends_with(['=', '+', '-', '−', '×', '÷', '*', '/', '(']);
        if algebra || !next.starts_with(|ch: char| ch.is_alphanumeric() || "₺$€£".contains(ch))
        {
            return;
        }
        let said: Vec<usize> = (0..self.parts.len())
            .filter(|at| !matches!(self.parts[*at], Part::Gap(_)))
            .collect();
        let Some((&times, count)) = said.split_last() else {
            return;
        };
        let cross = match &self.parts[times] {
            Part::Letters(letters) => matches!(letters.as_str(), "iks" | "İks"),
            Part::Name(word, suffixes) => word.text == "iks" && suffixes.is_empty(),
            _ => false,
        };
        let counted = !count.is_empty()
            && count.last() == Some(&(times - 1))
            && count.iter().all(|at| match &self.parts[*at] {
                Part::Number(_, suffixes) => suffixes.is_empty(),
                Part::Name(word, _) => matches!(word.text, "nokta" | "virgül"),
                _ => false,
            });
        if !cross || !counted || !matches!(self.parts[times - 1], Part::Number(..)) {
            return;
        }
        let word = if next.starts_with(|ch: char| ch.is_ascii_digit() || "₺$€£".contains(ch)) {
            TIMES
        } else if next.starts_with(char::is_uppercase) {
            PIECES
        } else {
            FOLD
        };
        self.parts[times] = Part::Name(word, Vec::new());
        self.decimal_point();
    }
    pub(crate) fn parts(&self) -> &[Part] {
        &self.parts
    }
}

const SLASH: Word = Word::new("slaş", BackFlat, Voiceless);

/// The lexicon's reading of a word: an approved abbreviation or a currency code anywhere
/// (the lowercase `tl` too), a unit of the table but before a number (`5 GB`, `GB
/// cinsinden`, not `GB 18030`), a unit label right after a number or its power (`10⁸ m/s`),
/// and a time unit after a unit and a slash (`km/s` is kilometre slaş saniye).
fn label(word: &str, parts: &[Part], number_next: bool) -> Option<Lexeme> {
    let mut said = parts
        .iter()
        .rev()
        .filter(|part| !matches!(part, Part::Gap(_)));
    let (last, before) = (said.next(), said.next());
    let power = |word: &Word| word.text.starts_with("üssü") || matches!(word.text, "kare" | "küp");
    let after_number = match last {
        Some(Part::Number(..)) => true,
        Some(Part::Name(word, suffixes)) => suffixes.is_empty() && power(word),
        _ => false,
    };
    let per_unit = matches!((last, before), (Some(Part::Name(name, _)), Some(Part::Label(..))) if name.text == SLASH.text);
    known(word)
        .or_else(|| Currency::parse(word).map(|currency| currency.lexeme(word)))
        .or_else(|| matches!(word, "tl" | "Tl").then(|| Currency::Try.lexeme("TL")))
        .or_else(|| {
            (after_number || !number_next)
                .then(|| lexicon::measure(word))
                .flatten()
        })
        .or_else(|| after_number.then(|| unit(word)).flatten())
        .or_else(|| per_unit.then(|| per(word)).flatten())
}

/// The lexicon's reading of a word as written, or written in other case: three consonants or
/// more in lowercase or after one capital are the initialism (`Kdv` and `kdv` are KDV), and
/// anything else after one capital a lowercase abbreviation (`Bkz.` is bkz.).
fn known(word: &str) -> Option<Lexeme> {
    lexicon::abbreviation(word).or_else(|| {
        let mut chars = word.chars();
        let first = chars.next()?;
        let rest = chars.as_str();
        if rest.chars().any(char::is_uppercase) {
            return None;
        }
        let letters: Vec<char> = word.chars().filter(|ch| ch.is_alphabetic()).collect();
        if letters.len() > 2 && letters.iter().all(|letter| letter_name(*letter).is_some()) {
            return lexicon::abbreviation(&word.to_uppercase());
        }
        first
            .is_uppercase()
            .then(|| lexicon::abbreviation(&format!("{}{rest}", first.to_lowercase())))
            .flatten()
    })
}

/// A unit the lexicon spells this way, or a unit of two letters or more in other capitals:
/// `5 KG` and `5 Kg` are kilograms. A single letter is exact, since `5G` is no gram.
fn unit(word: &str) -> Option<Lexeme> {
    lexicon::unit(word).or_else(|| {
        let letters = word.chars().count();
        (letters > 1)
            .then(|| lexicon::unit(&lexicon::lookup_key(word)))
            .flatten()
    })
}

/// The time unit a single letter is after a slash: `s` saniye, `h` saat.
fn per(word: &str) -> Option<Lexeme> {
    match word {
        "s" => lexicon::unit("sn"),
        "h" => lexicon::unit("sa"),
        _ => None,
    }
}

/// A word spelled out is said letter by letter, a consonant by its name and a vowel as itself
/// (a capital `I` as İ, as an initialism says it), and the last name takes a suffix written
/// after an apostrophe, in lowercase: `X'te` is İks'te, `TK'YI` is Te Ke'yı. Two capitals
/// that make a word are said as that word (`MI` is mı, `IN` in), and another word in capitals
/// is said in lowercase.
fn spelled(word: &[&str], before_digit: bool) -> String {
    let stem = word
        .iter()
        .position(|grapheme| matches!(*grapheme, "'" | "’" | "`" | "´"))
        .unwrap_or(word.len());
    let letters: Option<Vec<char>> = word[..stem]
        .iter()
        .map(|grapheme| {
            let mut chars = grapheme.chars();
            chars.next().filter(|_| chars.next().is_none())
        })
        .collect();
    let suffix = || lexicon::lookup_key(&word[stem..].concat());
    match letters {
        Some(letters) if spelled_out(&letters, before_digit) => {
            let names: Vec<_> = letters
                .into_iter()
                .map(|letter| match letter_name(letter) {
                    Some(name) => name.to_owned(),
                    None if letter == 'I' => "İ".to_owned(),
                    None => letter.to_string(),
                })
                .collect();
            format!("{}{}", names.join(" "), suffix())
        }
        Some(letters) => match pair(&letters) {
            Some(said) => format!("{said}{}", suffix()),
            None if capitals_word(&letters) => format!("{}{}", lowered(&letters), suffix()),
            None => word.concat(),
        },
        None => word.concat(),
    }
}

/// Capitals said in lowercase, a Turkish `I` as ı, unless the word cannot be Turkish: one
/// with `W`, `Q` or `X`, or one that starts with two consonants no word starts with, is a
/// foreign word whose `I` is i (`WINDOWS` windows, `PFIZER` pfizer).
fn lowered(letters: &[char]) -> String {
    let key = lexicon::lookup_key(&String::from_iter(letters));
    let foreign = letters
        .iter()
        .any(|letter| matches!(letter, 'W' | 'Q' | 'X'))
        || matches!(letters, [first, second, ..]
            if letter_name(*first).is_some()
                && letter_name(*second).is_some()
                && !ONSETS.contains(&lexicon::lookup_key(&String::from_iter([*first, *second])).as_str()));
    if foreign { key.replace('ı', "i") } else { key }
}

/// A code such as `1Z999AA10123456784`: eight characters or more, no space, that switch
/// between letters and digits at least twice, with two letter runs or more, short and no
/// words, not all of them units or abbreviations (`12GB/512GB` is two amounts, `1920x1080` a
/// size). Its numbers are said digit by digit, except round ones.
fn code(text: &str) -> bool {
    let digits: Vec<bool> = text
        .chars()
        .filter(|ch| ch.is_alphanumeric())
        .map(|ch| ch.is_ascii_digit())
        .collect();
    let switches = digits.windows(2).filter(|pair| pair[0] != pair[1]).count();
    let runs: Vec<&str> = text
        .split(|ch: char| !ch.is_alphabetic())
        .filter(|run| !run.is_empty())
        .collect();
    let short = runs.iter().all(|run| {
        run.chars().count() <= 3 && run.chars().filter(|ch| ch.is_lowercase()).count() <= 1
    });
    let known = |run: &&str| labelled(run) || unit(run).is_some();
    text.chars().count() >= 8
        && !text.contains(char::is_whitespace)
        && switches >= 2
        && runs.len() > 1
        && short
        && !runs.iter().all(known)
}

/// A count written with one `x` glued after it: `3x`, `1.5x`, `2×`, but not `0-x`.
fn multiplied(written: &str) -> bool {
    written.strip_suffix(['x', 'X', '×']).is_some_and(|count| {
        count.starts_with(|ch: char| ch.is_ascii_digit())
            && count.ends_with(|ch: char| ch.is_ascii_digit())
            && count
                .chars()
                .all(|ch| ch.is_ascii_digit() || matches!(ch, '.' | ','))
    })
}

/// A round number in a code is said as a number, as a model's is: `i9-14900K` is i dokuz
/// tire on dört bin dokuz yüz Ke.
fn round(digits: &str) -> bool {
    (3..=6).contains(&digits.len()) && !digits.starts_with('0') && digits.ends_with("00")
}

/// Suffixes casual writing glues to a number without an apostrophe: `5te`, `3ü`, `90lar`.
fn glued(letters: &str) -> bool {
    Suffix::new(letters).family().is_some()
        || matches!(
            letters,
            "ler"
                | "lar"
                | "li"
                | "lı"
                | "lu"
                | "lü"
                | "deki"
                | "daki"
                | "teki"
                | "taki"
                | "dir"
                | "dır"
                | "dur"
                | "dür"
                | "tir"
                | "tır"
                | "tur"
                | "tür"
                | "ken"
        )
}

fn currency_sign(grapheme: &str) -> bool {
    matches!(
        grapheme,
        "₺" | "$"
            | "€"
            | "£"
            | "¥"
            | "₽"
            | "₹"
            | "₩"
            | "₼"
            | "₾"
            | "₴"
            | "₸"
            | "₿"
            | "¢"
    )
}

/// What the reading has said so far, and what the next part needs to know of it.
struct Reading {
    parts: Vec<Part>,
    /// The last number said.
    amount: Option<Amount>,
    /// A degree sign was said, so `'` and `"` after a number are minutes and seconds.
    degree: bool,
}

/// Where the last number starts among the parts, where it ends among the graphemes, and how
/// many graphemes it is written with.
#[derive(Clone, Copy)]
struct Amount {
    part: usize,
    end: usize,
    width: usize,
}

/// The graphemes of a span with their classes; a combining mark follows its base character.
struct Span<'a> {
    graphemes: Vec<&'a str>,
    classes: Vec<Class>,
    code: bool,
    /// Two hyphens or more, each of them said: `SKU-48291-BLK-XL`.
    dashed: bool,
    /// Symbols with no letter or digit, as a password's rules list them: `!@#$`.
    symbolic: bool,
}

impl<'a> Span<'a> {
    fn new(text: &'a str) -> Self {
        let graphemes: Vec<_> = text.graphemes(true).collect();
        let classes: Vec<Class> = graphemes.iter().map(|grapheme| class(grapheme)).collect();
        let dashed = graphemes
            .iter()
            .filter(|grapheme| **grapheme == "-")
            .count()
            > 1;
        let symbolic = classes.contains(&Class::Symbol)
            && !classes
                .iter()
                .any(|class| matches!(class, Class::Digit | Class::Letter));
        Self {
            graphemes,
            classes,
            code: code(text),
            dashed,
            symbolic,
        }
    }
    /// `!` and `?` among symbols only are named: `!@#` is ünlem et heşteg.
    fn mark_name(&self, index: usize) -> Option<Word> {
        match self.graphemes[index] {
            "!" if self.symbolic => Some(EXCLAMATION),
            "?" if self.symbolic => Some(QUESTION),
            _ => None,
        }
    }
    /// From the first to the last digit, letter or symbol; everything outside is an edge.
    fn core(&self) -> Range<usize> {
        let content = |class: &Class| matches!(class, Class::Digit | Class::Letter | Class::Symbol);
        match (
            self.classes.iter().position(content),
            self.classes.iter().rposition(content),
        ) {
            (Some(first), Some(last)) => first..last + 1,
            _ => 0..0,
        }
    }
    fn run_end(&self, start: usize, belongs: impl Fn(usize) -> bool) -> usize {
        (start..self.graphemes.len())
            .find(|at| !belongs(*at))
            .unwrap_or(self.graphemes.len())
    }
    fn is(&self, at: usize, class: Class) -> bool {
        self.classes.get(at) == Some(&class)
    }
    /// At an edge, whitespace and punctuation stay as written and anything else is removed.
    /// A hyphen directly before the first digit is a minus sign, and `''`, `″`, or `"` after a
    /// decimal, right after the last digit are inches: `55''` is elli beş inç.
    fn edge(&self, index: usize, core: &Range<usize>, reading: &mut Reading) -> usize {
        if index == core.end
            && let Some(marks) = self.inch_marks(index, core)
        {
            reading.parts.push(Part::Name(INCH, Vec::new()));
            return index + marks;
        }
        if let Some(word) = self.mark_name(index) {
            reading.parts.push(Part::Name(word, Vec::new()));
            return index + 1;
        }
        // An apostrophe after the last number with no suffix is a minute's mark, said with the
        // number: a goal at `12'` is on iki. After a quote that opens the span it closes it:
        // `'5'` keeps both.
        if index == core.end
            && matches!(self.graphemes[index], "'" | "’")
            && !core.is_empty()
            && self.is(core.end - 1, Class::Digit)
            && !matches!(self.graphemes.first(), Some(&("'" | "’" | "‘")))
        {
            return index + 1;
        }
        let minus = self.graphemes[index] == "-"
            && index + 1 == core.start
            && self.is(core.start, Class::Digit);
        if minus {
            reading
                .parts
                .extend(spoken_name('−').map(|word| Part::Name(word, Vec::new())));
        } else if self.classes[index] != Class::Other {
            push_gap(&mut reading.parts, self.graphemes[index]);
        }
        index + 1
    }
    fn inch_marks(&self, index: usize, core: &Range<usize>) -> Option<usize> {
        if core.is_empty() || !self.is(core.end - 1, Class::Digit) {
            return None;
        }
        let decimal = self.graphemes[core.clone()]
            .iter()
            .any(|grapheme| matches!(*grapheme, "." | ","));
        match (self.graphemes.get(index), self.graphemes.get(index + 1)) {
            (Some(&"'"), Some(&"'")) | (Some(&"’"), Some(&"’")) => Some(2),
            (Some(&"″"), _) => Some(1),
            (Some(&"\""), _) if decimal => Some(1),
            _ => None,
        }
    }
    /// Reads the part that starts at `index` inside the core and returns where the next starts.
    fn read(&self, index: usize, reading: &mut Reading) -> usize {
        if matches!(self.classes[index], Class::Symbol | Class::Other) {
            if let Some(next) = self.footnote(index) {
                return next;
            }
            if let Some(next) = self.exponent(index, reading) {
                return next;
            }
        }
        match self.classes[index] {
            Class::Digit => self.amount(index, reading),
            Class::Letter => self.word(index, reading),
            Class::Space => {
                push_gap(&mut reading.parts, self.graphemes[index]);
                index + 1
            }
            Class::Separator
                if self.initialism(index)
                    || self.joins_word_and_number(index)
                    || self.joins_words(index)
                    || self.after_greek(index) =>
            {
                index + 1
            }
            // A period, comma or colon followed by a space closes a phrase: `5, a` is beş, a.
            Class::Separator
                if matches!(self.graphemes[index], "." | "," | ":")
                    && self.is(index + 1, Class::Space) =>
            {
                push_gap(&mut reading.parts, self.graphemes[index]);
                index + 1
            }
            Class::Separator if self.punctuates(index) => {
                push_gap(&mut reading.parts, self.graphemes[index]);
                push_gap(&mut reading.parts, " ");
                index + 1
            }
            Class::Separator | Class::Symbol => self.sign(index, reading),
            Class::Apostrophe => self.apostrophe(index, reading),
            Class::Punctuation => self.inner_punctuation(index, reading),
            Class::Other => index + 1,
        }
    }
    fn digits(&self, run: Range<usize>) -> String {
        self.graphemes[run]
            .iter()
            .filter_map(|grapheme| ascii_digit(grapheme))
            .collect()
    }
    /// A number. In groups of thousands (`1.500`, `9.876,54`, `1,000,000`, `1 000 000`) it is
    /// one amount; in a code its digits are said one by one. A currency sign written before
    /// it is said after it, and suffix letters glued to it are its suffix: `5te` is beşte.
    fn amount(&self, index: usize, reading: &mut Reading) -> usize {
        let start = reading.parts.len();
        let run_end = self.run_end(index, |at| self.is(at, Class::Digit));
        let end = if self.code {
            let digits = self.digits(index..run_end);
            if round(&digits) {
                push_number(&mut reading.parts, &digits);
            } else {
                for digit in digits.chars() {
                    push_number(&mut reading.parts, &digit.to_string());
                }
            }
            run_end
        } else if let Some((end, integer, fraction)) = self.grouped(index, run_end) {
            match Number::parse(&integer) {
                Some(number) => reading
                    .parts
                    .push(Part::Number(number.integer(), Vec::new())),
                None => push_digits(&mut reading.parts, &integer),
            }
            if let Some(fraction) = fraction {
                reading
                    .parts
                    .extend(spoken_name(',').map(|word| Part::Name(word, Vec::new())));
                push_fraction(&mut reading.parts, &fraction);
            }
            end
        } else {
            let digits = self.digits(index..run_end);
            // The digits after a decimal comma are a fraction: `6,022` is altı virgül sıfır
            // yirmi iki.
            let decimal = index > 0
                && self.graphemes[index - 1] == ","
                && matches!(reading.parts.as_slice(), [.., Part::Number(..), Part::Name(word, _)] if word.text == "virgül");
            // Minutes and seconds of a degree keep no leading zero: `41°05'` is beş dakika.
            if decimal {
                push_fraction(&mut reading.parts, &digits);
            } else if reading.degree && digits.len() <= 2 {
                let trimmed = digits.trim_start_matches('0');
                push_number(
                    &mut reading.parts,
                    if trimmed.is_empty() { "0" } else { trimmed },
                );
            } else {
                push_number(&mut reading.parts, &digits);
            }
            if let Some(next) = self.ordinal_word(index, run_end, reading) {
                return next;
            }
            run_end
        };
        let mut part = start;
        if start > 0
            && index > 0
            && currency_sign(self.graphemes[index - 1])
            && matches!(reading.parts[start - 1], Part::Name(..))
        {
            let currency = reading.parts.remove(start - 1);
            reading.parts.push(currency);
            part = start - 1;
        }
        reading.amount = Some(Amount {
            part,
            end,
            width: end - index,
        });
        self.glued_suffix(end, reading)
    }
    /// The digits of a number written in groups of thousands, and of its fraction: Turkish
    /// `9.876,54`, English `1,299.99` and `1,000,000`, or `1 000 000`. A first group of one to
    /// three digits must be followed by whole groups of three, so `2.5.1` and `192.168.1.1`
    /// are no such number, and one English or spaced group needs a fraction or zeros.
    fn grouped(&self, index: usize, run_end: usize) -> Option<(usize, String, Option<String>)> {
        if !(1..=3).contains(&(run_end - index)) || self.graphemes[index] == "0" {
            return None;
        }
        for (thousands, decimal) in [(".", ","), (",", "."), (" ", "")] {
            let mut end = run_end;
            let mut groups = 0;
            while self.graphemes.get(end) == Some(&thousands) && self.group_of_three(end + 1) {
                end += 4;
                groups += 1;
            }
            if groups == 0
                || (self.graphemes.get(end) == Some(&thousands) && self.is(end + 1, Class::Digit))
            {
                continue;
            }
            let fraction_end = (!decimal.is_empty()
                && self.graphemes.get(end) == Some(&decimal)
                && self.is(end + 1, Class::Digit))
            .then(|| self.run_end(end + 1, |at| self.is(at, Class::Digit)));
            let enough = match thousands {
                "." => true,
                "," => groups > 1 || fraction_end.is_some(),
                _ => groups > 1 || self.digits(end - 3..end) == "000",
            };
            if !enough {
                continue;
            }
            let integer = self.digits(index..end);
            let fraction = fraction_end.map(|fraction_end| self.digits(end + 1..fraction_end));
            return Some((fraction_end.unwrap_or(end), integer, fraction));
        }
        None
    }
    /// A number that starts a word with a period glued after it, before lowercase letters, is
    /// an ordinal: the letters are its suffix when they are one (`1.si` birincisi, `4.lük`
    /// dördüncülük) and the word after it otherwise (`3.kat` üçüncü kat, `21.yy'ın` yirmi
    /// birinci yüzyılın).
    fn ordinal_word(&self, index: usize, end: usize, reading: &mut Reading) -> Option<usize> {
        let starts_word =
            index == 0 || matches!(self.classes[index - 1], Class::Space | Class::Punctuation);
        if !starts_word || self.graphemes.get(end) != Some(&".") {
            return None;
        }
        let letters_end = self.run_end(end + 1, |at| self.is(at, Class::Letter));
        let letters = self.graphemes[end + 1..letters_end].concat();
        // A single letter is a label or a version, not a word: `2.a` is iki nokta a, `3.x`
        // üç nokta iks.
        if letters_end < end + 3 || !letters.chars().all(char::is_lowercase) {
            return None;
        }
        let Some(Part::Number(_, suffixes)) = reading.parts.last_mut() else {
            return None;
        };
        if !suffixes.is_empty() {
            return None;
        }
        suffixes.push(Suffix::new("inci"));
        let family = Suffix::new(&letters).family();
        if family == Some(Inflection::Ordinal) {
            return Some(letters_end);
        }
        let suffix = family.is_some()
            || glued(&letters)
            || matches!(
                letters.as_str(),
                "si" | "sı" | "su" | "sü" | "yi" | "yı" | "yu" | "yü"
            );
        if suffix && !self.is(letters_end, Class::Apostrophe) {
            suffixes.push(Suffix::new(&letters));
            return Some(letters_end);
        }
        Some(end + 1)
    }
    fn group_of_three(&self, at: usize) -> bool {
        (at..at + 3).all(|digit| self.is(digit, Class::Digit)) && !self.is(at + 3, Class::Digit)
    }
    fn glued_suffix(&self, end: usize, reading: &mut Reading) -> usize {
        if !self.is(end, Class::Letter) {
            return end;
        }
        let letters_end = self.run_end(end, |at| self.is(at, Class::Letter));
        let letters = self.graphemes[end..letters_end].concat();
        if !letters.chars().all(char::is_lowercase) || !glued(&letters) {
            return end;
        }
        match reading.parts.last_mut() {
            Some(Part::Number(_, suffixes)) => {
                suffixes.push(Suffix::new(&letters));
                letters_end
            }
            _ => end,
        }
    }
    /// A word the lexicon reads becomes a label; any other is kept, or spelled out by name.
    fn word(&self, index: usize, reading: &mut Reading) -> usize {
        let stem = self.run_end(index, |at| self.is(at, Class::Letter));
        let letters = self.graphemes[index..stem].concat();
        // An abbreviation written with its period or colon, as a word of its own: `md.'si`,
        // `No:4`, `D:3`, but not the `D:` of `4D:5E`. A number right after it is the one some
        // are read before.
        let numbered = |written: &str| {
            let digit = self
                .graphemes
                .get(stem + 1)
                .copied()
                .filter(|grapheme| ascii_digit(grapheme).is_some());
            digit.and_then(|digit| {
                lexicon::abbreviation_around(
                    written,
                    lexicon::Around {
                        previous: None,
                        next: Some(digit),
                    },
                )
            })
        };
        if !(index > 0 && self.is(index - 1, Class::Digit))
            && let Some(mark) = self
                .graphemes
                .get(stem)
                .filter(|mark| matches!(**mark, "." | ":"))
            && let Some(lexeme) =
                known(&format!("{letters}{mark}")).or_else(|| numbered(&format!("{letters}{mark}")))
        {
            reading.parts.push(Part::Label(lexeme, Vec::new()));
            return stem + 1;
        }
        // One of two letters or more written without its period before a suffix: `yy'ın` is
        // yüzyılın. A single letter is no such abbreviation: the `s` of `m/s'dir` is saniye.
        if stem > index + 1
            && self.is(stem, Class::Apostrophe)
            && lexicon::abbreviation(&letters).is_none()
            && let Some(lexeme) = lexicon::abbreviation(&format!("{letters}."))
        {
            reading.parts.push(Part::Label(lexeme, Vec::new()));
            return stem;
        }
        // A Greek letter alone is said by its Turkish name: `β` is beta.
        if stem == index + 1
            && let Some(name) = letters.chars().next().and_then(greek_name)
        {
            reading.parts.push(Part::Name(name, Vec::new()));
            return stem;
        }
        if let Some(lexeme) = self.powered_unit(&letters, stem) {
            reading.parts.push(Part::Label(lexeme, Vec::new()));
            return stem + 1;
        }
        // A domain's `com` after a point is said as people say it: `.com.tr` is nokta kom
        // nokta te re.
        if letters == "com"
            && matches!(reading.parts.last(), Some(Part::Name(word, _)) if word.text == "nokta")
        {
            let end = self.run_end(index, |at| self.in_word(at));
            let suffix = self.graphemes[stem..end].concat();
            reading.parts.push(Part::Letters(format!("kom{suffix}")));
            return end;
        }
        let before_digit = self.is(stem, Class::Digit);
        if letters == "v" && before_digit {
            reading.parts.push(Part::Name(VERSION, Vec::new()));
            return stem;
        }
        // `45K` and `1M` count thousands and millions, `4K` stays a resolution and `14900K`
        // a model; `65W` is watts, `5V` volts and `200MP` megapixels. A code has no such
        // amounts.
        if let Some(amount) = reading
            .amount
            .filter(|amount| !self.code && amount.end == index)
        {
            // The fraction of a decimal count: `1.5K` is bir virgül beş bin.
            let fraction = amount.part >= 2
                && matches!(&reading.parts[amount.part - 1], Part::Name(word, _) if matches!(word.text, "nokta" | "virgül"))
                && matches!(reading.parts[amount.part - 2], Part::Number(..));
            let scale = match letters.as_str() {
                "K" if fraction || (2..=3).contains(&amount.width) => Some(THOUSAND),
                "M" if amount.width <= 3 => Some(MILLION),
                "W" => Some(WATT),
                "V" => Some(VOLT),
                "MP" => Some(MEGAPIXEL),
                _ => None,
            };
            if let Some(scale) = scale {
                reading.parts.push(Part::Name(scale, Vec::new()));
                return stem;
            }
        }
        let number_next = self.is(
            self.run_end(stem, |at| self.is(at, Class::Space)),
            Class::Digit,
        );
        if let Some(lexeme) = label(&letters, &reading.parts, number_next) {
            reading.parts.push(Part::Label(lexeme, Vec::new()));
            return stem;
        }
        // Lowercase consonants are a unit (`88/dk`) or letters (`tcp`, `.tr`).
        if stem - index > 1
            && letters
                .chars()
                .all(|letter| letter.is_lowercase() && letter_name(letter).is_some())
        {
            let part = match unit(&letters) {
                Some(lexeme) => Part::Label(lexeme, Vec::new()),
                None => Part::Letters(
                    letters
                        .chars()
                        .filter_map(letter_name)
                        .collect::<Vec<_>>()
                        .join(" "),
                ),
            };
            reading.parts.push(part);
            return stem;
        }
        let end = self.run_end(index, |at| self.in_word(at));
        reading.parts.push(Part::Letters(spelled(
            &self.graphemes[index..end],
            before_digit,
        )));
        end
    }
    /// A unit with its power, read wherever it stands: `m²` and `m2` are metrekare, `km2`
    /// kilometrekare and `m³` metreküp. A digit is the power only when no digit follows it.
    fn powered_unit(&self, letters: &str, at: usize) -> Option<Lexeme> {
        let power = match *self.graphemes.get(at)? {
            "²" | "2" => '²',
            "³" | "3" => '³',
            _ => return None,
        };
        if self.is(at + 1, Class::Digit) {
            return None;
        }
        lexicon::unit(&format!("{letters}{power}"))
    }
    /// An apostrophe between two letters belongs to its word.
    fn in_word(&self, at: usize) -> bool {
        self.is(at, Class::Letter)
            || (self.is(at, Class::Apostrophe) && self.is(at + 1, Class::Letter))
    }
    /// A letter with no letter on either side.
    fn lone_letter(&self, at: usize) -> bool {
        self.is(at, Class::Letter)
            && !(at > 0 && self.is(at - 1, Class::Letter))
            && !self.is(at + 1, Class::Letter)
    }
    /// A period between two single letters is not said: `T.C` is two letters.
    fn initialism(&self, index: usize) -> bool {
        self.graphemes[index] == "."
            && index > 0
            && self.lone_letter(index - 1)
            && self.lone_letter(index + 1)
    }
    /// A hyphen between two words is not said: `E-POSTA` is e posta, `T-shirt` Te shirt. Between
    /// two single letters it is a range (`A-Z`), and in a code with more hyphens each is said.
    fn joins_words(&self, index: usize) -> bool {
        self.graphemes[index] == "-"
            && !self.dashed
            && index > 0
            && self.is(index - 1, Class::Letter)
            && self.is(index + 1, Class::Letter)
            && !(self.lone_letter(index - 1) && self.lone_letter(index + 1))
    }
    /// Whether a number, a lone letter or a bracket comes before `index`, past whitespace.
    fn operand_before(&self, index: usize) -> bool {
        let at = (0..index)
            .rev()
            .find(|at| !self.is(*at, Class::Space))
            .unwrap_or(index);
        at < index
            && (self.is(at, Class::Digit)
                || matches!(self.graphemes[at], ")" | "]")
                || self.lone_letter(at))
    }
    /// Whether a number, a lone letter or a bracket comes after `index`, past whitespace; a
    /// comparison may be followed by `=` too.
    fn operand_after(&self, index: usize, comparison: bool) -> bool {
        let at = self.run_end(index + 1, |at| self.is(at, Class::Space));
        self.is(at, Class::Digit)
            || matches!(self.graphemes.get(at), Some(&("(" | "[")))
            || (comparison && self.graphemes.get(at) == Some(&"="))
            || self.lone_letter(at)
    }
    /// Superscript digits written right after a word, two letters or more with a vowel, are
    /// its footnote, which says nothing: `Yazar¹`, `kaynak²³`. After a number, a single letter,
    /// letters of a formula or a unit they are a power: `10²`, `x²`, `mc²`, `cm³`.
    fn footnote(&self, index: usize) -> Option<usize> {
        if superscript(self.graphemes[index]).is_none()
            || !self.is(index.checked_sub(1)?, Class::Letter)
        {
            return None;
        }
        let start = (0..index)
            .rev()
            .take_while(|at| self.is(*at, Class::Letter))
            .last()?;
        let letters = self.graphemes[start..index].concat();
        let vowel = letters
            .chars()
            .any(|letter| letter.is_alphabetic() && letter_name(letter).is_none());
        if index - start < 2 || !vowel || unit(&letters).is_some() {
            return None;
        }
        Some(self.run_end(index, |at| superscript(self.graphemes[at]).is_some()))
    }
    /// A hyphen between a word and a number is not said: `COVID-19` is kovid on dokuz. In a
    /// code with more hyphens, each is said.
    fn joins_word_and_number(&self, index: usize) -> bool {
        let class = |at: usize| self.classes.get(at).copied();
        self.graphemes[index] == "-"
            && !self.dashed
            && index > 0
            && matches!(
                (class(index - 1), class(index + 1)),
                (Some(Class::Letter), Some(Class::Digit))
                    | (Some(Class::Digit), Some(Class::Letter))
            )
    }
    /// A colon or comma written with no space between a word and a number is punctuation, not
    /// a separator to name: `Fiyat:1.250TL` is Fiyat: bin iki yüz elli lira. Letters glued to
    /// a digit are no word, so the colons of `00:1A:2B` are named.
    fn punctuates(&self, index: usize) -> bool {
        let content = |at: usize| self.is(at, Class::Digit) || self.is(at, Class::Symbol);
        index > 0
            && match self.graphemes[index] {
                ":" => {
                    self.is(index - 1, Class::Letter)
                        && !self.after_digit(index - 1)
                        && content(index + 1)
                }
                "," => {
                    (self.is(index - 1, Class::Letter) && content(index + 1))
                        || (content(index - 1) && self.is(index + 1, Class::Letter))
                }
                _ => false,
            }
    }
    /// Whether the letters that end at `at` start right after a digit, as in `1A`.
    fn after_digit(&self, at: usize) -> bool {
        let start = (0..=at)
            .rev()
            .take_while(|letter| self.is(*letter, Class::Letter))
            .last()
            .unwrap_or(at);
        start > 0 && self.is(start - 1, Class::Digit)
    }
    /// `#` before a number or a code is numara: `#123`, `#TR-2026-000123`. Before a word it is
    /// a hashtag.
    fn numbered(&self, index: usize) -> bool {
        let end = self.run_end(index + 1, |at| {
            self.is(at, Class::Digit) || self.is(at, Class::Letter) || self.graphemes[at] == "-"
        });
        let code = &self.graphemes[index + 1..end];
        code.iter().any(|grapheme| ascii_digit(grapheme).is_some())
            && !code
                .iter()
                .any(|grapheme| grapheme.chars().any(char::is_lowercase))
            && (code.iter().all(|grapheme| ascii_digit(grapheme).is_some()) || code.contains(&"-"))
    }
    /// A power written in two superscript digits or more, or with a superscript minus: `10²³`
    /// is on üssü yirmi üç, `10⁻³` on üssü eksi üç. One superscript digit is named alone.
    fn exponent(&self, index: usize, reading: &mut Reading) -> Option<usize> {
        let minus = self.graphemes[index] == "⁻";
        let start = index + usize::from(minus);
        let end = self.run_end(start, |at| superscript(self.graphemes[at]).is_some());
        if end == start || (!minus && end - start < 2) {
            return None;
        }
        reading.parts.push(Part::Name(POWER, Vec::new()));
        if minus {
            reading
                .parts
                .extend(spoken_name('−').map(|word| Part::Name(word, Vec::new())));
        }
        let digits: String = self.graphemes[start..end]
            .iter()
            .filter_map(|grapheme| superscript(grapheme))
            .filter_map(|digit| char::from_digit(u32::try_from(digit).ok()?, 10))
            .collect();
        push_number(&mut reading.parts, &digits);
        Some(end)
    }
    /// A hyphen after a Greek letter alone is not said: `β-karoten` is beta karoten.
    fn after_greek(&self, index: usize) -> bool {
        self.graphemes[index] == "-"
            && index > 0
            && self.lone_letter(index - 1)
            && self.graphemes[index - 1]
                .chars()
                .next()
                .and_then(greek_name)
                .is_some()
            && self.is(index + 1, Class::Letter)
    }
    /// `°C` and `°F` are one sign: the letter after the degree is the scale.
    fn scale(&self, index: usize, letter: &str) -> bool {
        self.graphemes[index] == "°"
            && self.graphemes.get(index + 1) == Some(&letter)
            && self.lone_letter(index + 1)
    }
    /// Whether only whitespace lies between the last number and `index`.
    fn after_amount(&self, index: usize, reading: &Reading) -> bool {
        reading.amount.is_some_and(|amount| {
            amount.end <= index && (amount.end..index).all(|at| self.is(at, Class::Space))
        })
    }
    /// A separator or symbol by its name. `%` after a number is said before it, as Turkish
    /// says a percentage (`5%` is yüzde beş); `~` is yaklaşık only before a number, and `^`
    /// is üssü between two numbers or letters and şapka elsewhere; `½` after a digit is its
    /// half. `*`, `<` and `>` are said only between two numbers or single letters, or a
    /// comparison before a number, and `_` only inside a word: around words they are marks of
    /// emphasis, a footnote or a menu path and say nothing (`**Önemli**`, `Fiyat*`,
    /// `_vurgu_`). A `*` right after a count is its stars: `5*` is beş yıldız. Among symbols
    /// only, each is named.
    fn sign(&self, index: usize, reading: &mut Reading) -> usize {
        let grapheme = self.graphemes[index];
        let next_digit = self.is(index + 1, Class::Digit);
        let operand = |at: usize| self.is(at, Class::Digit) || self.is(at, Class::Letter);
        let sign = symbol(grapheme).or_else(|| grapheme.chars().next());
        let silent = !self.symbolic
            && match sign {
                Some('*') => !(self.operand_before(index) && self.operand_after(index, false)),
                Some('<' | '>') => {
                    !(self.operand_before(index) && self.operand_after(index, true))
                        && !self.is(
                            self.run_end(index + 1, |at| self.is(at, Class::Space)),
                            Class::Digit,
                        )
                }
                Some('_') => !(index > 0 && operand(index - 1) && operand(index + 1)),
                _ => false,
            };
        if silent {
            if sign == Some('*') && index > 0 && self.is(index - 1, Class::Digit) {
                reading.parts.push(Part::Name(STAR, Vec::new()));
            }
            return index + 1;
        }
        match sign {
            Some('%') if !next_digit && self.after_amount(index, reading) => {
                if let Some(amount) = reading.amount {
                    while matches!(reading.parts.last(), Some(Part::Gap(gap)) if gap.trim().is_empty())
                    {
                        reading.parts.pop();
                    }
                    let percent = spoken_name('%').map(|word| Part::Name(word, Vec::new()));
                    if let Some(percent) = percent {
                        reading
                            .parts
                            .insert(amount.part.min(reading.parts.len()), percent);
                    }
                }
                return index + 1;
            }
            Some('~') if !next_digit => return index + 1,
            Some('^') if !(index > 0 && operand(index - 1) && operand(index + 1)) => {
                reading.parts.push(Part::Name(CARET, Vec::new()));
                return index + 1;
            }
            Some('°' | '℃' | '℉') => reading.degree = true,
            _ => {}
        }
        let fahrenheit = self.scale(index, "F");
        let word = match sign {
            Some('½') if index > 0 && self.is(index - 1, Class::Digit) => Some(AND_A_HALF),
            Some('°') if fahrenheit => Some(FAHRENHEIT),
            Some('#') if self.numbered(index) => Some(NUMBER_SIGN),
            Some(sign) => spoken_name(sign),
            None => None,
        };
        reading
            .parts
            .extend(word.map(|word| Part::Name(word, Vec::new())));
        index
            + if fahrenheit || self.scale(index, "C") {
                2
            } else {
                1
            }
    }
    /// An apostrophe gives the letters after it to the spoken part before it; between two
    /// numbers after a degree it marks minutes.
    fn apostrophe(&self, index: usize, reading: &mut Reading) -> usize {
        if reading.degree && self.after_amount(index, reading) && self.is(index + 1, Class::Digit) {
            reading.parts.push(Part::Name(MINUTE, Vec::new()));
            return index + 1;
        }
        self.suffix(index, &mut reading.parts).unwrap_or(index + 1)
    }
    /// Brackets and quotes inside a span stay as written, set apart from the words; `"` after
    /// a number in a coordinate is seconds, and `–` between two numbers is a dash.
    fn inner_punctuation(&self, index: usize, reading: &mut Reading) -> usize {
        let grapheme = self.graphemes[index];
        let between_digits =
            index > 0 && self.is(index - 1, Class::Digit) && self.is(index + 1, Class::Digit);
        if let Some(word) = self.mark_name(index) {
            reading.parts.push(Part::Name(word, Vec::new()));
        } else if matches!(grapheme, "\"" | "″")
            && reading.degree
            && self.after_amount(index, reading)
        {
            reading.parts.push(Part::Name(SECOND, Vec::new()));
        } else if grapheme == "–" && between_digits {
            reading
                .parts
                .extend(spoken_name('-').map(|word| Part::Name(word, Vec::new())));
        } else if matches!(grapheme, "(" | "[" | "{" | "«" | "“" | "‘") {
            if !matches!(reading.parts.last(), None | Some(Part::Gap(_))) {
                push_gap(&mut reading.parts, " ");
            }
            push_gap(&mut reading.parts, grapheme);
        } else {
            push_gap(&mut reading.parts, grapheme);
            let content = self.is(index + 1, Class::Letter)
                || self.is(index + 1, Class::Digit)
                || self.is(index + 1, Class::Symbol);
            if content {
                push_gap(&mut reading.parts, " ");
            }
        }
        index + 1
    }
    /// Gives the letters after the apostrophe at `index` to the spoken part before it.
    fn suffix(&self, index: usize, parts: &mut [Part]) -> Option<usize> {
        let (Part::Number(_, suffixes) | Part::Name(_, suffixes) | Part::Label(_, suffixes)) =
            parts.last_mut()?
        else {
            return None;
        };
        let end = self.run_end(index + 1, |at| self.is(at, Class::Letter));
        let letters = self.graphemes.get(index + 1..end)?.concat();
        if letters.is_empty() {
            return None;
        }
        suffixes.push(Suffix::new(&letters));
        Some(end)
    }
}

fn push_gap(parts: &mut Vec<Part>, written: &str) {
    if let Some(Part::Gap(gap)) = parts.last_mut() {
        gap.push_str(written);
    } else {
        parts.push(Part::Gap(written.to_owned()));
    }
}

/// The digits after a decimal comma, as the spoken style says them: leading zeros one by one
/// and up to three digits after them as a number, more one by one: `6,022` is altı virgül
/// sıfır yirmi iki.
fn push_fraction(parts: &mut Vec<Part>, run: &str) {
    let zeros = run.len() - run.trim_start_matches('0').len();
    let rest = &run[zeros..];
    if (1..=3).contains(&rest.len()) {
        push_digits(parts, &run[..zeros]);
        push_number(parts, rest);
    } else {
        push_digits(parts, run);
    }
}

fn push_digits(parts: &mut Vec<Part>, run: &str) {
    parts.extend(
        run.chars()
            .filter_map(|digit| digit.to_digit(10))
            .map(|digit| Part::Number(u64::from(digit), Vec::new())),
    );
}

/// Digits that make an identifier rather than an amount: seven or more.
const IDENTIFIER_DIGITS: usize = 7;

/// One cardinal, or digit by digit for a zero-padded run, one beyond the cardinal reader or
/// an identifier. A ten-digit identifier that starts with 5 is a mobile number, said in its
/// groups: `5321234567` is beş yüz otuz iki yüz yirmi üç kırk beş altmış yedi.
fn push_number(parts: &mut Vec<Part>, run: &str) {
    if run.len() == 10 && run.starts_with('5') {
        for group in [&run[..3], &run[3..6], &run[6..8], &run[8..]] {
            push_number(parts, group);
        }
        return;
    }
    match Number::parse(run).filter(|_| !identifier(run)) {
        Some(number) => parts.push(Part::Number(number.integer(), Vec::new())),
        None => push_digits(parts, run),
    }
}

/// Whether a plain digit run is an identifier: seven digits or more, not zero-padded, and
/// not a round amount ending in thousands, which the literal reading says digit by digit.
pub(crate) fn identifier(text: &str) -> bool {
    text.len() >= IDENTIFIER_DIGITS
        && !text.starts_with('0')
        && !text.ends_with("000")
        && text.bytes().all(|byte| byte.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{morphology::Style, verbalize::literal_text};

    fn read(text: &str) -> String {
        literal_text(&Literal::parse(text), Style::Spoken)
    }

    #[test]
    fn digit_runs_are_cardinals_unless_zero_padded_or_beyond_the_cardinal_reader() {
        for (written, spoken) in [
            ("0", "sıfır"),
            ("10", "on"),
            ("00", "sıfır sıfır"),
            ("007", "sıfır sıfır yedi"),
            (
                "999999",
                "dokuz yüz doksan dokuz bin dokuz yüz doksan dokuz",
            ),
            // Seven digits or more are an identifier; ten that start with 5 a mobile number.
            ("1234567", "bir iki üç dört beş altı yedi"),
            (
                "5321234567",
                "beş yüz otuz iki yüz yirmi üç kırk beş altmış yedi",
            ),
            (
                "5321230567",
                "beş yüz otuz iki yüz yirmi üç sıfır beş altmış yedi",
            ),
            (
                "1000000000000000000",
                "bir sıfır sıfır sıfır sıfır sıfır sıfır sıfır sıfır sıfır sıfır sıfır sıfır sıfır sıfır sıfır sıfır sıfır sıfır",
            ),
        ] {
            assert_eq!(read(written), spoken, "{written}");
        }
    }

    #[test]
    fn separators_are_named_between_parts_and_kept_at_the_edges() {
        for (written, spoken) in [
            ("2.5.1", "iki nokta beş nokta bir"),
            ("1,5", "bir virgül beş"),
            ("24:00", "yirmi dört iki nokta sıfır sıfır"),
            ("10-15", "on tire on beş"),
            ("1/2", "bir slaş iki"),
            ("1 / 2", "bir slaş iki"),
            ("24/7", "yirmi dört slaş yedi"),
            ("km/sn", "kilometre slaş saniye"),
            ("5/ab", "beş slaş ab"),
            // A hyphen between a word and a number is not said.
            ("AB-12", "a be on iki"),
            ("F-16", "Fe on altı"),
            ("12-B", "on iki Be"),
            ("25.", "yirmi beş."),
            ("12..", "on iki.."),
            ("12,,", "on iki,,"),
            ("5-", "beş-"),
            ("- 5", "- beş"),
            ("(12)34", "(on iki) otuz dört"),
            ("1, 2, 3", "bir, iki, üç"),
            ("...", "..."),
        ] {
            assert_eq!(read(written), spoken, "{written}");
        }
    }

    #[test]
    fn every_symbol_in_the_table_is_read_by_its_name() {
        // `~` is named only before a number and `^` only between two. Between two words a
        // hyphen, `<`, `>` and a superscript say nothing, and `*` marks emphasis inside a word,
        // which then is one word.
        let unsaid = |symbol: char| {
            matches!(symbol, '-' | '<' | '>') || superscript(&symbol.to_string()).is_some()
        };
        for (symbol, word) in NAMES
            .iter()
            .filter(|(symbol, _)| !matches!(symbol, '~' | '^'))
        {
            if !SEPARATORS.contains(symbol) {
                assert_eq!(read(&symbol.to_string()), word.text, "{symbol}");
            }
            let glued = if *symbol == '*' {
                "abel".to_owned()
            } else if unsaid(*symbol) {
                "ab el".to_owned()
            } else {
                format!("ab {} el", word.text)
            };
            assert_eq!(read(&format!("ab{symbol}el")), glued, "{symbol}");
        }
        assert_eq!(read("a*b"), "a çarpı be");
        assert_eq!(read("**kişi**den"), "kişiden");
        assert_eq!(read("kullanın.*"), "kullanın.");
        assert_eq!(read("__APPNAME__"), "appname");
        assert_eq!(read("E=mc²"), "E eşittir me ce kare");
        assert_eq!(read("Yazar¹"), "Yazar");
        assert_eq!(read("A-Z"), "A tire Ze");
        assert_eq!(read("E-POSTA"), "E posta");
        assert_eq!(read("x<y"), "iks küçüktür ye");
        assert_eq!(read("<50"), "küçüktür elli");
        assert_eq!(read("~3 dk"), "yaklaşık üç dakika");
        assert_eq!(read("10^6"), "on üssü altı");
        assert_eq!(
            read("x ≥ 5, a² ≠ √16 ± 1, π ≈ 3"),
            "iks büyük eşittir beş, a kare eşit değildir karekök on altı artı eksi bir, pi yaklaşık üç"
        );
        assert_eq!(
            read("3 * 4 ÷ 2 < 7 > 5"),
            "üç çarpı dört bölü iki küçüktür yedi büyüktür beş"
        );
        assert_eq!(read("2× 30°"), "iki iks otuz derece");
        assert_eq!(read("ab_el#ef"), "ab alt çizgi el heşteg ef");
        assert_eq!(
            read("₺$€£%&@#+=/*<>°_|"),
            "lira dolar avro sterlin yüzde ve et heşteg artı eşittir slaş çarpı küçüktür büyüktür derece alt çizgi"
        );
    }

    #[test]
    fn a_minus_a_degree_and_a_fraction_are_read_for_what_they_are() {
        for (written, spoken) in [
            ("-5", "eksi beş"),
            ("-5:", "eksi beş:"),
            ("(-5)", "(eksi beş)"),
            ("−5", "eksi beş"),
            ("5°C", "beş derece"),
            ("-5°C'den", "eksi beş dereceden"),
            ("12°C'ye", "on iki dereceye"),
            ("5℃", "beş derece"),
            ("5°Ce", "beş derece Ce"),
            ("70 °F", "yetmiş derece Fahrenhayt"),
            ("70°F'de", "yetmiş derece Fahrenhaytta"),
            ("350℉", "üç yüz elli derece Fahrenhayt"),
            ("½", "yarım"),
            ("¾ fincan", "üç çeyrek fincan"),
            ("¼", "çeyrek"),
            ("2½", "iki buçuk"),
            ("2½'ta", "iki buçukta"),
        ] {
            assert_eq!(read(written), spoken, "{written}");
        }
    }

    #[test]
    fn a_single_letter_and_capital_consonants_are_said_by_name() {
        for (written, spoken) in [
            ("A", "A"),
            ("B", "Be"),
            ("C", "Ce"),
            ("D", "De"),
            ("b", "be"),
            ("ş", "şe"),
            ("Ğ", "Yumuşak ge"),
            ("q", "kü"),
            ("W", "Çift ve"),
            ("x", "iks"),
            ("X", "İks"),
            ("o", "o"),
            ("H2O", "He iki O"),
            ("CO2", "Ce O iki"),
            ("4K", "dört Ke"),
            ("5G", "beş Ge"),
            ("x2", "iks iki"),
            ("1920x1080", "bin dokuz yüz yirmi iks bin seksen"),
            ("B12", "Be on iki"),
            ("Q4'te", "Kü dörtte"),
            ("X'te", "İks'te"),
            ("B’ye", "Be’ye"),
            // An abbreviation read before a number only.
            ("D:", "De:"),
            ("D:5", "daire beş"),
            ("M.", "Me."),
            // A period between two single letters is not said.
            ("T.K.", "Te Ke."),
            ("A.Ş", "A Şe"),
            ("v.2", "ve nokta iki"),
            // Capitals without a vowel are spelled, and the last name takes the suffix.
            ("XL", "İks Le"),
            ("TK1956", "Te Ke bin dokuz yüz elli altı"),
            ("TR12", "Te Re on iki"),
            ("TK'yı", "Te Ke'yı"),
            ("TK'YI", "Te Ke'yı"),
            ("ZDF'den", "Ze De Fe'den"),
            ("ŞĞ", "Şe Yumuşak ge"),
            // So are three capitals ending in two consonants no Turkish word ends in.
            ("ADB", "A De Be"),
            ("ABC", "A Be Ce"),
            ("ADB'ye", "A De Be'ye"),
            // Other capitals are a word, said in lowercase; Roman numeral letters stay.
            ("ASK", "ask"),
            ("OFF", "off"),
            ("SU", "su"),
            ("İSTANBUL'U", "istanbul'u"),
            ("IĞDIR'A", "ığdır'a"),
            ("CLI", "CLI"),
            // A word with a lowercase letter stays as written.
            ("Hz", "He ze"),
            ("Tk", "Te ke"),
            ("NaCl", "Ne a Ce le"),
            ("YouTube", "YouTube"),
            // Lowercase consonants the lexicon knows in capitals are that initialism.
            ("bbc", "bi bi si"),
            ("kdv", "katma değer vergisi"),
            ("cvx", "ce ve iks"),
            ("Ankara", "Ankara"),
            ("1 q\u{308}", "bir q\u{308}"),
        ] {
            assert_eq!(read(written), spoken, "{written}");
        }
    }

    #[test]
    fn the_lexicon_reads_the_labels_it_knows() {
        for (written, spoken) in [
            ("5kg", "beş kilogram"),
            ("5 kg + 2 kg", "beş kilogram artı iki kilogram"),
            ("5 kg'den", "beş kilogramdan"),
            ("2 sa'tan", "iki saatten"),
            ("3 km'lik", "üç kilometrelik"),
            ("1,005 TL", "bir virgül sıfır sıfır beş lira"),
            ("25TL", "yirmi beş lira"),
            ("25 TL'dan", "yirmi beş liradan"),
            ("5 TL'lik", "beş liralık"),
            ("5 TL'ye", "beş liraya"),
            ("1,005 USD", "bir virgül sıfır sıfır beş dolar"),
            ("TL/USD", "lira slaş dolar"),
            ("ABD'li", "a be deli"),
            ("BBC'den", "bi bi siden"),
            ("5 GB", "beş gigabayt"),
            ("TBMM'ya", "te be me meye"),
            ("TBMM'DE", "te be me mede"),
            ("KDV'Lİ", "katma değer vergili"),
            ("5 KG'LIK", "beş kilogramlık"),
            ("IBAN:", "iban:"),
            // A unit with its power is read wherever it stands, the power as a superscript
            // or as a single digit.
            ("m2", "metrekare"),
            ("km2", "kilometrekare"),
            ("120 m2'lik", "yüz yirmi metrekarelik"),
            ("120 m²'lik", "yüz yirmi metrekarelik"),
            ("5 m³'lük", "beş metreküplük"),
            ("m3 su", "metreküp su"),
            ("m23", "me yirmi üç"),
            ("mm2", "milimetre iki"),
            // Any other unit is a label only right after a number; one letter is exact, more
            // may be written in other capitals.
            ("km", "kilometre"),
            ("5 KG", "beş kilogram"),
            ("250 GR'lık", "iki yüz elli gramlık"),
            ("5 Km", "beş kilometre"),
            ("100 ML", "yüz mililitre"),
            ("5 G", "beş Ge"),
            ("KG", "Ke Ge"),
            ("25 JPY", "yirmi beş Je Pe Ye"),
        ] {
            assert_eq!(read(written), spoken, "{written}");
        }
        for (written, exact) in [
            ("1,005 TL", "bir virgül sıfır sıfır beş Türk lirası"),
            ("25 TL'dan", "yirmi beş Türk lirasından"),
            ("5 TL'lik", "beş liralık"),
        ] {
            let literal = Literal::parse(written);
            assert_eq!(literal_text(&literal, Style::Exact), exact, "{written}");
        }
    }

    #[test]
    fn other_words_keep_their_letters_marks_and_inner_apostrophes() {
        for (written, spoken) in [
            ("abc123", "abc yüz yirmi üç"),
            ("ab''el", "ab el"),
            ("Ali'nin 5", "Ali'nin beş"),
            ("COVID-19", "kovid on dokuz"),
            ("COVID-19'dan", "kovid on dokuzdan"),
        ] {
            assert_eq!(read(written), spoken, "{written}");
        }
    }

    #[test]
    fn apostrophe_suffixes_are_harmonized_to_the_last_spoken_word() {
        for (written, spoken) in [
            ("1900'de", "bin dokuz yüzde"),
            ("12:60'de", "on iki iki nokta altmışta"),
            ("24:00'da", "yirmi dört iki nokta sıfır sıfırda"),
            ("%4'lik", "yüzde dörtlük"),
            ("%'lik", "yüzdelik"),
            ("4'üncü", "dördüncü"),
            ("6'ncı'ya", "altıncıya"),
            ("0532’e", "sıfır beş üç ikiye"),
            ("90'lar", "doksanlar"),
            ("1990'larda", "bin dokuz yüz doksanlarda"),
            ("90'lar'ın", "doksanların"),
            ("3'ünci", "üçüncü"),
            ("5 'te", "beş te"),
            ("5'6", "beş altı"),
            ("5'", "beş"),
        ] {
            assert_eq!(read(written), spoken, "{written}");
        }
    }

    #[test]
    fn other_characters_are_removed_and_no_text_is_rejected() {
        for (written, spoken) in [
            ("5^2", "beş üssü iki"),
            ("x^y", "iks üssü ye"),
            ("ab ^ el", "ab şapka el"),
            ("5¤", "beş"),
            ("|5~", "beş"),
            ("①", ""),
            ("١٢٣", ""),
            ("1\u{fe0f}\u{20e3}", "bir"),
            (";\u{301}12", ";\u{301}on iki"),
            ("", ""),
        ] {
            assert_eq!(read(written), spoken, "{written}");
        }
        // A digit or a symbol stays what it is inside a cluster that a prepended mark begins.
        for (written, word) in [
            ("\u{d4e}5", "beş"),
            ("\u{600}5", "beş"),
            ("\u{d4e}+", "artı"),
        ] {
            let spoken = read(written);
            assert!(spoken.contains(word), "{written}: {spoken}");
            assert!(!spoken.contains(['5', '+']), "{written}: {spoken}");
        }
    }

    #[test]
    fn codes_counts_and_symbol_runs_are_said_part_by_part() {
        for (written, spoken) in [
            // Letters glued to a digit are no word: each colon of a hardware address is said.
            ("00:1A:2B", "sıfır sıfır iki nokta bir A iki nokta iki Be"),
            ("Fiyat:1.250TL", "Fiyat: bin iki yüz elli lira"),
            // In a code with more hyphens, each of them is said.
            (
                "SKU-48291-BLK-XL",
                "se ke u tire dört sekiz iki dokuz bir tire Be Le Ke tire İks Le",
            ),
            (
                "TR-2026-000123",
                "Te Re tire iki bin yirmi altı tire sıfır sıfır sıfır bir iki üç",
            ),
            // A round number in a code is a number.
            ("i9-14900K", "i dokuz tire on dört bin dokuz yüz Ke"),
            // A count takes its scale after up to three digits or after a fraction.
            ("45K", "kırk beş bin"),
            ("1.5K", "bir virgül beş bin"),
            ("2.5M", "iki virgül beş milyon"),
            ("14900K", "on dört bin dokuz yüz Ke"),
            ("200MP", "iki yüz megapiksel"),
            ("MP3", "Me Pe üç"),
            // Symbols alone are all named; `^` that is no power is şapka.
            ("!@#$%^&*", "ünlem et heşteg dolar yüzde şapka ve çarpı"),
            ("?!", "?!"),
            // A domain's `com` after a point.
            ("şirket.com.tr", "şirket nokta kom nokta te re"),
        ] {
            assert_eq!(read(written), spoken, "{written}");
        }
    }

    #[test]
    fn capitals_are_letters_unless_they_make_a_word() {
        for (written, spoken) in [
            // Two capitals are letters, unless they are a word or a Roman numeral.
            ("ID", "İ De"),
            ("BA", "Be A"),
            ("ID'si", "İ De'si"),
            ("BU", "bu"),
            ("OL", "ol"),
            ("IV", "IV"),
            // Capitals that start with a pair no word starts with are letters.
            ("NGO", "Ne Ge O"),
            ("NGO'nun", "Ne Ge O'nun"),
            ("SPOR", "spor"),
            ("SHOP", "shop"),
        ] {
            assert_eq!(read(written), spoken, "{written}");
        }
    }

    #[test]
    fn signs_marks_and_greek_letters_are_named_or_dropped() {
        for (written, spoken) in [
            ("⅓", "bir bölü üç"),
            ("⅔'si", "iki bölü üçü"),
            ("¥500", "beş yüz yen"),
            ("₼20", "yirmi manat"),
            ("§ 5", "paragraf beş"),
            ("№ 12", "numara on iki"),
            ("★★★", "yıldız yıldız yıldız"),
            ("Marka®", "Marka"),
            ("→ adım", "adım"),
            ("β-karoten", "beta karoten"),
            ("α", "alfa"),
            ("#123", "numara yüz yirmi üç"),
            ("#TR-26-1", "numara Te Re tire yirmi altı tire bir"),
            ("#yapay", "heşteg yapay"),
            // A phone's keypad.
            ("*5678#", "yıldız beş bin altı yüz yetmiş sekiz kare"),
            ("1234#", "bin iki yüz otuz dört kare"),
            // Said as it is said.
            ("ve/veya", "ve veya"),
            ("Euro/dolar", "Euro slaş dolar"),
        ] {
            assert_eq!(read(written), spoken, "{written}");
        }
    }

    #[test]
    fn powers_fractions_and_glued_ordinals_are_read_as_numbers() {
        for (written, spoken) in [
            ("10²³", "on üssü yirmi üç"),
            ("10⁻³", "on üssü eksi üç"),
            ("3×10⁸ m/s'dir", "üç iks on üssü sekiz metre slaş saniyedir"),
            ("6,022", "altı virgül sıfır yirmi iki"),
            ("1,23456", "bir virgül iki üç dört beş altı"),
            ("1.si", "birincisi"),
            ("3.sü", "üçüncüsü"),
            ("4.lük", "dördüncülük"),
            ("3.kat", "üçüncü kat"),
            ("21.yy'ın", "yirmi birinci yüzyılın"),
            ("Cad'de", "caddesinde"),
            ("CD'yi", "si diyi"),
            ("12'", "on iki"),
            ("5`i", "beşi"),
            ("Mehmet`in", "Mehmet`in"),
        ] {
            assert_eq!(read(written), spoken, "{written}");
        }
        let unit = |word: &str| {
            Literal::unit_after_number(word).map(|unit| literal_text(&unit, Style::Spoken))
        };
        assert_eq!(unit("mm").as_deref(), Some("milimetre"));
        assert_eq!(unit("kg'lık").as_deref(), Some("kilogramlık"));
        assert_eq!(unit("Ahmet"), None);
    }

    #[test]
    fn an_x_after_a_count_says_what_it_multiplies() {
        for (before, written, following, spoken) in [
            ("", "3x", " faydalı", "üç kat"),
            ("", "2x", " Intel", "iki adet"),
            ("", "3x", " 1.250 TL", "üç kere"),
            ("", "3×", " ₺50", "üç kere"),
            ("", "1.5x", " arttı", "bir virgül beş kat"),
            // A variable, or nothing said after it: the x is read as written.
            ("y = ", "2x", " daha", "iki iks"),
            ("", "2x", "", "iki iks"),
            ("", "2x", ".", "iki iks"),
            ("", "GDDR6X", " bellek", "Ge De De Re altı İks"),
            ("", "2x1", " tablet", "iki iks bir"),
            ("", "0-x", " 0", "sıfır iks"),
            ("", "3xx", " daha", "üç iks iks"),
        ] {
            let mut literal = Literal::parse(written);
            literal.multiplier(written, before, following);
            assert_eq!(
                literal_text(&literal, Style::Spoken),
                spoken,
                "{before}{written}{following}"
            );
        }
    }

    #[test]
    fn a_symbol_spelled_letters_or_capitals_make_an_ordinary_word_respelled() {
        for word in [
            "+",
            "C++",
            "R&D",
            "a_b",
            "B",
            "x",
            "D:",
            "B'ye",
            "½",
            "TK",
            "CD'yi",
            "XL",
            "ŞŞ",
            "T.C",
            "ADB",
            "ve/veya",
            "Euro/dolar",
            "β",
            "Marka®",
            "AB'yi",
            "ID",
            // Capitals said in lowercase, and lowercase consonants spelled.
            "OFF",
            "ASK",
            "SAAT",
            "VE",
            "vs",
            "cvp",
        ] {
            assert!(respells(word), "{word}");
        }
        for word in [
            "A",
            "o",
            "ve",
            "e-posta",
            "saat:",
            "Ali'nin",
            "…",
            "YouTube",
            "km",
            "Москва",
            // A sound written in consonants stays as written.
            "hmm",
            "Pff",
            "mm",
            "Brr",
        ] {
            assert!(!respells(word), "{word}");
        }
    }
}
