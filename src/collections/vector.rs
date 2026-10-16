use std::alloc::{ alloc, dealloc, handle_alloc_error, Layout };
use std::ptr;
use std::slice;
use std::iter::FusedIterator;
use std::ops::{ Deref, DerefMut, Index, IndexMut };
use std::fmt::Debug;
use std::iter::{ Extend, FromIterator };
use std::cmp::{ PartialEq, Eq };
pub struct MyVec<T> {
    ptr: *mut T,
    len: usize,
    capacity: usize,
}

impl<T> MyVec<T> {
    pub fn new() -> Self {
        Self {
            ptr: ptr::null_mut(),
            len: 0,
            capacity: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            unsafe { Some(&*self.ptr.add(index)) }
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.len {
            unsafe { Some(&mut *self.ptr.add(index)) }
        } else {
            None
        }
    }

    fn layout(capacity: usize) -> Layout {
        Layout::array::<T>(capacity).expect("capacity overflow")
    }

    fn grow(&mut self) {
        let new_capacity = if self.capacity == 0 {
            4
        } else {
            self.capacity.checked_mul(2).expect("capacity overflow")
        };

        if std::mem::size_of::<T>() == 0 {
            self.capacity = usize::MAX;
            return;
        }

        let new_layout = Self::layout(new_capacity);

        let new_ptr = unsafe {
            let allocation = alloc(new_layout) as *mut T;

            if allocation.is_null() {
                handle_alloc_error(new_layout);
            }

            if self.len > 0 {
                ptr::copy_nonoverlapping(self.ptr, allocation, self.len);
            }

            if self.capacity > 0 {
                dealloc(self.ptr as *mut u8, Self::layout(self.capacity));
            }

            allocation
        };

        self.ptr = new_ptr;
        self.capacity = new_capacity;
    }

    pub fn push(&mut self, value: T) {
        if self.len == self.capacity {
            self.grow();
        }

        unsafe {
            self.ptr.add(self.len).write(value);
        }

        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }

        Some(self.remove(self.len - 1))
    }

    pub fn find_index<F>(&self, mut predicate: F) -> Option<usize> where F: FnMut(&T) -> bool {
        (0..self.len).find(|&index| predicate(unsafe { &*self.ptr.add(index) }))
    }

    pub fn count_matches<F>(&self, mut predicate: F) -> usize where F: FnMut(&T) -> bool {
        (0..self.len).filter(|&index| predicate(unsafe { &*self.ptr.add(index) })).count()
    }

    pub fn remove_first(&mut self, value: &T) -> bool where T: PartialEq {
        if let Some(index) = self.find_index(|item| item == value) {
            self.remove(index);
            true
        } else {
            false
        }
    }

    pub fn remove(&mut self, index: usize) -> T {
        assert!(index < self.len, "index out of bounds");

        unsafe {
            let element = self.ptr.add(index).read();
            ptr::copy(self.ptr.add(index + 1), self.ptr.add(index), self.len - index - 1);
            self.len -= 1;
            element
        }
    }

    pub fn indices_of<F>(&self, mut predicate: F) -> MyVec<usize> where F: FnMut(&T) -> bool {
        let mut indices = MyVec::new();

        for index in 0..self.len {
            if predicate(unsafe { &*self.ptr.add(index) }) {
                indices.push(index);
            }
        }

        indices
    }

