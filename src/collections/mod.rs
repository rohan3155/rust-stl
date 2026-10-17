//! Classic STL-style containers.
//!
//! Each container lives in its own module and is re-exported only from that
//! module, so imports read as `rust_stl::collections::vector::Vector`.
//!
//! | Container | Module | Status |
//! |---|---|---|
//! | [`Vector<T>`](vector::Vector) | [`vector`] | Implemented |
//!
//! ## Status of other containers
//!
//! The repository roadmap mentions a doubly linked list, a hash map and a
//! stack/queue, but **none of them are implemented**. Do not expect modules or
//! types for them to exist.

pub mod vector;
pub mod stack;