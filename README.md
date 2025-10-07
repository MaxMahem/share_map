# SwapMap

`SwapMap` is a **thread-safe, lock-free, frozen map** that is immutable but can be atomically swapped out for a new version.  

It is designed for scenarios with **frequent reads** and **occasional bulk updates**, such as configuration reloading, caching, or periodically rebuilt lookup tables.

Readers always see a consistent snapshot without blocking writers, and writers can atomically replace the entire map without disrupting ongoing reads.

## Features

- **Thread-Safe**: Safe concurrent access from multiple threads via [`ArcSwap`](https://docs.rs/arc-swap).  
- **Immutable Snapshots**: Readers work with immutable views of the map. Swaps don’t affect existing `FrozenMap`s or `ValueRef`s.  
- **Customizable Map Implementation**: By default, it uses [`HashMap`](https://doc.rust-lang.org/std/collections/struct.HashMap.html), but you can plug in [`BTreeMap`](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html), any of the maps from [`frozen_collections`](https://docs.rs/frozen-collections/latest/frozen_collections/), [`hashbrown::HashMap`](https://docs.rs/hashbrown/latest/hashbrown/), or any type implementing [`MapQuery`](https://docs.rs/frozen_collections/latest/frozen_collections/trait.MapQuery.html), [`Len`](https://docs.rs/frozen_collections/latest/frozen_collections/trait.Len.html), and [`FromIterator`](https://doc.rust-lang.org/std/iter/trait.FromIterator.html).
- **Value References**: `SwapMap::get` returns a `ValueRef` that stays valid even after a swap, pointing into the snapshot it came from.

## Limitations

- **No Borrows**: Values inside a `SwapMap` cannot be borrowed directly. This prevents direct iteration or direct `get` or `index` calls. Instead, to iterate, call `SwapMap::snapshot` and use the iterators provided by `FrozenMap`, or call `SwapMap::get` and use the returned `ValueRef` for reference access.
- **Immutable Access Only**: No mutable access to values is exposed, directly or indirectly. If you need mutability, use thread-safe constructs that provide interior mutability, such as [`Mutex`](https://doc.rust-lang.org/std/sync/struct.Mutex.html), [`RwLock`](https://doc.rust-lang.org/std/sync/struct.RwLock.html), or [`Atomic*`](https://doc.rust-lang.org/std/sync/atomic/index.html).
- **No Value Move Semantics** Values inside a `SwapMap` are ultimately owned by an [`Arc<[T]>`](https://doc.rust-lang.org/std/sync/struct.Arc.html), and cannot be moved out of without unsafe code. Thus, to take ownership of held values, [`Clone`](https://doc.rust-lang.org/std/clone/trait.Clone.html) (or [`Copy`](https://doc.rust-lang.org/std/marker/trait.Copy.html)) is required.

## Map Dependent Behavior

The map types used with `SwapMap` determine many aspects of how it operates — including which key types are valid, how lookups are performed, and the iteration order of entries.

For example:
- [`HashMap`](https://doc.rust-lang.org/std/collections/struct.HashMap.html) requires keys to implement [`Eq`](https://doc.rust-lang.org/std/cmp/trait.Eq.html) and [`Hash`](https://doc.rust-lang.org/std/hash/trait.Hash.html).
- [`BTreeMap`](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html) requires keys to also implement [`Ord`](https://doc.rust-lang.org/std/cmp/trait.Ord.html).
- [`HashMap`](https://doc.rust-lang.org/std/collections/struct.HashMap.html) supports querying with any key type that implements [`Borrow<K>`](https://doc.rust-lang.org/std/borrow/trait.Borrow.html).
- The iteration order of [`HashMap`](https://doc.rust-lang.org/std/collections/struct.HashMap.html) values is **undefined**, while [`BTreeMap`](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html) values are always **sorted by key**.

## Examples

### Basic usage

```rust
use swap_map::{SwapMap, ValueRef};

let swap_map = SwapMap::<&str, i32>::new();

// Store a new version of the map
swap_map.store([("key1", 42), ("key2", 100)]).expect("Duplicate Key");

// Access by key 
let value_ref: ValueRef<i32> = swap_map.get("key1").expect("Key not found");
assert_eq!(*value_ref, 42);

// value ref remains valid after a swap or a drop, but points to old data
swap_map.store([("key1", 21), ("key2", 200)]).expect("Duplicate Key");
assert_eq!(*value_ref, 42); 

// new queries point to new data.
let new_value_ref: ValueRef<i32> = swap_map.get("key1").expect("Key not found");
assert_eq!(*new_value_ref, 21);
```

### Taking a snapshot for iteration

```rust
use swap_map::SwapMap;

let swap_map = SwapMap::<&str, i32>::new();
swap_map.store([("apple", 3), ("banana", 7), ("pear", 5)]).expect("duplicate key");

// Take a frozen snapshot
let snapshot = swap_map.snapshot();

// Iterate keys and values
for (key, value) in snapshot.iter() {
    println!("{key} = {value}");
}
```

### Using a custom map type

With [`BTreeMap`](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html)
```rust
use std::collections::BTreeMap;
use swap_map::SwapMap;

let swap_map = SwapMap::<&str, i32, BTreeMap<&str, usize>>::new();

// Values are ordered by key when iterating via snapshot
swap_map.store([("c", 3), ("a", 1), ("b", 2)]).expect("duplicate key");

let snapshot = swap_map.snapshot();
let keys: Vec<_> = snapshot.keys().collect();

assert_eq!(keys, [&"a", &"b", &"c"]);
```

## Included Types

### FrozenMap

An immutable snapshot of the current map. Unlike `SwapMap`, a `FrozenMap` allows borrowing, which means it can be iterated or accessed via direct reference without having to use `ValueRef`. It also provides `FrozenMap::get_value_ref` if you need a `ValueRef`.

### ValueRef

A small (index + pointer) owned reference-like type, which contains a reference to the snapshot storing the value. It ensures the snapshot’s lifetime, allowing the value reference to be stored and passed around safely, particularly between threads.