    pub fn clear(&mut self) {
        while self.len > 0 {
            self.len -= 1;
            unsafe {
                ptr::drop_in_place(self.ptr.add(self.len));
            }
        }
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.ptr, self.len) }
    }
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.ptr, self.len) }
    }

    pub fn first(&self) -> Option<&T> {
        if self.len > 0 {
            unsafe { Some(&*self.ptr) }
        } else {
            None
        }
    }

    pub fn first_mut(&mut self) -> Option<&mut T> {
        if self.len > 0 {
            unsafe { Some(&mut *self.ptr) }
        } else {
            None
        }
    }

    pub fn last(&self) -> Option<&T> {
        if self.len > 0 {
            unsafe { Some(&*self.ptr.add(self.len - 1)) }
        } else {
            None
        }
    }
    pub fn swap(&mut self, index1: usize, index2: usize) {
        assert!(index1 < self.len, "index1 out of bounds");
        assert!(index2 < self.len, "index2 out of bounds");

        unsafe {
            std::ptr::swap(self.ptr.add(index1), self.ptr.add(index2));
        }
    }

    pub fn last_mut(&mut self) -> Option<&mut T> {
        if self.len > 0 {
            unsafe { Some(&mut *self.ptr.add(self.len - 1)) }
        } else {
            None
        }
    }
    pub fn swap_remove(&mut self, index: usize) -> Option<T> {
        if index >= self.len {
            return None;
        }

        self.swap(index, self.len - 1);
        self.remove(self.len - 1).into()
    }
    pub fn truncate(&mut self, new_len: usize) {
        if new_len >= self.len {
            return;
        }

        while self.len > new_len {
            self.len -= 1;
            unsafe {
                std::ptr::drop_in_place(self.ptr.add(self.len));
            }
        }
    }
    pub fn retain<F>(&mut self, mut predicate: F) where F: FnMut(&T) -> bool {
        let mut i = 0;
        while i < self.len {
            if !predicate(unsafe { &*self.ptr.add(i) }) {
                self.remove(i);
            } else {
                i += 1;
            }
        }
    }

    pub fn retain_mut<F>(&mut self, mut predicate: F) where F: FnMut(&mut T) -> bool {
        let mut i = 0;
        while i < self.len {
            if !predicate(unsafe { &mut *self.ptr.add(i) }) {
                self.remove(i);
            } else {
                i += 1;
            }
        }
    }
    pub fn remove_all(&mut self, value: T) where T: PartialEq {
        let mut i = 0;
        while i < self.len {
            if (unsafe { &*self.ptr.add(i) }) == &value {
                self.remove(i);
            } else {
                i += 1;
            }
        }
    }
    pub fn remove_where<F>(&mut self, mut predicate: F) where F: FnMut(&T) -> bool {
        let mut i = 0;
        while i < self.len {
            if predicate(unsafe { &*self.ptr.add(i) }) {
                self.remove(i);
            } else {
                i += 1;
            }
        }
    }
    pub fn remove_where_mut<F>(&mut self, mut predicate: F) where F: FnMut(&mut T) -> bool {
        let mut i = 0;
        while i < self.len {
            if predicate(unsafe { &mut *self.ptr.add(i) }) {
                self.remove(i);
            } else {
                i += 1;
            }
        }
    }
    pub fn remove_duplicates(&mut self) where T: PartialEq {
        let mut i = 0;
        while i < self.len {
            let mut j = i + 1;
            while j < self.len {
                if (unsafe { &*self.ptr.add(i) }) == (unsafe { &*self.ptr.add(j) }) {
                    self.remove(j);
                } else {
                    j += 1;
                }
            }
            i += 1;
        }
    }
    pub fn dedup(&mut self) where T: PartialEq {
        if self.len < 2 {
            return;
        }

        let mut i = 1;

        while i < self.len {
            let duplicate = unsafe { &*self.ptr.add(i - 1) == &*self.ptr.add(i) };

            if duplicate {
                self.remove(i);
            } else {
                i += 1;
            }
        }
    }
    pub fn reserve(&mut self, additional: usize) {
        let required_capacity = self.len + additional;

        if required_capacity > self.capacity {
            let new_capacity = required_capacity.max(self.capacity * 2);
            let new_layout = Self::layout(new_capacity);

            let new_ptr = unsafe {
                let allocation = alloc(new_layout) as *mut T;

                if allocation.is_null() {
                    handle_alloc_error(new_layout);
                }

                if self.len > 0 {
                    ptr::copy_nonoverlapping(self.ptr, allocation, self.len);
                }

                if self.capacity > 0 {
                    dealloc(self.ptr as *mut u8, Self::layout(self.capacity));
                }

                allocation
            };

            self.ptr = new_ptr;
            self.capacity = new_capacity;
        }
    }
    pub fn shrink_to_fit(&mut self) {
        if self.len < self.capacity {
            let new_layout = Self::layout(self.len);

            let new_ptr = unsafe {
                let allocation = alloc(new_layout) as *mut T;

                if allocation.is_null() {
                    handle_alloc_error(new_layout);
                }

                if self.len > 0 {
                    ptr::copy_nonoverlapping(self.ptr, allocation, self.len);
                }

                if self.capacity > 0 {
                    dealloc(self.ptr as *mut u8, Self::layout(self.capacity));
                }

                allocation
            };

            self.ptr = new_ptr;
            self.capacity = self.len;
        }
    }
    pub fn reserve_exact(&mut self, additional: usize) {
        let required_capacity = self.len + additional;

        if required_capacity > self.capacity {
            let new_layout = Self::layout(required_capacity);

            let new_ptr = unsafe {
                let allocation = alloc(new_layout) as *mut T;

                if allocation.is_null() {
                    handle_alloc_error(new_layout);
                }

                if self.len > 0 {
                    ptr::copy_nonoverlapping(self.ptr, allocation, self.len);
                }

                if self.capacity > 0 {
                    dealloc(self.ptr as *mut u8, Self::layout(self.capacity));
                }

                allocation
            };

            self.ptr = new_ptr;
            self.capacity = required_capacity;
        }
    }
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), &'static str> {
        let required_capacity = self.len + additional;

        if required_capacity > self.capacity {
            let new_capacity = required_capacity.max(self.capacity * 2);
            let new_layout = Self::layout(new_capacity);

            let new_ptr = unsafe {
                let allocation = alloc(new_layout) as *mut T;

                if allocation.is_null() {
                    return Err("Memory allocation failed");
                }

                if self.len > 0 {
                    ptr::copy_nonoverlapping(self.ptr, allocation, self.len);
                }

                if self.capacity > 0 {
                    dealloc(self.ptr as *mut u8, Self::layout(self.capacity));
                }

                allocation
            };

            self.ptr = new_ptr;
            self.capacity = new_capacity;
        }

        Ok(())
    }
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), &'static str> {
        let required_capacity = self.len + additional;

        if required_capacity > self.capacity {
            let new_layout = Self::layout(required_capacity);

            let new_ptr = unsafe {
                let allocation = alloc(new_layout) as *mut T;

                if allocation.is_null() {
                    return Err("Memory allocation failed");
                }

                if self.len > 0 {
                    ptr::copy_nonoverlapping(self.ptr, allocation, self.len);
                }

                if self.capacity > 0 {
                    dealloc(self.ptr as *mut u8, Self::layout(self.capacity));
                }

                allocation
            };

            self.ptr = new_ptr;
            self.capacity = required_capacity;
        }

        Ok(())
    }
    pub fn insert(&mut self, index: usize, value: T) {
        assert!(index <= self.len, "index out of bounds");

        if self.len == self.capacity {
            self.grow();
        }

        unsafe {
            ptr::copy(self.ptr.add(index), self.ptr.add(index + 1), self.len - index);
            self.ptr.add(index).write(value);
        }

        self.len += 1;
    }
    pub fn append(&mut self, other: &mut MyVec<T>) {
        let total_len = self.len + other.len;

        if total_len > self.capacity {
            self.reserve(total_len - self.len);
        }

        unsafe {
            ptr::copy_nonoverlapping(other.ptr, self.ptr.add(self.len), other.len);
        }

        self.len = total_len;
        other.clear();
    }
    pub fn extend(&mut self, iter: impl IntoIterator<Item = T>) {
        for item in iter {
            self.push(item);
        }
    }
    pub fn extend_from_slice(&mut self, slice: &[T]) where T: Clone {
        self.reserve(slice.len());

        for item in slice {
            self.push(item.clone());
        }
    }
    pub fn resize(&mut self, new_len: usize, value: T) where T: Clone {
        if new_len > self.len {
            self.reserve(new_len - self.len);
            for _ in self.len..new_len {
                self.push(value.clone());
            }
        } else {
            while self.len > new_len {
                self.pop();
            }
        }
    }
    pub fn resize_with<F>(&mut self, new_len: usize, mut f: F) where F: FnMut() -> T {
        if new_len > self.len {
            self.reserve(new_len - self.len);
            for _ in self.len..new_len {
                self.push(f());
            }
        } else {
            while self.len > new_len {
                self.pop();
            }
        }
    }
    pub fn split_off(&mut self, at: usize) -> MyVec<T> {
        assert!(at <= self.len, "index out of bounds");

        let mut new_vec = MyVec::new();
        new_vec.reserve(self.len - at);

        unsafe {
            ptr::copy_nonoverlapping(self.ptr.add(at), new_vec.ptr, self.len - at);
        }

        new_vec.len = self.len - at;
        self.len = at;

        new_vec
    }

    pub fn iter(&self) -> MyVecIter<'_, T> {
        MyVecIter {
            vec: self,
            index: 0,
        }
    }

    pub fn iter_mut(&mut self) -> MyVecIterMut<'_, T> {
        MyVecIterMut {
            ptr: self.ptr,
            len: self.len,
            index: 0,
            marker: std::marker::PhantomData,
        }
    }
    pub fn reverse(&mut self) {
        let mut left = 0;
        let mut right = self.len - 1;

        while left < right {
            self.swap(left, right);
            left += 1;
            right -= 1;
        }
    }

    pub fn sort(&mut self) where T: Ord {
        let slice = self.as_mut_slice();
        slice.sort();
    }
    pub fn sort_by<F>(&mut self, compare: F) where F: FnMut(&T, &T) -> std::cmp::Ordering {
        let slice = self.as_mut_slice();
        slice.sort_by(compare);
    }
    pub fn begin(&self) -> *const T {
        self.ptr
    }
    pub fn end(&self) -> *const T {
        unsafe { self.ptr.add(self.len) }
    }
    pub fn sort_range(&mut self, start: usize, end: usize) where T: Ord {
        assert!(start <= end, "invalid range");
        assert!(end <= self.len, "index out of bounds");

        self.as_mut_slice()[start..end].sort();
    }
    pub fn binary_search(&self, value: &T) -> Result<usize, usize> where T: Ord {
        self.as_slice().binary_search(value)
    }
    pub fn rotate_left(&mut self, mid: usize) {
        assert!(mid <= self.len, "mid out of bounds");

        self.as_mut_slice().rotate_left(mid);
    }
    pub fn rotate_right(&mut self, mid: usize) {
        assert!(mid <= self.len, "mid out of bounds");

        self.as_mut_slice().rotate_right(mid);
    }
}

