use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use collect_failable::TryCollectEx;

use share_map::ShareMap;

static TEST_DATA: [(&str, u8); 5] = [
    ("key1", 1),
    ("key2", 2),
    ("key3", 3),
    ("key4", 4),
    ("key5", 5),
];

static DUPLICATE_DATA: [(&str, u8); 6] = [
    ("key1", 1),
    ("key2", 2),
    ("key3", 3),
    ("key4", 4),
    ("key5", 5),
    ("key1", 6),
];

#[test]
fn default_is_empty() {
    let map: ShareMap<u8, ()> = ShareMap::default();
    assert_eq!(map.len(), 0, "should be empty");
    assert!(map.is_empty(), "should be empty");
}

#[test]
fn debug_matches_btreemap() {
    // Test against BTreeMap for reliability because HashMap does not guarantee iteration order
    let btree_map = BTreeMap::from(TEST_DATA);
    let swap_map: ShareMap<_, _, BTreeMap<_, _>> = btree_map.clone().into_iter().collect();

    let swap_debug = format!("{swap_map:?}");
    let btree_debug = format!("{btree_map:?}");

    assert_eq!(swap_debug, btree_debug, "should be equal");
}

#[test]
fn try_from_iter_no_duplicates_correct_data() {
    let map = ShareMap::<_, _>::try_from_iter(TEST_DATA).expect("should be ok");

    assert_eq!(map.len(), 5);
    assert_eq!(map["key1"], 1);
    assert_eq!(map["key2"], 2);
    assert_eq!(map["key3"], 3);
    assert_eq!(map["key4"], 4);
    assert_eq!(map["key5"], 5);
}

#[test]
fn try_from_iter_with_duplicates_returns_err() {
    let result = ShareMap::<_, _>::try_from_iter(DUPLICATE_DATA);
    assert!(result.is_err());
}

#[test]
fn from_iter_with_duplicates_correct_data() {
    let map = ShareMap::<_, _>::from_iter(DUPLICATE_DATA);

    assert_eq!(map.len(), 5);
    // hashmap keeps the last value
    assert_eq!(map["key1"], 6);
    assert_eq!(map["key2"], 2);
    assert_eq!(map["key3"], 3);
    assert_eq!(map["key4"], 4);
    assert_eq!(map["key5"], 5);
}

#[test]
fn from_iter_no_duplicates_correct_data() {
    let map = ShareMap::<_, _>::from_iter(TEST_DATA);

    assert_eq!(map.len(), 5);
    assert_eq!(map["key1"], 1);
    assert_eq!(map["key2"], 2);
    assert_eq!(map["key3"], 3);
    assert_eq!(map["key4"], 4);
    assert_eq!(map["key5"], 5);
}

#[test]
fn try_from_array() {
    let map = ShareMap::<_, _>::try_from(TEST_DATA).expect("should be Ok");

    assert_eq!(map.len(), 5);
    assert_eq!(map["key1"], 1);
    assert_eq!(map["key2"], 2);
    assert_eq!(map["key3"], 3);
    assert_eq!(map["key4"], 4);
    assert_eq!(map["key5"], 5);
}

#[test]
fn from_hashmap_roundtrip() {
    let hash_map_in: HashMap<_, _> = TEST_DATA.into();

    let map = ShareMap::<_, _>::from(hash_map_in.clone());

    assert_eq!(map.len(), 5);
    assert_eq!(map["key1"], 1);
    assert_eq!(map["key2"], 2);
    assert_eq!(map["key3"], 3);
    assert_eq!(map["key4"], 4);
    assert_eq!(map["key5"], 5);

    let hash_map_out: HashMap<_, _> = map.into();
    assert_eq!(hash_map_in, hash_map_out);
}

#[test]
fn from_btreemap_roundtrip() {
    let btree_map_in: BTreeMap<_, _> = TEST_DATA.into();

    let map = ShareMap::<_, _>::from(btree_map_in.clone());

    assert_eq!(map.len(), 5);
    assert_eq!(map["key1"], 1);
    assert_eq!(map["key2"], 2);
    assert_eq!(map["key3"], 3);
    assert_eq!(map["key4"], 4);
    assert_eq!(map["key5"], 5);

    let btree_map_out: BTreeMap<_, _> = map.into();
    assert_eq!(btree_map_in, btree_map_out);
}

#[test]
fn get_valid_key_returns_some_value() {
    let map = ShareMap::<_, _>::try_from_iter(TEST_DATA).expect("should be ok");

    assert_eq!(map.get("key1"), Some(&1));
    assert_eq!(map.get("key2"), Some(&2));
    assert_eq!(map.get("key3"), Some(&3));
    assert_eq!(map.get("key4"), Some(&4));
    assert_eq!(map.get("key5"), Some(&5));
}

