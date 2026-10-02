//! Current normalization preserves original grapheme partitions.
use normalizer_tr::{
    AmbiguityPolicy, Hint, HintKind, NORMALIZER_ID, NormalizeError, NormalizeOptions, Normalizer,
    SegmentKind, SourceRange,
};
use proptest::prelude::*;
use unicode_segmentation::UnicodeSegmentation;

fn partition(input: &str, n: &Normalizer) {
    let r = n.normalize(input, &NormalizeOptions::default()).unwrap();
    let mut bounds: Vec<_> = input.grapheme_indices(true).map(|(i, _)| i).collect();
    bounds.push(input.len());
    let mut cursor = 0;
    for s in r.segments() {
        assert_eq!(s.range().start(), cursor);
        cursor = s.range().end();
        assert!(bounds.contains(&s.range().start()) && bounds.contains(&cursor));
        if matches!(s.kind(), SegmentKind::Verbatim | SegmentKind::Unresolved) {
            assert_eq!(s.text(), &input[s.range().start()..cursor]);
        }
    }
    assert_eq!(cursor, input.len());
    assert_eq!(
        r.segments().iter().map(|s| s.text()).collect::<String>(),
        r.normalized_text()
    );
    assert_eq!(r.complete(), r.issues().is_empty());
    assert_eq!(n.normalize(input, &NormalizeOptions::default()).unwrap(), r);
    if !r.complete() {
        assert_eq!(
            n.normalize(
                input,
                &NormalizeOptions {
                    ambiguity_policy: AmbiguityPolicy::Reject,
                    ..Default::default()
                }
            ),
            Err(NormalizeError::Unresolved(r.issues().to_vec()))
        );
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]
    #[test]
    fn mixed_new_compounds_keep_grapheme_coordinates(parts in prop::collection::vec(
        prop_oneof![Just("ö 1.'nin"),Just("TR33 0006 1005 1978 6457 8413 26"),Just("http://ornek.com/a01?x=%20#A"),
        Just("0850 222 33 44"),Just("2 m²"),Just("5 kg'dan"),Just("3'üncünün"),Just("II. Dünya Savaşı"),
        Just("#yapayzeka & PTT"),Just("10-15 kişi"),Just(";\u{301}12"),Just("4’u\u{308}n"),Just("bir\r\niki")],1..20)) {
        let text=parts.join(" ");
        partition(&text,&Normalizer::new().unwrap());
    }
}

#[test]
fn hints_use_original_unicode_coordinates() {
    let text = "ö IV";
    let hint = Hint::new(SourceRange::new(3, 5), HintKind::Roman);
    let options = NormalizeOptions {
        hints: vec![hint],
        ..Default::default()
    };
    let r = Normalizer::new()
        .unwrap()
        .normalize(text, &options)
        .unwrap();
    assert_eq!(r.normalized_text(), "ö dört");
    assert_eq!(r.segments()[1].range(), hint.range());
    for (input, end, kind) in [
        ("info@ornek.com", 4, HintKind::Electronic),
        ("0850 222 33 44", 4, HintKind::Telephone),
        ("10-15 kg", 5, HintKind::Range),
        ("II. Dünya Savaşı", 2, HintKind::Roman),
    ] {
        assert_eq!(
            Normalizer::new().unwrap().normalize(
                input,
                &NormalizeOptions {
                    hints: vec![Hint::new(SourceRange::new(0, end), kind)],
                    ..Default::default()
                }
            ),
            Err(NormalizeError::InvalidHint),
            "{input}"
        );
    }
}

#[test]
fn financial_is_exact_in_both_policies() {
    let input = include_str!("fixtures/financial-input.txt").trim_end_matches(['\r', '\n']);
    let expected = include_str!("fixtures/financial-output.txt").trim_end_matches(['\r', '\n']);
    assert_eq!(input.len(), 108);
    let n = Normalizer::new().unwrap();
    assert_eq!(n.normalizer_id(), NORMALIZER_ID);
    for policy in [AmbiguityPolicy::Preserve, AmbiguityPolicy::Reject] {
        let r = n
            .normalize(
                input,
                &NormalizeOptions {
                    ambiguity_policy: policy,
                    ..Default::default()
                },
            )
            .unwrap();
        assert_eq!(r.normalizer_id(), NORMALIZER_ID);
        assert!(r.complete());
        assert!(r.issues().is_empty());
        assert_eq!(r.normalized_text(), expected);
        assert_eq!(
            r.segments()
                .iter()
                .filter(|s| s.kind() != SegmentKind::Verbatim)
                .map(|s| (s.range().start(), s.range().end()))
                .collect::<Vec<_>>(),
            [(0, 13), (19, 27), (51, 60), (70, 73), (80, 95)]
        );
    }
}

#[test]
fn shared_resources_keep_thread_control_isolation() {
    let workers: Vec<_> = (0..16)
        .map(|_| {
            std::thread::spawn(move || {
                let n = Normalizer::new().unwrap();
                assert_eq!(n.normalizer_id(), NORMALIZER_ID);
                let r = n.normalize("25 USD", &NormalizeOptions::default()).unwrap();
                assert_eq!(r.normalizer_id(), NORMALIZER_ID);
                assert!(r.complete());
                let control = normalizer_tr::WorkControl::default();
                control.cancel();
                assert_eq!(
                    n.normalize_controlled("25 USD", &NormalizeOptions::default(), &control),
                    Err(NormalizeError::Cancelled)
                );
                assert_eq!(
                    n.normalize("25 USD", &NormalizeOptions::default()).unwrap(),
                    r
                );
            })
        })
        .collect();
    for worker in workers {
        worker.join().unwrap();
    }
}
