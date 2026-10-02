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
fn engineering_failures_are_errors_in_both_policies() {
    let normalizer = Normalizer::new().unwrap();
    for policy in [AmbiguityPolicy::Preserve, AmbiguityPolicy::Reject] {
        let options = NormalizeOptions {
            ambiguity_policy: policy,
            ..Default::default()
        };
        for input in [
            "",
            " \n\t\r ",
            "\u{2003}",
            "a\0b",
            "a\u{202e}b",
            "a\u{2066}b",
            "\u{061c}12",
        ] {
            assert_eq!(
                normalizer.normalize(input, &options),
                Err(NormalizeError::InvalidInput)
            );
        }
        for bidi in [
            '\u{061c}', '\u{200e}', '\u{200f}', '\u{202a}', '\u{202b}', '\u{202c}', '\u{202d}',
            '\u{202e}', '\u{2066}', '\u{2067}', '\u{2068}', '\u{2069}',
        ] {
            assert_eq!(
                normalizer.normalize(&format!("a{bidi}12"), &options),
                Err(NormalizeError::InvalidInput)
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
    // Fewer than 4096 candidates; spoken large integers exceed the result budget.
    let input = "999999999999999999 ".repeat(1500);
    assert!(input.len() < MAX_INPUT_BYTES);
    for policy in [AmbiguityPolicy::Preserve, AmbiguityPolicy::Reject] {
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