#[test]
fn get_invalid_key_returns_none() {
    let map = ShareMap::<_, _>::try_from_iter(TEST_DATA).expect("should be ok");

    assert_eq!(map.get("key6"), None);
}

#[test]
fn contains_key_valid_key_returns_true() {
    let map = ShareMap::<_, _>::try_from_iter(TEST_DATA).expect("should be ok");

    assert!(map.contains_key("key1"));
    assert!(map.contains_key("key2"));
    assert!(map.contains_key("key3"));
    assert!(map.contains_key("key4"));
    assert!(map.contains_key("key5"));
}

#[test]
fn contains_key_invalid_key_returns_false() {
    let map = ShareMap::<_, _>::try_from_iter(TEST_DATA).expect("should be ok");

    assert!(!map.contains_key("key6"));
}

#[test]
#[should_panic(expected = "no entry found for key")]
fn index_invalid_key_panics() {
    let map = ShareMap::<_, _>::try_from_iter(TEST_DATA).expect("should be ok");

    _ = map["key6"];
}

#[test]
fn eq_maps_are_equal() {
    let map1 = ShareMap::<_, _>::try_from_iter(TEST_DATA).expect("should be ok");
    let map2 = ShareMap::<_, _>::try_from_iter(TEST_DATA).expect("should be ok");

    assert_eq!(map1, map2);
}

#[test]
fn ne_maps_same_keys_different_values_not_equal() {
    let map1 = ShareMap::<_, _>::try_from_iter(TEST_DATA).expect("should be ok");
    let map2 = ShareMap::<_, _>::from_iter(DUPLICATE_DATA);

    assert_ne!(map1, map2);
}

#[test]
fn ne_maps_different_lengths_not_equal() {
    let map1 = ShareMap::<_, _>::try_from_iter(TEST_DATA).expect("should be ok");
    let map2 = ShareMap::<_, _>::default();

    assert_ne!(map1, map2);
}

#[test]
fn ne_maps_different_keys_not_equal() {
    let map1: ShareMap<_, _> = TEST_DATA
        .map(|(k, v)| (k.to_lowercase(), v))
        .into_iter()
        .try_collect_ex()
        .expect("should be ok");
    let map2: ShareMap<_, _> = TEST_DATA
        .map(|(k, v)| (k.to_uppercase(), v))
        .into_iter()
        .try_collect_ex()
        .expect("should be ok");

    assert_ne!(map1, map2);
}

// Use BtreeMap for testing because HashMap does not guarantee iteration order
#[test]
fn keys_returns_borrowed_keys() {
    let map = ShareMap::<_, _, BTreeMap<_, _>>::try_from_iter(TEST_DATA).expect("should be ok");

    let map_keys: Vec<_> = map.keys().collect();
    let data_keys: Vec<_> = TEST_DATA.iter().map(|(k, _)| k).collect();

    assert_eq!(map_keys, data_keys);
}

#[test]
fn into_keys_returns_keys() {
    let map = ShareMap::<_, _, BTreeMap<_, _>>::try_from_iter(TEST_DATA).expect("should be ok");

    let map_keys: Vec<_> = map.into_keys().collect();
    let data_keys: Vec<_> = TEST_DATA.into_iter().map(|(k, _)| k).collect();

    assert_eq!(map_keys, data_keys);
}

#[test]
fn values_returns_borrowed_values() {
    let map = ShareMap::<_, _>::try_from_iter(TEST_DATA).expect("should be ok");

    let map_values: Vec<_> = map.values().collect();
    let data_values: Vec<_> = TEST_DATA.iter().map(|(_, v)| v).collect();

    assert_eq!(map_values, data_values);
}

#[test]
fn into_values_returns_values() {
    let map = ShareMap::<_, _>::try_from_iter(TEST_DATA).expect("should be ok");

    let map_values = map.into_values();
    let data_values: Arc<[_]> = TEST_DATA.into_iter().map(|(_, v)| v).collect();

    assert_eq!(map_values, data_values);
}

#[test]
fn map_into_iter_borrowed() {
    let map = ShareMap::<_, _, BTreeMap<_, _>>::try_from_iter(TEST_DATA).expect("should be ok");

    let borrowed_vec: Vec<_> = TEST_DATA.iter().map(|(k, v)| (k, v)).collect();
    let frozen_vec: Vec<_> = map.into_iter().collect();

    assert_eq!(borrowed_vec, frozen_vec);
}
