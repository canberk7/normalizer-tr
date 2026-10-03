//! Opt-in forced readings: fixed order, spoken style, literal reading, unchanged diagnostics.
use std::sync::OnceLock;

use normalizer_tr::{
    AmbiguityPolicy, Hint, HintKind, IssueCategory, LimitKind, MAX_INPUT_BYTES, NormalizeError,
    NormalizeOptions, NormalizeResult, Normalizer, Segment, SegmentKind, SourceRange,
};
use proptest::prelude::*;
use unicode_segmentation::UnicodeSegmentation;

/// Every symbol the literal reading names; none may survive where Forced has read the text.
const SYMBOLS: &str = "+−=×÷*%&@#<>°℃℉_½¼¾⅓⅔⅛₺$€£¥₽₹₼§№★";

fn normalizer() -> &'static Normalizer {
    static NORMALIZER: OnceLock<Normalizer> = OnceLock::new();
    NORMALIZER.get_or_init(|| Normalizer::new().unwrap())
}

fn normalize(
    input: &str,
    policy: AmbiguityPolicy,
    hints: Vec<Hint>,
) -> Result<NormalizeResult, NormalizeError> {
    normalizer().normalize(
        input,
        &NormalizeOptions {
            ambiguity_policy: policy,
            hints,
        },
    )
}

fn preserve(input: &str) -> NormalizeResult {
    normalize(input, AmbiguityPolicy::Preserve, Vec::new()).unwrap()
}

fn forced(input: &str) -> NormalizeResult {
    normalize(input, AmbiguityPolicy::Forced, Vec::new()).unwrap()
}

fn segment_at(result: &NormalizeResult, range: SourceRange) -> &Segment {
    result
        .segments()
        .iter()
        .find(|segment| segment.range() == range)
        .unwrap()
}

/// The forced segment of every issue, as (written span, rule id, kind).
fn readings<'a>(
    input: &'a str,
    result: &NormalizeResult,
) -> Vec<(&'a str, &'static str, SegmentKind)> {
    result
        .issues()
        .iter()
        .map(|issue| {
            let segment = segment_at(result, issue.range());
            (
                &input[issue.range().start()..issue.range().end()],
                segment.rule_id(),
                segment.kind(),
            )
        })
        .collect()
}

#[test]
fn an_unresolved_span_takes_the_first_reading_of_the_forced_order() {
    for (input, written, rule_id, kind, spoken) in [
        (
            "1.234",
            "1.234",
            "forced.cardinal",
            SegmentKind::Cardinal,
            "bin iki yüz otuz dört",
        ),
        (
            "Toplantı 09:30'da",
            "09:30'da",
            "forced.time",
            SegmentKind::Time,
            "Toplantı dokuz otuzda",
        ),
        (
            "10-15",
            "10-15",
            "forced.range",
            SegmentKind::Range,
            "on tire on beş",
        ),
        // A Cardinal hint does not read a trailing period, and no word follows it: literal.
        (
            "Toplam 25.",
            "25.",
            "forced.literal",
            SegmentKind::Literal,
            "Toplam yirmi beş.",
        ),
        (
            "0532",
            "0532",
            "forced.digits",
            SegmentKind::Digits,
            "sıfır beş üç iki",
        ),
        (
            "29.02.1900",
            "29.02.1900",
            "forced.date",
            SegmentKind::Date,
            "yirmi dokuz Şubat bin dokuz yüz",
        ),
        (
            "2. Dünya Savaşı",
            "2.",
            "forced.ordinal",
            SegmentKind::Ordinal,
            "ikinci Dünya Savaşı",
        ),
        (
            "TK1956 uçuşu",
            "TK1956",
            "forced.literal",
            SegmentKind::Literal,
            "Te Ke bin dokuz yüz elli altı uçuşu",
        ),
        (
            "5 KG un",
            "5 KG",
            "forced.literal",
            SegmentKind::Literal,
            "beş kilogram un",
        ),
        (
            "3 + 4 = 7",
            "3 + 4 = 7",
            "forced.literal",
            SegmentKind::Literal,
            "üç artı dört eşittir yedi",
        ),
        (
            "1,005 TL",
            "1,005 TL",
            "forced.literal",
            SegmentKind::Literal,
            "bir virgül sıfır sıfır beş lira",
        ),
        ("IV", "IV", "forced.roman", SegmentKind::Roman, "dört"),
        (
            "IV. Murat",
            "IV.",
            "forced.roman",
            SegmentKind::Roman,
            "dördüncü Murat",
        ),
        (
            "01.02.2026 tarihinde",
            "01.02.2026",
            "forced.date",
            SegmentKind::Date,
            "bir Şubat iki bin yirmi altı tarihinde",
        ),
        (
            "02/10/2026 olarak",
            "02/10/2026",
            "forced.date",
            SegmentKind::Date,
            "iki Ekim iki bin yirmi altı olarak",
        ),
        (
            "kapanış 24:00",
            "24:00",
            "forced.time",
            SegmentKind::Time,
            "kapanış yirmi dört",
        ),
        (
            "mesai 09:00-17:30 arası",
            "09:00-17:30",
            "forced.range",
            SegmentKind::Range,
            "mesai dokuz tire on yedi otuz arası",
        ),
        (
            "Maç 3-1 bitti",
            "3-1",
            "forced.range",
            SegmentKind::Range,
            "Maç üç tire bir bitti",
        ),
        (
            "oranı 1:3",
            "1:3",
            "forced.ratio",
            SegmentKind::Cardinal,
            "oranı bire üç",
        ),
        (
            "21. yüzyılda",
            "21.",
            "forced.ordinal",
            SegmentKind::Ordinal,
            "yirmi birinci yüzyılda",
        ),
        (
            "532 123 45 67",
            "532 123 45 67",
            "forced.telephone",
            SegmentKind::Telephone,
            "beş yüz otuz iki yüz yirmi üç kırk beş altmış yedi",
        ),
        (
            "TR12 0006 1005 1978 6457 8413 26",
            "TR12 0006 1005 1978 6457 8413 26",
            "forced.iban",
            SegmentKind::Iban,
            "te re bir iki, sıfır sıfır sıfır altı, bir sıfır sıfır beş, bir dokuz yedi sekiz, altı dört beş yedi, sekiz dört bir üç, iki altı",
        ),
        (
            "Python 3.12",
            "3.12",
            "forced.literal",
            SegmentKind::Literal,
            "Python üç nokta on iki",
        ),
    ] {
        let result = forced(input);
        assert_eq!(result.normalized_text(), spoken, "{input}");
        assert_eq!(
            readings(input, &result),
            [(written, rule_id, kind)],
            "{input}"
        );
        assert!(!result.complete(), "{input}");
        assert_eq!(result.issues(), preserve(input).issues(), "{input}");
    }
}

