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
    let normalizer = Normalizer::new().unwrap();
    for case in corpus {
        for reject in [false, true] {
            let mut options = NormalizeOptions {
                ambiguity_policy: if reject {
                    AmbiguityPolicy::Reject
                } else {
                    AmbiguityPolicy::Preserve
                },
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
            let key = format!(
                "{}:{}",
                case["id"].as_str().unwrap(),
                if reject { "reject" } else { "preserve" }
            );
            assert_eq!(actual, expected["outcomes"][&key], "{key}");
        }
    }
}
