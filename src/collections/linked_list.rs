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
use std::iter::{Extend, FromIterator};
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
        unsafe { Some(&tail.as_ref().value) }
    }

    /// Returns a mutable reference to the last element.
    pub fn back_mut(&mut self) -> Option<&mut T> {
        let mut tail = self.tail?;
        // SAFETY: The tail pointer must point to a live node owned by `head`.
        // The mutable borrow of `self` prevents another safe mutable access
        // to this list for the lifetime of the returned reference.
        unsafe { Some(&mut tail.as_mut().value) }
    }
    /// Removes all elements from the list.
    pub fn clear(&mut self) {
        self.head = None;
        self.tail = None;
        self.len = 0;
    }
    pub fn push_back(&mut self, value: T) {
        let mut node = Box::new(Node::new(value));
        let node_ptr = NonNull::from(node.as_mut());

        match self.tail {
            Some(mut tail) => unsafe {
                node.prev = Some(tail);
                tail.as_mut().next = Some(node);
            },
            None => {
                self.head = Some(node);
            }
        }

        self.tail = Some(node_ptr);
        self.len += 1;
    }

    pub fn push_front(&mut self, value: T) {
        let mut node = Box::new(Node::new(value));
        let node_ptr = NonNull::from(node.as_mut());

        match self.head.take() {
            Some(mut head) => {
                head.prev = Some(node_ptr);
                node.next = Some(head);
                self.head = Some(node);
            }
            None => {
                self.tail = Some(node_ptr);
                self.head = Some(node);
            }
        }

        self.len += 1;
    }
    pub fn pop_back(&mut self) -> Option<T> {
        let mut tail = self.tail?;

        unsafe {
            if let Some(mut prev) = tail.as_mut().prev {
                prev.as_mut().next = None;
                self.tail = Some(prev);
            } else {
                self.head = None;
                self.tail = None;
            }

            self.len -= 1;

            let node = *Box::from_raw(tail.as_ptr());
            Some(node.value)
        }
    }
    /// Removes the first element and returns it, or `None` if the list is empty.
    pub fn pop_front(&mut self) -> Option<T> {
        let mut node = self.head.take()?;

        self.head = node.next.take();

        if let Some(new_head) = self.head.as_mut() {
            new_head.prev = None;
        } else {
            self.tail = None;
        }

        self.len -= 1;

        Some(node.value)
    }
    pub fn insert(&mut self, index: usize, value: T) {
        if index == 0 {
            self.push_front(value);
            return;
        }

        if index == self.len {
            self.push_back(value);
            return;
        }

        let mut current = self.head.as_mut().unwrap();

        for _ in 0..index - 1 {
            current = current.next.as_mut().unwrap();
        }

        let mut new_node = Box::new(Node::new(value));
        new_node.next = current.next.take();
        current.next = Some(new_node);
        self.len += 1;
    }
    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index >= self.len {
            return None;
        }

        if index == 0 {
            return self.pop_front();
        }

        if index == self.len - 1 {
            return self.pop_back();
        }

        let mut current = self.head.as_mut()?;

        for _ in 0..index - 1 {
            current = current.next.as_mut()?;
        }

        let mut removed = current.next.take()?;
        current.next = removed.next.take();
        self.len -= 1;

        Some(removed.value)
    }
    pub fn remove_first(&mut self, value: &T) -> bool
    where
        T: PartialEq,
    {
        let mut current = &mut self.head;

        while let Some(node) = current {
            if node.value == *value {
                let mut removed = current.take().unwrap();
                *current = removed.next.take();

                if let Some(next) = current.as_mut() {
                    next.prev = removed.prev;
                } else {
                    self.tail = removed.prev;
                }

                self.len -= 1;
                return true;
            }

            current = &mut current.as_mut().unwrap().next;
        }

        false
    }
    pub fn remove_all(&mut self, value: T)
    where
        T: PartialEq,
    {
        let mut current = &mut self.head;

        while let Some(node) = current.as_mut() {
            if node.value == value {
                let mut removed = current.take().unwrap();
                *current = removed.next.take();

                if let Some(next) = current.as_mut() {
                    next.prev = removed.prev;
                } else {
                    self.tail = removed.prev;
                }

                self.len -= 1;
            } else {
                current = &mut current.as_mut().unwrap().next;
            }
        }
    }
}

impl<T> Default for LinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}
