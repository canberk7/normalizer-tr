//! Explicit reader precedence and centrally sealed source claims.
use normalizer_tr::{
    Hint, HintKind, NormalizeError, NormalizeOptions, Normalizer, SegmentKind, SourceRange,
};

#[test]
fn structured_claims_cannot_be_cut_by_numeric_hints() {
    let n = Normalizer::new().unwrap();
    for (text, end) in [
        ("3 + 4", 1),
        ("5 MG", 1),
        ("25 TL", 2),
        ("10-15 kişi", 2),
        ("AB12 25 TL", 2),
    ] {
        assert_eq!(
            n.normalize(
                text,
                &NormalizeOptions {
                    hints: vec![Hint::new(SourceRange::new(0, end), HintKind::Cardinal)],
                    ..Default::default()
                }
            ),
            Err(NormalizeError::InvalidHint),
            "{text}"
        );
    }
    let input = "0850 222 33 44";
    let r = n
        .normalize(
            input,
            &NormalizeOptions {
                hints: vec![Hint::new(
                    SourceRange::new(0, input.len()),
                    HintKind::Digits,
                )],
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(r.segments()[0].kind(), SegmentKind::Digits);
    assert_eq!(
        r.normalized_text(),
        "sıfır sekiz beş sıfır iki iki iki üç üç dört dört"
    );
}

#[test]
fn addresses_and_quantities_win_before_component_protection() {
    let n = Normalizer::new().unwrap();
    let input = "https://ornek.com/a@b?x=2&y=3";
    let r = n.normalize(input, &NormalizeOptions::default()).unwrap();
    assert!(r.complete());
    assert_eq!(r.segments()[0].kind(), SegmentKind::Electronic);
    assert_eq!(r.segments()[0].range(), SourceRange::new(0, input.len()));
    let r = n
        .normalize("5 kg + 2 kg", &NormalizeOptions::default())
        .unwrap();
    assert!(!r.complete());
    assert_eq!(r.normalized_text(), "5 kg + 2 kg");
    assert_eq!(r.segments()[0].range(), SourceRange::new(0, 11));
    let r = n
        .normalize("tarih 14.03.2026 kg", &NormalizeOptions::default())
        .unwrap();
    assert!(!r.complete());
    assert_eq!(r.normalized_text(), "tarih 14.03.2026 kg");
}

#[test]
fn explicit_context_roles_are_carried_into_preserved_words() {
    let r = Normalizer::new()
        .unwrap()
        .normalize(
            "II. DÜNYA SAVAŞI ve XXI. YÜZYIL",
            &NormalizeOptions::default(),
        )
        .unwrap();
    assert!(r.complete());
    assert_eq!(
        r.normalized_text(),
        "ikinci DÜNYA SAVAŞI ve yirmi birinci YÜZYIL"
    );
}
