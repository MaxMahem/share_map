#![cfg(feature = "serde")]

use share_map::{ensure_unqiue, ShareMap};

static TEST_DATA: [(&str, u8); 5] = [
    ("key1", 1),
    ("key2", 2),
    ("key3", 3),
    ("key4", 4),
    ("key5", 5),
];

#[test]
fn serde_roundtrip() {
    let map = ShareMap::<_, _>::try_from_iter(TEST_DATA).expect("should be ok");

    let serialized = serde_json::to_string(&map).expect("should be ok");
    let deserialized: ShareMap<&str, u8> = serde_json::from_str(&serialized).expect("should be ok");

    assert_eq!(map, deserialized);
}

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct TestContainer {
    #[serde(with = "ensure_unqiue")]
    map: ShareMap<String, u8>,
}

#[test]
fn deserialize_ensure_unqiue_duplicate_keys_errors() {
    let data = r#"{"map": {"key1": 1, "key2": 2, "key1": 3}}"#;

    let err = serde_json::from_str::<TestContainer>(data).expect_err("should Err");

    assert!(err.is_data());
}

#[test]
fn serde_ensured_unique_roundtrip() {
    let test_data = TEST_DATA.into_iter().map(|(k, v)| (k.to_string(), v));
    let map = ShareMap::<String, _>::try_from_iter(test_data).expect("should be ok");
    let test_container = TestContainer { map };

    let serialized = serde_json::to_string(&test_container).expect("should be ok");
    let deserialized: TestContainer = serde_json::from_str(&serialized).expect("should be ok");

    assert_eq!(test_container, deserialized);
}

#[test]
fn deserialize_ensure_unqiue_wrong_type_uses_expecting() {
    // Test with a string instead of a map
    let data = r#"{"map": "not a map"}"#;
    let err = serde_json::from_str::<TestContainer>(data).expect_err("should Err");
    assert!(err.to_string().contains("a map with unique keys"));

    // Test with a number instead of a map
    let data = r#"{"map": 123}"#;
    let err = serde_json::from_str::<TestContainer>(data).expect_err("should Err");
    assert!(err.to_string().contains("a map with unique keys"));

    // Test with an array instead of a map
    let data = r#"{"map": [1, 2, 3]}"#;
    let err = serde_json::from_str::<TestContainer>(data).expect_err("should Err");
    assert!(err.to_string().contains("a map with unique keys"));
}

#[test]
fn deserialize_ensure_unqiue_malformed_entry_errors() {    
    // Map expects String keys, but we provide a number as a key
    let data = r#"{"map": {123: "value"}}"#;
    let err = serde_json::from_str::<TestContainer>(data).expect_err("should Err");
    assert!(err.is_syntax());
}
