# rust-stl

An educational, dependency-free reimplementation of classic STL-style
containers in Rust, written with raw pointers and manual allocation so the
memory-management machinery is visible instead of hidden.

The crate currently provides one container: **`Vector<T>`**, a growable,
contiguous, heap-allocated array.

## Status and maturity

**Development-ready, not published.**

* `Vector<T>` is implemented and covered by an extensive test suite, including
  exact destructor-count, panic-safety and zero-sized-type tests.
* The crate has **not** been published to crates.io.
* It is **not** benchmarked against `std::collections`. No performance claims
  are made anywhere in this repository.
* The API is a **subset** of `std::vec::Vec`. Breaking changes are expected
  before `1.0`.

## Features

* `Vector<T>` with indexing (`vector[i]`, including ranges like `vector[1..3]`),
  `get`/`get_mut`, `first`/`last`, `push`/`pop`, `insert`/`remove`/
  `swap_remove`, `clear`/`truncate`.
* Capacity control: `reserve`, `reserve_exact`, `shrink_to_fit`, and the
  fallible `try_reserve` / `try_reserve_exact`.
* Bulk operations: `append`, `extend`, `extend_from_slice`, `resize`,
  `resize_with`, `split_off`.
* Filtering and de-duplication: `retain`, `retain_mut`, `remove_where`,
  `remove_where_mut`, `remove_all`, `remove_first`, `dedup`,
  `remove_duplicates`.
* Sorting and searching: `sort`, `sort_by`, `sort_range`, `binary_search`,
  `binary_search_by`, `binary_search_by_key`; plus `reverse`, `rotate_left`,
  `rotate_right`, `split_at`, `windows`, `chunks`.
* All three iteration forms (`&Vector<T>`, `&mut Vector<T>`, `Vector<T>`) with
  `ExactSizeIterator`, `FusedIterator` and `DoubleEndedIterator` on the owned
  iterator.
* Full support for **zero-sized element types**, including ZSTs with `Drop`.
* No runtime dependencies.

## Minimum supported Rust version

Rust **1.85**, the first release supporting the 2024 edition this crate is
built against. The MSRV is declared as `rust-version` in `Cargo.toml`.

## Installation

### From crates.io

Not yet available — the crate has not been published.

### From this repository (local development)

```sh
git clone https://github.com/rohan3155/rust-stl.git
cd rust-stl
cargo build
```

To use it from another local crate, add a path dependency:

```toml
[dependencies]
rust-stl = { path = "../rust-stl" }
```

## Quick start

```rust
use rust_stl::collections::vector::Vector;

let mut numbers: Vector<i32> = Vector::new();
numbers.push(30);
numbers.push(10);
numbers.push(20);

numbers.sort();
assert_eq!(numbers.as_slice(), &[10, 20, 30]);

// `Ok(index)` if found, `Err(insertion_index)` if not.
assert_eq!(numbers.binary_search(&20), Ok(1));

// Iterate by reference, in place, or by value.
let total: i32 = (&numbers).into_iter().sum();
assert_eq!(total, 60);

let owned: Vec<i32> = numbers.into_iter().collect();
assert_eq!(owned, vec![10, 20, 30]);
```

## API examples

### Constructing and inserting

```rust
use rust_stl::collections::vector::Vector;

let mut vector: Vector<i32> = Vector::new(); // no allocation yet
vector.reserve(8);                           // one allocation, room for 8

vector.push(1);
vector.insert(0, 0);                         // O(len): shifts the tail right
vector.extend([2, 3]);
assert_eq!(vector.as_slice(), &[0, 1, 2, 3]);
```

### Indexing and safe access

```rust
use rust_stl::collections::vector::Vector;

let mut vector: Vector<i32> = (0..5).collect();

assert_eq!(vector[2], 2);          // panics if out of bounds
assert_eq!(vector.get(2), Some(&2));
assert_eq!(vector.get(99), None);  // no panic
assert_eq!(&vector[1..3], &[1, 2]); // range indexing

vector[0] = 10;
*vector.last_mut().unwrap() = 40;
assert_eq!(vector.as_slice(), &[10, 1, 2, 3, 40]);
```

### Iteration

```rust
use rust_stl::collections::vector::Vector;

let mut vector: Vector<i32> = (0..4).collect();

for value in &vector {
    print!("{value} ");            // &i32
}

for value in &mut vector {
    *value += 1;                   // &mut i32
}

// Consumes the vector; the iterator can also be driven from both ends.
let owned: Vec<i32> = vector.into_iter().rev().collect();
assert_eq!(owned, vec![4, 3, 2, 1]);
```

