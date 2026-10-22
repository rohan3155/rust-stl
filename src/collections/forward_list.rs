//! A singly linked list.
//!
//! [`ForwardList<T>`] stores its elements in individually allocated nodes linked
//! by `next` pointers. It is the "forward list" of this crate: the one to
//! reach for when only front insertion and removal are needed and a small
//! per-node footprint matters.
//!
//! ```
//! use rust_stl::collections::forward_list::ForwardList;
//!
//! let mut list = ForwardList::new();
//! list.push_front(3);
//! list.push_front(2);
//! list.push_front(1);
//!
//! assert_eq!(list.front(), Some(&1));
//! assert_eq!(list.pop_front(), Some(1));
//! assert_eq!(list.len(), 2);
//! ```

use std::fmt::Debug;
use std::iter::Extend;
use std::iter::FromIterator;
use std::ops::{Index, IndexMut};

struct Node<T> {
    value: T,
    next: Option<Box<Node<T>>>,
}

impl<T> Node<T> {
    fn new(value: T) -> Self {
        Self { value, next: None }
    }
}

/// A growable singly linked list.
///
/// `ForwardList<T>` keeps a `head` pointer and a length. There is no tail
/// pointer, so appending is O(`len`); front insertion and removal are O(1).
pub struct ForwardList<T> {
    head: Option<Box<Node<T>>>,
    len: usize,
}

/// A cursor into a [`ForwardList`].
///
/// Position `0` is the sentinel before the first element; position `n` refers
/// to the `n`-th element (1-based). The cursor is a lightweight, copyable
/// handle rather than a borrow of the list.
#[derive(Clone, Copy)]
pub struct Cursor {
    pos: usize,
}

/// An iterator over shared references to the elements of a [`ForwardList`].
pub struct ForwardListIter<'a, T> {
    current: Option<&'a Node<T>>,
    remaining: usize,
}

impl<'a, T> Iterator for ForwardListIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.current?;
        self.current = node.next.as_deref();
        self.remaining = self.remaining.saturating_sub(1);
        Some(&node.value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<T> ExactSizeIterator for ForwardListIter<'_, T> {}

impl<T> std::iter::FusedIterator for ForwardListIter<'_, T> {}

/// An iterator over mutable references to the elements of a [`ForwardList`].
pub struct ForwardListIterMut<'a, T> {
    current: Option<&'a mut Node<T>>,
    remaining: usize,
}

