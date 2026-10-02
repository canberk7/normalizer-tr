//! Current bounded coverage, conservative ambiguity and typed intent.
use normalizer_tr::{
    AmbiguityPolicy, Hint, HintKind, NORMALIZER_ID, NormalizeError, NormalizeOptions, Normalizer,
    SegmentKind, SourceRange,
};

fn complete(input: &str, expected: &str) {
    let result = Normalizer::new()
        .unwrap()
        .normalize(input, &NormalizeOptions::default())
        .unwrap();
    assert!(result.complete(), "{input}: {:?}", result.issues());
    assert_eq!(result.normalized_text(), expected, "{input}");
    assert_eq!(result.normalizer_id(), NORMALIZER_ID);
}

#[test]
fn composite_source_target_and_derived_tail_goldens() {
    for (input, output) in [
        ("%3'ten", "yüzde üçten"),
        ("%3,25'ten", "yüzde üç virgül iki beşten"),
        ("TL'den", "Türk lirasından"),
        ("25 TL'den", "yirmi beş Türk lirasından"),
        ("5 kg'dan", "beş kilogramdan"),
        ("1.'nin", "birincinin"),
        ("4.'ye", "dördüncüye"),
        ("3'üncü'nün", "üçüncünün"),
        ("3'üncünün", "üçüncünün"),
        ("1'incinin", "birincinin"),
        ("6'ncı'ya", "altıncıya"),
        ("25,05 TL'den", "yirmi beş Türk lirası beş kuruştan"),
        ("25 USD'den", "yirmi beş dolardan"),
        ("25,05 USD'den", "yirmi beş dolar beş sentten"),
        ("KDV'den", "katma değer vergisinden"),
        ("PTT'ye", "pe te teye"),
        ("NATO'da", "natoda"),
        ("IBAN'ın", "ibanın"),
        ("TBMM'nin", "te be me menin"),
        ("Prof. Ali", "profesör Ali"),
        ("vb.", "ve benzeri"),
        ("PTT NATO IBAN", "pe te te nato iban"),
    ] {
        complete(input, output);
    }
}

#[test]
fn explicitly_allowed_unit_aliases_area_volume_rates() {
    for (symbol, word) in [
        ("mg", "miligram"),
        ("µg", "mikrogram"),
        ("μg", "mikrogram"),
        ("gr", "gram"),
        ("ml", "mililitre"),
        ("lt", "litre"),
        ("dk", "dakika"),
        ("sn", "saniye"),
        ("sa", "saat"),
        ("m²", "metrekare"),
        ("cm²", "santimetrekare"),
        ("km²", "kilometrekare"),
        ("m³", "metreküp"),
    ] {
        complete(&format!("2 {symbol}"), &format!("iki {word}"));
    }
    complete("90 km/sa", "saatte doksan kilometre");
    complete("90 km/h", "saatte doksan kilometre");
    complete("5 m/s", "saniyede beş metre");
    complete("-0,05 ml'den", "eksi sıfır virgül sıfır beş mililitreden");
    complete("5 m²'ye", "beş metrekareye");
    complete("5 m³'e", "beş metrekübe");
    complete("5 m³'ün", "beş metrekübün");
    complete("5 m³'te", "beş metreküpte");
    complete("2 sa'ten", "iki saatten");
    complete("2 sa'te", "iki saatte");
}

#[test]
fn four_currency_exact_positions_minor_units_and_signs() {
    for (input, output) in [
        ("$40", "kırk dolar"),
        ("€14,05", "on dört avro beş sent"),
        ("25 GBP", "yirmi beş sterlin"),
        ("-0,50 USD", "eksi sıfır dolar elli sent"),
        ("+0,00 EUR", "artı sıfır avro"),
        ("1.234,5 £", "bin iki yüz otuz dört sterlin elli peni"),
        ("GBP 25", "yirmi beş sterlin"),
        ("TL 25", "yirmi beş Türk lirası"),
        ("25€", "yirmi beş avro"),
        ("25€'ya", "yirmi beş avroya"),
        ("25₺'dan", "yirmi beş Türk lirasından"),
        ("₺25", "yirmi beş Türk lirası"),
    ] {
        complete(input, output);
    }
}