### Filtering and de-duplication

```rust
use rust_stl::collections::vector::Vector;

let mut kept: Vector<i32> = (0..6).collect();
kept.retain(|n| n % 2 == 0);          // KEEPS matches
assert_eq!(kept.as_slice(), &[0, 2, 4]);

let mut removed: Vector<i32> = (0..6).collect();
removed.remove_where(|n| n % 2 == 0); // REMOVES matches
assert_eq!(removed.as_slice(), &[1, 3, 5]);

let mut consecutive: Vector<i32> = vec![1, 1, 2, 1].into_iter().collect();
consecutive.dedup();                   // consecutive duplicates only
assert_eq!(consecutive.as_slice(), &[1, 2, 1]);

let mut distinct: Vector<i32> = vec![1, 1, 2, 1].into_iter().collect();
distinct.remove_duplicates();          // first occurrence of each value
assert_eq!(distinct.as_slice(), &[1, 2]);
```

### Searching and sorting

```rust
use rust_stl::collections::vector::Vector;

let mut vector: Vector<i32> = vec![5, 1, 4, 2, 3].into_iter().collect();

vector.sort();                          // stable, ascending
assert_eq!(vector.as_slice(), &[1, 2, 3, 4, 5]);
assert_eq!(vector.binary_search(&3), Ok(2));
assert_eq!(vector.binary_search(&9), Err(5)); // insertion point

vector.rotate_left(1);
assert_eq!(vector.as_slice(), &[2, 3, 4, 5, 1]);

// `sort_range` sorts the half-open range `start..end`, leaving `end` alone.
let mut partial: Vector<i32> = vec![9, 8, 3, 1, 2, 0].into_iter().collect();
partial.sort_range(2, 5);
assert_eq!(partial.as_slice(), &[9, 8, 1, 2, 3, 0]);
```

### Capacity management

```rust
use rust_stl::collections::vector::Vector;

let mut vector: Vector<i32> = Vector::new();
vector.reserve(1_000);                 // avoid intermediate reallocations
vector.extend(0..1_000);
assert!(vector.capacity() >= 1_000);

vector.clear();                        // capacity is retained
assert!(vector.is_empty());

vector.shrink_to_fit();                // release the allocation (len == 0)
assert_eq!(vector.capacity(), 0);

// Fallible reservation: an error instead of a panic, and the contents are kept.
vector.push(1);
assert!(vector.try_reserve_exact(usize::MAX).is_err());
assert_eq!(vector.as_slice(), &[1]);
```

### Collecting from an iterator

```rust
use rust_stl::collections::vector::Vector;

let vector: Vector<String> = ["a", "b", "c"].into_iter().map(String::from).collect();
let rejoined = vector.into_iter().collect::<Vec<_>>().join(",");
assert_eq!(rejoined, "a,b,c");
```

## Capabilities and limitations

* **Subset of `Vec`.** `with_capacity`, `drain`, `splice`, `into_iter` adapters
  beyond `DoubleEndedIterator`, and fallible range indexing are not provided.
  `get` takes a `usize` index only, not a range.
* **Fallible reservation is coarse.** `try_reserve` and `try_reserve_exact`
  return `Result<(), &'static str>`. Capacity overflow and allocation failure
  both surface as a string and are not distinguished.
* **Removal-by-shifting is quadratic.** `retain`, `retain_mut`, `remove_where`,
  `remove_where_mut`, `remove_all`, `dedup` and `remove_duplicates` remove
  elements by shifting the tail, so their worst case is O(n²), unlike the
  linear-time implementations in the standard library. They are correct and
  panic-safe; they are simply not optimised.
* **Not `no_std`.** The crate uses `std::alloc` and the standard library
  directly.
* **Not a drop-in replacement.** Where a method name matches `std::vec::Vec`, the
  behaviour is intended to match and is documented individually, but the two are
  separate types and do not interoperate implicitly.
* **`swap_remove` returns `Option<T>`** and returns `None` for an out-of-range
  index, whereas `Vec::swap_remove` panics.

## Performance and complexity

Complexity is documented per method in the API docs. The summary for
`Vector<T>`:

| Operation | Average | Worst case |
|---|---|---|
| `push` / `pop` | O(1) | O(n) when the capacity is exceeded |
| `get` / indexing | O(1) | O(1) |
| `insert` / `remove` | O(n) | O(n) |
| `swap_remove` | O(1) | O(1) |
| `sort` | O(n log n) | O(n log n) |
| `binary_search` | O(log n) | O(log n) |
| `retain` / `dedup` / `remove_where` | O(n) | O(n²) |

