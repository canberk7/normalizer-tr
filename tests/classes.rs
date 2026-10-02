//! Positive bounded-profile class fixtures.
use normalizer_tr::{Hint, HintKind, NormalizeOptions, Normalizer, SourceRange};

fn normalized(input: &str) -> String {
    let result = Normalizer::new()
        .unwrap()
        .normalize(input, &NormalizeOptions::default())
        .unwrap();
    assert!(result.complete(), "{input}: {:?}", result.issues());
    result.normalized_text().to_owned()
}

#[test]
fn exact_cardinals_decimals_and_signs() {
    for (input, expected) in [
        ("0", "sıfır"),
        ("100", "yüz"),
        ("1000", "bin"),
        ("1001", "bin bir"),
        ("101000", "yüz bir bin"),
        ("1000000", "bir milyon"),
        ("1000000000", "bir milyar"),
        ("1000000000000", "bir trilyon"),
        ("1000000000000000", "bir katrilyon"),
        (
            "999999999999999999",
            "dokuz yüz doksan dokuz katrilyon dokuz yüz doksan dokuz trilyon dokuz yüz doksan dokuz milyar dokuz yüz doksan dokuz milyon dokuz yüz doksan dokuz bin dokuz yüz doksan dokuz",
        ),
        ("-0", "eksi sıfır"),
        ("+12", "artı on iki"),
        ("12,05", "on iki virgül sıfır beş"),
        (
            "-0,000000001",
            "eksi sıfır virgül sıfır sıfır sıfır sıfır sıfır sıfır sıfır sıfır bir",
        ),
        ("%12,5", "yüzde on iki virgül beş"),
    ] {
        assert_eq!(normalized(input), expected, "{input}");
    }
}

#[test]
fn exact_money_never_rounds() {
    for (input, expected) in [
        (
            "1.234,50 TL",
            "bin iki yüz otuz dört Türk lirası elli kuruş",
        ),
        ("-0,50 TL", "eksi sıfır Türk lirası elli kuruş"),
        ("-0,00 TRY", "eksi sıfır Türk lirası"),
        ("25 ₺", "yirmi beş Türk lirası"),
        ("₺25,01", "yirmi beş Türk lirası bir kuruş"),
        ("25,5₺", "yirmi beş Türk lirası elli kuruş"),
        ("+25,00 TL", "artı yirmi beş Türk lirası"),
        ("TL 25", "yirmi beş Türk lirası"),
        ("1.000.000,09 TRY", "bir milyon Türk lirası dokuz kuruş"),
    ] {
        assert_eq!(normalized(input), expected, "{input}");
    }
}

#[test]
fn shared_numeric_inflection() {
    for (input, expected) in [
        ("3'üncü", "üçüncü"),
        ("6'ncı", "altıncı"),
        ("1'inci", "birinci"),
        ("2'nci", "ikinci"),
        ("4'üncü", "dördüncü"),
        ("4'e", "dörde"),
        ("4'ü", "dördü"),
        ("4'ün", "dördün"),
        ("4'te", "dörtte"),
        ("4'ten", "dörtten"),
        ("6'ya", "altıya"),
        ("6'yı", "altıyı"),
        ("6'nın", "altının"),
        ("100'ün", "yüzün"),
        ("%4'lük", "yüzde dörtlük"),
        ("%6'lık", "yüzde altılık"),
        ("%10'luk", "yüzde onluk"),
        ("%12,5'lik", "yüzde on iki virgül beşlik"),
    ] {
        assert_eq!(normalized(input), expected, "{input}");
    }
}