#[test]
fn a_forced_reading_taken_as_written_equals_the_explicit_hint_of_its_kind() {
    for (input, kind, rule_id) in [
        ("0532", HintKind::Digits, "forced.digits"),
        ("ö 00042", HintKind::Digits, "forced.digits"),
        ("Maç 3-1 bitti", HintKind::Range, "forced.range"),
        ("Toplantı 09:30'da", HintKind::Time, "forced.time"),
        ("Mesai 09.30", HintKind::Time, "forced.time"),
        ("14.03.2026 günü", HintKind::Date, "forced.date"),
        ("2026-03-14", HintKind::Date, "forced.date"),
        ("10-15", HintKind::Range, "forced.range"),
        ("1,25-2,50", HintKind::Range, "forced.range"),
        ("532 123 45 67", HintKind::Telephone, "forced.telephone"),
        ("1.234", HintKind::Cardinal, "forced.cardinal"),
        ("1.234.567'nin", HintKind::Cardinal, "forced.cardinal"),
        ("Bölüm IV", HintKind::Roman, "forced.roman"),
        ("IV. Murat", HintKind::Roman, "forced.roman"),
        ("21. yüzyılda", HintKind::Ordinal, "forced.ordinal"),
        ("2. Dünya Savaşı", HintKind::Ordinal, "forced.ordinal"),
        ("32.01.2026'de", HintKind::Literal, "forced.literal"),
        ("5 KG", HintKind::Literal, "forced.literal"),
        ("3 + 4 = 7", HintKind::Literal, "forced.literal"),
        ("Toplam 25.", HintKind::Literal, "forced.literal"),
    ] {
        let guessed = forced(input);
        let [issue] = guessed.issues() else {
            panic!("{input}: expected one issue");
        };
        let guess = segment_at(&guessed, issue.range());
        assert_eq!(guess.rule_id(), rule_id, "{input}");
        let hint = Hint::new(issue.range(), kind);
        let hinted = normalize(input, AmbiguityPolicy::Forced, vec![hint]).unwrap();
        let explicit = segment_at(&hinted, issue.range());
        assert_eq!(
            (explicit.kind(), explicit.text()),
            (guess.kind(), guess.text()),
            "{input}"
        );
        assert_eq!(
            hinted.normalized_text(),
            guessed.normalized_text(),
            "{input}"
        );
        assert!(hinted.complete(), "{input}");
    }
}

#[test]
fn a_suffix_no_guess_reads_is_split_off_and_attached_to_the_reading() {
    for (input, expected, spoken) in [
        (
            "09:30'de",
            vec![("09:30'de", "forced.time", SegmentKind::Time)],
            "dokuz otuzda",
        ),
        (
            "Toplantı 09:30'dan 11:00'e kadar sürecek",
            vec![
                ("09:30'dan", "forced.time", SegmentKind::Time),
                ("11:00'e", "forced.time", SegmentKind::Time),
            ],
            "Toplantı dokuz otuzdan on bire kadar sürecek",
        ),
        (
            "kapanış 24:00'te",
            vec![("24:00'te", "forced.time", SegmentKind::Time)],
            "kapanış yirmi dörtte",
        ),
        (
            "14.03.2026'daki toplantı",
            vec![("14.03.2026'daki", "forced.date", SegmentKind::Date)],
            "on dört Mart iki bin yirmi altıdaki toplantı",
        ),
        (
            "tarih 2026-03-14'te",
            vec![("2026-03-14'te", "forced.date", SegmentKind::Date)],
            "tarih on dört Mart iki bin yirmi altıda",
        ),
        (
            "Louis XIV'ün sarayı",
            vec![("XIV'ün", "forced.roman", SegmentKind::Roman)],
            "Louis on dördün sarayı",
        ),
        (
            "1.'nın",
            vec![("1.'nın", "forced.ordinal", SegmentKind::Ordinal)],
            "birincinin",
        ),
        (
            "1.234'inci",
            vec![("1.234'inci", "forced.cardinal", SegmentKind::Ordinal)],
            "bin iki yüz otuz dördüncü",
        ),
        (
            "1990'larda",
            vec![("1990'larda", "forced.cardinal", SegmentKind::Cardinal)],
            "bin dokuz yüz doksanlarda",
        ),
        (
            "0532'yi ara",
            vec![("0532'yi", "forced.digits", SegmentKind::Digits)],
            "sıfır beş üç ikiyi ara",
        ),
        (
            "sorular info@ornek.com'a",
            vec![(
                "info@ornek.com'a",
                "forced.electronic",
                SegmentKind::Electronic,
            )],
            "sorular info et ornek nokta koma",
        ),
        (
            "24:01'de",
            vec![("24:01'de", "forced.literal", SegmentKind::Literal)],
            "yirmi dört iki nokta sıfır birde",
        ),
    ] {
        let result = forced(input);
        assert_eq!(result.normalized_text(), spoken, "{input}");
        assert_eq!(readings(input, &result), expected, "{input}");
        assert_eq!(result.issues(), preserve(input).issues(), "{input}");
    }
    // Only the forced order splits the suffix: the same explicit hint is rejected as before.
    for (input, kind) in [
        ("09:30'de", HintKind::Time),
        ("14.03.2026'ya", HintKind::Date),
        ("XIV'ün", HintKind::Roman),
    ] {
        let hint = Hint::new(SourceRange::new(0, input.len()), kind);
        for policy in [AmbiguityPolicy::Preserve, AmbiguityPolicy::Forced] {
            assert_eq!(
                normalize(input, policy, vec![hint]),
                Err(NormalizeError::InvalidHint),
                "{input}"
            );
        }
    }
}

#[test]
fn roman_is_guessed_only_where_it_is_not_ordinary_text() {
    for (input, spoken) in [
        ("Bölüm IV", "Bölüm dört"),
        ("II. Abdülhamit", "ikinci Abdülhamit"),
        ("X. Kalkınma Planı", "onuncu Kalkınma Planı"),
        ("XX'nci yüzyıl", "yirminci yüzyıl"),
        ("MMXXIV yılında", "iki bin yirmi dört yılında"),
    ] {
        let result = forced(input);
        assert_eq!(result.normalized_text(), spoken, "{input}");
        assert_eq!(readings(input, &result)[0].1, "forced.roman", "{input}");
    }
    // Not guessed: consonants are spelled by name, as any capitals without a vowel are.
    for (input, spoken) in [
        ("X platformu", "İks platformu"),
        ("X'te paylaştı", "İks'te paylaştı"),
        ("Rocky V", "Rocky Ve"),
        ("C vitamini", "Ce vitamini"),
        ("M. Kemal Atatürk", "Me. Kemal Atatürk"),
        ("XL beden", "İks Le beden"),
    ] {
        let result = forced(input);
        assert_eq!(result.normalized_text(), spoken, "{input}");
        assert_eq!(readings(input, &result)[0].1, "forced.literal", "{input}");
    }
    // An abbreviation the lexicon knows is no numeral and no issue in any policy.
    for (input, spoken) in [("CD çalar", "si di çalar"), ("CV gönder", "si vi gönder")] {
        assert_eq!(preserve(input).normalized_text(), spoken, "{input}");
        assert!(forced(input).issues().is_empty(), "{input}");
    }
}

