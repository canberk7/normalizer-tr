//! Current semantic characterization; not a compatibility mode or old-release promise.
#![cfg(feature = "serde")]

use normalizer_tr::{
    AmbiguityPolicy, Hint, HintKind, NormalizeError, NormalizeOptions, Normalizer, SourceRange,
};
use serde_json::{Value, json};

#[test]
fn current_readings_ranges_issues_and_errors_survive_refactoring() {
    let mut corpus: Vec<Value> =
        serde_json::from_str(include_str!("../benches/corpus.json")).unwrap();
    corpus.extend(
        serde_json::from_str::<Vec<Value>>(include_str!("../benches/intent-corpus.json")).unwrap(),
    );
    let expected: Value =
        serde_json::from_str(include_str!("fixtures/current-contract.json")).unwrap();
    let forced: Value =
        serde_json::from_str(include_str!("fixtures/forced-contract.json")).unwrap();
    let normalizer = Normalizer::new().unwrap();
    for case in corpus {
        for (policy, name, expected) in [
            (AmbiguityPolicy::Preserve, "preserve", &expected),
            (AmbiguityPolicy::Reject, "reject", &expected),
            (AmbiguityPolicy::Forced, "forced", &forced),
        ] {
            let mut options = NormalizeOptions {
                ambiguity_policy: policy,
                ..Default::default()
            };
            if let Some(hint) = case.get("hint") {
                let kind = match hint["kind"].as_str().unwrap() {
                    "cardinal" => HintKind::Cardinal,
                    "digits" => HintKind::Digits,
                    "date" => HintKind::Date,
                    "time" => HintKind::Time,
                    "ordinal" => HintKind::Ordinal,
                    "roman" => HintKind::Roman,
                    "range" => HintKind::Range,
                    "telephone" => HintKind::Telephone,
                    "electronic" => HintKind::Electronic,
                    _ => unreachable!(),
                };
                options.hints.push(Hint::new(
                    SourceRange::new(
                        hint["start"].as_u64().unwrap() as usize,
                        hint["end"].as_u64().unwrap() as usize,
                    ),
                    kind,
                ));
            }
            let mut actual = match normalizer.normalize(case["text"].as_str().unwrap(), &options) {
                Ok(result) => json!({"result":result}),
                Err(NormalizeError::Unresolved(issues)) => {
                    json!({"error":"unresolved","issues":issues})
                }
                Err(error) => json!({"error":format!("{error:?}")}),
            };
            if let Some(result) = actual.get_mut("result").and_then(Value::as_object_mut) {
                result.remove("normalizer_id");
                for segment in result.get_mut("segments").unwrap().as_array_mut().unwrap() {
                    segment.as_object_mut().unwrap().remove("rule_id");
                }
            }
            let key = format!("{}:{name}", case["id"].as_str().unwrap());
            assert_eq!(actual, expected["outcomes"][&key], "{key}");
        }
    }
}

#[test]
fn forced_outcomes_keep_the_issues_ranges_and_resolved_kinds_of_preserve() {
    let preserve: Value =
        serde_json::from_str(include_str!("fixtures/current-contract.json")).unwrap();
    let forced: Value =
        serde_json::from_str(include_str!("fixtures/forced-contract.json")).unwrap();
    let forced = forced["outcomes"].as_object().unwrap();
    assert_eq!(
        forced.len() * 2,
        preserve["outcomes"].as_object().unwrap().len()
    );
    let mut respoken = 0;
    for (key, outcome) in forced {
        let kept = &preserve["outcomes"][key.replace(":forced", ":preserve")];
        // An error is the same error; a result keeps its diagnostics and its partition.
        if kept.get("error").is_some() {
            assert_eq!(outcome, kept, "{key}");
            continue;
        }
        let (outcome, kept) = (&outcome["result"], &kept["result"]);
        assert_eq!(outcome["issues"], kept["issues"], "{key}");
        assert_eq!(outcome["complete"], kept["complete"], "{key}");
        let segments = |value: &Value| value["segments"].as_array().unwrap().clone();
        assert_eq!(segments(outcome).len(), segments(kept).len(), "{key}");
        for (segment, source) in segments(outcome).iter().zip(segments(kept)) {
            assert_eq!(segment["range"], source["range"], "{key}");
            if source["kind"] == "Unresolved" {
                assert_ne!(segment["kind"], "Unresolved", "{key}");
            } else {
                assert_eq!(segment["kind"], source["kind"], "{key}");
            }
        }
        respoken += usize::from(outcome["normalized_text"] != kept["normalized_text"]);
    }
    // Forced readings and the spoken style (a fraction, the lira, a range's dash, a slash in
    // an address, a long number said as an identifier, a hashtag) change eighteen cases.
    assert_eq!(respoken, 18);
}
