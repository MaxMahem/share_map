use std::borrow::Borrow;
use std::error::Error;
use std::hash::BuildHasher;

use crate::{Handle, ShareMap};

#[test]
fn deref_matches_value() {
    let value = 42;

    let handle = ShareMap::<_, _>::try_from_iter([("key1", value)])
        .expect("should be Ok")
        .get_handle("key1")
        .expect("should be Some");

    assert_eq!(*handle, value);
}

#[test]
fn as_ref_matches_value() {
    let value = 42;

    let handle = ShareMap::<_, _>::try_from_iter([("key1", value)])
        .expect("should be Ok")
        .get_handle("key1")
        .expect("should be Some");

    assert_eq!(handle.as_ref(), &value);
}

#[test]
fn borrow_matches_value() {
    let value = 42;

    let handle = ShareMap::<_, _>::try_from_iter([("key1", value)])
        .expect("should be Ok")
        .get_handle("key1")
        .expect("should be Some");

    let borrow: &i32 = handle.borrow();

    assert_eq!(borrow, &value);
}

#[test]
fn debug_matches_value() {
    let value = 42;

    let handle = ShareMap::<_, _>::try_from_iter([("key1", value)])
        .expect("should be Ok")
        .get_handle("key1")
        .expect("should be Some");

    let handle_debug = format!("{:?}", handle);
    let value_debug = format!("{:?}", value);

    assert_eq!(handle_debug, value_debug);
}

#[test]
fn display_matches_value() {
    let value = 42;

    let handle = ShareMap::<_, _>::try_from_iter([("key1", value)])
        .expect("should be Ok")
        .get_handle("key1")
        .expect("should be Some");

    let handle_display = format!("{}", handle);
    let value_display = format!("{}", value);

    assert_eq!(handle_display, value_display);
}

#[test]
fn cloned_value_matches() {
    let value = "hello world";

    let handle = ShareMap::<_, _>::try_from_iter([("key1", value)])
        .expect("should be Ok")
        .get_handle("key1")
        .expect("should be Some");

    let cloned_handle = handle.clone();

    assert_eq!(cloned_handle, handle);
    assert_eq!(*cloned_handle, *handle);
    assert!(Handle::eq(&cloned_handle, &handle));
}

#[test]
fn cloned_handle_eq() {
    let value = "hello world";

    let handle = ShareMap::<_, _>::try_from_iter([("key1", value)])
        .expect("should be Ok")
        .get_handle("key1")
        .expect("should be Some");

    let cloned_handle = handle.clone();

    assert!(Handle::ref_eq(&cloned_handle, &handle));
}

#[derive(Debug, thiserror::Error, Clone, Copy, PartialEq, Eq)]
#[error("inner error")]
struct InnerError;

#[derive(Debug, thiserror::Error, Clone, Copy, PartialEq, Eq)]
#[error("test error")]
struct TestError(#[source] InnerError);

#[test]
fn handle_delegates_source() {
    let error = TestError(InnerError);

    let handle = ShareMap::<_, _>::try_from_iter([("key1", error)])
        .expect("should be Ok")
        .get_handle("key1")
        .expect("should be Some");

    let handle_error_source_dbg = format!("{:?}", handle.source());
    let error_source_dbg = format!("{:?}", error.source());

    assert_eq!(handle_error_source_dbg, error_source_dbg);
}

#[test]
fn eq_ne_same_ref() {
    let value = 42;

    let map = ShareMap::<_, _>::try_from_iter([("key1", value)]).expect("should be Ok");

    let handle1 = map.get_handle("key1").expect("should be Some");
    let handle2 = map.get_handle("key1").expect("should be Some");

    assert!(Handle::eq(&handle1, &handle2));
    assert!(!Handle::ne(&handle1, &handle2));
    assert_eq!(handle1, handle2);
    assert_eq!(handle1.cmp(&handle2), std::cmp::Ordering::Equal);
}

#[test]
fn eq_ne_different_ref_same_value() {
    let value = 42;

    let handle1 = ShareMap::<_, _>::try_from_iter([("key1", value)])
        .expect("should be Ok")
        .get_handle("key1")
        .expect("should be Some");
    let handle2 = ShareMap::<_, _>::try_from_iter([("key1", value)])
        .expect("should be Ok")
        .get_handle("key1")
        .expect("should be Some");

    assert!(Handle::eq(&handle1, &handle2));
    assert!(!Handle::ne(&handle1, &handle2));
    assert_eq!(handle1, handle2);
    assert_eq!(handle1.cmp(&handle2), std::cmp::Ordering::Equal);
}

#[test]
fn ref_eq_ne_same_ref() {
    let value = 42;

    let map = ShareMap::<_, _>::try_from_iter([("key1", value)]).expect("should be Ok");

    let handle1 = map.get_handle("key1").expect("should be Some");
    let handle2 = map.get_handle("key1").expect("should be Some");

    assert!(Handle::ref_eq(&handle1, &handle2));
    assert!(!Handle::ref_ne(&handle1, &handle2));
}

#[test]
fn ref_eq_different_ref_same_value() {
    let value = 42;

    let handle1 = ShareMap::<_, _>::try_from_iter([("key1", value)])
        .expect("should be Ok")
        .get_handle("key1")
        .expect("should be Some");
    let handle2 = ShareMap::<_, _>::try_from_iter([("key1", value)])
        .expect("should be Ok")
        .get_handle("key1")
        .expect("should be Some");

    assert!(!Handle::ref_eq(&handle1, &handle2));
    assert!(Handle::ref_ne(&handle1, &handle2));
}

#[test]
fn hash_matches_value() {
    let value = 42;

    let handle = ShareMap::<_, _>::try_from_iter([("key1", value)])
        .expect("should be Ok")
        .get_handle("key1")
        .expect("should be Some");

    let hasher = std::hash::RandomState::new();
    let value_hash = hasher.hash_one(value);
    let handle_hash = hasher.hash_one(handle);

    assert_eq!(handle_hash, value_hash);
}

#[test]
fn partial_cmp_matches_value() {
    let value = 42.0;

    let handle1 = ShareMap::<_, _>::try_from_iter([("key1", value)])
        .expect("should be Ok")
        .get_handle("key1")
        .expect("should be Some");

    let handle2 = ShareMap::<_, _>::try_from_iter([("key1", value)])
        .expect("should be Ok")
        .get_handle("key1")
        .expect("should be Some");

    assert_eq!(
        handle1.partial_cmp(&handle2),
        Some(std::cmp::Ordering::Equal)
    );
}

#[test]
fn deserialize_matches_value_deserialize() {
    let value = 42.0;

    let handle = ShareMap::<_, _>::try_from_iter([("key1", value)])
        .expect("should be Ok")
        .get_handle("key1")
        .expect("should be Some");

    let handle_serialized = serde_json::to_string(&handle).expect("should be Ok");
    let value_serialized = serde_json::to_string(&value).expect("should be Ok");

    assert_eq!(handle_serialized, value_serialized);
}