#[test]
fn literal_reads_a_span_as_written() {
    for (input, spoken) in [
        (
            "32.01.2026",
            "otuz iki nokta sıfır bir nokta iki bin yirmi altı",
        ),
        (
            "32.01.2026'de",
            "otuz iki nokta sıfır bir nokta iki bin yirmi altıda",
        ),
        ("3 + 4 = 7", "üç artı dört eşittir yedi"),
        ("1,005 TL", "bir virgül sıfır sıfır beş lira"),
        ("5kg", "beş kilogram"),
        ("5 KG", "beş kilogram"),
        ("KDV'li", "katma değer vergili"),
        ("TBMM'DE", "te be me mede"),
        ("TK1956", "Te Ke bin dokuz yüz elli altı"),
        ("2.5.1", "iki nokta beş nokta bir"),
        ("3 × 4", "üç iks dört"),
        ("5 * 3", "beş çarpı üç"),
        ("6 x 7 = 42", "altı iks yedi eşittir kırk iki"),
        ("-5°C'den", "eksi beş dereceden"),
        ("H2O", "He iki O"),
        ("4K", "dört Ke"),
        ("1920x1080", "bin dokuz yüz yirmi iks bin seksen"),
        ("½", "yarım"),
        ("2½", "iki buçuk"),
        (
            "₺$€£%&@#+=/*<>°_|",
            "lira dolar avro sterlin yüzde ve et heşteg artı eşittir slaş çarpı küçüktür büyüktür derece alt çizgi",
        ),
    ] {
        let result = forced(input);
        assert_eq!(result.normalized_text(), spoken, "{input}");
        let [segment] = result.segments() else {
            panic!("{input}: expected one segment");
        };
        assert_eq!(
            (segment.kind(), segment.rule_id(), segment.range()),
            (
                SegmentKind::Literal,
                "forced.literal",
                SourceRange::new(0, input.len())
            ),
            "{input}"
        );
    }
    // A period between two single letters is silent; the sentence period stays outside.
    assert_eq!(forced("Y.Z. projesi").normalized_text(), "Ye Ze. projesi");
    // The lexicon reads the initialisms it knows, dots and all.
    assert_eq!(
        forced("T.C. kimlik numarası").normalized_text(),
        "Türkiye Cumhuriyeti kimlik numarası"
    );
}

#[test]
fn a_date_the_calendar_lacks_is_still_read_as_a_date() {
    for (input, spoken) in [
        ("29.02.1900", "yirmi dokuz Şubat bin dokuz yüz"),
        (
            "29.02.1900'de doğdu",
            "yirmi dokuz Şubat bin dokuz yüzde doğdu",
        ),
        (
            "tarih 31.04.2026",
            "tarih otuz bir Nisan iki bin yirmi altı",
        ),
        ("31/04/2026", "otuz bir Nisan iki bin yirmi altı"),
        ("2026-02-30", "otuz Şubat iki bin yirmi altı"),
        (
            "30.02.2026-31.02.2026 arası",
            "otuz Şubat iki bin yirmi altı tire otuz bir Şubat iki bin yirmi altı arası",
        ),
    ] {
        let result = forced(input);
        assert_eq!(result.normalized_text(), spoken, "{input}");
        // The day is still invalid: the issue stays, only the reading is a date.
        assert_eq!(result.issues(), preserve(input).issues(), "{input}");
        assert!(!result.complete(), "{input}");
        assert!(
            readings(input, &result)
                .iter()
                .all(|(_, rule_id, _)| ["forced.date", "forced.range"].contains(rule_id)),
            "{input}"
        );
    }
    // Not the shape of a date: day 1-31, month 1-12 and a four-digit year.
    for (input, spoken) in [
        (
            "32.01.2026",
            "otuz iki nokta sıfır bir nokta iki bin yirmi altı",
        ),
        (
            "01.13.2026",
            "sıfır bir nokta on üç nokta iki bin yirmi altı",
        ),
    ] {
        let result = forced(input);
        assert_eq!(result.normalized_text(), spoken, "{input}");
        assert_eq!(readings(input, &result)[0].1, "forced.literal", "{input}");
    }
}

#[test]
fn a_period_after_a_number_is_an_ordinal_unless_it_may_close_a_sentence() {
    for (input, spoken) in [
        ("1. Dünya Savaşı", "birinci Dünya Savaşı"),
        ("2. Abdülhamit tahta çıktı", "ikinci Abdülhamit tahta çıktı"),
        ("1. Giriş\n2. Yöntem", "birinci Giriş\nikinci Yöntem"),
        ("Osmanlı 16.\nyüzyılda", "Osmanlı on altıncı\nyüzyılda"),
        ("1923. yılında", "bin dokuz yüz yirmi üçüncü yılında"),
        // At the end of the text, before a capital on the next line or after a year, the
        // period may close a sentence.
        ("Toplam 25.", "Toplam yirmi beş."),
        ("Sayı 25.\nSonra", "Sayı yirmi beş.\nSonra"),
        (
            "Kuruluş 1923. Sonra büyüdü.",
            "Kuruluş bin dokuz yüz yirmi üç. Sonra büyüdü.",
        ),
    ] {
        let result = forced(input);
        assert_eq!(result.normalized_text(), spoken, "{input}");
        assert_eq!(result.issues(), preserve(input).issues(), "{input}");
    }
}

#[test]
fn capitals_are_spelled_read_by_the_lexicon_or_said_as_words() {
    for (input, spoken) in [
        ("Plan B ve TK ile XS", "Plan Be ve Te Ke ile İks Se"),
        ("XXL beden", "İks İks Le beden"),
        ("TK'YI ara", "Te Ke'yı ara"),
        // Three capitals ending in two consonants no Turkish word ends in are letters.
        ("ADB ve ZBD", "A De Be ve Ze Be De"),
        // Other capitals are a word, said in lowercase.
        (
            "ŞİŞLİ'DEKİ İŞYERİ IĞDIR'A TAŞINDI",
            "şişli'deki işyeri ığdır'a taşındı",
        ),
        // An initialism the lexicon knows is said as people say it, in any policy.
        ("BBC ve CHP", "bi bi si ve ce he pe"),
        ("ABD ve USB", "a be de ve u se be"),
        ("CD'Yİ ver", "si diyi ver"),
        // After a number a unit is read in any capitals; elsewhere it is spelled.
        (
            "5 KG un, 250 GR'lık paket, 3 Km",
            "beş kilogram un, iki yüz elli gramlık paket, üç kilometre",
        ),
        ("KG cinsinden", "Ke Ge cinsinden"),
    ] {
        let result = forced(input);
        assert_eq!(result.normalized_text(), spoken, "{input}");
        assert_eq!(result.issues(), preserve(input).issues(), "{input}");
    }
}