#[test]
fn units_and_approved_abbreviations() {
    for (input, expected) in [
        ("5 kg", "beş kilogram"),
        ("3 m", "üç metre"),
        ("2 g", "iki gram"),
        ("1 km", "bir kilometre"),
        ("12 cm", "on iki santimetre"),
        ("10 mm", "on milimetre"),
        ("2 L", "iki litre"),
        ("0,05 mL", "sıfır virgül sıfır beş mililitre"),
        ("Dr. Ali", "doktor Ali"),
        ("TBMM", "te be me me"),
        ("TBMM'ye", "te be me meye"),
        ("KDV", "katma değer vergisi"),
        ("TBMM’ye", "te be me meye"),
    ] {
        assert_eq!(normalized(input), expected, "{input}");
    }
}

#[test]
fn hinted_and_cued_temporal_values() {
    for (input, expected) in [
        ("tarih 01.02.2026", "tarih bir Şubat iki bin yirmi altı"),
        ("tarih: 01.02.2026", "tarih: bir Şubat iki bin yirmi altı"),
        ("TARİH:01.02.2026", "TARİH:bir Şubat iki bin yirmi altı"),
        ("tarih :01.02.2026", "tarih :bir Şubat iki bin yirmi altı"),
        ("tarih : 01.02.2026", "tarih : bir Şubat iki bin yirmi altı"),
        (
            "TARI\u{307}H:01.02.2026",
            "TARI\u{307}H:bir Şubat iki bin yirmi altı",
        ),
        ("saat: 09:30", "saat: dokuz otuz"),
        ("saat:09:30", "saat:dokuz otuz"),
        ("tarihi 29.02.2000", "tarihi yirmi dokuz Şubat iki bin"),
        ("tarih 2026-03-14", "tarih on dört Mart iki bin yirmi altı"),
        (
            "TARİH: 2024-02-29",
            "TARİH: yirmi dokuz Şubat iki bin yirmi dört",
        ),
        ("saat 12:30", "saat on iki otuz"),
        ("SAAT 12.30", "SAAT on iki otuz"),
        ("saat 00:00", "saat sıfır"),
        ("saat 09:00'da", "saat dokuzda"),
        ("saat 04:00'te", "saat dörtte"),
    ] {
        assert_eq!(normalized(input), expected, "{input}");
    }
    let normalizer = Normalizer::new().unwrap();
    for (input, kind, expected) in [
        ("1.234", HintKind::Cardinal, "bin iki yüz otuz dört"),
        ("00042", HintKind::Cardinal, "kırk iki"),
        ("001.234", HintKind::Cardinal, "bin iki yüz otuz dört"),
        ("-000", HintKind::Cardinal, "eksi sıfır"),
        ("1.234", HintKind::Digits, "bir iki üç dört"),
        ("12/34", HintKind::Digits, "bir iki üç dört"),
        ("(123)", HintKind::Digits, "bir iki üç"),
        (
            "+90(532)123",
            HintKind::Digits,
            "artı dokuz sıfır beş üç iki bir iki üç",
        ),
        ("12-34", HintKind::Digits, "bir iki üç dört"),
        ("-12", HintKind::Digits, "bir iki"),
        (
            "+90 (532) 123 45 67",
            HintKind::Digits,
            "artı dokuz sıfır beş üç iki bir iki üç dört beş altı yedi",
        ),
        ("00042", HintKind::Digits, "sıfır sıfır sıfır dört iki"),
        (
            "+90 532 123 45 67",
            HintKind::Digits,
            "artı dokuz sıfır beş üç iki bir iki üç dört beş altı yedi",
        ),
        ("01.02.2026", HintKind::Date, "bir Şubat iki bin yirmi altı"),
        (
            "2026-03-14",
            HintKind::Date,
            "on dört Mart iki bin yirmi altı",
        ),
        ("12:30", HintKind::Time, "on iki otuz"),
    ] {
        let result = normalizer
            .normalize(
                input,
                &NormalizeOptions {
                    hints: vec![Hint::new(SourceRange::new(0, input.len()), kind)],
                    ..Default::default()
                },
            )
            .unwrap();
        assert_eq!(result.normalized_text(), expected);
        assert!(result.complete());
    }
}
