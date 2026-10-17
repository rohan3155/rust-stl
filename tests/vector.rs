//! Integration tests for [`Vector<T>`](rust_stl::collections::vector::Vector).
//!
//! The suite is split by behaviour into the modules under `tests/vector/`. They
//! are stitched together here because Cargo only treats files directly under
//! `tests/` as integration-test crates; nested files must be included by path.

#[path = "common/mod.rs"]
mod common;

#[path = "vector/capacity.rs"]
mod capacity;

#[path = "vector/core_behavior.rs"]
mod core_behavior;

#[path = "vector/edge_cases.rs"]
mod edge_cases;

#[path = "vector/iterators.rs"]
mod iterators;

#[path = "vector/memory_safety.rs"]
mod memory_safety;

#[path = "vector/traits.rs"]
mod traits;