#[test]
fn a_unit_with_its_power_is_read_wherever_it_stands() {
    for (input, spoken) in [
        ("120 m2 daire", "yüz yirmi metrekare daire"),
        ("120 m²'lik daire", "yüz yirmi metrekarelik daire"),
        ("5 m³'lük depo", "beş metreküplük depo"),
        ("m2 fiyatı 50.000 TL", "metrekare fiyatı elli bin lira"),
        ("3+1 daire", "üç artı bir daire"),
    ] {
        let result = forced(input);
        assert_eq!(result.normalized_text(), spoken, "{input}");
        assert_eq!(result.issues(), preserve(input).issues(), "{input}");
    }
}

#[test]
fn a_suffix_is_fitted_to_the_word_the_forced_reading_says() {
    for (input, spoken) in [
        // Written for "te le" or "ka de ve", said after lira or vergi.
        ("5 TL'li ürün", "beş liralı ürün"),
        ("TL'ler", "liralar"),
        ("5 TL'si var", "beş lirası var"),
        (
            "KDV'li fiyat ve KDV'siz fiyat",
            "katma değer vergili fiyat ve katma değer vergisiz fiyat",
        ),
        (
            "KDV'deki ve USD'si, 6'yken",
            "katma değer vergisindeki ve doları, altıyken",
        ),
        // Capitals are the same suffix.
        (
            "TBMM'DE KDV'Lİ PTT'DEN",
            "te be me mede katma değer vergili pe te teden",
        ),
        (
            "09:30'DA ve 14.03.2026'DA",
            "dokuz otuzda ve on dört Mart iki bin yirmi altıda",
        ),
    ] {
        let result = forced(input);
        assert_eq!(result.normalized_text(), spoken, "{input}");
        assert_eq!(result.issues(), preserve(input).issues(), "{input}");
    }
}

#[test]
fn a_caller_can_hint_literal_explicitly() {
    let input = "Sürüm 2.5.1 çıktı; IV";
    let start = input.find("2.5.1").unwrap();
    let version = SourceRange::new(start, start + 5);
    let numeral = SourceRange::new(input.len() - 2, input.len());
    let hints = vec![
        Hint::new(version, HintKind::Literal),
        Hint::new(numeral, HintKind::Literal),
    ];
    for policy in [
        AmbiguityPolicy::Preserve,
        AmbiguityPolicy::Reject,
        AmbiguityPolicy::Forced,
    ] {
        let result = normalize(input, policy, hints.clone()).unwrap();
        // The hint decides: no issue, and no guess at the Roman numeral.
        assert_eq!(
            result.normalized_text(),
            "Sürüm iki nokta beş nokta bir çıktı; IV"
        );
        assert!(result.complete() && result.issues().is_empty());
        for range in [version, numeral] {
            let segment = segment_at(&result, range);
            assert_eq!(
                (segment.kind(), segment.rule_id()),
                (SegmentKind::Literal, "literal.hint")
            );
        }
    }
    // It reads any whole expression, including one the engine resolves itself.
    let clock = Hint::new(SourceRange::new(5, 10), HintKind::Literal);
    assert_eq!(
        normalize("saat 09:30", AmbiguityPolicy::Preserve, vec![clock])
            .unwrap()
            .normalized_text(),
        "saat sıfır dokuz iki nokta otuz"
    );
    // The lexicon's labels follow the policy's style: exact by default, spoken under Forced.
    let money = Hint::new(SourceRange::new(0, 8), HintKind::Literal);
    for (policy, spoken) in [
        (
            AmbiguityPolicy::Preserve,
            "bir virgül sıfır sıfır beş Türk lirası",
        ),
        (AmbiguityPolicy::Forced, "bir virgül sıfır sıfır beş lira"),
    ] {
        assert_eq!(
            normalize("1,005 TL", policy, vec![money])
                .unwrap()
                .normalized_text(),
            spoken
        );
    }
    // It is still a whole-expression hint.
    let cut = Hint::new(SourceRange::new(0, 1), HintKind::Literal);
    assert_eq!(
        normalize("3 + 4", AmbiguityPolicy::Preserve, vec![cut]),
        Err(NormalizeError::InvalidHint)
    );
}

#[test]
fn forced_keeps_issues_ranges_and_resolved_kinds() {
    let input = "25 TL; 1.234; IV; 10-15";
    let preserved = preserve(input);
    let result = forced(input);
    assert_eq!(
        result.normalized_text(),
        "yirmi beş lira; bin iki yüz otuz dört; dört; on tire on beş"
    );
    assert!(!result.complete());
    assert_eq!(result.issues(), preserved.issues());
    assert_eq!(
        result
            .issues()
            .iter()
            .map(|issue| issue.category())
            .collect::<Vec<_>>(),
        [
            IssueCategory::Ambiguous,
            IssueCategory::Ambiguous,
            IssueCategory::InvalidExpression
        ]
    );
    assert_eq!(
        result
            .segments()
            .iter()
            .map(|segment| (segment.kind(), segment.rule_id()))
            .collect::<Vec<_>>(),
        [
            (SegmentKind::Money, "quantity"),
            (SegmentKind::Verbatim, "source.verbatim"),
            (SegmentKind::Cardinal, "forced.cardinal"),
            (SegmentKind::Verbatim, "source.verbatim"),
            (SegmentKind::Roman, "forced.roman"),
            (SegmentKind::Verbatim, "source.verbatim"),
            (SegmentKind::Range, "forced.range"),
        ]
    );
    assert_eq!(
        result
            .segments()
            .iter()
            .map(|segment| segment.range())
            .collect::<Vec<_>>(),
        preserved
            .segments()
            .iter()
            .map(|segment| segment.range())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        normalize(input, AmbiguityPolicy::Reject, Vec::new()),
        Err(NormalizeError::Unresolved(preserved.issues().to_vec()))
    );
}

