use std::alloc::{ alloc, dealloc, handle_alloc_error, Layout };
use std::ptr;
use std::ptr::NonNull;
use std::slice;
use std::iter::FusedIterator;
use std::ops::{ Deref, DerefMut, Index, IndexMut };
use std::fmt::Debug;
use std::iter::{ Extend, FromIterator };
use std::cmp::{ PartialEq, Eq };
pub struct MyVec<T> {
    /// Pointer to the start of the allocation.
    ///
    /// Invariants:
    /// * The pointer is always non-null and aligned for `T`. Empty vectors that
    ///   never allocated use [`NonNull::dangling`], which satisfies both.
    /// * When `capacity > 0`, the pointer is the start of a heap block obtained
    ///   from [`alloc`] with `Layout::array::<T>(capacity)`, and the first `len`
    ///   elements of that block are initialized.
    /// * When `capacity == 0`, no allocation exists and the pointer is dangling.
    ptr: NonNull<T>,
    len: usize,
    capacity: usize,
}

impl<T> MyVec<T> {
    pub fn new() -> Self {
        Self {
            // A zero-capacity vector owns no allocation, so it stores a
            // dangling but non-null, correctly aligned pointer. Using a null
            // pointer here would make `NonNull`'s (and slice's) alignment and
            // non-null requirements unsatisfiable.
            ptr: NonNull::dangling(),
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
            // SAFETY: `index < len`, and the first `len` elements of the
            // allocation are initialized, so this pointer addresses a live `T`.
            unsafe { Some(&*self.ptr.as_ptr().add(index)) }
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.len {
            // SAFETY: `index < len`, and `&mut self` guarantees exclusive
            // access, so this pointer addresses a uniquely borrowed live `T`.
            unsafe { Some(&mut *self.ptr.as_ptr().add(index)) }
        } else {
            None
        }
    }

    /// Returns `true` when `T` occupies no memory.
    ///
    /// Zero-sized types are special-cased everywhere in this file: they never
    /// own a heap allocation (`alloc`/`dealloc` with a zero-sized layout is
    /// undefined behaviour), and their capacity is `usize::MAX` so that no
    /// amount of `push`es can trigger a reallocation.
    #[inline]
    fn is_zst() -> bool {
        std::mem::size_of::<T>() == 0
    }

    /// Computes the layout for `capacity` elements, panicking on overflow.
    fn layout(capacity: usize) -> Layout {
        Layout::array::<T>(capacity).expect("capacity overflow")
    }

    /// Computes the layout for `capacity` elements.
    ///
    /// Returns an error when `capacity * size_of::<T>()` overflows `isize`,
    /// which is the point at which a slice of that many elements could not
    /// exist and the allocation request is for a guaranteed-impossible size.
    fn try_layout(capacity: usize) -> Result<Layout, &'static str> {
        Layout::array::<T>(capacity).map_err(|_| "capacity overflow")
    }

    /// Allocates uninitialized storage for `capacity` elements.
    ///
    /// # Panics
    ///
    /// Aborts the process if the allocation fails.
    ///
    /// # Safety
    ///
    /// For `capacity == 0` or a zero-sized `T` the returned pointer must not be
    /// used to access memory; the caller is responsible for treating it as
    /// dangling. For every other case it is a valid write target for
    /// `capacity` elements.
    unsafe fn allocate(capacity: usize) -> NonNull<T> {
        debug_assert!(capacity > 0 && !Self::is_zst());

        let layout = Self::layout(capacity);

        // SAFETY: `capacity > 0` and `T` is not zero-sized, so `layout` has a
        // non-zero size, which is `alloc`'s precondition.
        let allocation = unsafe { alloc(layout) };

        if allocation.is_null() {
            handle_alloc_error(layout);
        }

        // SAFETY: just checked that `allocation` is non-null, and `alloc`
        // returns a pointer aligned for `layout`, hence for `T`.
        unsafe { NonNull::new_unchecked(allocation as *mut T) }
    }

