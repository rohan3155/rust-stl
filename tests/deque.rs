//! Integration tests for [`Deque<T>`](rust_stl::collections::deque::Deque).

#[path = "common/mod.rs"]
mod common;

#[path = "deque/core_behavior.rs"]
mod core_behavior;

#[path = "deque/capacity.rs"]
mod capacity;

#[path = "deque/edge_cases.rs"]
mod edge_cases;

#[path = "deque/iterators.rs"]
mod iterators;

#[path = "deque/memory_safety.rs"]
mod memory_safety;

#[path = "deque/traits.rs"]
mod traits;