#[test]
fn forced_says_resolved_readings_in_the_spoken_style() {
    for (input, exact, spoken) in [
        ("4,25", "dört virgül iki beş", "dört virgül yirmi beş"),
        ("0,18", "sıfır virgül bir sekiz", "sıfır virgül on sekiz"),
        (
            "12,05",
            "on iki virgül sıfır beş",
            "on iki virgül sıfır beş",
        ),
        (
            "%3,25'ten",
            "yüzde üç virgül iki beşten",
            "yüzde üç virgül yirmi beşten",
        ),
        (
            "1,05-2,50 kg",
            "bir virgül sıfır beş ila iki virgül beş sıfır kilogram",
            "bir virgül sıfır beş tire iki virgül elli kilogram",
        ),
        ("25 TL", "yirmi beş Türk lirası", "yirmi beş lira"),
        (
            "1.234,50 TL",
            "bin iki yüz otuz dört Türk lirası elli kuruş",
            "bin iki yüz otuz dört lira elli kuruş",
        ),
        (
            "25 TL'den",
            "yirmi beş Türk lirasından",
            "yirmi beş liradan",
        ),
        ("₺25", "yirmi beş Türk lirası", "yirmi beş lira"),
        ("TL'den", "Türk lirasından", "liradan"),
        ("25 USD", "yirmi beş dolar", "yirmi beş dolar"),
    ] {
        let preserved = preserve(input);
        let result = forced(input);
        assert_eq!(preserved.normalized_text(), exact, "{input}");
        assert_eq!(result.normalized_text(), spoken, "{input}");
        // A resolved span is still resolved: only the way it is said differs.
        assert!(
            preserved.complete() && result.complete() && result.issues().is_empty(),
            "{input}"
        );
        assert_eq!(result.segments().len(), preserved.segments().len());
        for (segment, kept) in result.segments().iter().zip(preserved.segments()) {
            assert_eq!(
                (segment.range(), segment.kind(), segment.rule_id()),
                (kept.range(), kept.kind(), kept.rule_id()),
                "{input}"
            );
        }
    }
}

#[test]
fn forced_speaks_stray_symbols_and_single_letters_without_an_issue() {
    for (input, spoken, strays) in [
        (
            "Fatura 25 USD + KDV",
            "Fatura yirmi beş dolar artı katma değer vergisi",
            vec!["+"],
        ),
        ("C++ ve a&b", "Ce artı artı ve a ve be", vec!["C++", "a&b"]),
        ("Plan B hazır", "Plan Be hazır", vec!["B"]),
        ("x ve y", "iks ve ye", vec!["x", "y"]),
        ("a > b", "a büyüktür be", vec![">", "b"]),
        ("A'dan Z'ye", "A'dan Ze'ye", vec!["Z'ye"]),
    ] {
        let preserved = preserve(input);
        let result = forced(input);
        assert_eq!(result.normalized_text(), spoken, "{input}");
        assert!(result.complete() && result.issues().is_empty(), "{input}");
        assert_eq!(result.issues(), preserved.issues(), "{input}");
        let spelled: Vec<_> = result
            .segments()
            .iter()
            .filter(|segment| segment.rule_id() == "forced.spoken")
            .map(|segment| {
                assert_eq!(segment.kind(), SegmentKind::Literal);
                &input[segment.range().start()..segment.range().end()]
            })
            .collect();
        assert_eq!(spelled, strays, "{input}");
    }
    // A vowel is its own name, and a word is not a letter: nothing to respell.
    for input in ["o geldi", "A şıkkı ve e-posta", "saat: Ali'nin"] {
        assert_eq!(forced(input), preserve(input), "{input}");
    }
}

#[test]
fn a_mark_the_reading_says_is_not_kept_as_written() {
    for (input, spoken, marks) in [
        ("6.8\" ekran", "altı virgül sekiz inç ekran", 1),
        (
            "41°00'49\"K 28°58'34\"D",
            "kırk bir derece sıfır dakika kırk dokuz saniye kuzey yirmi sekiz derece elli sekiz dakika otuz dört saniye doğu",
            2,
        ),
        (
            "maç 90+4. dakikada bitti",
            "maç doksan artı dördüncü dakikada bitti",
            1,
        ),
    ] {
        let result = forced(input);
        assert_eq!(result.normalized_text(), spoken, "{input}");
        // Each mark is a spoken segment of its own, with nothing more said for it.
        let said = result
            .segments()
            .iter()
            .filter(|segment| segment.rule_id() == "forced.spoken" && segment.text().is_empty())
            .count();
        assert_eq!(said, marks, "{input}");
    }
}

#[test]
fn codes_counts_durations_and_winds_are_said_as_people_say_them() {
    for (input, spoken) in [
        (
            "MAC 00:1A:2B:3C:4D:5E",
            "mek sıfır sıfır iki nokta bir A iki nokta iki Be iki nokta üç Ce iki nokta dört De iki nokta beş E",
        ),
        (
            "kod SKU-48291-BLK-XL",
            "kod se ke u tire dört sekiz iki dokuz bir tire Be Le Ke tire İks Le",
        ),
        (
            "i9-14900K işlemci",
            "i dokuz tire on dört bin dokuz yüz Ke işlemci",
        ),
        ("200MP kamera", "iki yüz megapiksel kamera"),
        ("1.5K beğeni", "bir virgül beş bin beğeni"),
        ("rekor 2:00:35", "rekor iki saat otuz beş saniye"),
        ("saat 14:30:00'da", "saat on dört otuzda"),
        (
            "rüzgâr K-KD 15 km/sa",
            "rüzgâr kuzey kuzeydoğu saatte on beş kilometre",
        ),
        ("ve Elizabeth II.", "ve Elizabeth ikinci."),
        ("f(x) = 2x", "fe (iks) eşittir iki iks"),
        (
            "şifre: !@#$%^&*",
            "şifre: ünlem et heşteg dolar yüzde şapka ve çarpı",
        ),
        (
            "destek@şirket.com.tr",
            "destek et şirket nokta kom nokta te re",
        ),
        (
            "https://www.ornek.com.tr",
            "he te te pe se iki nokta slaş slaş çift ve çift ve çift ve nokta ornek nokta kom nokta te re",
        ),
    ] {
        assert_eq!(forced(input).normalized_text(), spoken, "{input}");
    }
}

#[test]
fn fractions_multipliers_fahrenheit_and_compound_plurals_are_said_as_people_say_them() {
    for (input, spoken) in [
        // A fraction is said with bölü; other slashes are slaş.
        ("1/2 bardak, 2/3'ü", "bir bölü iki bardak, iki bölü üçü"),
        ("olasılık 1/6", "olasılık bir bölü altı"),
        (
            "7/24 açık, No: 3/5",
            "yedi slaş yirmi dört açık, numara üç slaş beş",
        ),
        // An `x` after a count multiplies what follows; after an operator it is a variable.
        ("3x faydalı", "üç kat faydalı"),
        ("2x Intel Xeon", "iki adet Intel Xeon"),
        (
            "3x 1.666,67 TL",
            "üç kere bin altı yüz altmış altı lira altmış yedi kuruş",
        ),
        ("y = 2x", "ye eşittir iki iks"),
        // Fahrenheit is said; Celsius is the degree alone.
        (
            "70 °F ve 21°C",
            "yetmiş derece Fahrenhayt ve yirmi bir derece",
        ),
        // A compound says its possessive after its plural.
        (
            "KDV'ler ve KDV'lerde",
            "katma değer vergileri ve katma değer vergilerinde",
        ),
    ] {
        assert_eq!(forced(input).normalized_text(), spoken, "{input}");
    }
}