    /// Releases the allocation owned by this vector.
    ///
    /// No-op for zero-sized `T` and for vectors that never allocated, which is
    /// what keeps the dangling pointer from being handed to `dealloc`.
    fn deallocate(ptr: NonNull<T>, capacity: usize) {
        if capacity == 0 || Self::is_zst() {
            return;
        }

        // SAFETY: `capacity > 0`, `T` is not zero-sized, and by this type's
        // invariant `ptr` is the base of a live allocation created with exactly
        // `Layout::array::<T>(capacity)`. Using the same layout is required by
        // `dealloc` and is what `allocate` / `reserve` guarantee.
        unsafe {
            dealloc(ptr.as_ptr() as *mut u8, Self::layout(capacity));
        }
    }

    fn grow(&mut self) {
        if Self::is_zst() {
            // ZSTs are stored inline in the pointer, so there is nothing to
            // allocate; report an effectively infinite capacity instead.
            self.capacity = usize::MAX;
            return;
        }

        // `checked_add`/`checked_mul` keep a pathological request from wrapping
        // around to a small capacity and silently under-allocating.
        let required = self
            .len
            .checked_add(1)
            .expect("capacity overflow: vector length exceeded");

        let new_capacity = if self.capacity == 0 {
            4
        } else {
            self.capacity
                .checked_mul(2)
                .expect("capacity overflow: vector capacity exceeded")
        };

        let new_capacity = new_capacity.max(required);

        // SAFETY: `new_capacity > 0` and `T` is not zero-sized here.
        let new_ptr = unsafe { Self::allocate(new_capacity) };

        // SAFETY: the destination is a fresh, unaliased allocation valid for
        // `new_capacity >= len` elements, so the two regions cannot overlap.
        // Only `len` elements are read, all of which are initialized.
        unsafe {
            ptr::copy_nonoverlapping(self.ptr.as_ptr(), new_ptr.as_ptr(), self.len);
        }

        Self::deallocate(self.ptr, self.capacity);

        self.ptr = new_ptr;
        self.capacity = new_capacity;
    }