pub struct MyVecIter<'a, T> {
    vec: &'a MyVec<T>,
    index: usize,
}
impl<'a, T> Iterator for MyVecIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.vec.len {
            return None;
        }

        let item = unsafe { &*self.vec.ptr.add(self.index) };

        self.index += 1;
        Some(item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.vec.len - self.index;
        (remaining, Some(remaining))
    }
}

impl<T> ExactSizeIterator for MyVecIter<'_, T> {}

impl<T> FusedIterator for MyVecIter<'_, T> {}

pub struct MyVecIterMut<'a, T> {
    ptr: *mut T,
    len: usize,
    index: usize,
    marker: std::marker::PhantomData<&'a mut T>,
}

impl<'a, T> Iterator for MyVecIterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            return None;
        }

        let item = unsafe { &mut *self.ptr.add(self.index) };

        self.index += 1;
        Some(item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.len - self.index;
        (remaining, Some(remaining))
    }
}

impl<T> ExactSizeIterator for MyVecIterMut<'_, T> {}

impl<T> FusedIterator for MyVecIterMut<'_, T> {}

impl<'a, T> IntoIterator for &'a MyVec<T> {
    type Item = &'a T;
    type IntoIter = MyVecIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut MyVec<T> {
    type Item = &'a mut T;
    type IntoIter = MyVecIterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

pub struct MyVecIntoIter<T> {
    vec: MyVec<T>,
    index: usize,
}

impl<T> Iterator for MyVecIntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.vec.len {
            return None;
        }

        let item = unsafe { self.vec.ptr.add(self.index).read() };

        self.index += 1;
        Some(item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.vec.len - self.index;
        (remaining, Some(remaining))
    }
}

impl<T> ExactSizeIterator for MyVecIntoIter<T> {}

impl<T> FusedIterator for MyVecIntoIter<T> {}

impl<T> IntoIterator for MyVec<T> {
    type Item = T;
    type IntoIter = MyVecIntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        let vec = unsafe {
            // Transfer ownership of the vector into the iterator.
            // Prevent the original vector from being dropped here.
            std::ptr::read(&self)
        };

