# rust-stl

An educational, dependency-free reimplementation of classic STL-style
containers in Rust, written with raw pointers and manual allocation so the
memory-management machinery is visible instead of hidden.

The crate currently provides two containers:

* **`Vector<T>`**, a growable, contiguous, heap-allocated array; and
* **`Stack<T>`**, a last-in, first-out (LIFO) stack that reuses `Vector<T>` as
  its backing storage rather than duplicating any memory-management code.

## Status and maturity

**Development-ready, not published.**

* `Vector<T>` and `Stack<T>` are implemented and covered by an extensive test
  suite, including exact destructor-count, panic-safety and zero-sized-type
  tests.
* The crate has **not** been published to crates.io.
* It ships only a small sanity micro-benchmark, not a rigorous benchmark. No
  performance claims are made anywhere in this repository.
* The API is a **subset** of `std::vec::Vec`. Breaking changes are expected
  before `1.0`.

## Features

### `Vector<T>`

* `Vector<T>` with indexing (`vector[i]`, including ranges like `vector[1..3]`),
  `get`/`get_mut`, `first`/`last`, `push`/`pop`, `push_back`/`push_front`, `pop_back`/`pop_front`, `insert`/`remove`/
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

### `Stack<T>`

* A LIFO stack: `push`, `pop`, `top`, `top_mut`, `len`, `is_empty`, `clear`.
* Capacity control: `with_capacity`, `capacity`, `reserve`, `reserve_exact`,
  `shrink_to_fit`.
* `Debug`, `Clone`, `PartialEq`/`Eq`, `Default`, `Extend`, `FromIterator` and all
  three `IntoIterator` forms.
* Backed by `Vector<T>`, so it inherits the vector's allocation strategy and
  zero-sized-type handling without reimplementing any of it.

### Shared

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

And a first look at the stack:

```rust
use rust_stl::collections::stack::Stack;

let mut history: Stack<String> = Stack::new();
history.push("first".to_string());
history.push("second".to_string());

// The most recent push is on top.
assert_eq!(history.top(), Some(&"second".to_string()));

// Popping is last-in, first-out; an empty stack yields `None`, never a panic.
assert_eq!(history.pop(), Some("second".to_string()));
assert_eq!(history.pop(), Some("first".to_string()));
assert_eq!(history.pop(), None);
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

### Stacks: push, inspect and pop

```rust
use rust_stl::collections::stack::Stack;

let mut stack: Stack<i32> = Stack::new();
stack.push(1);
stack.push(2);
stack.push(3);

// `top` borrows the newest element; `top_mut` updates it in place.
assert_eq!(stack.top(), Some(&3));
*stack.top_mut().unwrap() *= 10;
assert_eq!(stack.top(), Some(&30));

// Last-in, first-out removal.
assert_eq!(stack.pop(), Some(30));
assert_eq!(stack.pop(), Some(2));
assert_eq!(stack.len(), 1);
```

### Stacks: ownership and drop semantics

`pop` moves ownership of the top element to the caller. The stack stops
owning that value and will not drop it, even when the stack itself is later
destroyed. Elements still inside the stack are dropped exactly once when the
stack goes out of scope.

```rust
use rust_stl::collections::stack::Stack;

let mut stack: Stack<String> = Stack::new();
stack.push("bottom".to_string());
stack.push("top".to_string());

// Ownership of "top" transfers to `popped`; the stack no longer holds it.
let popped = stack.pop().expect("a value");
assert_eq!(popped, "top");
assert_eq!(stack.len(), 1);

// Dropping `popped` destroys that value now, not when the stack drops.
drop(popped);

// The remaining element is destroyed exactly once when the stack is dropped.
drop(stack);
```

### Stacks: iterating, collecting and traits

```rust
use rust_stl::collections::stack::Stack;

// `collect` pushes in order, so the last value ends up on top.
let stack: Stack<i32> = (0..4).collect();
assert_eq!(format!("{stack:?}"), "[0, 1, 2, 3]");

// Iteration walks from the bottom to the top. The owned iterator is
// double-ended, so `rev` visits the elements in pop order instead.
let bottom_up: Vec<i32> = (&stack).into_iter().copied().collect();
assert_eq!(bottom_up, vec![0, 1, 2, 3]);

let pop_order: Vec<i32> = stack.clone().into_iter().rev().collect();
assert_eq!(pop_order, vec![3, 2, 1, 0]);