    pub fn push(&mut self, value: T) {
        if self.len == self.capacity {
            self.grow();
        }

        // SAFETY: `len < capacity` after the growth check, so slot `len` is
        // inside the allocation (or, for a ZST, a valid dangling slot) and is
        // currently uninitialized.
        unsafe {
            self.ptr.as_ptr().add(self.len).write(value);
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
        (0..self.len)
            .find(|&index| predicate(unsafe { &*self.ptr.as_ptr().add(index) }))
    }

    pub fn count_matches<F>(&self, mut predicate: F) -> usize where F: FnMut(&T) -> bool {
        (0..self.len)
            .filter(|&index| predicate(unsafe { &*self.ptr.as_ptr().add(index) }))
            .count()
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

        // SAFETY: `index < len`, so slot `index` holds an initialized value.
        // `read` moves it out; the following overlapping copy is required
        // because the source and destination ranges overlap at `index`.
        unsafe {
            let element = self.ptr.as_ptr().add(index).read();
            let count = self.len - index - 1;
            ptr::copy(
                self.ptr.as_ptr().add(index + 1),
                self.ptr.as_ptr().add(index),
                count,
            );
            self.len -= 1;
            element
        }
    }

    pub fn indices_of<F>(&self, mut predicate: F) -> MyVec<usize> where F: FnMut(&T) -> bool {
        let mut indices = MyVec::new();

        for index in 0..self.len {
            // SAFETY: `index < len` and those slots are initialized.
            if predicate(unsafe { &*self.ptr.as_ptr().add(index) }) {
                indices.push(index);
            }
        }

        indices
    }

    pub fn clear(&mut self) {
        // Shorten first, then drop: this mirrors `ptr::drop_in_place` on each
        // element and stays correct even if a `Drop` impl observes the vector.
        while self.len > 0 {
            self.len -= 1;
            // SAFETY: `len < old_len`, so this slot still holds a live value
            // that has not been dropped yet.
            unsafe {
                ptr::drop_in_place(self.ptr.as_ptr().add(self.len));
            }
        }
    }

    pub fn as_slice(&self) -> &[T] {
        if self.len == 0 {
            // SAFETY: an empty slice only requires a non-null, aligned pointer;
            // `dangling()` provides one without dereferencing it.
            return unsafe { slice::from_raw_parts(NonNull::<T>::dangling().as_ptr(), 0) };
        }

        // SAFETY: the allocation (or, for a ZST, the dangling pointer) is valid
        // for `len` elements and all of them are initialized.
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        if self.len == 0 {
            // SAFETY: as in `as_slice`, and `&mut self` gives unique access.
            return unsafe { slice::from_raw_parts_mut(NonNull::<T>::dangling().as_ptr(), 0) };
        }

        // SAFETY: the allocation is valid for `len` initialized elements and
        // `&mut self` guarantees they are not aliased.
        unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }

    pub fn first(&self) -> Option<&T> {
        if self.len > 0 {
            // SAFETY: `len > 0` means slot 0 is initialized.
            unsafe { Some(&*self.ptr.as_ptr()) }
        } else {
            None
        }
    }

    pub fn first_mut(&mut self) -> Option<&mut T> {
        if self.len > 0 {
            // SAFETY: `len > 0` and `&mut self` gives unique access.
            unsafe { Some(&mut *self.ptr.as_ptr()) }
        } else {
            None
        }
    }

    pub fn last(&self) -> Option<&T> {
        if self.len > 0 {
            // SAFETY: `len > 0`, so slot `len - 1` is inside the allocation
            // and initialized.
            unsafe { Some(&*self.ptr.as_ptr().add(self.len - 1)) }
        } else {
            None
        }
    }

    pub fn swap(&mut self, index1: usize, index2: usize) {
        assert!(index1 < self.len, "index1 out of bounds");
        assert!(index2 < self.len, "index2 out of bounds");

        // SAFETY: both indices are in bounds, so both slots are initialized.
        // `ptr::swap` is valid for identical values and overlapping ranges.
        unsafe {
            std::ptr::swap(self.ptr.as_ptr().add(index1), self.ptr.as_ptr().add(index2));
        }
    }

    pub fn last_mut(&mut self) -> Option<&mut T> {
        if self.len > 0 {
            // SAFETY: `len > 0`, slot `len - 1` is initialized, and `&mut self`
            // guarantees unique access to it.
            unsafe { Some(&mut *self.ptr.as_ptr().add(self.len - 1)) }
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

        // Elements in `new_len..len` are dropped in place; nothing is moved, so
        // an interrupted `Drop` cannot leave the remaining prefix inconsistent.
        while self.len > new_len {
            self.len -= 1;
            // SAFETY: `len < old_len`, so this slot holds an un-dropped value.
            unsafe {
                std::ptr::drop_in_place(self.ptr.as_ptr().add(self.len));
            }
        }
    }

    pub fn retain<F>(&mut self, mut predicate: F) where F: FnMut(&T) -> bool {
        let mut i = 0;
        while i < self.len {
            // SAFETY: `i < len` and slot `i` is initialized.
            if !predicate(unsafe { &*self.ptr.as_ptr().add(i) }) {
                // `remove` fully updates `len` before the next predicate call,
                // so a panicking predicate cannot observe a half-removed
                // element.
                self.remove(i);
            } else {
                i += 1;
            }
        }
    }

    pub fn retain_mut<F>(&mut self, mut predicate: F) where F: FnMut(&mut T) -> bool {
        let mut i = 0;
        while i < self.len {
            // SAFETY: `i < len`, slot `i` is initialized, and the returned
            // borrow ends before the next iteration so no two `&mut` overlap.
            if !predicate(unsafe { &mut *self.ptr.as_ptr().add(i) }) {
                self.remove(i);
            } else {
                i += 1;
            }
        }
    }

    pub fn remove_all(&mut self, value: T) where T: PartialEq {
        let mut i = 0;
        while i < self.len {
            // SAFETY: `i < len` and slot `i` is initialized.
            if (unsafe { &*self.ptr.as_ptr().add(i) }) == &value {
                self.remove(i);
            } else {
                i += 1;
            }
        }
    }

    pub fn remove_where<F>(&mut self, mut predicate: F) where F: FnMut(&T) -> bool {
        let mut i = 0;
        while i < self.len {
            // SAFETY: `i < len` and slot `i` is initialized.
            if predicate(unsafe { &*self.ptr.as_ptr().add(i) }) {
                self.remove(i);
            } else {
                i += 1;
            }
        }
    }

    pub fn remove_where_mut<F>(&mut self, mut predicate: F) where F: FnMut(&mut T) -> bool {
        let mut i = 0;
        while i < self.len {
            // SAFETY: `i < len`, slot `i` is initialized, and each borrow ends
            // before the next one begins.
            if predicate(unsafe { &mut *self.ptr.as_ptr().add(i) }) {
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
                // SAFETY: `i < j < len` and both slots are initialized.
                let equal = unsafe { *self.ptr.as_ptr().add(i) == *self.ptr.as_ptr().add(j) };
                if equal {
                    // Removing shifts the tail left, so `j` now points at the
                    // element that was at `j + 1`; it is deliberately not
                    // advanced.
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
            // SAFETY: `i >= 1` and `i < len`, so slots `i - 1` and `i` are
            // distinct, initialized, and simultaneously borrowed immutably.
            let duplicate = unsafe { *self.ptr.as_ptr().add(i - 1) == *self.ptr.as_ptr().add(i) };

            if duplicate {
                self.remove(i);
            } else {
                i += 1;
            }
        }
    }
    /// Reallocates storage so that `new_capacity` elements fit, moving the
    /// currently initialized `len` elements into the new block.
    ///
    /// On success the old block is released. On failure the vector is left
    /// completely untouched: same pointer, length, capacity and elements.
    fn try_reallocate(&mut self, new_capacity: usize) -> Result<(), &'static str> {
        if Self::is_zst() {
            // Nothing is heap-allocated for a ZST, so "reserving" is just
            // recording an unbounded capacity.
            self.capacity = usize::MAX;
            return Ok(());
        }

        if new_capacity <= self.capacity {
            return Ok(());
        }

        let new_layout = Self::try_layout(new_capacity)?;
        let new_ptr = unsafe { alloc(new_layout) } as *mut T;

        if new_ptr.is_null() {
            return Err("memory allocation failed");
        }

        // SAFETY: `new_ptr` is a live, uniquely owned allocation of
        // `new_capacity >= len` elements, so it cannot overlap the old block.
        // The first `len` slots of the old block are all initialized.
        unsafe {
            ptr::copy_nonoverlapping(self.ptr.as_ptr(), new_ptr, self.len);
        }

        // SAFETY: the old pointer was produced by `allocate`/a previous
        // `try_reallocate` with exactly `Layout::array::<T>(self.capacity)`,
        // which is what `deallocate` frees.
        unsafe {
            dealloc(self.ptr.as_ptr() as *mut u8, Self::layout(self.capacity));
        }

        // SAFETY: the null check above guarantees `new_ptr` is non-null, and
        // `alloc` returns a pointer aligned for `Layout::array::<T>`, hence `T`.
        self.ptr = unsafe { NonNull::new_unchecked(new_ptr) };
        self.capacity = new_capacity;

        Ok(())
    }

    /// Like `try_reallocate`, but aborts the process on failure instead of
    /// returning an error.
    fn reallocate(&mut self, new_capacity: usize) {
        if Self::is_zst() {
            self.capacity = usize::MAX;
            return;
        }

        if new_capacity <= self.capacity {
            return;
        }

        // SAFETY: `new_capacity > self.capacity >= 0`, and `T` is not a ZST, so
        // the layout has a non-zero size and `allocate` returns a live block.
        let new_ptr = unsafe { Self::allocate(new_capacity) };

        // SAFETY: as in `try_reallocate`, the fresh block cannot overlap the old
        // one and the first `len` slots are initialized.
        unsafe {
            ptr::copy_nonoverlapping(self.ptr.as_ptr(), new_ptr.as_ptr(), self.len);
        }

        Self::deallocate(self.ptr, self.capacity);

        self.ptr = new_ptr;
        self.capacity = new_capacity;
    }

    /// Validates `len + additional` and returns the minimum required capacity.
    fn required_capacity(&self, additional: usize) -> Result<usize, &'static str> {
        self.len
            .checked_add(additional)
            .ok_or("capacity overflow")
    }

    /// The amortized target capacity used by `reserve` / `try_reserve`.
    fn amortized_capacity(&self, required: usize) -> usize {
        // Doubling keeps `push` amortized O(1). `saturating_mul` avoids
        // wrapping here: an over-large target is rejected later by the layout
        // check rather than silently rounded down.
        required.max(self.capacity.saturating_mul(2)).max(4)
    }

    pub fn reserve(&mut self, additional: usize) {
        let required = self
            .required_capacity(additional)
            .expect("capacity overflow: vector length exceeded");

        self.reallocate(self.amortized_capacity(required));
    }

    pub fn shrink_to_fit(&mut self) {
        if Self::is_zst() {
            // A ZST owns no allocation, so there is nothing to shrink.
            return;
        }

        if self.len == self.capacity {
            return;
        }

        if self.len == 0 {
            // Growing is not required here: releasing the block outright keeps
            // us from ever calling `alloc` with a zero-sized layout, which is
            // undefined behaviour.
            Self::deallocate(self.ptr, self.capacity);
            self.ptr = NonNull::dangling();
            self.capacity = 0;
            return;
        }

        // `len > 0`, so `allocate` is called with a non-zero size.
        let new_ptr = unsafe { Self::allocate(self.len) };

        // SAFETY: the destination is a fresh allocation of exactly `len`
        // elements, so the regions cannot overlap; all `len` elements are
        // initialized.
        unsafe {
            ptr::copy_nonoverlapping(self.ptr.as_ptr(), new_ptr.as_ptr(), self.len);
        }

        Self::deallocate(self.ptr, self.capacity);

        self.ptr = new_ptr;
        self.capacity = self.len;
    }

    pub fn reserve_exact(&mut self, additional: usize) {
        let required = self
            .required_capacity(additional)
            .expect("capacity overflow: vector length exceeded");

        self.reallocate(required);
    }

    pub fn try_reserve(&mut self, additional: usize) -> Result<(), &'static str> {
        let required = self.required_capacity(additional)?;
        let target = self.amortized_capacity(required);

        self.try_reallocate(target)
    }

    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), &'static str> {
        let required = self.required_capacity(additional)?;

