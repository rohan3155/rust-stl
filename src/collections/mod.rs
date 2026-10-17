//! Classic STL-style containers.
//!
//! Each container lives in its own module and is re-exported only from that
//! module, so imports read as `rust_stl::collections::vector::Vector` or
//! `rust_stl::collections::stack::Stack`.
//!
//! | Container | Module | Status |
//! |---|---|---|
//! | [`Vector<T>`](vector::Vector) | [`vector`] | Implemented |
//! | [`Stack<T>`](stack::Stack) | [`stack`] | Implemented |
//!
//! ## Status of other containers
//!
//! The repository roadmap also mentions a doubly linked list and a hash map, but
//! **they are not implemented**. Do not expect modules or types for them to
//! exist.

pub mod stack;
pub mod vector;
