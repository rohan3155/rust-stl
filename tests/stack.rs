//! Integration tests for [`Stack<T>`](rust_stl::collections::stack::Stack).
//!
//! The suite is split by behaviour into the modules under `tests/stack/`. They
//! are stitched together here because Cargo only treats files directly under
//! `tests/` as integration-test crates; nested files must be included by path.

#[path = "common/mod.rs"]
mod common;

#[path = "stack/capacity.rs"]
mod capacity;

#[path = "stack/core_behavior.rs"]
mod core_behavior;

#[path = "stack/edge_cases.rs"]
mod edge_cases;

#[path = "stack/memory_safety.rs"]
mod memory_safety;