        self.try_reallocate(required)
    }

    pub fn insert(&mut self, index: usize, value: T) {
        assert!(index <= self.len, "index out of bounds");

        if self.len == self.capacity {
            self.grow();
        }

        // SAFETY: after the growth check `len < capacity`, so both
        // `ptr.add(index)` and `ptr.add(index + 1)` are within the allocation
        // (`index <= len`), and `ptr::copy` is required because the ranges
        // overlap. Slot `index` is vacated by the shift and is then written.
        unsafe {
            ptr::copy(
                self.ptr.as_ptr().add(index),
                self.ptr.as_ptr().add(index + 1),
                self.len - index,
            );
            self.ptr.as_ptr().add(index).write(value);
        }

        self.len += 1;
    }

    pub fn append(&mut self, other: &mut MyVec<T>) {
        let other_len = other.len;

        if other_len == 0 {
            return;
        }

        let total_len = self
            .len
            .checked_add(other_len)
            .expect("capacity overflow: vector length exceeded");

        if total_len > self.capacity {
            self.reserve(total_len - self.len);
        }

        // SAFETY: `reserve` guaranteed `total_len <= self.capacity`, so slots
        // `self.len..total_len` exist. `other` is exclusively borrowed, so its
        // allocation cannot alias `self`'s, making the regions disjoint (the
        // `len == 0` case leaves two zero-sized ranges, which is fine).
        unsafe {
            ptr::copy_nonoverlapping(other.ptr.as_ptr(), self.ptr.as_ptr().add(self.len), other_len);
        }

        // Ownership of the copied values has moved to `self`. `other` must be
        // emptied *without* running their destructors: `copy_nonoverlapping`
        // duplicated the values, so clearing `other` here would drop every
        // appended element twice (once via `self` and once via `other`).
        // Truncating the length is enough; `other` keeps its now-unused
        // capacity.
        self.len = total_len;
        other.len = 0;
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

        // SAFETY: `at <= len` and `new_vec` has capacity for `len - at`
        // elements, so the destination is large enough. The two vectors own
        // distinct allocations, so the regions cannot overlap. On return
        // `self.len` is shortened first, so each element is owned exactly once.
        unsafe {
            ptr::copy_nonoverlapping(
                self.ptr.as_ptr().add(at),
                new_vec.ptr.as_ptr(),
                self.len - at,
            );
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
            ptr: self.ptr.as_ptr(),
            len: self.len,
            index: 0,
            marker: std::marker::PhantomData,
        }
    }

    pub fn reverse(&mut self) {
        // `len - 1` would underflow on an empty vector, so guard first. This is
        // also the natural "nothing to do" case.
        if self.len < 2 {
            return;
        }

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
        // Never null: an empty, never-allocated vector yields a dangling but
        // aligned pointer, which is what pointer arithmetic and comparisons
        // expect.
        self.ptr.as_ptr()
    }

    pub fn end(&self) -> *const T {
        // SAFETY: `len <= capacity`, so one-past-the-end is in bounds. For a
        // ZST the pointer never moves, which is also correct for `len` elements.
        unsafe { self.ptr.as_ptr().add(self.len) }
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
    pub fn split_at(&self, mid: usize) -> (&[T], &[T]) {
        self.as_slice().split_at(mid)
    }
    pub fn windows(&self, size: usize) -> std::slice::Windows<'_, T> {
        self.as_slice().windows(size)
    }
    pub fn chunks(&self, size: usize) -> std::slice::Chunks<'_, T> {
        self.as_slice().chunks(size)
    }
    pub fn binary_search_by<F>(&self, compare: F) -> Result<usize, usize>
        where F: FnMut(&T) -> std::cmp::Ordering
    {
        self.as_slice().binary_search_by(compare)
    }

    pub fn binary_search_by_key<B, F>(&self, key: &B,  f: F) -> Result<usize, usize>
        where B: Ord, F: FnMut(&T) -> B
    {
        self.as_slice().binary_search_by_key(key, f)
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

        let item = unsafe { &*self.vec.ptr.as_ptr().add(self.index) };

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

/// Owning iterator over a [`MyVec`].
///
/// Created by [`MyVec::into_iter`] (or, equivalently, `for value in vec`).
pub struct MyVecIntoIter<T> {
    /// Base of the allocation being drained. Still owned by this iterator.
    ptr: NonNull<T>,
    /// The half-open range of the allocation that is still initialized and has
    /// not been yielded yet.
    ///
    /// This is the single source of truth for the iterator's state: elements
    /// before `remaining.start` have been moved out and must not be touched
    /// again, elements from `remaining.end` onwards were already moved out by
    /// [`DoubleEndedIterator::next_back`], and `remaining` itself still holds
    /// live values that this iterator owns.
    remaining: std::ops::Range<usize>,
    /// Capacity of the allocation, needed to release it with the exact layout
    /// it was created with.
    capacity: usize,
    /// Marks the allocation as owned, so dropping the iterator knows it must
    /// release it, and keeps `T`'s variance/drop-check behaviour correct.
    marker: std::marker::PhantomData<T>,
}

impl<T> Iterator for MyVecIntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.start >= self.remaining.end {
            return None;
        }

        let index = self.remaining.start;
        // Advance first: if the value's destructor runs later and panics, the
        // element must already be marked as consumed, or it would be dropped a
        // second time by `Drop`.
        self.remaining.start += 1;

        // SAFETY: `index < remaining.end <= capacity`, so the slot is inside the
        // allocation, and everything in `remaining` is still initialized.
        // Advancing the range first means no other method will read the slot
        // again, so producing the value by value is sound.
        Some(unsafe { self.ptr.as_ptr().add(index).read() })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.remaining.end - self.remaining.start;
        (remaining, Some(remaining))
    }
}

