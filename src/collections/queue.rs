use crate::collections::vector::Vector;
use crate::collections::vector::VectorIntoIter;
use crate::collections::vector::VectorIter;
use crate::collections::vector::VectorIterMut;
use std::fmt::Debug;
use std::iter::Extend;
use std::iter::FromIterator;

pub struct Queue<T> {
    data: Vector<T>,
}

impl<T> Queue<T> {
    pub fn new() -> Self {
        Queue { data: Vector::new() }
    }
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn enqueue(&mut self, item: T) {
        self.data.push_back(item);
    }

    pub fn dequeue(&mut self) -> Option<T> {
        if self.is_empty() { None } else { Some(self.data.remove(0)) }
    }
    pub fn peek(&self) -> Option<&T> {
        self.data.get(0)
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.data.get_mut(0)
    }
    pub fn with_capacity(capacity: usize) -> Self {
        Queue { data: Vector::with_capacity(capacity) }
    }
    pub fn front(&self) -> Option<&T> {
        self.data.get(0)
    }
    pub fn front_mut(&mut self) -> Option<&mut T> {
        self.data.get_mut(0)
    }
    pub fn back(&self) -> Option<&T> {
        self.data.get(self.len() - 1)
    }
    pub fn back_mut(&mut self) -> Option<&mut T> {
        self.data.get_mut(self.len() - 1)
    }
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }
    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn reserve(&mut self, additional: usize) {
        self.data.reserve(additional);
    }
    pub fn reserve_exact(&mut self, additional: usize) {
        self.data.reserve_exact(additional);
    }
    pub fn shrink_to_fit(&mut self) {
        self.data.shrink_to_fit();
    }
    pub fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index)
    }
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.data.get_mut(index)
    }
    pub fn as_slice(&self) -> &[T] {
        self.data.as_slice()
    }

    pub fn try_reserve(&mut self, additional: usize) -> Result<(), &'static str> {
        self.data.try_reserve(additional)
    }
    pub fn iter(&self) -> VectorIter<'_, T> {
        self.data.iter()
    }
    pub fn iter_mut(&mut self) -> VectorIterMut<'_, T> {
        self.data.iter_mut()
    }
}

impl<T> Default for Queue<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Debug for Queue<T> where T: Debug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.data.iter()).finish()
    }
}

impl<T> Clone for Queue<T> where T: Clone {
    fn clone(&self) -> Self {
        Self { data: self.data.clone() }
    }
}

impl<T> PartialEq for Queue<T> where T: PartialEq {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl<T> Eq for Queue<T> where T: Eq {}

impl<T> Extend<T> for Queue<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.data.extend(iter);
    }
}

impl<T> FromIterator<T> for Queue<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut q = Queue::new();
        q.extend(iter);
        q
    }
}
impl<T> IntoIterator for Queue<T> {
    type Item = T;
    type IntoIter = VectorIntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a Queue<T> {
    type Item = &'a T;
    type IntoIter = VectorIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Queue<T> {
    type Item = &'a mut T;
    type IntoIter = VectorIterMut<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.iter_mut()
    }
}
