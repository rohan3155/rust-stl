//! Integration tests for [`Queue<T>`](rust_stl::collections::queue::Queue).
//!
//! The suite is split by behaviour into the modules under `tests/queue/`. They
//! are stitched together here because Cargo only treats files directly under
//! `tests/` as integration-test crates; nested files must be included by path.

#[path = "common/mod.rs"]
mod common;

#[path = "queue/capacity.rs"]
mod capacity;

#[path = "queue/core_behavior.rs"]
mod core_behavior;

#[path = "queue/edge_cases.rs"]
mod edge_cases;

#[path = "queue/iterators.rs"]
mod iterators;

#[path = "queue/memory_safety.rs"]
mod memory_safety;

#[path = "queue/traits.rs"]
mod traits;
