//! Engineering limits, cooperative controls, and thread safety.
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use normalizer_tr::{
    AmbiguityPolicy, Hint, HintKind, LimitKind, MAX_HINTS, MAX_INPUT_BYTES, NormalizeError,
    NormalizeOptions, Normalizer, SourceRange, WorkControl,
};

#[test]
fn engineering_failures_are_errors_in_every_policy() {
    let normalizer = Normalizer::new().unwrap();
    for policy in [
        AmbiguityPolicy::Preserve,
        AmbiguityPolicy::Reject,
        AmbiguityPolicy::Forced,
    ] {
        let options = NormalizeOptions {
            ambiguity_policy: policy,
            ..Default::default()
        };
        for input in [
            "",
            " \n\t\r ",
            "\u{2003}",
            "a\0b",
            "a\u{200f}\0",
            "\u{200f} \u{202e}",
        ] {
            assert_eq!(
                normalizer.normalize(input, &options),
                Err(NormalizeError::InvalidInput)
            );
        }
        // A bidirectional control is invalid input, except under Forced, which reads the text
        // as if it were not there and says nothing for it.
        for bidi in [
            '\u{061c}', '\u{200e}', '\u{200f}', '\u{202a}', '\u{202b}', '\u{202c}', '\u{202d}',
            '\u{202e}', '\u{2066}', '\u{2067}', '\u{2068}', '\u{2069}',
        ] {
            let result = normalizer.normalize(&format!("a{bidi}12"), &options);
            if policy != AmbiguityPolicy::Forced {
                assert_eq!(result, Err(NormalizeError::InvalidInput));
                continue;
            }
            let result = result.unwrap();
            let unmarked = normalizer.normalize("a12", &options).unwrap();
            assert_eq!(result.normalized_text(), unmarked.normalized_text());
            // The control stays inside the original coordinates, read by the span around it.
            let ends: Vec<_> = result.segments().iter().map(|s| s.range().end()).collect();
            assert_eq!(ends.last(), Some(&(3 + bidi.len_utf8())), "{bidi:?}");
            let spaced = normalizer
                .normalize(&format!("a {bidi} 12"), &options)
                .unwrap();
            assert!(
                spaced.segments().iter().any(|segment| {
                    segment.range().start() <= 2
                        && segment.range().end() >= 2 + bidi.len_utf8()
                        && segment.text().is_empty()
                        && segment.rule_id() == "forced.spoken"
                }),
                "{bidi:?}"
            );
        }
        assert_eq!(
            normalizer.normalize(&"a".repeat(MAX_INPUT_BYTES + 1), &options),
            Err(NormalizeError::LimitExceeded(LimitKind::Input))
        );
        let mut many_hints = options.clone();
        many_hints.hints =
            vec![Hint::new(SourceRange::new(0, 1), HintKind::Cardinal); MAX_HINTS + 1];
        assert_eq!(
            normalizer.normalize("1", &many_hints),
            Err(NormalizeError::LimitExceeded(LimitKind::Hints))
        );
        assert_eq!(
            normalizer.normalize(&"1 ".repeat(4097), &options),
            Err(NormalizeError::LimitExceeded(LimitKind::Candidates))
        );
        assert_eq!(
            normalizer.normalize(&"0 ".repeat(4097), &options),
            Err(NormalizeError::LimitExceeded(LimitKind::Candidates))
        );
        let cancelled = WorkControl::default();
        cancelled.clone().cancel();
        assert_eq!(
            normalizer.normalize_controlled("25 TL", &options, &cancelled),
            Err(NormalizeError::Cancelled)
        );
        let deadline = WorkControl::new(Some(Instant::now() - Duration::from_secs(1)));
        assert_eq!(
            normalizer.normalize_controlled("25 TL", &options, &deadline),
            Err(NormalizeError::Cancelled)
        );
    }
    assert!(
        normalizer
            .normalize(&"a".repeat(MAX_INPUT_BYTES), &NormalizeOptions::default())
            .unwrap()
            .complete()
    );
}

#[test]
fn result_allocation_limit_counts_segment_and_final_text_buffers() {
    let normalizer = Normalizer::new().unwrap();
    // Fewer than 4096 candidates; spoken large quantities exceed the result budget. A bare
    // digit run this long would be an identifier under Forced, and said digit by digit.
    let input = "999999999999999999 kg ".repeat(1400);
    assert!(input.len() < MAX_INPUT_BYTES);
    for policy in [
        AmbiguityPolicy::Preserve,
        AmbiguityPolicy::Reject,
        AmbiguityPolicy::Forced,
    ] {
        assert!(matches!(
            normalizer.normalize(
                &input,
                &NormalizeOptions {
                    ambiguity_policy: policy,
                    ..Default::default()
                }
            ),
            Err(NormalizeError::LimitExceeded(LimitKind::Result))
        ));
    }
}

#[test]
fn immutable_normalizer_is_send_sync_and_concurrent() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Normalizer>();
    assert_send_sync::<WorkControl>();
    let normalizer = Arc::new(Normalizer::new().unwrap());
    let expected = normalizer
        .normalize("25 TL; %4'lük", &NormalizeOptions::default())
        .unwrap();
    let threads: Vec<_> = (0..8)
        .map(|_| {
            let normalizer = Arc::clone(&normalizer);
            std::thread::spawn(move || {
                normalizer
                    .normalize("25 TL; %4'lük", &NormalizeOptions::default())
                    .unwrap()
            })
        })
        .collect();
    for thread in threads {
        assert_eq!(thread.join().unwrap(), expected);
    }
}