#[test]
fn everyday_texts_of_every_kind_are_spoken_as_people_say_them() {
    for (input, spoken) in [
        (
            "Kredi kartı: 4111 1111 1111 1111",
            "Kredi kartı: dört bir bir bir bir bir bir bir bir bir bir bir bir bir bir bir",
        ),
        ("#1 çok satan ürün", "bir numara çok satan ürün"),
        (
            "Fiyat: 50 TL | Kargo: 10 TL",
            "Fiyat: elli lira Kargo: on lira",
        ),
        (
            "→ Adım 1: Giriş yap • Adım 2: Ödeme ✓",
            "Adım bir: Giriş yap Adım iki: Ödeme",
        ),
        (
            "Boyutlar: 159,9 x 76,7 x 8,25 mm.",
            "Boyutlar: yüz elli dokuz virgül dokuz iks yetmiş altı virgül yedi iks sekiz virgül yirmi beş milimetre.",
        ),
        (
            "Puan: 4,5/5, not: 85/100",
            "Puan: dört virgül beş bölü beş, not: seksen beş bölü yüz",
        ),
        (
            "Merkez: Atatürk Mah. 1234. Sok. No: 5",
            "Merkez: Atatürk mahallesi bin iki yüz otuz dördüncü sokak numara beş",
        ),
        (
            "5 No'lu ve 3 nolu dispanser",
            "beş numaralı ve üç nolu dispanser",
        ),
        (
            "Verstappen 1:23.456'lık turla",
            "Verstappen bir dakika yirmi üç virgül dört yüz elli altı saniyelik turla",
        ),
        (
            "Avogadro sayısı 6,022×10²³'tür.",
            "Avogadro sayısı altı virgül sıfır yirmi iki iks on üssü yirmi üçtür.",
        ),
        (
            "1.si, 2.si ve 3.sü ödül aldı; 21.yy'ın icadı",
            "birincisi, ikincisi ve üçüncüsü ödül aldı; yirmi birinci yüzyılın icadı",
        ),
        ("α-tokoferol ve β-karoten", "alfa tokoferol ve beta karoten"),
        (
            "Kapı kodu 1234#, alarm *5678#",
            "Kapı kodu bin iki yüz otuz dört kare, alarm yıldız beş bin altı yüz yetmiş sekiz kare",
        ),
        (
            "Fiyat ¥500, ₽1000, ₹200",
            "Fiyat beş yüz yen, bin ruble, iki yüz rupi",
        ),
        ("Marka® ve Ürün™ © 2025", "Marka ve Ürün iki bin yirmi beş"),
        (
            "harf notu BA, kimlik ID'si",
            "harf notu Be A, kimlik İ De'si",
        ),
        (
            "⅓ kısmı, § 5, № 12",
            "bir bölü üç kısmı, paragraf beş, numara on iki",
        ),
        (
            "golleri 12' ve 45+2'",
            "golleri on iki ve kırk beş artı iki",
        ),
        ("Euro/dolar ve/veya TL", "Euro slaş dolar ve veya lira"),
    ] {
        assert_eq!(forced(input).normalized_text(), spoken, "{input}");
    }
}

/// The bidirectional controls, invalid input under Preserve and Reject.
const BIDI: [char; 12] = [
    '\u{061c}', '\u{200e}', '\u{200f}', '\u{202a}', '\u{202b}', '\u{202c}', '\u{202d}', '\u{202e}',
    '\u{2066}', '\u{2067}', '\u{2068}', '\u{2069}',
];

/// Forced reads text with bidirectional controls as the text without them, in original
/// coordinates: verbatim text is cut around each control, and no control is said.
fn controlled_invariants(input: &str) {
    let unmarked: String = input.chars().filter(|ch| !BIDI.contains(ch)).collect();
    let result = normalize(input, AmbiguityPolicy::Forced, Vec::new());
    let Ok(expected) = normalize(&unmarked, AmbiguityPolicy::Forced, Vec::new()) else {
        assert!(result.is_err(), "{input:?}");
        return;
    };
    let result = match result {
        Ok(result) => result,
        // A control between two halves of a flag would join them once removed.
        Err(NormalizeError::InvalidInput) => {
            assert!(
                input
                    .chars()
                    .any(|ch| ('\u{1f1e6}'..='\u{1f1ff}').contains(&ch)),
                "{input:?}"
            );
            return;
        }
        Err(error) => panic!("{input:?}: {error:?}"),
    };
    // A control dropped between two spaces takes one of them with it.
    let collapsed = |text: &str| text.split_whitespace().collect::<Vec<_>>().join(" ");
    assert_eq!(
        collapsed(result.normalized_text()),
        collapsed(expected.normalized_text()),
        "{input:?}"
    );
    assert_eq!(result.issues().len(), expected.issues().len(), "{input:?}");
    let mut boundaries: Vec<_> = input.grapheme_indices(true).map(|(at, _)| at).collect();
    boundaries.push(input.len());
    let mut cursor = 0;
    for segment in result.segments() {
        assert_eq!(segment.range().start(), cursor, "{input:?}");
        cursor = segment.range().end();
        assert!(boundaries.contains(&segment.range().start()), "{input:?}");
        assert!(boundaries.contains(&cursor), "{input:?}");
        if segment.kind() == SegmentKind::Verbatim {
            assert_eq!(segment.text(), &input[segment.range().start()..cursor]);
        }
        assert!(!segment.text().contains(&BIDI[..]), "{input:?}");
    }
    assert_eq!(cursor, input.len(), "{input:?}");
}

#[test]
fn forced_reads_text_with_bidirectional_controls_as_if_they_were_not_there() {
    for input in [
        "Yön işareti \u{200f} içeren metin.",
        "Fiyat\u{200f} 5\u{200f} TL, Dr.\u{200f} Ahmet",
        "\u{202e}ters\u{202c} ve \u{2066}izole\u{2069}",
        "\u{200e}12.03.2026\u{200e} tarihinde",
    ] {
        assert_eq!(
            normalize(input, AmbiguityPolicy::Preserve, Vec::new()),
            Err(NormalizeError::InvalidInput)
        );
        controlled_invariants(input);
    }
    assert_eq!(
        forced("Fiyat\u{200f} 5\u{200f} TL, Dr.\u{200f} Ahmet").normalized_text(),
        "Fiyat beş lira, doktor Ahmet"
    );
    // A hint is given in original coordinates.
    let hinted = normalize(
        "\u{200f}1.234",
        AmbiguityPolicy::Forced,
        vec![Hint::new(SourceRange::new(3, 8), HintKind::Digits)],
    )
    .unwrap();
    assert_eq!(hinted.normalized_text(), "bir iki üç dört");
    assert_eq!(
        normalize(
            "\u{200f}1",
            AmbiguityPolicy::Forced,
            vec![Hint::new(SourceRange::new(1, 4), HintKind::Cardinal)],
        ),
        Err(NormalizeError::InvalidHint)
    );
}

