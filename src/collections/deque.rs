//! A double-ended queue.
//!
//! [`Deque<T>`] stores its elements in a growable
//! [`Vector<T>`](crate::collections::vector::Vector) but tracks a *head* index
//! so that popping from either end does not shift the remaining elements. The
//! live elements always occupy a contiguous range `[head, tail)` of the backing
//! storage, so slice and iterator access stay honest.

use crate::collections::vector::Vector;
use crate::collections::vector::VectorIntoIter;
use std::fmt::Debug;
use std::iter::Extend;
use std::iter::FromIterator;
use std::mem::ManuallyDrop;
use std::ops::{Index, IndexMut};

/// A growable double-ended queue.
pub struct Deque<T> {
    data: Vector<T>,
    head: usize,
}

impl<T> Deque<T> {
    pub fn new() -> Self {
        Deque {
            data: Vector::new(),
            head: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Deque {
            data: Vector::with_capacity(capacity),
            head: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.data.len().saturating_sub(self.head)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    pub fn push_front(&mut self, value: T) {
        if self.head > 0 {
            self.head -= 1;
            unsafe {
                (self.data.begin() as *mut T).add(self.head).write(value);
            }
        } else {
            self.data.insert(0, value);
        }
    }

    pub fn push_back(&mut self, value: T) {
        if self.head > 0 && self.data.len() == self.data.capacity() {
            self.compact();
        }
        self.data.push(value);
    }

    pub fn pop_front(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let value = unsafe { (self.data.begin() as *mut T).add(self.head).read() };
        self.head += 1;

        let live = self.data.len().saturating_sub(self.head);
        if self.head >= live || self.head == self.data.len() {
            self.compact();
        }

        Some(value)
    }

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

    pub fn front(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            unsafe { Some(&*self.data.begin().add(self.head)) }
        }
    }

    pub fn front_mut(&mut self) -> Option<&mut T> {
        if self.is_empty() {
            None
        } else {
            unsafe { Some(&mut *(self.data.begin() as *mut T).add(self.head)) }
        }
    }

    pub fn back(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            self.data.last()
        }
    }

    pub fn back_mut(&mut self) -> Option<&mut T> {
        if self.is_empty() {
            None
        } else {
            self.data.last_mut()
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len() {
            return None;
        }
        unsafe { Some(&*self.data.begin().add(self.head + index)) }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.len() {
            return None;
        }
        unsafe { Some(&mut *(self.data.begin() as *mut T).add(self.head + index)) }
    }

    pub fn clear(&mut self) {
        self.compact();
        self.data.clear();
        self.head = 0;
    }

    pub fn truncate(&mut self, new_len: usize) {
        if new_len >= self.len() {
            return;
        }
        self.data.truncate(self.head + new_len);
    }

    pub fn as_slice(&self) -> &[T] {
        let len = self.data.len().saturating_sub(self.head);
        unsafe { std::slice::from_raw_parts(self.data.begin().add(self.head), len) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        let len = self.data.len().saturating_sub(self.head);
        unsafe { std::slice::from_raw_parts_mut(self.data.begin().add(self.head) as *mut T, len) }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.as_slice().iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.as_mut_slice().iter_mut()
    }

    pub fn reserve(&mut self, additional: usize) {
        self.compact();
        self.data.reserve(additional);
    }

    pub fn reserve_exact(&mut self, additional: usize) {
        self.compact();
        self.data.reserve_exact(additional);
    }

    pub fn shrink_to_fit(&mut self) {
        self.compact();
        self.data.shrink_to_fit();
    }

    pub fn try_reserve(&mut self, additional: usize) -> Result<(), &'static str> {
        self.compact();
        self.data.try_reserve(additional)
    }

    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), &'static str> {
        self.compact();
        self.data.try_reserve_exact(additional)
    }

    pub fn retain<F>(&mut self, predicate: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.compact();
        self.data.retain(predicate);
    }

    pub fn retain_mut<F>(&mut self, predicate: F)
    where
        F: FnMut(&mut T) -> bool,
    {
        self.compact();
        self.data.retain_mut(predicate);
    }

    pub fn append(&mut self, other: &mut Deque<T>) {
        self.compact();
        other.compact();
        self.data.append(&mut other.data);
        other.head = 0;
    }

    pub fn swap(&mut self, other: &mut Self) {
        std::mem::swap(self, other);
    }

    fn compact(&mut self) {
        if self.head == 0 {
            return;
        }
        let live = self.data.len().saturating_sub(self.head);
        if live > 0 {
            unsafe {
                std::ptr::copy(
                    self.data.begin().add(self.head),
                    self.data.begin() as *mut T,
                    live,
                );
                self.data.set_len(live);
            }
        } else {
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