#[test]
fn contextual_ranges_keep_written_order_and_exact_fractional_zeros() {
    complete("10-15 Kişi", "on ila on beş Kişi");
    for (input, output) in [
        ("10-15 kişi", "on ila on beş kişi"),
        ("15-10 adet", "on beş ila on adet"),
        (
            "1,05-2,50 kg",
            "bir virgül sıfır beş ila iki virgül beş sıfır kilogram",
        ),
        ("-5--2 gün", "eksi beş ila eksi iki gün"),
        ("3–4 yaş", "üç ila dört yaş"),
    ] {
        complete(input, output);
    }
}

#[test]
fn national_phone_grouping_retains_zeros_and_international_prefix() {
    complete("telefon 25 TL", "telefon yirmi beş Türk lirası");
    complete("telefon 3 adet", "telefon üç adet");
    complete(
        "TELEFON 5321234567",
        "TELEFON beş yüz otuz iki yüz yirmi üç kırk beş altmış yedi",
    );
    complete(
        "0005 001 02 03",
        "sıfır sıfır sıfır beş sıfır sıfır bir sıfır iki sıfır üç",
    );
    complete("+90", "artı doksan");
    complete("+90532", "artı doksan bin beş yüz otuz iki");
    complete("0 1 2", "sıfır bir iki");
    complete("0 5", "sıfır beş");
    complete(
        "0 0 0 0 0 0 0 0 0 0 0",
        "sıfır sıfır sıfır sıfır sıfır sıfır sıfır sıfır sıfır sıfır sıfır",
    );
    complete(
        "0,000000000",
        "sıfır virgül sıfır sıfır sıfır sıfır sıfır sıfır sıfır sıfır sıfır",
    );
    complete(
        "0-123456789 kişi",
        "sıfır ila yüz yirmi üç milyon dört yüz elli altı bin yedi yüz seksen dokuz kişi",
    );
    complete(
        "0850 222 33 44",
        "sıfır sekiz yüz elli iki yüz yirmi iki otuz üç kırk dört",
    );
    complete(
        "0850-222-33-44",
        "sıfır sekiz yüz elli iki yüz yirmi iki otuz üç kırk dört",
    );
    complete(
        "08502223344",
        "sıfır sekiz yüz elli iki yüz yirmi iki otuz üç kırk dört",
    );
    complete(
        "+90 (532) 123 45 67",
        "artı doksan beş yüz otuz iki yüz yirmi üç kırk beş altmış yedi",
    );
    complete(
        "0505 001 02 03",
        "sıfır beş yüz beş sıfır sıfır bir sıfır iki sıfır üç",
    );
    complete(
        "telefon 5321234567",
        "telefon beş yüz otuz iki yüz yirmi üç kırk beş altmış yedi",
    );
}

#[test]
fn full_turkish_iban_checksum_and_groups() {
    let expected = "te re üç üç, sıfır sıfır sıfır altı, bir sıfır sıfır beş, bir dokuz yedi sekiz, altı dört beş yedi, sekiz dört bir üç, iki altı";
    complete("TR330006100519786457841326", expected);
    complete("TR33 0006 1005 1978 6457 8413 26", expected);
    let n = Normalizer::new().unwrap();
    for last in '0'..='9' {
        if last == '6' {
            continue;
        }
        let input = format!("TR33000610051978645784132{last}");
        let result = n.normalize(&input, &NormalizeOptions::default()).unwrap();
        assert!(!result.complete());
        assert_eq!(result.normalized_text(), input);
    }
}

