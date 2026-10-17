# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project intends to follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

> **No version has been published yet.** Everything below is unreleased work in
> the repository; `Cargo.toml` currently reads `version = "0.1.0"` but the crate
> is not on crates.io. Until the first release the API may change without notice.

## [Unreleased]

### Added

* `Vector<T>`, a growable, contiguous, heap-allocated array, with:
  * indexing and access: `Index`/`IndexMut` (including range indexing), `get`,
    `get_mut`, `first`, `first_mut`, `last`, `last_mut`, `as_slice`,
    `as_mut_slice`;
  * mutation: `push`, `pop`, `insert`, `remove`, `swap`, `swap_remove`, `clear`,
    `truncate`, `reverse`;
  * capacity: `capacity`, `reserve`, `reserve_exact`, `try_reserve`,
    `try_reserve_exact`, `shrink_to_fit`;
  * bulk operations: `append`, `extend`, `extend_from_slice`, `resize`,
    `resize_with`, `split_off`;
  * filtering and de-duplication: `find_index`, `count_matches`, `indices_of`,
    `remove_first`, `retain`, `retain_mut`, `remove_all`, `remove_where`,
    `remove_where_mut`, `dedup`, `remove_duplicates`;
  * ordering and searching: `sort`, `sort_by`, `sort_range`, `binary_search`,
    `binary_search_by`, `binary_search_by_key`, `rotate_left`, `rotate_right`;
  * slicing: `split_at`, `windows`, `chunks`, `begin`, `end`.
* `Stack<T>`, a last-in, first-out (LIFO) stack backed by `Vector<T>`, with:
  * core operations: `new`, `push`, `pop`, `top`, `top_mut`, `len`, `is_empty`,
    `clear`;
  * capacity: `with_capacity`, `capacity`, `reserve`, `reserve_exact`,
    `shrink_to_fit`;
  * `swap`, plus documented `get`/`get_mut` escape hatches into the backing
    storage;
  * standard traits: `Debug`, `Clone`, `PartialEq`, `Eq`, `Default`, `Extend`,
    `FromIterator` and `IntoIterator` for `Stack<T>`, `&Stack<T>` and
    `&mut Stack<T>`.
* Iteration in all three ownership forms — `&Vector<T>`, `&mut Vector<T>` and
  `Vector<T>` — with `ExactSizeIterator`, `FusedIterator` and (for the owned
  iterator) `DoubleEndedIterator`.
* Standard trait implementations: `Debug`, `Clone`, `PartialEq`, `Eq`,
  `Default`, `Deref`, `DerefMut`, `Extend`, `FromIterator`, `IntoIterator`.
* Support for zero-sized element types, including ZSTs that implement `Drop`.
* Crate-level, module-level and per-method documentation, with compiled and run
  doctests.
* Runnable examples grouped by collection: `basic_usage`, `iteration`,
  `filtering_and_sorting` and `capacity_management` for the vector, plus
  `stack_basic_usage` and `stack_iteration` for the stack.
* An integration test suite organised by collection and behaviour, plus unit
  tests for private invariants. It covers exact destructor counts, panic safety,
  zero-sized types, capacity/overflow behaviour and all three iteration forms,
  for both `Vector<T>` and `Stack<T>`.
* A dependency-free micro-benchmark (`cargo bench --bench vector`).
* `README.md`, `CONTRIBUTING.md`, `SECURITY.md`, and dual `MIT OR Apache-2.0`
  licensing.
* GitHub Actions CI: formatting, clippy with warnings denied, tests, doctests,
  example compilation/execution, documentation with warnings denied, and a
  build against the declared MSRV.

### Fixed

These are defects that existed in the initial development of the container.
Several were memory-safety bugs (undefined behaviour or double drops), so this
section is worth reading in full.

