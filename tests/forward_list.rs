//! Integration tests for [`ForwardList<T>`](rust_stl::collections::forward_list::ForwardList).

#[path = "common/mod.rs"]
mod common;

#[path = "forward_list/core_behavior.rs"]
mod core_behavior;

#[path = "forward_list/edge_cases.rs"]
mod edge_cases;

#[path = "forward_list/iterators.rs"]
mod iterators;

#[path = "forward_list/memory_safety.rs"]
mod memory_safety;

#[path = "forward_list/traits.rs"]
mod traits;