#[test]
fn inputs_with_nothing_to_force_or_respell_are_byte_identical() {
    for input in [
        "5 kg; 3'üncü",
        "saat 09:30'da 5 kg malzeme ve %12,5'lik fark",
        "tarih 01.02.2026 ve 3'üncü",
        "0850 222 33 44",
        "TR33 0006 1005 1978 6457 8413 26",
        "II. Dünya Savaşı ve XXI. yüzyıl",
        "info@ornek.com",
        "ordinary words remain unchanged.",
        "İstanbul 👩‍👩‍👧‍👦: o\u{308}n 4’u\u{308}n ve 25 USD",
    ] {
        let preserved = preserve(input);
        assert!(preserved.complete(), "{input}");
        assert_eq!(forced(input), preserved, "{input}");
    }
}

#[test]
fn explicit_hints_are_applied_before_forced_readings() {
    let input = "1.234 ve 10-15";
    let digits = Hint::new(SourceRange::new(0, 5), HintKind::Digits);
    let result = normalize(input, AmbiguityPolicy::Forced, vec![digits]).unwrap();
    assert_eq!(
        result.normalized_text(),
        "bir iki üç dört ve on tire on beş"
    );
    assert_eq!(result.segments()[0].rule_id(), "digits.hint");
    assert_eq!(
        readings(input, &result),
        [("10-15", "forced.range", SegmentKind::Range)]
    );
    for (input, end, kind) in [
        ("IIII", 4, HintKind::Roman),
        ("31.04.2026", 10, HintKind::Date),
        ("3 + 4", 1, HintKind::Cardinal),
    ] {
        let hint = Hint::new(SourceRange::new(0, end), kind);
        assert_eq!(
            normalize(input, AmbiguityPolicy::Forced, vec![hint]),
            Err(NormalizeError::InvalidHint),
            "{input}"
        );
    }
}

#[test]
fn forced_reads_the_nfc_text_at_original_coordinates() {
    let input = "o\u{308}n 1.234'u\u{308} ve A\u{30a}B-12";
    let result = forced(input);
    assert_eq!(
        result.normalized_text(),
        "o\u{308}n bin iki yüz otuz dördü ve \u{c5} Be on iki"
    );
    assert_eq!(
        readings(input, &result),
        [
            ("1.234'u\u{308}", "forced.cardinal", SegmentKind::Cardinal),
            ("A\u{30a}B-12", "forced.literal", SegmentKind::Literal),
        ]
    );
    assert_eq!(result.issues(), preserve(input).issues());
}

#[test]
fn forced_readings_count_against_the_limits() {
    // Preserve keeps these spans as written; their forced readings outgrow the result budget.
    let input = "999.999.999.999.999.999 ".repeat(1300);
    assert!(input.len() < MAX_INPUT_BYTES);
    assert!(normalize(&input, AmbiguityPolicy::Preserve, Vec::new()).is_ok());
    assert_eq!(
        normalize(&input, AmbiguityPolicy::Forced, Vec::new()),
        Err(NormalizeError::LimitExceeded(LimitKind::Result))
    );
    // A stray letter is a candidate record only where Forced speaks it.
    let input = "x ".repeat(4097);
    assert!(normalize(&input, AmbiguityPolicy::Preserve, Vec::new()).is_ok());
    assert_eq!(
        normalize(&input, AmbiguityPolicy::Forced, Vec::new()),
        Err(NormalizeError::LimitExceeded(LimitKind::Candidates))
    );
}

/// The hint kind whose reader a forced rule id names, where there is one.
fn hint_kind(rule_id: &str) -> Option<HintKind> {
    Some(match rule_id {
        "forced.digits" => HintKind::Digits,
        "forced.time" => HintKind::Time,
        "forced.date" => HintKind::Date,
        "forced.range" => HintKind::Range,
        "forced.telephone" => HintKind::Telephone,
        "forced.cardinal" => HintKind::Cardinal,
        "forced.roman" => HintKind::Roman,
        "forced.ordinal" => HintKind::Ordinal,
        "forced.electronic" => HintKind::Electronic,
        "forced.literal" => HintKind::Literal,
        "forced.ratio" | "forced.fraction" | "forced.iban" | "forced.money" => return None,
        other => panic!("unexpected forced rule id {other}"),
    })
}

fn unread(text: &str) -> bool {
    text.chars()
        .any(|ch| ch.is_ascii_digit() || SYMBOLS.contains(ch))
}

