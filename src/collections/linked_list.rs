//! A doubly linked list.
//!
//! [`LinkedList<T>`] stores elements in individually allocated nodes linked
//! in both directions. Insertion and removal at either end are O(1).
//!
//! ```
//! use rust_stl::collections::linked_list::LinkedList;
//!
//! let mut list = LinkedList::new();
//! assert!(list.is_empty());
//! assert_eq!(list.len(), 0);
//! ```

use std::fmt;
use std::iter::{ Extend, FromIterator };
use std::ptr::NonNull;

/// A node in the doubly linked list.
struct Node<T> {
    value: T,
    next: Option<Box<Node<T>>>,
    prev: Option<NonNull<Node<T>>>,
}

impl<T> Node<T> {
    fn new(value: T) -> Self {
        Self {
            value,
            next: None,
            prev: None,
        }
    }
}

/// A doubly linked list.
///
/// The list owns its nodes through the forward `next` chain.
/// The `prev` links and `tail` pointer are non-owning pointers.
///
/// # Invariants
///
/// - `head.is_none()` if and only if `tail.is_none()`.
/// - `len == 0` if and only if the list is empty.
/// - `head` has no previous node.
/// - `tail` has no next node.
/// - Every backward link points to the preceding node in the forward chain.
///
/// # Safety
///
/// Any implementation that dereferences `prev` or `tail` must ensure
/// the pointed-to node is still alive and belongs to this list.
pub struct LinkedList<T> {
    head: Option<Box<Node<T>>>,
    tail: Option<NonNull<Node<T>>>,
    len: usize,
}

impl<T> LinkedList<T> {
    /// Creates an empty list.
    pub const fn new() -> Self {
        Self {
            head: None,
            tail: None,
            len: 0,
        }
    }

    /// Returns the number of elements in the list.
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the list contains no elements.
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns a reference to the first element.
    pub fn front(&self) -> Option<&T> {
        self.head.as_deref().map(|node| &node.value)
    }

    /// Returns a mutable reference to the first element.
    pub fn front_mut(&mut self) -> Option<&mut T> {
        self.head.as_deref_mut().map(|node| &mut node.value)
    }
    /// Returns a reference to the last element.
    pub fn back(&self) -> Option<&T> {
        let tail = self.tail?;
        // SAFETY: The tail pointer must point to a live node owned by `head`.
        // This invariant must be maintained by every mutating operation.
        unsafe {
            Some(&tail.as_ref().value)
        }
    }

    /// Returns a mutable reference to the last element.
    pub fn back_mut(&mut self) -> Option<&mut T> {
        let mut tail = self.tail?;
        // SAFETY: The tail pointer must point to a live node owned by `head`.
        // The mutable borrow of `self` prevents another safe mutable access
        // to this list for the lifetime of the returned reference.
        unsafe {
            Some(&mut tail.as_mut().value)
        }
    }
    /// Removes all elements from the list.
    pub fn clear(&mut self) {
        self.head = None;
        self.tail = None;
        self.len = 0;
    }
}

impl<T> Default for LinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}