#[test]
fn iban_capture_does_not_claim_ordinary_words_or_following_lexical_quantities() {
    complete("tren ve trafik bugün sakin", "tren ve trafik bugün sakin");
    complete("trafik 25 örnek içerir", "trafik yirmi beş örnek içerir");
    let product = Normalizer::new()
        .unwrap()
        .normalize("AB12 25 TL", &NormalizeOptions::default())
        .unwrap();
    assert_eq!(product.normalized_text(), "AB12 yirmi beş Türk lirası");
    assert!(!product.complete());
    assert_eq!(product.issues().len(), 1);
    assert_eq!(product.issues()[0].range(), SourceRange::new(0, 4));
    assert_eq!(product.segments()[2].kind(), SegmentKind::Money);
    assert_eq!(product.segments()[2].range(), SourceRange::new(5, 10));
    let iban = "TR33 0006 1005 1978 6457 8413 26";
    let spoken = "te re üç üç, sıfır sıfır sıfır altı, bir sıfır sıfır beş, bir dokuz yedi sekiz, altı dört beş yedi, sekiz dört bir üç, iki altı";
    let n = Normalizer::new().unwrap();
    for (tail, expected) in [
        ("NATO", "nato"),
        ("PTT", "pe te te"),
        ("25 TL", "yirmi beş Türk lirası"),
    ] {
        let source = format!("{iban} {tail}");
        let r = n.normalize(&source, &NormalizeOptions::default()).unwrap();
        assert!(r.complete());
        assert!(r.issues().is_empty());
        assert_eq!(r.normalized_text(), format!("{spoken} {expected}"));
        assert_eq!(r.segments()[0].kind(), SegmentKind::Iban);
        assert_eq!(r.segments()[0].range(), SourceRange::new(0, iban.len()));
        assert_eq!(
            r.segments()[2].range(),
            SourceRange::new(iban.len() + 1, source.len())
        );
    }
    for expression in [
        "TR3A 0006 1005 1978 6457 8413 26",
        "TR33 0006 1005 1978 6457 8413 2A",
        "TR33 00A6 1005 1978 6457 8413 26",
        "TR33 0006 1005 1978 6457 8413 26 00",
        "TRAA 0006 1005 1978 6457 8413 26",
    ] {
        let source = format!("{expression} sonraki sözcükler");
        let r = n.normalize(&source, &NormalizeOptions::default()).unwrap();
        assert!(!r.complete());
        assert_eq!(r.normalized_text(), source);
        assert_eq!(r.issues().len(), 1);
        assert_eq!(r.issues()[0].range(), SourceRange::new(0, expression.len()));
        assert_eq!(r.segments()[0].kind(), SegmentKind::Unresolved);
        assert_eq!(
            r.segments()[0].range(),
            SourceRange::new(0, expression.len())
        );
    }
}

#[test]
fn web_addresses_and_role_aware_symbols_preserve_all_components() {
    complete("WEB ornek.net", "WEB ornek nokta net");
    let long = format!("https://ornek.com/{}", "a".repeat(4096));
    complete(
        &long,
        &format!(
            "ha te te pe es iki nokta eğik çizgi eğik çizgi ornek nokta kom eğik çizgi {}",
            "a".repeat(4096)
        ),
    );
    complete("info@ornek.com", "info et ornek nokta kom");
    complete(
        "User01@Ornek.com.tr",
        "User sıfır bir et Ornek nokta com nokta te re",
    );
    complete("#yapayzeka & araştırma", "hashtag yapayzeka ve araştırma");
    complete(
        "www.ornek.com",
        "çift ve çift ve çift ve nokta ornek nokta kom",
    );
    complete("web ornek.net", "web ornek nokta net");
    complete(
        "HTTPS://Ornek.com/A00",
        "ha te te pe es iki nokta eğik çizgi eğik çizgi Ornek nokta kom eğik çizgi A sıfır sıfır",
    );
    complete(
        "https://ornek.com/a@b?x=2",
        "ha te te pe es iki nokta eğik çizgi eğik çizgi ornek nokta kom eğik çizgi a et b soru işareti x eşittir iki",
    );
    complete(
        "https://ornek.com:8080/a01?x=2&y=%20#bolum",
        "ha te te pe es iki nokta eğik çizgi eğik çizgi ornek nokta kom iki nokta sekiz sıfır sekiz sıfır eğik çizgi a sıfır bir soru işareti x eşittir iki ve y eşittir yüzde iki sıfır kare bolum",
    );
    complete(
        "(https://ornek.com/a(b)).",
        "(ha te te pe es iki nokta eğik çizgi eğik çizgi ornek nokta kom eğik çizgi a aç parantez b kapat parantez).",
    );
}