impl<T> DoubleEndedIterator for MyVecIntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.remaining.start >= self.remaining.end {
            return None;
        }

        self.remaining.end -= 1;
        let index = self.remaining.end;

        // SAFETY: `remaining.start <= index < remaining.end <= capacity`, so the
        // slot is inside the allocation and still initialized. The range was
        // shrunk before the read so the slot is never yielded twice.
        Some(unsafe { self.ptr.as_ptr().add(index).read() })
    }
}

impl<T> ExactSizeIterator for MyVecIntoIter<T> {}

impl<T> FusedIterator for MyVecIntoIter<T> {}

impl<T> Drop for MyVecIntoIter<T> {
    fn drop(&mut self) {
        // SAFETY: `remaining` is exactly the set of live, owned, not-yet-yielded
        // elements, and it is always a sub-range of the allocation. Dropping
        // them in place leaves no element un-dropped and none dropped twice;
        // elements outside the range were already moved out of the iterator.
        unsafe {
            ptr::drop_in_place(ptr::slice_from_raw_parts_mut(
                self.ptr.as_ptr().add(self.remaining.start),
                self.remaining.end - self.remaining.start,
            ));
        }

        // Release the allocation exactly once. `deallocate` is a no-op for a
        // zero-sized `T` or a zero capacity, matching how the allocation was
        // (not) created.
        MyVec::<T>::deallocate(self.ptr, self.capacity);
    }
}

