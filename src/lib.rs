//! # rust-stl
//!
//! A small, educational reimplementation of classic STL-style containers in
//! safe-encapsulating Rust, built from raw pointers and manual allocation so the
//! memory-management machinery is visible rather than hidden.
//!
//! ## Design goals
//!
//! * **Readable, auditable internals.** Each container owns a small number of
//!   private invariants, and every `unsafe` block carries a `// SAFETY:` comment
//!   explaining the invariant that makes it sound.
//! * **No hidden dependencies.** `rust-stl` has no runtime dependencies.
//! * **Honest scope.** This is a learning project, not a competitor to the
//!   standard library. The implementations are believed to be correct and are
//!   exercised by an extensive test suite, but they are not optimised or
//!   benchmarked against `std::collections`.
//!
//! ## Available containers
//!
//! | Type | Module | Description |
//! |---|---|---|
//! | [`Vector<T>`](collections::vector::Vector) | [`collections::vector`] | Growable, contiguous, heap-allocated array |
//! | [`Stack<T>`](collections::stack::Stack) | [`collections::stack`] | Last-in, first-out (LIFO) stack backed by a `Vector` |
//!
//! These two containers are the implemented API today; the other containers
//! listed in the repository's roadmap do not exist yet and are not part of this
//! crate.
//!
//! ## Quick start
//!
//! ```
//! use rust_stl::collections::vector::Vector;
//!
//! let mut numbers: Vector<i32> = Vector::new();
//! numbers.push(30);
//! numbers.push(10);
//! numbers.push(20);
//!
//! numbers.sort();
//! assert_eq!(numbers.as_slice(), &[10, 20, 30]);
//!
//! assert_eq!(numbers.binary_search(&20), Ok(1));
//!
//! // Iterate by reference, in place, or by value.
//! let total: i32 = (&numbers).into_iter().sum();
//! assert_eq!(total, 60);
//!
//! let owned: Vec<i32> = numbers.into_iter().collect();
//! assert_eq!(owned, vec![10, 20, 30]);
//! ```
//!
//! ## Important limitations
//!
//! * The API covers a subset of [`std::vec::Vec`]; see the
//!   [`Vector`](collections::vector::Vector) documentation for the specific
//!   differences.
//! * [`try_reserve`](collections::vector::Vector::try_reserve) reports failures
//!   as a `&'static str` rather than a structured error type.
//! * Operations that remove an element by shifting the tail
//!   ([`retain`](collections::vector::Vector::retain),
//!   [`dedup`](collections::vector::Vector::dedup),
//!   [`remove_where`](collections::vector::Vector::remove_where), …) are O(`n²`)
//!   in the worst case, unlike the linear-time implementations in the standard
//!   library.
//! * The containers are not `no_std` compatible.
//!
//! ## Safety
//!
//! The public API contains no `unsafe` items — only the internal implementation
//! does. If you find a way to trigger undefined behaviour through the public
//! API, please report it as described in the repository's `SECURITY.md`.
//!
//! ## Crate layout
//!
//! * [`collections`] — one module per container.
//!   * [`collections::vector`] — [`Vector<T>`](collections::vector::Vector) and
//!     its iterators.
//!   * [`collections::stack`] — [`Stack<T>`](collections::stack::Stack), a LIFO
//!     adaptor over `Vector`.

pub mod collections;