* **Empty vectors held a null pointer.** `new()` stored `ptr::null_mut()` and
  handed it to `slice::from_raw_parts` and `dealloc`, which is undefined
  behaviour. The pointer is now a `NonNull<T>` initialised to
  `NonNull::dangling()`, which is non-null and correctly aligned.
* **Zero-sized types were mishandled.** `alloc`/`dealloc` were called with a
  zero-sized layout (undefined behaviour, and an abort in practice), and the
  capacity bookkeeping conflicted with the ZST special case. Zero-sized types
  now never touch the allocator, report `usize::MAX` capacity, and still have
  each value dropped exactly once.
* **`append` double-dropped every appended element.** It bit-copied the values
  and then called `other.clear()`, running their destructors even though
  ownership had moved. `other` is now emptied by resetting its length.
* **`reverse` underflowed on an empty vector.** It computed `len - 1` without a
  guard and panicked. Short vectors are now handled explicitly.
* **Reservation used unchecked arithmetic.** `len + additional` and
  `capacity * 2` could wrap and silently under-allocate.
  `try_reserve`/`try_reserve_exact` now report capacity and layout overflow as
  errors instead of panicking, and `reserve`/`reserve_exact` panic with a clear
  message.
* **`try_reallocate` freed a zero-sized layout.** On the first successful
  reservation it called `dealloc` with a zero-sized layout on a dangling
  pointer, aborting the process. It now goes through the shared deallocation
  helper, which skips that case.
* **`reserve(0)` reallocated.** The amortized target was computed
  unconditionally, so a request that already fit would still double the
  capacity. Growth is now gated on `len + additional > capacity`.
* **`shrink_to_fit` on an empty vector called `alloc` with a zero-sized
  layout.** It now releases the block and restores the dangling pointer.
* **The owned iterator dropped already-yielded elements.** `VectorIntoIter`
  tracked only an index and left the `Vector` intact, so dropping a partially
  consumed iterator dropped the yielded elements a second time. It now owns the
  allocation and an explicit `remaining: Range<usize>`, drops exactly that range
  on drop, and frees the allocation once.

### Changed

* **Breaking: `MyVec<T>` was renamed to `Vector<T>`.** The iterator types were
  renamed to match: `MyVecIter` → `VectorIter`, `MyVecIterMut` →
  `VectorIterMut`, `MyVecIntoIter` → `VectorIntoIter`. No compatibility aliases
  are provided, because this is the first public API. The Cargo package name
  (`rust-stl`) is unchanged.
* Indexing now delegates to slice indexing, so range indexing such as
  `vector[1..3]` works and two hand-written `unsafe` blocks were removed.
* Integration tests, examples and the benchmark are organised by collection
  (`tests/vector/`, `tests/stack/`, `examples/vector/`, `examples/stack/`,
  `benches/vector/`). Existing test, example and benchmark target names are
  unchanged.
* The whole repository is formatted with `rustfmt` and lints clean under
  `cargo clippy --all-targets --all-features -- -D warnings`.

### Known limitations

* `try_reserve`/`try_reserve_exact` return `Result<(), &'static str>` and do not
  distinguish capacity overflow from allocation failure.
* `retain`, `retain_mut`, `remove_where`, `remove_where_mut`, `remove_all`,
  `dedup` and `remove_duplicates` remove elements by shifting the tail, so their
  worst case is O(n²).
* The API is a subset of `std::vec::Vec`; `drain`, `splice` and fallible range
  indexing are not implemented.
* `Stack<T>` iterates from the bottom to the top — the reverse of its pop order —
  because it forwards to the backing `Vector`. Use `into_iter().rev()` for pop
  order. `get`/`get_mut` expose interior positions by index as a documented
  escape hatch.
* `Clone` (and other bulk operations) apply `clone()` element by element and do
  not specialise for `T: Copy` the way `Vec` does, so cloning a vector of `Copy`
  types is slower. Correct, just not `memcpy`-optimised.
* The crate is not `no_std` compatible.
* No performance claims are made, and the micro-benchmark is not statistically
  rigorous.
