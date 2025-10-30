# ShareMap

[![CI](https://github.com/MaxMahem/share_map/workflows/CI/badge.svg)](https://github.com/MaxMahem/share_map/actions)
![GitHub License](https://img.shields.io/github/license/maxmahem/share_map)
[![dependency status](https://deps.rs/repo/github/maxmahem/share_map/status.svg)](https://deps.rs/repo/github/maxmahem/share_map)
[![codecov](https://codecov.io/github/MaxMahem/share_map/graph/badge.svg?token=N5JJLLQ04L)](https://codecov.io/github/MaxMahem/share_map)

`ShareMap` is a **configurable, thread-safe, immutable map** of values that supports shared read access and provides access to stable, sharable value references to its owned values (`Handle`s).

It is designed for scenarios with **frequent reads** and **occasional bulk updates**, such as configuration reloading, caching, or periodically rebuilt lookup tables. Where data needs to be queried on one thread while values are handled on another. `Handle`s provide persistent access to owned data, so the `ShareMap` can be dropped or replaced without affecting current readers who will continue to see the previous value.

## ✨ Features

- 🔒 **Immutable design** — once created, data cannot be mutated.
- 🔗 **Stable handles** — each entry can be accessed through a persistent `Handle<T>` which can outlive the map.
- ⚡ **Iterator support** — borrow-based iterators allow efficient traversal.
- 🧠 **Customizable Map Implementation**: By default `SharedMap` uses [`HashMap`](https://doc.rust-lang.org/std/collections/struct.HashMap.html) for its key lookups. But you can plug in [`BTreeMap`](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html), any of the maps from [`frozen_collections`](https://docs.rs/frozen-collections/latest/frozen_collections/), [`hashbrown::HashMap`](https://docs.rs/hashbrown/latest/hashbrown/), or any type implementing [`MapQuery`](https://docs.rs/frozen_collections/latest/frozen_collections/trait.MapQuery.html), [`Len`](https://docs.rs/frozen_collections/latest/frozen_collections/trait.Len.html), and [`FromIterator`](https://doc.rust-lang.org/std/iter/trait.FromIterator.html).
- ❌ **Failure-aware construction** — integrates with `TryFromIterator` for fallible initialization.

## Limitations

- **Immutable Access Only**: No mutable access to values is exposed, directly or indirectly. If you need mutability, use thread-safe constructs that provide interior mutability, such as [`Mutex`](https://doc.rust-lang.org/std/sync/struct.Mutex.html), [`RwLock`](https://doc.rust-lang.org/std/sync/struct.RwLock.html), or [`Atomic*`](https://doc.rust-lang.org/std/sync/atomic/index.html).
- **No Value Move Semantics** Values inside a `ShareMap` are ultimately owned by an [`Arc<[T]>`](https://doc.rust-lang.org/std/sync/struct.Arc.html), and cannot be moved out of without unsafe code. Thus, to take ownership of held values, [`Clone`](https://doc.rust-lang.org/std/clone/trait.Clone.html) or [`Copy`](https://doc.rust-lang.org/std/marker/trait.Copy.html) is required.

## Map Dependent Behavior

The map types used with `ShareMap` determine many aspects of its operation — including which key types are valid, how lookups are performed, and the iteration order of entries.

For example:
- [`HashMap`](https://doc.rust-lang.org/std/collections/struct.HashMap.html) requires keys to implement [`Eq`](https://doc.rust-lang.org/std/cmp/trait.Eq.html) and [`Hash`](https://doc.rust-lang.org/std/hash/trait.Hash.html).
- [`BTreeMap`](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html) requires keys to also implement [`Ord`](https://doc.rust-lang.org/std/cmp/trait.Ord.html).
- [`HashMap`](https://doc.rust-lang.org/std/collections/struct.HashMap.html) supports querying with any key type that implements [`Borrow<K>`](https://doc.rust-lang.org/std/borrow/trait.Borrow.html).
- Key iteration order is undefined for [`HashMap`](https://doc.rust-lang.org/std/collections/struct.HashMap.html) but always in order for [`BTreeMap`](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html).

## Examples

### Basic usage

```rust
use share_map::{ShareMap, Handle};

let data = [("key1", 42), ("key2", 100)];
let share_map = ShareMap::<_, _>::try_from_iter(data).expect("Duplicate Key");

// Access by key 
let handle: Handle<i32> = share_map.get_handle("key1").expect("Key not found");
assert_eq!(*handle, 42);

// value ref remains valid after a swap or a drop, but points to old data
drop(share_map);
assert_eq!(*handle, 42); 
```

### Using a custom map type

With [`BTreeMap`](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html)
```rust
use std::collections::BTreeMap;
use share_map::ShareMap;

let data = [("a", 42), ("b", 100), ("c", 200)];
let share_map = ShareMap::<_, _, BTreeMap<_, _>>::try_from_iter(data).expect("Duplicate Key");

let keys: Vec<_> = share_map.keys().collect();

assert_eq!(keys, [&"a", &"b", &"c"]);
```