// A clone owns independent values; equality compares contents, not capacity.
let clone = stack.clone();
assert_eq!(clone, stack);
```

## Capabilities and limitations

* **Subset of `Vec`.** `drain`, `splice`, `into_iter` adapters beyond
  `DoubleEndedIterator`, and fallible range indexing are not provided. `get`
  takes a `usize` index only, not a range.
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
* **No `Copy` specialisation.** Bulk operations such as `Clone` map `clone()`
  over the elements rather than specialising for `T: Copy` to a `memcpy`, so
  cloning a vector of `Copy` types is slower than `Vec::clone`. Behaviour is
  identical; only the constant factor differs.
* **The stack iterates bottom-to-top.** `Stack<T>` iterates from the bottom of
  the stack to the top — the reverse of its pop order — because it forwards to
  the backing `Vector`. Use `into_iter().rev()` for pop order. The `get` and
  `get_mut` accessors expose interior positions by index as a documented escape
  hatch; ordinary stack use goes through `top`/`top_mut` and `pop`.

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

The summary for `Stack<T>`:

| Operation | Average | Worst case |
|---|---|---|
| `push` | O(1) | O(n) when the capacity is exceeded |
| `pop` / `top` / `top_mut` | O(1) | O(1) |
| `clear` | O(n) | O(n) |
| `reserve` / `reserve_exact` / `shrink_to_fit` | O(n) | O(n) |

These are algorithmic bounds, not measured results, and no claims are made about
constant factors relative to `std::vec::Vec`.

### Benchmarking

The repository ships small, dependency-free micro-benchmarks that run a few
representative operations against the standard library side by side:

```sh
cargo bench --bench vector   # Vector<T> vs std::vec::Vec
cargo bench --bench stack    # Stack<T>  vs std::vec::Vec used as a stack
```

Each prints the best of a handful of repetitions for each operation. They are
sanity harnesses for spotting gross regressions while working on the
implementation — **not** rigorous benchmarks. There is no statistical analysis
or load isolation, so their numbers are not guarantees and are deliberately not
quoted in this README.

## Testing

```sh
cargo test                  # unit tests, integration tests and doctests
cargo test --examples       # compile and run the example programs
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check
```

The test suite is organised by collection and by behaviour. Cargo only treats
files directly under `tests/` as test crates, so `tests/vector.rs` and
`tests/stack.rs` each include their behaviour modules by path.

For `Vector<T>`:

| File | Covers |
|---|---|
| `tests/vector/core_behavior.rs` | Construction, growth, indexing, mutation, filtering, sorting, searching, rotation and slicing |
| `tests/vector/capacity.rs` | Reservation, shrinking, retained capacity, and overflow/error paths |
| `tests/vector/edge_cases.rs` | Empty and single-element vectors, zero-sized types, non-`Copy` values, nested vectors, empty `append`/`split_off` |
| `tests/vector/iterators.rs` | The three iteration forms, `size_hint`, `ExactSizeIterator`, `FusedIterator`, `DoubleEndedIterator`, early drop |
| `tests/vector/memory_safety.rs` | Exact destructor counts for every destructive operation, plus panicking predicates, `Clone` and `PartialEq` |
| `tests/vector/traits.rs` | `Debug`, `Clone`, `PartialEq`/`Eq`, `Deref`/`DerefMut`, `Index`/`IndexMut`, `Extend`, `FromIterator`, `IntoIterator` |

For `Stack<T>`:

| File | Covers |
|---|---|
| `tests/stack/core_behavior.rs` | Empty-stack behaviour, LIFO ordering, push/pop sequences, `top`/`top_mut`, `clear`, single-element and non-`Copy` values |
| `tests/stack/capacity.rs` | Growth, `with_capacity`, `reserve`, `reserve_exact`, `shrink_to_fit`, retained capacity |
| `tests/stack/edge_cases.rs` | Empty and single-element stacks, `swap`, zero-sized and nested values, large stacks |
| `tests/stack/memory_safety.rs` | Exact destructor counts for drop, clear and pop, plus zero-sized values with `Drop` |
| `tests/stack/traits.rs` | `Debug`, `Clone`, `PartialEq`/`Eq`, `Default`, `Extend`, `FromIterator` and all three iteration forms |

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

| Example (target name) | Source | Demonstrates |
|---|---|---|
| `basic_usage` | `examples/vector/basic_usage.rs` | Construction, push vs. insert, indexing vs. `get`, updating, iteration, `remove` vs. `swap_remove`, sorting, searching |
| `iteration` | `examples/vector/iteration.rs` | All three iteration forms, in-place mutation, the double-ended owned iterator, zero-sized types |
| `filtering_and_sorting` | `examples/vector/filtering_and_sorting.rs` | `retain` vs. `remove_where`, `dedup` vs. `remove_duplicates`, sorting, binary search, rotation, `windows` vs. `chunks` |
| `capacity_management` | `examples/vector/capacity_management.rs` | Reservation, retained capacity, `shrink_to_fit`, fallible reservation |
| `stack_basic_usage` | `examples/stack/basic_usage.rs` | Stack construction, `push`, `top`/`top_mut`, LIFO `pop`, empty-stack behaviour, `collect`, `clear` |
| `stack_iteration` | `examples/stack/iteration.rs` | All three iteration forms over a stack, in-place mutation, pop order via `rev`, zero-sized types |

The stack examples use a `stack_` prefix because Cargo derives a target name from
the file name, and both collections have a `basic_usage` and an `iteration`.

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
├── .github/workflows/ci.yml    formatting, lints, tests, docs, MSRV
├── src/
│   ├── lib.rs                  crate-level documentation
│   └── collections/
│       ├── mod.rs
│       ├── stack.rs            Stack<T> and its trait implementations
│       └── vector.rs           Vector<T>, its iterators and unit tests
├── examples/
│   ├── stack/                  Stack examples (see above)
│   └── vector/                 Vector examples (see above)
├── benches/
│   ├── stack/main.rs           dependency-free micro-benchmark
│   └── vector/main.rs          dependency-free micro-benchmark
└── tests/
    ├── common/mod.rs           shared test helpers
    ├── stack.rs                stack test crate root
    ├── stack/                  stack behaviour modules (see above)
    ├── vector.rs               vector test crate root
    └── vector/                 vector behaviour modules (see above)
```

## Roadmap

### Implemented

* `Vector<T>` — a growable, contiguous, heap-allocated array.
* `Stack<T>` — a LIFO adaptor over `Vector<T>`.

### Planned

* A doubly linked list and a hash map. These are ideas, **not** existing code —
  this README will be updated when and only if they exist.
* Improvements to `Vector<T>`:
  * linear-time `retain`/`dedup`/`remove_where`;
  * a structured error type for the fallible reservation methods;
  * a statistically sound benchmark suite (the current one is a sanity check).
* A Miri job in CI to exercise the `unsafe` code under the interpreter.

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
