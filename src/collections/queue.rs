//! A first-in, first-out (FIFO) queue.
//!
//! [`Queue<T>`] stores its elements in a growable
//! [`Vector<T>`](crate::collections::vector::Vector) but tracks a *head* index
//! so that dequeueing does not shift the remaining elements. The live elements
//! always occupy a contiguous slice `[head, len)` of the backing storage, so
//! slice and iterator access stay honest.

use crate::collections::vector::Vector;
use crate::collections::vector::VectorIntoIter;
use std::fmt::Debug;
use std::iter::Extend;
use std::iter::FromIterator;
use std::mem::ManuallyDrop;

/// A growable first-in, first-out (FIFO) queue.
///
/// `Queue<T>` stores its elements in a [`Vector<T>`](crate::collections::vector::Vector)
/// and tracks how many slots at the front have been dequeued. Only the
/// remaining slots hold live elements, and they are always contiguous, so
/// [`as_slice`](Queue::as_slice) and the iterators expose exactly the queue
/// contents in FIFO order.
///
/// ```
/// use rust_stl::collections::queue::Queue;
///
/// let mut queue = Queue::new();
/// queue.enqueue(1);
/// queue.enqueue(2);
///
/// assert_eq!(queue.front(), Some(&1));
/// assert_eq!(queue.dequeue(), Some(1));
/// assert_eq!(queue.dequeue(), Some(2));
/// assert_eq!(queue.dequeue(), None);
/// ```
pub struct Queue<T> {
    data: Vector<T>,
    head: usize,
}

impl<T> Queue<T> {
    /// Creates an empty queue.
    pub fn new() -> Self {
        Queue {
            data: Vector::new(),
            head: 0,
        }
    }

    /// Returns the number of elements in the queue.
    pub fn len(&self) -> usize {
        self.data.len().saturating_sub(self.head)
    }

    /// Returns `true` if the queue holds no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Adds `item` to the back of the queue.
    ///
    /// This is O(1) amortized.
    ///
    /// # Panics
    ///
    /// Panics if the additional element would overflow the queue's maximum
    /// length.
    pub fn enqueue(&mut self, item: T) {
        // Reclaim dead slots at the front before growing the allocation.
        if self.head > 0 && self.data.len() == self.data.capacity() {
            self.compact();
        }
        self.data.push(item);
    }

    /// Removes and returns the element at the front of the queue.
    ///
    /// Returns `None` if the queue is empty. This is O(1) amortized.
    pub fn dequeue(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        // SAFETY: `head < data.len()`, so the slot is initialised and holds a
        // live value. Reading it moves ownership to the caller; the slot is left
        // uninitialised and is excluded from future drops by `head`.
        let value = unsafe { (self.data.begin() as *mut T).add(self.head).read() };
        self.head += 1;

        // Compact when the dead prefix is at least as large as the live suffix,
        // keeping the dead prefix from dominating the storage.
        let live = self.data.len() - self.head;
        if self.head >= live || self.head == self.data.len() {
            self.compact();
        }

        Some(value)
    }