fn forced_invariants(input: &str) {
    let Ok(preserved) = normalize(input, AmbiguityPolicy::Preserve, Vec::new()) else {
        return;
    };
    let result = forced(input);
    assert_eq!(result, forced(input));
    assert_eq!(result.issues(), preserved.issues());
    assert_eq!(result.complete(), preserved.complete());
    assert!(
        !result.normalized_text().bytes().any(|b| b.is_ascii_digit()),
        "{:?}",
        result.normalized_text()
    );
    assert_eq!(
        result
            .segments()
            .iter()
            .map(|segment| segment.text())
            .collect::<String>(),
        result.normalized_text()
    );
    // Forced segments follow the Preserve partition; only verbatim text may be cut finer.
    let mut segments = result.segments().iter().peekable();
    for kept in preserved.segments() {
        if kept.kind() != SegmentKind::Verbatim {
            let segment = segments.next().unwrap();
            assert_eq!(segment.range(), kept.range());
            if kept.kind() == SegmentKind::Unresolved {
                assert!(segment.rule_id().starts_with("forced."));
                assert_ne!(segment.rule_id(), "forced.spoken");
                assert_ne!(segment.kind(), SegmentKind::Unresolved);
                assert!(!unread(segment.text()), "{:?}", segment.text());
            } else {
                assert_eq!(
                    (segment.kind(), segment.rule_id()),
                    (kept.kind(), kept.rule_id())
                );
                // Only the spoken style says a resolved span differently: a fraction, the
                // lira, a range's dash, a clock's zero, a slash in an address, or a long
                // plain number as the identifier it is.
                let written = &input[kept.range().start()..kept.range().end()];
                let restyled = [
                    "virgül",
                    "Türk lirası",
                    " ila ",
                    "eğik çizgi",
                    "hashtag",
                    "ha te te pe",
                ]
                .iter()
                .any(|word| kept.text().contains(word))
                    || kept.kind() == SegmentKind::Time
                    || (kept.kind() == SegmentKind::Cardinal
                        && written.len() >= 7
                        && written.bytes().all(|b| b.is_ascii_digit()));
                if !restyled {
                    assert_eq!(segment.text(), kept.text());
                }
            }
            continue;
        }
        let mut cursor = kept.range().start();
        while let Some(segment) = segments.next_if(|s| s.range().end() <= kept.range().end()) {
            assert_eq!(segment.range().start(), cursor);
            cursor = segment.range().end();
            if segment.kind() == SegmentKind::Verbatim {
                assert_eq!(segment.text(), &input[segment.range().start()..cursor]);
            } else {
                assert_eq!(
                    (segment.kind(), segment.rule_id()),
                    (SegmentKind::Literal, "forced.spoken")
                );
            }
            assert!(!unread(segment.text()), "{:?}", segment.text());
        }
        assert_eq!(cursor, kept.range().end());
    }
    assert!(segments.next().is_none());
    for issue in result.issues().iter().take(3) {
        let guess = segment_at(&result, issue.range());
        let Some(kind) = hint_kind(guess.rule_id()) else {
            continue;
        };
        // Where the same explicit hint is accepted, it reads exactly what Forced read, unless
        // Forced also read the text around the span: an ordinal period, an inch or seconds
        // mark, a unit after a decimal, a compass letter after a coordinate, the word after a
        // count's `x`, or the end of the text after a Roman ordinal, whose period then closes
        // the sentence too.
        let (before, after) = (
            &input[..issue.range().start()],
            &input[issue.range().end()..],
        );
        let written = &input[issue.range().start()..issue.range().end()];
        // A count with one `x` glued after it: `3x`, `1.5x`.
        let multiplied = written.strip_suffix(['x', 'X', '×']).is_some_and(|count| {
            count.starts_with(|ch: char| ch.is_ascii_digit())
                && count.ends_with(|ch: char| ch.is_ascii_digit())
                && count
                    .chars()
                    .all(|ch| ch.is_ascii_digit() || matches!(ch, '.' | ','))
        });
        // A lone `*`, `<`, `>` or `#` is said or not by what stands around it: `3 * 4`,
        // `Kategori > Telefon`, `# Başlık`.
        let marked = matches!(written, "*" | "<" | ">") || written.bytes().all(|b| b == b'#');
        let context = before.ends_with(['"', '″'])
            || after.starts_with(['"', '″', '.'])
            || after.trim_start().starts_with(char::is_alphabetic)
            || multiplied
            || marked;
        if (kind == HintKind::Literal && context)
            || (kind == HintKind::Roman && guess.text().ends_with('.'))
        {
            continue;
        }
        let hint = Hint::new(issue.range(), kind);
        match normalize(input, AmbiguityPolicy::Forced, vec![hint]) {
            Ok(hinted) => {
                let explicit = segment_at(&hinted, issue.range());
                assert_eq!(
                    (explicit.kind(), explicit.text()),
                    (guess.kind(), guess.text())
                );
            }
            Err(error) => assert_eq!(error, NormalizeError::InvalidHint),
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]
    #[test]
    fn forced_reads_every_unresolved_span(
        fragments in prop::collection::vec(prop_oneof![
            "[a-zA-Z0-9.,:%'/_+@-]{0,30}",
            "[0-9IVXLCDMabx.,:'’/ +=×÷%&@#*<>°_₺$€£½²³-]{1,24}",
            Just("İstanbul".to_owned()),
            Just("o\u{308} 4’u\u{308}n".to_owned()),
            Just("👨‍👩‍👧".to_owned()),
            Just("25 TL".to_owned()),
            Just("4,25 TL'lik".to_owned()),
            Just("14.03.2026'da saat 09:30'da".to_owned()),
            Just("09:30'dan 11:00'e".to_owned()),
            Just("09:00-17:30".to_owned()),
            Just("21. yüzyılda".to_owned()),
            Just("1. Dünya Savaşı".to_owned()),
            Just("29.02.1900'de 31/04/2026".to_owned()),
            Just("TK1956 CD'Yİ TBMM'DE".to_owned()),
            Just("5 KG'lık 120 m2'lik 5 m³'lük".to_owned()),
            Just("KDV'li TL'ler 6'yken".to_owned()),
            Just("Cad. No: 5 D: 3 ABD'de Ltd. Şti.".to_owned()),
            Just("24/7 10-15 1234567 5321234567".to_owned()),
            Just("ŞİŞLİ'DEKİ DR. AHMET CEVABI BUL.".to_owned()),
            Just("COVID-19 F-16 ADB 12:05".to_owned()),
            Just("1.'nın".to_owned()),
            Just("XIV'ün".to_owned()),
            Just("0532'yi".to_owned()),
            Just("532 123 45 67".to_owned()),
            Just("3 + 4 = 7".to_owned()),
            Just("TR12 0006 1005".to_owned()),
            Just("TR12 0006 1005 1978 6457 8413 26".to_owned()),
            Just("https://ornek.com/%ZZ".to_owned()),
            Just("info@ornek.com'a".to_owned()),
            Just("#yapay_zeka".to_owned()),
            Just("C++ & R&D".to_owned()),
            Just(";\u{301}12".to_owned()),
            Just("①".to_owned()),
        ], 0..30)
    ) {
        forced_invariants(&fragments.join(" "));
    }

    #[test]
    fn forced_survives_arbitrary_unicode(chars in prop::collection::vec(any::<char>(), 0..150)) {
        let input: String = chars.into_iter().filter(|ch| !ch.is_control()).collect();
        forced_invariants(&input);
    }

    #[test]
    fn forced_reads_digits_and_symbols_inside_arbitrary_unicode(
        parts in prop::collection::vec(prop_oneof![
            any::<char>().prop_filter("no controls", |ch| !ch.is_control()).prop_map(String::from),
            "[0-9]{1,4}",
            "[.,:'’/ -]",
            "[+=×÷*%&@#<>°_₺$€£½⅓¥₽§№★©®™•→✓⁻²³⁸`´αβ]",
            "[bcdxBCDX]",
        ], 0..60)
    ) {
        forced_invariants(&parts.concat());
    }

    #[test]
    fn forced_reads_around_bidirectional_controls(
        parts in prop::collection::vec(prop_oneof![
            "[a-zA-Z0-9.,:%'/ +=x-]{0,12}",
            Just("25 TL".to_owned()),
            Just("Dr. Ahmet".to_owned()),
            Just("1.234'ü".to_owned()),
            Just("🇹🇷".to_owned()),
            Just("🇹".to_owned()),
            Just("e\u{301}".to_owned()),
            prop::sample::select(BIDI.to_vec()).prop_map(String::from),
        ], 0..20)
    ) {
        controlled_invariants(&parts.concat());
    }
}
