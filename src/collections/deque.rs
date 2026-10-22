//! A double-ended queue.
//!
//! [`Deque<T>`] stores its elements in a growable
//! [`Vector<T>`](crate::collections::vector::Vector) but tracks a *head* index
//! so that popping from either end does not shift the remaining elements. The
//! live elements always occupy a contiguous range `[head, tail)` of the backing
//! storage, so slice and iterator access stay honest.
//!
//! ```
//! use rust_stl::collections::deque::Deque;
//!
//! let mut deque = Deque::new();
//! deque.push_back(2);
//! deque.push_front(1);
//!
//! assert_eq!(deque.front(), Some(&1));
//! assert_eq!(deque.back(), Some(&2));
//! assert_eq!(deque.pop_front(), Some(1));
//! assert_eq!(deque.pop_back(), Some(2));
//! assert_eq!(deque.pop_front(), None);
//! ```

use crate::collections::vector::Vector;
use crate::collections::vector::VectorIntoIter;
use std::fmt::Debug;
use std::iter::Extend;
use std::iter::FromIterator;
use std::mem::ManuallyDrop;
use std::ops::{Index, IndexMut};

/// A growable double-ended queue.
///
/// `Deque<T>` stores its elements in a [`Vector<T>`](crate::collections::vector::Vector)
/// and tracks how many slots at the front have been popped. Only the remaining
/// slots hold live elements, and they are always contiguous, so
/// [`as_slice`](Deque::as_slice) and the iterators expose exactly the deque
/// contents from front to back.
pub struct Deque<T> {
    data: Vector<T>,
    head: usize,
}

impl<T> Deque<T> {
    /// Creates an empty deque.
    ///
    /// No allocation is performed; the first push triggers the first
    /// allocation. Use [`with_capacity`](Deque::with_capacity) when the final
    /// size is known in advance.
    pub fn new() -> Self {
        Deque {
            data: Vector::new(),
            head: 0,
        }
    }

    /// Creates an empty deque with room for at least `capacity` elements.
    pub fn with_capacity(capacity: usize) -> Self {
        Deque {
            data: Vector::with_capacity(capacity),
            head: 0,
        }
    }

    /// Returns the number of elements in the deque.
    pub fn len(&self) -> usize {
        self.data.len().saturating_sub(self.head)
    }

    /// Returns `true` if the deque holds no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of elements the deque can hold without reallocating.
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    /// Adds `value` to the front of the deque.
    pub fn push_front(&mut self, value: T) {
        if self.head > 0 {
            self.head -= 1;
            // SAFETY: `head` pointed at the first live slot, so `head - 1` is a
            // dead slot still inside the allocation and is now initialised.
            unsafe {
                (self.data.begin() as *mut T).add(self.head).write(value);
            }
        } else {
            self.data.insert(0, value);
        }
    }

    /// Adds `value` to the back of the deque.
    pub fn push_back(&mut self, value: T) {
        if self.head > 0 && self.data.len() == self.data.capacity() {
            self.compact();
        }
        self.data.push(value);
    }

    /// Removes and returns the element at the front of the deque.
    pub fn pop_front(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        // SAFETY: `head < data.len()`, so the slot is initialised and holds a
        // live value. Reading it moves ownership to the caller.
        let value = unsafe { (self.data.begin() as *mut T).add(self.head).read() };
        self.head += 1;

        let live = self.data.len().saturating_sub(self.head);
        if self.head >= live || self.head == self.data.len() {
            self.compact();
        }

        Some(value)
    }

    /// Removes and returns the element at the back of the deque.
    pub fn pop_back(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let value = self.data.pop().expect("deque non-empty");
        if self.is_empty() {
            self.compact();
        }
        Some(value)
    }