    /// Returns a shared reference to the element at the front of the queue.
    pub fn front(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            // SAFETY: `head < data.len()`.
            unsafe { Some(&*(self.data.begin().add(self.head))) }
        }
    }

    /// Returns a mutable reference to the element at the front of the queue.
    pub fn front_mut(&mut self) -> Option<&mut T> {
        if self.is_empty() {
            None
        } else {
            // SAFETY: `head < data.len()` and `&mut self` gives unique access.
            unsafe { Some(&mut *(self.data.begin() as *mut T).add(self.head)) }
        }
    }

    /// Returns a shared reference to the element at the back of the queue.
    pub fn back(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            self.data.last()
        }
    }

    /// Returns a mutable reference to the element at the back of the queue.
    pub fn back_mut(&mut self) -> Option<&mut T> {
        if self.is_empty() {
            None
        } else {
            self.data.last_mut()
        }
    }

    /// Returns a shared reference to the element at the front of the queue.
    ///
    /// This is an alias for [`front`](Queue::front).
    pub fn peek(&self) -> Option<&T> {
        self.front()
    }

    /// Returns a mutable reference to the element at the front of the queue.
    ///
    /// This is an alias for [`front_mut`](Queue::front_mut).
    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.front_mut()
    }

    /// Creates an empty queue with room for at least `capacity` elements.
    pub fn with_capacity(capacity: usize) -> Self {
        Queue {
            data: Vector::with_capacity(capacity),
            head: 0,
        }
    }

    /// Returns the number of elements the queue can hold without reallocating.
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    /// Removes every element, dropping each one exactly once.
    pub fn clear(&mut self) {
        self.compact();
        self.data.clear();
        self.head = 0;
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

    /// Fallible version of [`reserve`](Queue::reserve).
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), &'static str> {
        self.compact();
        self.data.try_reserve(additional)
    }

    /// Fallible version of [`reserve_exact`](Queue::reserve_exact).
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), &'static str> {
        self.compact();
        self.data.try_reserve_exact(additional)
    }

    /// Returns a shared reference to the element at `index`, counting from the
    /// front, or `None` if `index` is out of bounds.
    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len() {
            return None;
        }
        // SAFETY: `head + index < data.len()`.
        unsafe { Some(&*self.data.begin().add(self.head + index)) }
    }

    /// Returns a mutable reference to the element at `index`, counting from the
    /// front, or `None` if `index` is out of bounds.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.len() {
            return None;
        }
        // SAFETY: in bounds and `&mut self` gives unique access.
        unsafe { Some(&mut *(self.data.begin() as *mut T).add(self.head + index)) }
    }

    /// Returns the queue's contents as a contiguous slice, front to back.
    pub fn as_slice(&self) -> &[T] {
        // SAFETY: `head <= data.len()`, and the live elements are exactly that
        // range; they are initialised.
        unsafe {
            std::slice::from_raw_parts(
                self.data.begin().add(self.head),
                self.data.len() - self.head,
            )
        }
    }

    /// Returns the queue's contents as a mutable contiguous slice, front to
    /// back.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        let len = self.data.len() - self.head;
        // SAFETY: `head <= data.len()`, live elements are initialised, and
        // `&mut self` gives unique access.
        unsafe { std::slice::from_raw_parts_mut(self.data.begin().add(self.head) as *mut T, len) }
    }

    /// Returns an iterator over shared references to the elements, from front to
    /// back.
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.as_slice().iter()
    }

    /// Returns an iterator over mutable references to the elements, from front
    /// to back.
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.as_mut_slice().iter_mut()
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

    /// Moves every element of `other` to the back of this queue.
    pub fn append(&mut self, other: &mut Queue<T>) {
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
    ///
    /// Dead (already dequeued) slots are discarded without being dropped, since
    /// their values were moved out by [`dequeue`](Queue::dequeue).
    fn compact(&mut self) {
        if self.head == 0 {
            return;
        }
        let live = self.data.len() - self.head;
        if live > 0 {
            // SAFETY: source and destination are disjoint? They may overlap when
            // head < live; use copy (memmove) which handles overlap. The
            // destination has room for `live` elements because `live <= len`.
            unsafe {
                std::ptr::copy(
                    self.data.begin().add(self.head),
                    self.data.begin() as *mut T,
                    live,
                );
                self.data.set_len(live);
            }
        } else {
            // SAFETY: setting length to 0 drops nothing and leaves the
            // (uninitialised) dead slots outside the vector's logical range.
            unsafe {
                self.data.set_len(0);
            }
        }
        self.head = 0;
    }
}

impl<T> Drop for Queue<T> {
    fn drop(&mut self) {
        // Compact first so the backing vector owns exactly the live elements and
        // drops each of them exactly once.
        self.compact();
    }
}

impl<T> Default for Queue<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Debug for Queue<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.as_slice().iter()).finish()
    }
}

impl<T> Clone for Queue<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        let mut new_queue = Queue::new();
        new_queue.extend(self.iter().cloned());
        new_queue
    }
}

impl<T> PartialEq for Queue<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T> Eq for Queue<T> where T: Eq {}

impl<T> Extend<T> for Queue<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.enqueue(item);
        }
    }
}

impl<T> FromIterator<T> for Queue<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut queue = Queue::new();
        queue.extend(iter);
        queue
    }
}

impl<T> IntoIterator for Queue<T> {
    type Item = T;
    type IntoIter = VectorIntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        // Compact first so only live elements are owned, then hand the storage
        // to the vector's owned iterator. Use `ManuallyDrop` to avoid running
        // `Queue::drop` after the data has been moved out.
        let mut self_md = ManuallyDrop::new(self);
        self_md.compact();
        unsafe {
            let data = std::ptr::read(&self_md.data);
            data.into_iter()
        }
    }
}

impl<'a, T> IntoIterator for &'a Queue<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Queue<T> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_mut_slice().iter_mut()
    }
}