These are algorithmic bounds, not measured results, and no claims are made about
constant factors relative to `std::vec::Vec`.

### Benchmarking

The repository ships a small, dependency-free micro-benchmark that runs a few
representative operations against `std::vec::Vec` side by side:

```sh
cargo bench --bench vector
```

It prints the best of a handful of repetitions for each operation. It is a
sanity harness for spotting gross regressions while working on the
implementation — **not** a rigorous benchmark. There is no statistical analysis
or load isolation, so its numbers are not guarantees and are deliberately not
quoted in this README.

## Testing

```sh
cargo test                  # unit tests, integration tests and doctests
cargo test --examples       # compile and run the example programs
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check
```

The test suite is organised by behaviour:

| File | Covers |
|---|---|
| `tests/core_behavior.rs` | Construction, growth, indexing, mutation, filtering, sorting, searching, rotation and slicing |
| `tests/capacity.rs` | Reservation, shrinking, retained capacity, and overflow/error paths |
| `tests/edge_cases.rs` | Empty and single-element vectors, zero-sized types, non-`Copy` values, nested vectors, empty `append`/`split_off` |
| `tests/iterators.rs` | The three iteration forms, `size_hint`, `ExactSizeIterator`, `FusedIterator`, `DoubleEndedIterator`, early drop |
| `tests/memory_safety.rs` | Exact destructor counts for every destructive operation, plus panicking predicates, `Clone` and `PartialEq` |
| `tests/traits.rs` | `Debug`, `Clone`, `PartialEq`/`Eq`, `Deref`/`DerefMut`, `Index`/`IndexMut`, `Extend`, `FromIterator`, `IntoIterator` |

The unit tests in `src/collections/vector.rs` additionally check private
invariants (layout overflow, growth arithmetic, dangling-pointer handling, the
owned iterator's remaining range).

Miri is recommended when working on the `unsafe` code, but it is **not** part of
the default checks here because `cargo miri` requires a nightly toolchain:

```sh
rustup +nightly component add miri
cargo +nightly miri test
```

CI runs the same checks on every push to `main` and on every pull request, plus a
build against the declared MSRV (see `.github/workflows/ci.yml`).

## Documentation

```sh
cargo doc --no-deps --open
```

Published API docs (once the crate is released) will be at
<https://docs.rs/rust-stl>.

## Examples

Each example is self-contained and runs with `cargo run --example <name>`:

| Example | Demonstrates |
|---|---|
| `examples/basic_usage.rs` | Construction, push vs. insert, indexing vs. `get`, updating, iteration, `remove` vs. `swap_remove`, sorting, searching |
| `examples/iteration.rs` | All three iteration forms, in-place mutation, the double-ended owned iterator, zero-sized types |
| `examples/filtering_and_sorting.rs` | `retain` vs. `remove_where`, `dedup` vs. `remove_duplicates`, sorting, binary search, rotation, `windows` vs. `chunks` |
| `examples/capacity_management.rs` | Reservation, retained capacity, `shrink_to_fit`, fallible reservation |

## Project structure

```text
rust-stl/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── CONTRIBUTING.md
├── CHANGELOG.md
├── SECURITY.md
├── LICENSE-MIT
├── LICENSE-APACHE
├── .github/workflows/ci.yml   formatting, lints, tests, docs, MSRV
├── src/
│   ├── lib.rs                 crate-level documentation
│   └── collections/
│       ├── mod.rs
│       └── vector.rs          Vector<T>, its iterators and unit tests
├── examples/                  runnable examples (see above)
├── benches/vector.rs          dependency-free micro-benchmark
└── tests/                     integration test suite (see above)
```

## Roadmap

* No new containers are implemented yet. A doubly linked list, a hash map and a
  stack/queue are ideas, **not** existing code — this README will be updated when
  and only if they exist.
* Planned improvements to `Vector<T>`:
  * linear-time `retain`/`dedup`/`remove_where`;
  * a structured error type for the fallible reservation methods;
  * a statistically sound benchmark suite (the current one is a sanity check);
  * a Miri job in CI.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). In short: run `cargo fmt`, `cargo test`
and `cargo clippy --all-targets --all-features -- -D warnings` before opening a
pull request, and add a regression test for every bug fix.

## Security

See [SECURITY.md](SECURITY.md). Bugs that can cause undefined behaviour through
the public API are treated as security issues; the internal implementation uses
`unsafe` heavily, so such reports are especially valuable.

## License

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
* MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