        std::mem::forget(self);

        MyVecIntoIter {
            vec,
            index: 0,
        }
    }
}
impl<T> Default for MyVec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for MyVec<T> {
    fn drop(&mut self) {
        self.clear();

        if self.capacity > 0 && std::mem::size_of::<T>() != 0 {
            unsafe {
                dealloc(self.ptr as *mut u8, Self::layout(self.capacity));
            }
        }
    }
}

/// Implementing Index trait for MyVec
impl<T> Index<usize> for MyVec<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.len, "index out of bounds");

        unsafe { &*self.ptr.add(index) }
    }
}

impl<T> IndexMut<usize> for MyVec<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(index < self.len, "index out of bounds");

        unsafe { &mut *self.ptr.add(index) }
    }
}

impl<T> Debug for MyVec<T> where T: Debug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}
impl<T> Clone for MyVec<T> where T: Clone {
    fn clone(&self) -> Self {
        let mut new_vec = MyVec::new();
        new_vec.reserve(self.len);

        for item in self.iter() {
            new_vec.push(item.clone());
        }

        new_vec
    }
}

impl<T> PartialEq for MyVec<T> where T: PartialEq {
    fn eq(&self, other: &Self) -> bool {
        if self.len != other.len {
            return false;
        }

        for i in 0..self.len {
            if self[i] != other[i] {
                return false;
            }
        }

        true
    }
}

impl<T> Eq for MyVec<T> where T: Eq {}

impl<T> Extend<T> for MyVec<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.push(item);
        }
    }
}

impl<T> FromIterator<T> for MyVec<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut vec = MyVec::new();
        vec.extend(iter);
        vec
    }
}

impl<T> Deref for MyVec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T> DerefMut for MyVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}
