//! Feature-independent semantics and optional result serialization.
#![cfg(feature = "serde")]

use normalizer_tr::{NormalizeOptions, Normalizer};

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
