//! Original UTF-8 grapheme mapping and hint validity.
use normalizer_tr::{
    Hint, HintKind, NormalizeError, NormalizeOptions, Normalizer, SegmentKind, SourceRange,
};

#[test]
fn nfc_recognition_keeps_original_graphemes_and_case() {
    let input = "İstanbul 👩‍👩‍👧‍👦: o\u{308}n 4’u\u{308}n ve 25 TL";
    let result = Normalizer::new()
        .unwrap()
        .normalize(input, &NormalizeOptions::default())
        .unwrap();
    assert_eq!(
        result.normalized_text(),
        "İstanbul 👩‍👩‍👧‍👦: o\u{308}n dördün ve yirmi beş Türk lirası"
    );
    assert!(result.complete());
    let inflected = result
        .segments()
        .iter()
        .find(|s| s.kind() == SegmentKind::Cardinal)
        .unwrap();
    assert_eq!(
        &input[inflected.range().start()..inflected.range().end()],
        "4’u\u{308}n"
    );
    assert_eq!(
        result
            .segments()
            .iter()
            .map(|s| s.text())
            .collect::<String>(),
        result.normalized_text()
    );
}

#[test]
fn hint_ranges_are_original_non_overlapping_grapheme_safe_whole_expressions() {
    let normalizer = Normalizer::new().unwrap();
    let input = "e\u{301} 00042";
    let start = input.find('0').unwrap();
    let result = normalizer
        .normalize(
            input,
            &NormalizeOptions {
                hints: vec![Hint::new(
                    SourceRange::new(start, input.len()),
                    HintKind::Digits,
                )],
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(
        result.normalized_text(),
        "e\u{301} sıfır sıfır sıfır dört iki"
    );
    for (input, hints) in [
        (
            "e\u{301} 12",
            vec![Hint::new(SourceRange::new(0, 1), HintKind::Digits)],
        ),
        (
            "ü 12",
            vec![Hint::new(SourceRange::new(1, 2), HintKind::Digits)],
        ),
        (
            "12",
            vec![Hint::new(SourceRange::new(0, 3), HintKind::Cardinal)],
        ),
        (
            "12",
            vec![Hint::new(SourceRange::new(1, 1), HintKind::Cardinal)],
        ),
        (
            "123",
            vec![Hint::new(SourceRange::new(0, 2), HintKind::Cardinal)],
        ),
        (
            "AB123",
            vec![Hint::new(SourceRange::new(2, 5), HintKind::Cardinal)],
        ),
        (
            "25 TL",
            vec![Hint::new(SourceRange::new(0, 2), HintKind::Cardinal)],
        ),
        (
            "12",
            vec![Hint::new(SourceRange::new(0, 2), HintKind::Cardinal); 2],
        ),
        (
            "00,42",
            vec![Hint::new(SourceRange::new(0, 5), HintKind::Cardinal)],
        ),
        (
            "31.04.2026",
            vec![Hint::new(SourceRange::new(0, 10), HintKind::Date)],
        ),
        (
            "24:00",
            vec![Hint::new(SourceRange::new(0, 5), HintKind::Time)],
        ),
        (
            "2026-03-14'te",
            vec![Hint::new(SourceRange::new(0, 13), HintKind::Date)],
        ),
        (
            "+90 532 123",
            vec![Hint::new(SourceRange::new(0, 3), HintKind::Digits)],
        ),
    ] {
        assert_eq!(
            normalizer.normalize(
                input,
                &NormalizeOptions {
                    hints,
                    ..Default::default()
                }
            ),
            Err(NormalizeError::InvalidHint),
            "{input}"
        );
    }
}

#[test]
fn punctuation_with_combining_marks_remains_grapheme_safe() {
    let normalizer = Normalizer::new().unwrap();
    for input in [";\u{301}12", ",\u{301}12"] {
        let result = normalizer
            .normalize(input, &NormalizeOptions::default())
            .unwrap();
        assert_eq!(result.normalized_text(), input);
        assert!(!result.complete());
        assert_eq!(result.issues()[0].range(), SourceRange::new(0, input.len()));
    }
    for input in ["e;\u{301}12", "12;\u{301}34", "\u{301} 25 TL"] {
        let result = normalizer
            .normalize(input, &NormalizeOptions::default())
            .unwrap();
        assert_eq!(
            result
                .segments()
                .iter()
                .map(|s| s.text())
                .collect::<String>(),
            result.normalized_text()
        );
        for segment in result.segments() {
            assert!(input.is_char_boundary(segment.range().start()));
            assert!(input.is_char_boundary(segment.range().end()));
        }
    }
}

#[test]
fn curly_abbreviation_lookup_preserves_the_original_whole_range() {
    let input = "ü TBMM’ye!";
    let result = Normalizer::new()
        .unwrap()
        .normalize(input, &NormalizeOptions::default())
        .unwrap();
    assert_eq!(result.normalized_text(), "ü te be me meye!");
    assert!(result.complete());
    let abbreviation = result
        .segments()
        .iter()
        .find(|s| s.kind() == SegmentKind::Abbreviation)
        .unwrap();
    assert_eq!(
        &input[abbreviation.range().start()..abbreviation.range().end()],
        "TBMM’ye"
    );
}

#[test]
fn maximum_hints_are_accepted_and_sorted_without_changing_gaps() {
    let input = "1;".repeat(normalizer_tr::MAX_HINTS);
    let hints = (0..normalizer_tr::MAX_HINTS)
        .rev()
        .map(|index| {
            Hint::new(
                SourceRange::new(index * 2, index * 2 + 1),
                HintKind::Cardinal,
            )
        })
        .collect();
    let result = Normalizer::new()
        .unwrap()
        .normalize(
            &input,
            &NormalizeOptions {
                hints,
                ..Default::default()
            },
        )
        .unwrap();
    assert!(result.complete());
    assert_eq!(
        result.normalized_text(),
        "bir;".repeat(normalizer_tr::MAX_HINTS)
    );
}

#[test]
fn digits_hints_consume_all_promised_separators_at_original_ranges() {
    let normalizer = Normalizer::new().unwrap();
    for expression in ["1.234", "12/34", "(123)", "+90 (532) 123 45 67"] {
        let input = format!("ö\u{308} {expression} sonra");
        let start = "ö\u{308} ".len();
        let range = SourceRange::new(start, start + expression.len());
        let result = normalizer
            .normalize(
                &input,
                &NormalizeOptions {
                    hints: vec![Hint::new(range, HintKind::Digits)],
                    ..Default::default()
                },
            )
            .unwrap();
        assert!(result.complete());
        let digits = result
            .segments()
            .iter()
            .find(|s| s.kind() == SegmentKind::Digits)
            .unwrap();
        assert_eq!(digits.range(), range);
        assert_eq!(&input[range.start()..range.end()], expression);
    }
    for (input, range) in [
        ("12/34", SourceRange::new(0, 2)),
        ("+90 (532) 123 45 67", SourceRange::new(5, 8)),
        ("12@34", SourceRange::new(0, 5)),
        ("12:34", SourceRange::new(0, 5)),
        ("12\t34", SourceRange::new(0, 5)),
        ("12+34", SourceRange::new(0, 5)),
    ] {
        assert_eq!(
            normalizer.normalize(
                input,
                &NormalizeOptions {
                    hints: vec![Hint::new(range, HintKind::Digits)],
                    ..Default::default()
                }
            ),
            Err(NormalizeError::InvalidHint)
        );
    }
}

#[test]
fn colon_cues_use_nfc_lookup_but_keep_original_case_and_coordinates() {
    let input = "TARI\u{307}H:01.02.2026 ve saat:09:30";
    let result = Normalizer::new()
        .unwrap()
        .normalize(input, &NormalizeOptions::default())
        .unwrap();
    assert_eq!(
        result.normalized_text(),
        "TARI\u{307}H:bir Şubat iki bin yirmi altı ve saat:dokuz otuz"
    );
    assert!(result.complete());
    for (kind, expression) in [
        (SegmentKind::Date, "01.02.2026"),
        (SegmentKind::Time, "09:30"),
    ] {
        let start = input.find(expression).unwrap();
        let segment = result.segments().iter().find(|s| s.kind() == kind).unwrap();
        assert_eq!(
            segment.range(),
            SourceRange::new(start, start + expression.len())
        );
    }
}