#[test]
fn canonical_contextual_romans_and_all_new_hints() {
    complete("II. DÜNYA SAVAŞI", "ikinci DÜNYA SAVAŞI");
    complete("II. dünya savaşı", "ikinci dünya savaşı");
    complete("II. Dünya Savaşı", "ikinci Dünya Savaşı");
    complete("XXI. yüzyıl", "yirmi birinci yüzyıl");
    let n = Normalizer::new().unwrap();
    for (input, kind, output, segment) in [
        (
            "25.",
            HintKind::Ordinal,
            "yirmi beşinci",
            SegmentKind::Ordinal,
        ),
        (
            "1.'nin",
            HintKind::Ordinal,
            "birincinin",
            SegmentKind::Ordinal,
        ),
        ("IV", HintKind::Roman, "dört", SegmentKind::Roman),
        ("II.", HintKind::Roman, "ikinci", SegmentKind::Roman),
        ("IV'üncü", HintKind::Roman, "dördüncü", SegmentKind::Roman),
        (
            "IV'üncünün",
            HintKind::Roman,
            "dördüncünün",
            SegmentKind::Roman,
        ),
        ("II.'nin", HintKind::Roman, "ikincinin", SegmentKind::Roman),
        (
            "MMMCMXCIX",
            HintKind::Roman,
            "üç bin dokuz yüz doksan dokuz",
            SegmentKind::Roman,
        ),
        (
            "10-15",
            HintKind::Range,
            "on ila on beş",
            SegmentKind::Range,
        ),
        (
            "5321234567",
            HintKind::Telephone,
            "beş yüz otuz iki yüz yirmi üç kırk beş altmış yedi",
            SegmentKind::Telephone,
        ),
        (
            "ornek.com",
            HintKind::Electronic,
            "ornek nokta kom",
            SegmentKind::Electronic,
        ),
    ] {
        let options = NormalizeOptions {
            hints: vec![Hint::new(SourceRange::new(0, input.len()), kind)],
            ..Default::default()
        };
        let result = n.normalize(input, &options).unwrap();
        assert!(result.complete());
        assert_eq!(result.normalized_text(), output);
        assert_eq!(result.segments()[0].kind(), segment);
    }
}

#[test]
fn invalid_ambiguous_and_unsupported_composites_are_atomic() {
    let n = Normalizer::new().unwrap();
    for input in [
        "IV",
        "I.",
        "Toplam 25.",
        "10-15",
        "TR12 0006 1005",
        "tr330006100519786457841326",
        "TR330006100519786457841327",
        "TR000006100519786457841326",
        "TR990006100519786457841326",
        "DE330006100519786457841326",
        "TR33 00061005 1978 6457 8413 26",
        "TR33 0006 1005 1978 6457 8413 2A",
        "TR3A 0006 1005 1978 6457 8413 26",
        "TR33 00A6 1005 1978 6457 8413 26",
        "TR33 0006 1005 1978 6457 8413 26 00",
        "%3'den",
        "%3,25'den",
        "%3'ten'de",
        "25 TL'dan",
        "5 kg'den",
        "1.'nın",
        "4.'ya",
        "1,005 USD",
        "25 JPY",
        "25 XAU",
        "25 USD EUR",
        "$25 USD",
        "25€ EUR",
        "5 kg + 2 kg",
        "10-15 kişi = 2",
        "5 MG",
        "Prof.'e'nin'den",
        "vb.'nin'in",
        "5 Μg",
        "5 km/s",
        "5 m^2",
        "2 sa'tan",
        "2 sa'ta",
        "1e3 kg",
        "90 mph",
        "3 + 4 = 7",
        "0850 22 33 44",
        "tel 123",
        "0532 123 45 6X",
        "+90 1 2",
        "www.örnek.com",
        "https://xn--rnek-4qa.com/a",
        "info@xn--rnek-4qa.com",
        "user@örnek.com",
        "\"user\"@ornek.com",
        "\"user name\"@ornek.com",
        "ftp://ornek.com/a",
        "https://user:secret@ornek.com/a",
        "https://ornek.com/%ZZ",
        "https://ornek.com:99999/a",
        "https://ornek.invalid/a",
        "ornek.com",
    ] {
        let r = n.normalize(input, &NormalizeOptions::default()).unwrap();
        assert!(!r.complete(), "{input}");
        assert_eq!(r.normalized_text(), input, "{input}");
        assert_eq!(
            n.normalize(
                input,
                &NormalizeOptions {
                    ambiguity_policy: AmbiguityPolicy::Reject,
                    ..Default::default()
                }
            ),
            Err(NormalizeError::Unresolved(r.issues().to_vec())),
            "{input}"
        );
    }
}

#[test]
fn malformed_new_hints_do_not_legalize_values_or_partial_ranges() {
    let n = Normalizer::new().unwrap();
    for (input, kind) in [
        ("IIII", HintKind::Roman),
        ("VX", HintKind::Roman),
        ("MMMM", HintKind::Roman),
        ("25", HintKind::Ordinal),
        ("1,2.", HintKind::Ordinal),
        ("12:34", HintKind::Range),
        ("+44 5321234567", HintKind::Telephone),
        ("https://ornek.com/%G0", HintKind::Electronic),
    ] {
        let r = n.normalize(
            input,
            &NormalizeOptions {
                hints: vec![Hint::new(SourceRange::new(0, input.len()), kind)],
                ..Default::default()
            },
        );
        assert_eq!(r, Err(NormalizeError::InvalidHint), "{input}");
    }
}