    /// Returns a shared reference to the front element.
    pub fn front(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            // SAFETY: `head < data.len()`.
            unsafe { Some(&*self.data.begin().add(self.head)) }
        }
    }

    /// Returns a mutable reference to the front element.
    pub fn front_mut(&mut self) -> Option<&mut T> {
        if self.is_empty() {
            None
        } else {
            // SAFETY: in bounds and `&mut self` gives unique access.
            unsafe { Some(&mut *(self.data.begin() as *mut T).add(self.head)) }
        }
    }

    /// Returns a shared reference to the back element.
    pub fn back(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            self.data.last()
        }
    }

    /// Returns a mutable reference to the back element.
    pub fn back_mut(&mut self) -> Option<&mut T> {
        if self.is_empty() {
            None
        } else {
            self.data.last_mut()
        }
    }

    /// Returns a shared reference to the element at `index`, counting from the
    /// front, or `None` if out of bounds.
    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len() {
            return None;
        }
        // SAFETY: `head + index < data.len()`.
        unsafe { Some(&*self.data.begin().add(self.head + index)) }
    }

    /// Returns a mutable reference to the element at `index`, counting from the
    /// front, or `None` if out of bounds.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.len() {
            return None;
        }
        // SAFETY: in bounds and `&mut self` gives unique access.
        unsafe { Some(&mut *(self.data.begin() as *mut T).add(self.head + index)) }
    }

    /// Removes every element, dropping each one exactly once.
    pub fn clear(&mut self) {
        self.compact();
        self.data.clear();
        self.head = 0;
    }

    /// Shortens the deque to at most `new_len` elements, dropping the removed
    /// tail.
    pub fn truncate(&mut self, new_len: usize) {
        if new_len >= self.len() {
            return;
        }
        self.data.truncate(self.head + new_len);
    }

    /// Returns the deque's contents as a contiguous slice, front to back.
    pub fn as_slice(&self) -> &[T] {
        let len = self.data.len().saturating_sub(self.head);
        // SAFETY: `head <= data.len()`, and the live elements are exactly that
        // range; they are initialised.
        unsafe { std::slice::from_raw_parts(self.data.begin().add(self.head), len) }
    }

    /// Returns the deque's contents as a mutable contiguous slice, front to
    /// back.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        let len = self.data.len().saturating_sub(self.head);
        // SAFETY: `head <= data.len()`, live elements are initialised, and
        // `&mut self` gives unique access.
        unsafe { std::slice::from_raw_parts_mut(self.data.begin().add(self.head) as *mut T, len) }
    }

    /// Returns an iterator over shared references to the elements, front to back.
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.as_slice().iter()
    }

    /// Returns an iterator over mutable references to the elements, front to back.
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.as_mut_slice().iter_mut()
    }

    /// Reserves capacity for at least `additional` more elements.
    pub fn reserve(&mut self, additional: usize) {
        self.compact();
        self.data.reserve(additional);
    }

    /// Reserves capacity for exactly `additional` more elements.
    pub fn reserve_exact(&mut self, additional: usize) {
        self.compact();
        self.data.reserve_exact(additional);
    }

    /// Reduces the capacity to the current length.
    pub fn shrink_to_fit(&mut self) {
        self.compact();
        self.data.shrink_to_fit();
    }

    /// Fallible version of [`reserve`](Deque::reserve).
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), &'static str> {
        self.compact();
        self.data.try_reserve(additional)
    }

    /// Fallible version of [`reserve_exact`](Deque::reserve_exact).
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), &'static str> {
        self.compact();
        self.data.try_reserve_exact(additional)
    }

    /// Keeps only the elements for which `predicate` returns `true`.
    pub fn retain<F>(&mut self, predicate: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.compact();
        self.data.retain(predicate);
    }

    /// Keeps only the elements for which `predicate` returns `true`, allowing
    /// mutation.
    pub fn retain_mut<F>(&mut self, predicate: F)
    where
        F: FnMut(&mut T) -> bool,
    {
        self.compact();
        self.data.retain_mut(predicate);
    }

    /// Moves every element of `other` to the back of this deque.
    pub fn append(&mut self, other: &mut Deque<T>) {
        self.compact();
        other.compact();
        self.data.append(&mut other.data);
        other.head = 0;
    }

    /// Swaps the contents of `self` and `other` in O(1).
    pub fn swap(&mut self, other: &mut Self) {
        std::mem::swap(self, other);
    }

    /// Compacts the storage so that the live elements start at index 0.
    fn compact(&mut self) {
        if self.head == 0 {
            return;
        }
        let live = self.data.len().saturating_sub(self.head);
        if live > 0 {
            // SAFETY: source and destination may overlap when head < live; use
            // copy (memmove) which handles overlap.
            unsafe {
                std::ptr::copy(
                    self.data.begin().add(self.head),
                    self.data.begin() as *mut T,
                    live,
                );
                self.data.set_len(live);
            }
        } else {
            // SAFETY: setting length to 0 drops nothing.
            unsafe {
                self.data.set_len(0);
            }
        }
        self.head = 0;
    }
}

impl<T> Drop for Deque<T> {
    fn drop(&mut self) {
        self.compact();
    }
}

impl<T> Default for Deque<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Debug for Deque<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.as_slice().iter()).finish()
    }
}

impl<T> Clone for Deque<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        let mut new_deque = Deque::new();
        new_deque.extend(self.iter().cloned());
        new_deque
    }
}

impl<T> PartialEq for Deque<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T> Eq for Deque<T> where T: Eq {}

impl<T> Extend<T> for Deque<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.push_back(item);
        }
    }
}

impl<T> FromIterator<T> for Deque<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut deque = Deque::new();
        deque.extend(iter);
        deque
    }
}

impl<T> IntoIterator for Deque<T> {
    type Item = T;
    type IntoIter = VectorIntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        let mut self_md = ManuallyDrop::new(self);
        self_md.compact();
        unsafe {
            let data = std::ptr::read(&self_md.data);
            data.into_iter()
        }
    }
}

impl<'a, T> IntoIterator for &'a Deque<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Deque<T> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_mut_slice().iter_mut()
    }
}

impl<T, I> Index<I> for Deque<T>
where
    I: std::slice::SliceIndex<[T]>,
{
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        &self.as_slice()[index]
    }
}

impl<T, I> IndexMut<I> for Deque<T>
where
    I: std::slice::SliceIndex<[T]>,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.as_mut_slice()[index]
    }
}