impl<'a, T> Iterator for ForwardListIterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.current.take()?;
        self.current = node.next.as_mut().map(|n| n.as_mut());
        self.remaining = self.remaining.saturating_sub(1);
        Some(&mut node.value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<T> ExactSizeIterator for ForwardListIterMut<'_, T> {}

impl<T> std::iter::FusedIterator for ForwardListIterMut<'_, T> {}

/// An owning iterator over a [`ForwardList`].
pub struct ForwardListIntoIter<T> {
    current: Option<Box<Node<T>>>,
    remaining: usize,
}

impl<T> Iterator for ForwardListIntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let mut node = self.current.take()?;
        self.current = node.next.take();
        self.remaining = self.remaining.saturating_sub(1);
        Some(node.value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<T> ExactSizeIterator for ForwardListIntoIter<T> {}

impl<T> std::iter::FusedIterator for ForwardListIntoIter<T> {}

impl<T> Drop for ForwardListIntoIter<T> {
    fn drop(&mut self) {
        // Drop any remaining nodes.
        while self.current.take().is_some() {}
    }
}

impl<T> ForwardList<T> {
    /// Creates an empty list.
    pub fn new() -> Self {
        Self { head: None, len: 0 }
    }

    /// Returns the number of elements in the list.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the list holds no elements.
    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    /// Adds `value` to the front of the list.
    pub fn push_front(&mut self, value: T) {
        let mut node = Box::new(Node::new(value));
        node.next = self.head.take();
        self.head = Some(node);
        self.len += 1;
    }

    /// Removes and returns the element at the front of the list.
    pub fn pop_front(&mut self) -> Option<T> {
        let mut node = self.head.take()?;
        self.head = node.next.take();
        self.len -= 1;
        Some(node.value)
    }

    /// Returns a shared reference to the front element.
    pub fn front(&self) -> Option<&T> {
        self.head.as_ref().map(|node| &node.value)
    }

    /// Returns a mutable reference to the front element.
    pub fn front_mut(&mut self) -> Option<&mut T> {
        self.head.as_mut().map(|node| &mut node.value)
    }

    /// Adds `value` to the back of the list.
    pub fn push_back(&mut self, value: T) {
        let mut current = &mut self.head;

        while let Some(node) = current {
            current = &mut node.next;
        }

        *current = Some(Box::new(Node::new(value)));
        self.len += 1;
    }

    /// Removes and returns the element at the back of the list.
    pub fn pop_back(&mut self) -> Option<T> {
        self.head.as_ref()?;

        if self.head.as_ref().unwrap().next.is_none() {
            self.len = 0;
            return self.head.take().map(|node| node.value);
        }

        let mut current = self.head.as_mut().unwrap();

        while current.next.as_ref().unwrap().next.is_some() {
            current = current.next.as_mut().unwrap();
        }

        let last = current.next.take().unwrap();
        self.len -= 1;

        Some(last.value)
    }

    /// Removes every element.
    pub fn clear(&mut self) {
        self.head = None;
        self.len = 0;
    }

    /// Inserts `value` at `index`, shifting later elements right.
    pub fn insert(&mut self, index: usize, value: T) {
        if index > self.len {
            panic!("index out of bounds");
        }

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

    /// Returns a cursor to the sentinel position before the first element.
    pub fn before_begin(&self) -> Cursor {
        Cursor { pos: 0 }
    }

    /// Inserts `value` after the element referenced by `pos`.
    ///
    /// `pos` may be the sentinel returned by [`before_begin`](Self::before_begin),
    /// in which case the value becomes the new first element. Returns a cursor
    /// to the newly inserted element.
    pub fn insert_after(&mut self, pos: Cursor, value: T) -> Cursor {
        if pos.pos == 0 {
            self.push_front(value);
            return Cursor { pos: 1 };
        }

        if pos.pos >= self.len {
            self.push_back(value);
            return Cursor { pos: self.len };
        }

        let mut current = self.head.as_mut().unwrap();
        for _ in 0..pos.pos - 1 {
            current = current.next.as_mut().unwrap();
        }

        let mut new_node = Box::new(Node::new(value));
        new_node.next = current.next.take();
        current.next = Some(new_node);
        self.len += 1;

        Cursor { pos: pos.pos + 1 }
    }

    /// Removes the element after the one referenced by `pos`.
    pub fn erase_after(&mut self, pos: Cursor) -> Option<T> {
        if pos.pos == 0 {
            return self.pop_front();
        }

        if pos.pos >= self.len {
            return None;
        }

        let mut current = self.head.as_mut()?;
        for _ in 0..pos.pos - 1 {
            current = current.next.as_mut()?;
        }

        let mut removed = current.next.take()?;
        current.next = removed.next.take();
        self.len -= 1;

        Some(removed.value)
    }

    /// Moves every element of `other` so that they follow the element referenced
    /// by `pos` in `self`.
    pub fn splice_after(&mut self, pos: Cursor, other: &mut Self) {
        if other.is_empty() {
            return;
        }

        if pos.pos == 0 {
            if self.is_empty() {
                self.head = other.head.take();
            } else {
                let mut tail = other.head.as_mut().unwrap();
                while tail.next.is_some() {
                    tail = tail.next.as_mut().unwrap();
                }
                tail.next = self.head.take();
                self.head = other.head.take();
            }
        } else {
            if pos.pos >= self.len {
                self.append(other);
                return;
            }

            let mut current = self.head.as_mut().unwrap();
            for _ in 0..pos.pos - 1 {
                current = current.next.as_mut().unwrap();
            }

            let mut tail = other.head.as_mut().unwrap();
            while tail.next.is_some() {
                tail = tail.next.as_mut().unwrap();
            }
            tail.next = current.next.take();
            current.next = other.head.take();
        }

        self.len += other.len;
        other.len = 0;
    }

    /// Moves every element of `other` to the back of `self`.
    pub fn append(&mut self, other: &mut Self) {
        if other.is_empty() {
            return;
        }

        if self.is_empty() {
            self.head = other.head.take();
        } else {
            let mut tail = self.head.as_mut().unwrap();
            while tail.next.is_some() {
                tail = tail.next.as_mut().unwrap();
            }
            tail.next = other.head.take();
        }

        self.len += other.len;
        other.len = 0;
    }

    /// Removes the element at `index`.
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

    /// Removes the element after `index`.
    pub fn remove_after(&mut self, index: usize) -> Option<T> {
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

    /// Removes the first element equal to `value`.
    pub fn remove_first(&mut self, value: &T) -> bool
    where
        T: PartialEq,
    {
        let mut current = &mut self.head;

        loop {
            match current {
                Some(node) if node.value == *value => {
                    let mut removed = current.take().unwrap();
                    *current = removed.next.take();
                    self.len -= 1;
                    return true;
                }
                Some(node) => {
                    current = &mut node.next;
                }
                None => return false,
            }
        }
    }

    /// Removes all elements equal to `value`.
    pub fn remove_all(&mut self, value: T)
    where
        T: PartialEq,
    {
        let mut current = &mut self.head;

        while let Some(node) = current.as_mut() {
            if node.value == value {
                let mut removed = current.take().unwrap();
                *current = removed.next.take();
                self.len -= 1;
            } else {
                current = &mut current.as_mut().unwrap().next;
            }
        }
    }

    /// Keeps only elements satisfying `predicate`.
    pub fn retain<F>(&mut self, mut predicate: F)
    where
        F: FnMut(&T) -> bool,
    {
        let mut current = &mut self.head;

        while current.is_some() {
            let keep = {
                let node = current.as_mut().unwrap();
                predicate(&node.value)
            };

            if keep {
                let node = current.as_mut().unwrap();
                current = &mut node.next;
            } else {
                let mut removed = current.take().unwrap();
                *current = removed.next.take();
                self.len -= 1;
            }
        }
    }

    /// Reverses the list in place.
    pub fn reverse(&mut self) {
        let mut prev = None;
        let mut current = self.head.take();

        while let Some(mut node) = current {
            let next = node.next.take();
            node.next = prev;
            prev = Some(node);
            current = next;
        }

        self.head = prev;
    }

    /// Resizes the list to `new_len`, filling new slots with clones of `value`.
    pub fn resize(&mut self, new_len: usize, value: T)
    where
        T: Clone,
    {
        if new_len < self.len {
            if new_len == 0 {
                self.clear();
                return;
            }

            let mut current = self.head.as_mut().unwrap();
            for _ in 0..new_len - 1 {
                current = current.next.as_mut().unwrap();
            }
            current.next = None;
            self.len = new_len;
        } else {
            for _ in self.len..new_len {
                self.push_back(value.clone());
            }
        }
    }

    /// Returns an iterator over shared references.
    pub fn iter(&self) -> ForwardListIter<'_, T> {
        ForwardListIter {
            current: self.head.as_deref(),
            remaining: self.len,
        }
    }

    /// Returns an iterator over mutable references.
    pub fn iter_mut(&mut self) -> ForwardListIterMut<'_, T> {
        ForwardListIterMut {
            current: self.head.as_mut().map(|n| n.as_mut()),
            remaining: self.len,
        }
    }

    /// Returns the index of the first element satisfying `predicate`.
    pub fn find_index<F>(&self, predicate: F) -> Option<usize>
    where
        F: FnMut(&T) -> bool,
    {
        self.iter().position(predicate)
    }

    /// Returns a shared reference to the element at `index`.
    pub fn get(&self, index: usize) -> Option<&T> {
        self.iter().nth(index)
    }

    /// Returns a mutable reference to the element at `index`.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.iter_mut().nth(index)
    }
}

impl<T> Default for ForwardList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Debug for ForwardList<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T> Clone for ForwardList<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        let mut list = ForwardList::new();
        for value in self.iter() {
            list.push_back(value.clone());
        }
        list
    }
}

impl<T> PartialEq for ForwardList<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.iter().eq(other.iter())
    }
}

impl<T> Eq for ForwardList<T> where T: Eq {}

impl<T> Extend<T> for ForwardList<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for value in iter {
            self.push_back(value);
        }
    }
}

impl<T> FromIterator<T> for ForwardList<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut list = ForwardList::new();
        list.extend(iter);
        list
    }
}

impl<T> IntoIterator for ForwardList<T> {
    type Item = T;
    type IntoIter = ForwardListIntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        let remaining = self.len;
        ForwardListIntoIter {
            current: self.head,
            remaining,
        }
    }
}

impl<'a, T> IntoIterator for &'a ForwardList<T> {
    type Item = &'a T;
    type IntoIter = ForwardListIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut ForwardList<T> {
    type Item = &'a mut T;
    type IntoIter = ForwardListIterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T> Index<usize> for ForwardList<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("index out of bounds")
    }
}

impl<T> IndexMut<usize> for ForwardList<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).expect("index out of bounds")
    }
}
