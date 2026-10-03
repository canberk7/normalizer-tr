//! Feature-independent semantics and optional result serialization.
#![cfg(feature = "serde")]

use normalizer_tr::{AmbiguityPolicy, HintKind, NormalizeOptions, Normalizer, SegmentKind};
use serde_json::json;

#[test]
fn optional_serialization_preserves_readings_and_source_coordinates() {
    let input = include_str!("fixtures/financial-input.txt").trim_end_matches(['\r', '\n']);
    let expected = include_str!("fixtures/financial-output.txt").trim_end_matches(['\r', '\n']);
    let result = Normalizer::new()
        .unwrap()
        .normalize(input, &NormalizeOptions::default())
        .unwrap();
    let json = serde_json::to_value(&result).unwrap();
    assert_eq!(json["normalized_text"], expected);
    assert_eq!(json["segments"][0]["range"]["start"], 0);
    assert_eq!(json["segments"][0]["range"]["end"], 13);
    assert_eq!(json["issues"], serde_json::json!([]));
    assert_eq!(json["complete"], true);
    let partial = Normalizer::new()
        .unwrap()
        .normalize("25 TL; 1.234", &NormalizeOptions::default())
        .unwrap();
    assert_eq!(serde_json::to_value(partial).unwrap()["complete"], false);
}

#[test]
fn forced_policy_and_literal_kinds_round_trip() {
    for policy in [
        AmbiguityPolicy::Preserve,
        AmbiguityPolicy::Reject,
        AmbiguityPolicy::Forced,
    ] {
        let json = serde_json::to_string(&policy).unwrap();
        assert_eq!(
            serde_json::from_str::<AmbiguityPolicy>(&json).unwrap(),
            policy
        );
    }
    assert_eq!(
        serde_json::to_value(AmbiguityPolicy::Forced).unwrap(),
        json!("Forced")
    );
    assert_eq!(
        serde_json::from_value::<AmbiguityPolicy>(json!("Forced")).unwrap(),
        AmbiguityPolicy::Forced
    );
    assert_eq!(
        serde_json::to_value(HintKind::Literal).unwrap(),
        json!("Literal")
    );
    assert_eq!(
        serde_json::from_value::<HintKind>(json!("Literal")).unwrap(),
        HintKind::Literal
    );
    // Segment kinds are serialize-only, like the rest of a result.
    assert_eq!(
        serde_json::to_value(SegmentKind::Literal).unwrap(),
        json!("Literal")
    );
    let forced = Normalizer::new()
        .unwrap()
        .normalize(
            "25 TL; 2.5.1",
            &NormalizeOptions {
                ambiguity_policy: AmbiguityPolicy::Forced,
                ..Default::default()
            },
        )
        .unwrap();
    let json = serde_json::to_value(&forced).unwrap();
    assert_eq!(
        json["normalized_text"],
        "yirmi beş lira; iki nokta beş nokta bir"
    );
    assert_eq!(json["complete"], false);
    assert_eq!(json["issues"].as_array().unwrap().len(), 1);
    assert_eq!(json["segments"][2]["kind"], "Literal");
    assert_eq!(json["segments"][2]["rule_id"], "forced.literal");
    assert_eq!(json["segments"][2]["range"], json!({"start": 7, "end": 12}));
}
