//! Current partial/strict and invalid temporal expression contracts.
use normalizer_tr::{AmbiguityPolicy, NormalizeError, NormalizeOptions, Normalizer};

#[test]
fn partial_and_strict_share_diagnostics() {
    let n = Normalizer::new().unwrap();
    let r = n
        .normalize("25 TL; 1.234", &NormalizeOptions::default())
        .unwrap();
    assert_eq!(r.normalized_text(), "yirmi beş Türk lirası; 1.234");
    assert!(!r.complete());
    assert_eq!(
        n.normalize(
            "25 TL; 1.234",
            &NormalizeOptions {
                ambiguity_policy: AmbiguityPolicy::Reject,
                ..Default::default()
            }
        ),
        Err(NormalizeError::Unresolved(r.issues().to_vec()))
    );
}

#[test]
fn unsupported_values_do_not_rewrite_numeric_fragments() {
    let n = Normalizer::new().unwrap();
    for text in [
        "00042",
        "1.234",
        "12.05",
        "12:30",
        "1000000000000000000",
        "1,1234567890",
        "1.23,50 TL",
        "1,001 TL",
        "25 JPY",
        "3'ünci",
        "4'a",
        "6'da'nın",
        "%4'lik",
        "%6'luk",
        "abc123",
        "AB-12",
        "192.168.1.1",
        "1/2",
        "3-5",
        "1e3",
        "v1.2.3",
        "3 - 5",
        "1 / 2",
        "5kg",
        "25TL",
        "12..",
        "12,,",
        "5 KG",
        "5 l",
        "5 m^2",
        "١٢٣",
        "①",
    ] {
        let r = n.normalize(text, &NormalizeOptions::default()).unwrap();
        assert_eq!(r.normalized_text(), text, "{text}");
        assert!(!r.complete(), "{text}");
    }
}

#[test]
fn invalid_temporal_tokens_and_suffixed_iso_are_preserved_whole() {
    let n = Normalizer::new().unwrap();
    for expression in [
        "2025-02-29",
        "1900-02-29",
        "2026-13-01",
        "2026-04-31",
        "2026-00-10",
        "0000-01-01",
        "2026-3-14",
        "2026-03-4",
        "2026--03-14",
        "2026-03-14-25",
        "2026-03-14'te",
    ] {
        let source = format!("tarih {expression}; 25 TL");
        let r = n.normalize(&source, &NormalizeOptions::default()).unwrap();
        assert_eq!(
            r.normalized_text(),
            format!("tarih {expression}; yirmi beş Türk lirası")
        );
        assert_eq!(r.issues().len(), 1);
        assert_eq!(r.issues()[0].range().start(), 6);
        assert_eq!(r.issues()[0].range().end(), 6 + expression.len());
    }
}

#[test]
fn corrupt_frame_cannot_enable_uncued_date() {
    let n = Normalizer::new().unwrap();
    for source in [
        "14.03.2026'da saat 24:00'da",
        "29.02.2025'te saat 09:30'da",
        "14.03.2026'da saat 09:30'de",
        "14.03.2026'da başka 09:30'da",
    ] {
        let r = n.normalize(source, &NormalizeOptions::default()).unwrap();
        assert!(!r.complete());
        assert_eq!(
            r.segments()[0].kind(),
            normalizer_tr::SegmentKind::Unresolved
        );
    }
}