impl<T> IntoIterator for MyVec<T> {
    type Item = T;
    type IntoIter = MyVecIntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        // Wrapping in `ManuallyDrop` disables `MyVec`'s destructor, so its
        // allocation and elements are transferred to the iterator without being
        // freed or dropped here.
        let vec = std::mem::ManuallyDrop::new(self);

        MyVecIntoIter {
            ptr: vec.ptr,
            remaining: 0..vec.len,
            capacity: vec.capacity,
            marker: std::marker::PhantomData,
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
        // `clear` drops every initialized element, then `deallocate` frees the
        // block. For a ZST both are no-ops with respect to the allocator: the
        // elements are still dropped by `clear`, but nothing is freed, because
        // there is nothing to free.
        self.clear();
        Self::deallocate(self.ptr, self.capacity);
    }
}

/// Implementing Index trait for MyVec
impl<T> Index<usize> for MyVec<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.len, "index out of bounds");

        // SAFETY: `index < len`, and those slots are initialized.
        unsafe { &*self.ptr.as_ptr().add(index) }
    }
}

impl<T> IndexMut<usize> for MyVec<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(index < self.len, "index out of bounds");

        // SAFETY: `index < len`, and `&mut self` guarantees unique access.
        unsafe { &mut *self.ptr.as_ptr().add(index) }
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
