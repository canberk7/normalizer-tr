//! Property coverage for mapped partitions and exact domain rendering.
use std::sync::OnceLock;

use normalizer_tr::{NormalizeOptions, Normalizer, SegmentKind};
use proptest::prelude::*;
use unicode_segmentation::UnicodeSegmentation;

fn normalizer() -> &'static Normalizer {
    static NORMALIZER: OnceLock<Normalizer> = OnceLock::new();
    NORMALIZER.get_or_init(|| Normalizer::new().unwrap())
}

fn partition(input: &str) {
    let result = normalizer().normalize(input, &NormalizeOptions::default());
    if input.trim().is_empty()
        || input.chars().any(|ch| {
            matches!(ch,
        '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
        })
    {
        assert_eq!(result, Err(normalizer_tr::NormalizeError::InvalidInput));
        return;
    }
    let result = result.unwrap();
    let repeated = normalizer()
        .normalize(input, &NormalizeOptions::default())
        .unwrap();
    assert_eq!(result, repeated);
    let mut boundaries: Vec<_> = input.grapheme_indices(true).map(|(i, _)| i).collect();
    boundaries.push(input.len());
    let mut cursor = 0;
    for segment in result.segments() {
        assert_eq!(segment.range().start(), cursor);
        assert!(boundaries.contains(&cursor));
        cursor = segment.range().end();
        assert!(boundaries.contains(&cursor));
        if matches!(
            segment.kind(),
            SegmentKind::Verbatim | SegmentKind::Unresolved
        ) {
            assert_eq!(segment.text(), &input[segment.range().start()..cursor]);
        }
    }
    assert_eq!(cursor, input.len());
    assert_eq!(
        result
            .segments()
            .iter()
            .map(|s| s.text())
            .collect::<String>(),
        result.normalized_text()
    );
    assert_eq!(result.complete(), result.issues().is_empty());
    let unresolved: Vec<_> = result
        .segments()
        .iter()
        .filter(|s| s.kind() == SegmentKind::Unresolved)
        .map(|s| s.range())
        .collect();
    assert_eq!(
        unresolved,
        result
            .issues()
            .iter()
            .map(|issue| issue.range())
            .collect::<Vec<_>>()
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]
    #[test]
    fn unicode_partition_and_determinism(
        fragments in prop::collection::vec(prop_oneof![
            "[a-zA-Z0-9.,:%'/_+@-]{0,30}",
            Just("İstanbul".to_owned()),
            Just("o\u{308} 4’u\u{308}n".to_owned()),
            Just("👨‍👩‍👧".to_owned()),
            Just("25 TL".to_owned()),
            Just("14.03.2026'da saat 09:30'da".to_owned()),
        ], 0..30)
    ) {
        partition(&fragments.join(" "));
    }

    #[test]
    fn every_bounded_integer_is_complete(value in 0_u64..1_000_000_000_000_000_000) {
        let input = value.to_string();
        let result = normalizer().normalize(&input, &NormalizeOptions::default()).unwrap();
        prop_assert!(result.complete());
        partition(&input);
    }

    #[test]
    fn arbitrary_unicode_never_breaks_grapheme_mapping(chars in prop::collection::vec(any::<char>(), 0..150)) {
        let input: String = chars.into_iter().filter(|ch| !ch.is_control()).collect();
        partition(&input);
    }

    #[test]
    fn all_exact_cents_match_integer_minor_renderer(major in 0_u32..1_000_000, cents in 0_u8..100) {
        let input = format!("{major},{cents:02} TL");
        let result = normalizer().normalize(&input, &NormalizeOptions::default()).unwrap();
        let whole = normalizer().normalize(&major.to_string(), &NormalizeOptions::default()).unwrap();
        let expected = if cents == 0 {
            format!("{} Türk lirası", whole.normalized_text())
        } else {
            let minor = normalizer().normalize(&cents.to_string(), &NormalizeOptions::default()).unwrap();
            format!("{} Türk lirası {} kuruş", whole.normalized_text(), minor.normalized_text())
        };
        prop_assert!(result.complete());
        prop_assert_eq!(result.normalized_text(), expected);
    }
}
