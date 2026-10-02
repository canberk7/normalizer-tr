//! Exact report baseline and reusable contrasting financial frames.
use normalizer_tr::{AmbiguityPolicy, NormalizeOptions, Normalizer, SegmentKind, SourceRange};

#[test]
fn exact_financial_baseline_and_original_partition() {
    let input = include_str!("fixtures/financial-input.txt").trim_end_matches(['\r', '\n']);
    let expected = include_str!("fixtures/financial-output.txt").trim_end_matches(['\r', '\n']);
    assert_eq!(input.len(), 108);
    let normalizer = Normalizer::new().unwrap();
    for policy in [AmbiguityPolicy::Preserve, AmbiguityPolicy::Reject] {
        let result = normalizer
            .normalize(
                input,
                &NormalizeOptions {
                    ambiguity_policy: policy,
                    ..Default::default()
                },
            )
            .unwrap();
        assert_eq!(result.normalized_text(), expected);
        assert!(result.complete());
        assert!(result.issues().is_empty());
        let transformed: Vec<_> = result
            .segments()
            .iter()
            .filter(|s| s.kind() != SegmentKind::Verbatim)
            .map(|s| (s.kind(), s.range()))
            .collect();
        assert_eq!(
            transformed,
            [
                (SegmentKind::Date, SourceRange::new(0, 13)),
                (SegmentKind::Time, SourceRange::new(19, 27)),
                (SegmentKind::Percent, SourceRange::new(51, 60)),
                (SegmentKind::Abbreviation, SourceRange::new(70, 73)),
                (SegmentKind::Money, SourceRange::new(80, 95)),
            ]
        );
        let mut cursor = 0;
        for segment in result.segments() {
            assert_eq!(segment.range().start(), cursor);
            cursor = segment.range().end();
            if segment.kind() == SegmentKind::Verbatim {
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
            expected
        );
    }
}

#[test]
fn reusable_temporal_financial_variants() {
    let normalizer = Normalizer::new().unwrap();
    let cases = [
        (
            "29.02.2024'te saat 10:00'da %4'lük ve 1.000,05 TL.",
            "yirmi dokuz Şubat iki bin yirmi dörtte saat onda yüzde dörtlük ve bin Türk lirası beş kuruş.",
        ),
        (
            "01.01.2000'de saat 01:04'te %6'lık KDV 0,5 TRY",
            "bir Ocak iki binde saat bir dörtte yüzde altılık katma değer vergisi sıfır Türk lirası elli kuruş",
        ),
        (
            "31.12.2025'te saat 23:59'da %10'luk -0,00 TL",
            "otuz bir Aralık iki bin yirmi beşte saat yirmi üç elli dokuzda yüzde onluk eksi sıfır Türk lirası",
        ),
    ];
    for (input, expected) in cases {
        let result = normalizer
            .normalize(input, &NormalizeOptions::default())
            .unwrap();
        assert_eq!(result.normalized_text(), expected, "{input}");
        assert!(result.complete(), "{input}: {:?}", result.issues());
    }
}
