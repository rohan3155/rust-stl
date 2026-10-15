use std::alloc::{ alloc, dealloc, handle_alloc_error, Layout };
use std::ptr;
use std::slice;

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
            if unsafe { &*self.ptr.add(i) } == &value {
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
