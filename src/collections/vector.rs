//! A growable, contiguous array — this crate's counterpart to
//! [`std::vec::Vec`].
//!
//! See [`Vector`] for the type-level documentation, examples and a description
//! of how it differs from the standard library's implementation.

use std::alloc::{Layout, alloc, dealloc, handle_alloc_error};
use std::cmp::{Eq, PartialEq};
use std::fmt::Debug;
use std::iter::FusedIterator;
use std::iter::{Extend, FromIterator};
use std::ops::{Deref, DerefMut, Index, IndexMut};
use std::ptr;
use std::ptr::NonNull;
use std::slice;

/// A growable, contiguous, heap-allocated array.
///
/// `Vector<T>` stores its elements in one contiguous heap allocation, exactly
/// like [`std::vec::Vec<T>`], and grows that allocation automatically as
/// elements are pushed. It is the "sequence container" of this crate: the one to
/// reach for when you need ordered, indexable storage.
///
/// # Length and capacity
///
/// * **Length** ([`len`](Vector::len)) is the number of elements actually
///   stored; those elements are initialized and owned by the vector.
/// * **Capacity** ([`capacity`](Vector::capacity)) is the number of
///   elements the current
///   allocation can hold without reallocating. It is always `>= len`.
///
/// Pushing past the capacity reallocates: the elements are moved to a larger
/// block (the capacity at least doubles) and the old block is freed. This makes
/// [`push`](Vector::push) O(1) *amortized* even though an individual push may be
/// O(`len`). Reserving in advance with [`reserve`](Vector::reserve) or
/// [`reserve_exact`](Vector::reserve_exact) avoids the intermediate
/// reallocations when the final size is known.
///
/// # Complexity
///
/// | Operation | Average | Worst case |
/// |---|---|---|
/// | [`push`](Vector::push) / [`pop`](Vector::pop) | O(1) | O(`len`) on reallocation |
/// | [`get`](Vector::get) / indexing | O(1) | O(1) |
/// | [`insert`](Vector::insert) / [`remove`](Vector::remove) | O(`len`) | O(`len`) |
/// | [`swap_remove`](Vector::swap_remove) | O(1) | O(1) |
/// | [`sort`](Vector::sort) | O(`len` log `len`) | O(`len` log `len`) |
/// | [`binary_search`](Vector::binary_search) | O(log `len`) | O(log `len`) |
/// | [`retain`](Vector::retain) / [`dedup`](Vector::dedup) | O(`len`) | O(`len²`) |
///
/// # Zero-sized types
///
/// `Vector<T>` supports zero-sized element types such as `()` or a custom
/// zero-sized struct. No allocation is performed for them, and
/// [`capacity`](Vector::capacity) reports [`usize::MAX`]. Destructors of
/// zero-sized values still run exactly once.
///
/// # Examples
///
/// Construction, insertion, indexing, iteration, removal and sorting:
///
/// ```
/// use rust_stl::collections::vector::Vector;
///
/// // Build a vector from an iterator.
/// let mut scores: Vector<i32> = (0..5).map(|n| n * 10).collect();
/// assert_eq!(scores.len(), 5);
/// assert_eq!(scores.capacity() >= 5, true);
///
/// // Insert in the middle, read it back safely, then index directly.
/// scores.insert(2, 15);
/// assert_eq!(scores.get(2), Some(&15));
/// assert_eq!(scores[2], 15);
///
/// // Iterate by reference and mutate in place.
/// for score in &mut scores {
///     *score += 1;
/// }
///
/// // Remove by value and by index.
/// assert!(scores.remove_first(&11));
/// assert_eq!(scores.remove(0), 1);
///
/// // Sort, then search. Note that `Ok`/`Err` distinguish found from not-found.
/// scores.sort();
/// assert!(scores.binary_search(&21).is_ok());
/// assert_eq!(scores.binary_search(&100), Err(scores.len()));
///
/// // Iteration by value consumes the vector.
/// let total: i32 = scores.into_iter().sum();
/// assert!(total > 0);
/// ```
///
/// # Differences from [`std::vec::Vec`]
///
/// This is an educational reimplementation, not a drop-in replacement. It is
/// not benchmarked against the standard library and makes no performance
/// claims.
///
/// * **API surface.** Only a subset of `Vec`'s API is provided, and some names
///   differ: [`remove_where`](Vector::remove_where) and
///   [`remove_all`](Vector::remove_all) have no `Vec` equivalent, while
///   `Vec::drain`, `Vec::splice`, `Vec::with_capacity` and fallible indexing of
///   ranges are absent.
/// * **Fallible reservation.** [`try_reserve`](Vector::try_reserve) returns
///   `Result<(), &'static str>` rather than a dedicated error type, and does not
///   distinguish capacity overflow from allocation failure.
/// * **Partial consumption.** [`Vector::into_iter`] supports
///   [`DoubleEndedIterator`]; `Vec`'s iterator additionally supports
///   in-place and slice-like adapters.
/// * **No `Copy` specialisation.** `Vec`'s `Clone` (and several other bulk
///   operations) specialise for `T: Copy` and reduce to `memcpy`; this
///   implementation always maps `clone()` over the elements, so cloning a
///   vector of `Copy` types is measurably slower. The behaviour is identical,
///   only the constant factor differs. Specialising like this needs unstable
///   `min_specialization` until Rust exposes a stable mechanism.
/// * **Removal by shifting.** `retain`, `retain_mut`, `remove_where`,
///   `remove_where_mut`, `remove_all`, `dedup` and `remove_duplicates` shift the
///   tail for each removal, so their worst case is O(`len²`) where the standard
///   library is O(`len`).
/// * **Following the standard library.** Methods that share a name with `Vec`
///   are intended to behave the same way, but they are documented individually
///   rather than assumed to be identical.
pub struct Vector<T> {
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

impl<T> Vector<T> {
    /// Creates an empty `Vector`.
    ///
    /// No allocation is performed. The first element pushed triggers the first
    /// allocation.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_stl::collections::vector::Vector;
    ///
    /// let mut vector: Vector<i32> = Vector::new();
    /// assert!(vector.is_empty());
    /// assert_eq!(vector.capacity(), 0);
    ///
    /// vector.push(1);
    /// assert_eq!(vector.len(), 1);
    /// ```
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

    /// Returns the number of elements the vector holds.
    ///
    /// This is O(1).
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns the number of elements the vector can hold without reallocating.
    ///
    /// This is O(1) and is an implementation detail: it is always `>= len`, but
    /// the exact value is not part of the contract, so do not depend on it.
    ///
    /// For a zero-sized element type this reports [`usize::MAX`], because such a
    /// vector never needs to allocate.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns `true` if the vector contains no elements.
    ///
    /// This is O(1) and is equivalent to `len() == 0`.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns a reference to the element at `index`, or `None` if `index` is
    /// out of bounds.
    ///
    /// This is O(1). Prefer it over the indexing operator when the index may be
    /// out of range, since indexing panics.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_stl::collections::vector::Vector;
    ///
    /// let vector: Vector<i32> = (0..3).collect();
    /// assert_eq!(vector.get(1), Some(&1));
    /// assert_eq!(vector.get(3), None);
    /// ```
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            // SAFETY: `index < len`, and the first `len` elements of the
            // allocation are initialized, so this pointer addresses a live `T`.
            unsafe { Some(&*self.ptr.as_ptr().add(index)) }
        } else {
            None
        }
    }

    /// Returns a mutable reference to the element at `index`, or `None` if
    /// `index` is out of bounds.
    ///
    /// This is O(1).
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

    /// Appends `value` to the back of the vector, growing it if necessary.
    ///
    /// # Complexity
    ///
    /// O(1) amortized. When `len == capacity` the backing storage is
    /// reallocated, which is O(`len`); the capacity doubles on each such step,
    /// so the cost averages out to constant time per push.
    ///
    /// # Panics
    ///
    /// Panics if the additional element would overflow the vector's maximum
    /// length.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_stl::collections::vector::Vector;
    ///
    /// let mut vector = Vector::new();
    /// vector.push(1);
    /// vector.push(2);
    /// assert_eq!(vector.as_slice(), &[1, 2]);
    /// ```
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

    /// Removes and returns the last element, or `None` if the vector is empty.
    ///
    /// This is O(1) and does not release capacity.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_stl::collections::vector::Vector;
    ///
    /// let mut vector: Vector<i32> = (0..2).collect();
    /// assert_eq!(vector.pop(), Some(1));
    /// assert_eq!(vector.pop(), Some(0));
    /// assert_eq!(vector.pop(), None);
    /// ```
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }

        Some(self.remove(self.len - 1))
    }

    /// Returns the index of the first element for which `predicate` returns
    /// `true`, or `None` if no element matches.
    ///
    /// The predicate is applied in index order. This is O(`len`).
    pub fn find_index<F>(&self, mut predicate: F) -> Option<usize>
    where
        F: FnMut(&T) -> bool,
    {
        (0..self.len).find(|&index| predicate(unsafe { &*self.ptr.as_ptr().add(index) }))
    }

    /// Returns how many elements satisfy `predicate`.
    ///
    /// The predicate is applied once per element, in index order. This is
    /// O(`len`).
    pub fn count_matches<F>(&self, mut predicate: F) -> usize
    where
        F: FnMut(&T) -> bool,
    {
        (0..self.len)
            .filter(|&index| predicate(unsafe { &*self.ptr.as_ptr().add(index) }))
            .count()
    }

    /// Removes the first element equal to `value`, returning `true` if one was
    /// found.
    ///
    /// The tail is shifted left, so it preserves the order of the remaining
    /// elements. This is O(`len`) plus the cost of the `PartialEq` comparisons.
    /// Removes and returns the element at `index`, shifting every later element
    /// one position to the left.
    ///
    /// Ownership of the removed value is transferred to the caller; the vector
    /// does not drop it.
    ///
    /// This is O(`len`). If the order of the remaining elements does not matter,
    /// [`swap_remove`](Self::swap_remove) is O(1).
    ///
    /// # Panics
    ///
    /// Panics if `index >= len`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_stl::collections::vector::Vector;
    ///
    /// let mut vector: Vector<i32> = (0..3).collect();
    /// assert_eq!(vector.remove(1), 1);
    /// assert_eq!(vector.as_slice(), &[0, 2]);
    /// ```
    pub fn remove_first(&mut self, value: &T) -> bool
    where
        T: PartialEq,
    {
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

    /// Returns a new `Vector` containing the indices of every element that
    /// satisfies `predicate`, in ascending order.
    ///
    /// This is O(`len`) and allocates a result vector.
    pub fn indices_of<F>(&self, mut predicate: F) -> Vector<usize>
    where
        F: FnMut(&T) -> bool,
    {
        let mut indices = Vector::new();

        for index in 0..self.len {
            // SAFETY: `index < len` and those slots are initialized.
            if predicate(unsafe { &*self.ptr.as_ptr().add(index) }) {
                indices.push(index);
            }
        }

        indices
    }

    /// Removes all elements, dropping each one exactly once.
    ///
    /// The allocated capacity is retained, so the vector can be refilled without
    /// reallocating. This is O(`len`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_stl::collections::vector::Vector;
    ///
    /// let mut vector: Vector<i32> = (0..4).collect();
    /// let capacity = vector.capacity();
    /// vector.clear();
    /// assert!(vector.is_empty());
    /// assert_eq!(vector.capacity(), capacity);
    /// ```
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

    /// Returns the contents as a slice.
    ///
    /// This is O(1) and performs no allocation. An empty vector yields an empty
    /// slice backed by a dangling, non-null pointer.
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

    /// Returns the contents as a mutable slice.
    ///
    /// This is O(1) and performs no allocation.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        if self.len == 0 {
            // SAFETY: as in `as_slice`, and `&mut self` gives unique access.
            return unsafe { slice::from_raw_parts_mut(NonNull::<T>::dangling().as_ptr(), 0) };
        }

        // SAFETY: the allocation is valid for `len` initialized elements and
        // `&mut self` guarantees they are not aliased.
        unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }

    /// Returns a reference to the first element, or `None` if the vector is
    /// empty. This is O(1).
    pub fn first(&self) -> Option<&T> {
        if self.len > 0 {
            // SAFETY: `len > 0` means slot 0 is initialized.
            unsafe { Some(&*self.ptr.as_ptr()) }
        } else {
            None
        }
    }

    /// Returns a mutable reference to the first element, or `None` if the
    /// vector is empty. This is O(1).
    pub fn first_mut(&mut self) -> Option<&mut T> {
        if self.len > 0 {
            // SAFETY: `len > 0` and `&mut self` gives unique access.
            unsafe { Some(&mut *self.ptr.as_ptr()) }
        } else {
            None
        }
    }

    /// Returns a reference to the last element, or `None` if the vector is
    /// empty. This is O(1).
    pub fn last(&self) -> Option<&T> {
        if self.len > 0 {
            // SAFETY: `len > 0`, so slot `len - 1` is inside the allocation
            // and initialized.
            unsafe { Some(&*self.ptr.as_ptr().add(self.len - 1)) }
        } else {
            None
        }
    }

    /// Swaps the elements at `index1` and `index2`.
    ///
    /// This is O(1) and moves no other elements.
    ///
    /// # Panics
    ///
    /// Panics if either index is out of bounds.
    pub fn swap(&mut self, index1: usize, index2: usize) {
        assert!(index1 < self.len, "index1 out of bounds");
        assert!(index2 < self.len, "index2 out of bounds");

        // SAFETY: both indices are in bounds, so both slots are initialized.
        // `ptr::swap` is valid for identical values and overlapping ranges.
        unsafe {
            std::ptr::swap(self.ptr.as_ptr().add(index1), self.ptr.as_ptr().add(index2));
        }
    }

    /// Returns a mutable reference to the last element, or `None` if the vector
    /// is empty. This is O(1).
    pub fn last_mut(&mut self) -> Option<&mut T> {
        if self.len > 0 {
            // SAFETY: `len > 0`, slot `len - 1` is initialized, and `&mut self`
            // guarantees unique access to it.
            unsafe { Some(&mut *self.ptr.as_ptr().add(self.len - 1)) }
        } else {
            None
        }
    }

    /// Removes the element at `index` and returns it, filling the gap with the
    /// last element.
    ///
    /// The element *at* `index` is the one returned; the order of the remaining
    /// elements is **not** preserved. Unlike [`remove`](Self::remove) this is
    /// O(1).
    ///
    /// Returns `None` if `index` is out of bounds rather than panicking.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_stl::collections::vector::Vector;
    ///
    /// let mut vector: Vector<i32> = (0..3).collect();
    /// assert_eq!(vector.swap_remove(0), Some(0));
    /// // The last element took the removed element's place.
    /// assert_eq!(vector.as_slice(), &[2, 1]);
    /// assert_eq!(vector.swap_remove(9), None);
    /// ```
    pub fn swap_remove(&mut self, index: usize) -> Option<T> {
        if index >= self.len {
            return None;
        }

        self.swap(index, self.len - 1);
        self.remove(self.len - 1).into()
    }

    /// Shortens the vector to at most `new_len` elements, dropping the removed
    /// tail.
    ///
    /// Does nothing if `new_len >= len`, so it is never an error to pass a
    /// length larger than the vector. Capacity is retained. This is O(number of
    /// elements removed).
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

    /// Keeps only the elements for which `predicate` returns `true`, preserving
    /// their relative order.
    ///
    /// The predicate is called with a shared reference to each element, in
    /// index order, and the rejected elements are dropped. Note the polarity:
    /// `retain` keeps matches, whereas
    /// [`remove_where`](Self::remove_where) removes them.
    ///
    /// This is O(`len²`) in the worst case (each removal shifts the tail).
    ///
    /// If the predicate panics, the vector remains valid and every element is
    /// still owned by exactly one place, but elements that were not yet visited
    /// are left in place rather than dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_stl::collections::vector::Vector;
    ///
    /// let mut vector: Vector<i32> = (0..6).collect();
    /// vector.retain(|n| n % 2 == 0);
    /// assert_eq!(vector.as_slice(), &[0, 2, 4]);
    /// ```
    pub fn retain<F>(&mut self, mut predicate: F)
    where
        F: FnMut(&T) -> bool,
    {
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

    /// Keeps only the elements for which `predicate` returns `true`, allowing the
    /// predicate to mutate the elements it keeps.
    ///
    /// The predicate is called with a mutable reference to each element, in
    /// index order. Rejected elements are dropped after the predicate returns.
    ///
    /// This is O(`len²`) in the worst case. The same panic-safety note as
    /// [`retain`](Self::retain) applies.
    pub fn retain_mut<F>(&mut self, mut predicate: F)
    where
        F: FnMut(&mut T) -> bool,
    {
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

    /// Removes every element equal to `value`.
    ///
    /// The relative order of the surviving elements is preserved. This is
    /// O(`len²`) in the worst case because each removal shifts the tail.
    ///
    /// Note that `value` is consumed: it is dropped along with the matching
    /// elements.
    pub fn remove_all(&mut self, value: T)
    where
        T: PartialEq,
    {
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

    /// Removes every element for which `predicate` returns `true`, preserving
    /// the relative order of the survivors.
    ///
    /// This is the mirror image of [`retain`](Self::retain): `remove_where`
    /// removes matches, `retain` keeps them.
    ///
    /// This is O(`len²`) in the worst case.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_stl::collections::vector::Vector;
    ///
    /// let mut vector: Vector<i32> = (0..6).collect();
    /// vector.remove_where(|n| n % 2 == 0);
    /// assert_eq!(vector.as_slice(), &[1, 3, 5]);
    /// ```
    pub fn remove_where<F>(&mut self, mut predicate: F)
    where
        F: FnMut(&T) -> bool,
    {
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

    /// Removes every element for which `predicate` returns `true`, passing each
    /// element to the predicate mutably so it can inspect or update it before
    /// the decision is made.
    ///
    /// This is O(`len²`) in the worst case. The predicate is called with a
    /// mutable reference, and any mutation it performs on a removed element is
    /// discarded along with the element.
    pub fn remove_where_mut<F>(&mut self, mut predicate: F)
    where
        F: FnMut(&mut T) -> bool,
    {
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

    /// Removes every repeated value, keeping only the first occurrence of each
    /// distinct value.
    ///
    /// This is a **set-like** operation: it compares every pair of elements, so
    /// duplicates that are not adjacent are removed. To remove only
    /// *consecutive* duplicates (the conventional vector behaviour), use
    /// [`dedup`](Self::dedup).
    ///
    /// The order of first occurrences is preserved. This is O(`len²`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_stl::collections::vector::Vector;
    ///
    /// let mut vector: Vector<i32> = vec![3, 1, 3, 2, 1].into_iter().collect();
    /// vector.remove_duplicates();
    /// assert_eq!(vector.as_slice(), &[3, 1, 2]);
    /// ```
    pub fn remove_duplicates(&mut self)
    where
        T: PartialEq,
    {
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

    /// Removes *consecutive* repeated elements, keeping the first of each run.
    ///
    /// This matches the conventional vector semantics: it only compares adjacent
    /// elements, so `[1, 1, 2, 1]` becomes `[1, 2, 1]`. To collapse duplicates
    /// wherever they appear, use
    /// [`remove_duplicates`](Self::remove_duplicates).
    ///
    /// This is O(`len²`) in the worst case because the tail is shifted for every
    /// removal. Calling [`sort`](Self::sort) first makes the operation O(`len`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_stl::collections::vector::Vector;
    ///
    /// let mut vector: Vector<i32> = vec![1, 1, 2, 2, 1].into_iter().collect();
    /// vector.dedup();
    /// assert_eq!(vector.as_slice(), &[1, 2, 1]);
    /// ```
    pub fn dedup(&mut self)
    where
        T: PartialEq,
    {
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

        // SAFETY: `reallocate` freed the same pointer, and `try_reallocate` is
        // called with `new_capacity > self.capacity`, so a real block exists.
        // Delegating to `deallocate` is required: it skips the
        // never-allocated (`capacity == 0`) and zero-sized cases, where calling
        // `dealloc` directly would pass a zero-sized layout — undefined
        // behaviour that `dealloc`'s precondition check turns into an abort.
        Self::deallocate(self.ptr, self.capacity);

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
        self.len.checked_add(additional).ok_or("capacity overflow")
    }

    /// The amortized target capacity used by `reserve` / `try_reserve`.
    ///
    /// Only meaningful when `required > self.capacity`; callers must check that
    /// first, otherwise a doubling of the current capacity could trigger a
    /// pointless reallocation for a request that already fits.
    fn amortized_capacity(&self, required: usize) -> usize {
        // Doubling keeps `push` amortized O(1). `saturating_mul` avoids
        // wrapping here: an over-large target is rejected later by the layout
        // check rather than silently rounded down.
        required.max(self.capacity.saturating_mul(2)).max(4)
    }

    /// Reserves capacity for at least `additional` more elements.
    ///
    /// The vector may allocate more than requested; `capacity()` afterwards is
    /// guaranteed to be at least `len + additional`. If the current capacity is
    /// already sufficient this does nothing, so `reserve(0)` never reallocates.
    /// Existing elements are preserved.
    ///
    /// # Panics
    ///
    /// Panics if `len + additional` overflows `usize`, or if the resulting
    /// element layout cannot be represented. Use
    /// [`try_reserve`](Self::try_reserve) to handle those cases instead.
    pub fn reserve(&mut self, additional: usize) {
        let required = self
            .required_capacity(additional)
            .expect("capacity overflow: vector length exceeded");

        // `reserve(0)` (or any request that already fits) must not grow the
        // vector: reporting a larger capacity would be a surprising allocation.
        if required > self.capacity {
            self.reallocate(self.amortized_capacity(required));
        }
    }

    /// Reduces the capacity to the current length.
    ///
    /// For an empty vector this releases the allocation entirely and leaves the
    /// vector able to allocate again on the next push. Non-empty vectors keep
    /// all their elements. Zero-sized element types own no allocation, so this
    /// is a no-op for them.
    ///
    /// The allocator is free to honour this request only approximately, so only
    /// the length and the contents are guaranteed afterwards.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_stl::collections::vector::Vector;
    ///
    /// let mut vector: Vector<i32> = Vector::new();
    /// vector.reserve(64);
    /// vector.extend(0..4);
    /// vector.shrink_to_fit();
    /// assert!(vector.capacity() >= 4);
    /// assert_eq!(vector.as_slice(), &[0, 1, 2, 3]);
    /// ```
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

    /// Reserves capacity for exactly `additional` more elements.
    ///
    /// Unlike [`reserve`](Self::reserve) this does not over-allocate, which is
    /// useful when the final size is known precisely. Existing elements are
    /// preserved.
    ///
    /// # Panics
    ///
    /// Panics if `len + additional` or the resulting element layout overflows.
    /// Use [`try_reserve_exact`](Self::try_reserve_exact) to handle those cases
    /// instead.
    pub fn reserve_exact(&mut self, additional: usize) {
        let required = self
            .required_capacity(additional)
            .expect("capacity overflow: vector length exceeded");

        self.reallocate(required);
    }

    /// Fallible version of [`reserve`](Self::reserve).
    ///
    /// # Errors
    ///
    /// Returns `Err` if `len + additional` overflows `usize`, if the element
    /// layout for the target capacity cannot be represented, or if the
    /// allocation itself fails. On error the vector is completely untouched:
    /// same pointer, length, capacity and contents.
    ///
    /// The error is a human-readable static string; the API deliberately does
    /// not distinguish the failure kinds.
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), &'static str> {
        let required = self.required_capacity(additional)?;

        if required > self.capacity {
            let target = self.amortized_capacity(required);
            self.try_reallocate(target)?;
        }

        Ok(())
    }

    /// Fallible version of [`reserve_exact`](Self::reserve_exact).
    ///
    /// # Errors
    ///
    /// As [`try_reserve`](Self::try_reserve), and on failure the vector is left
    /// exactly as it was.
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), &'static str> {
        let required = self.required_capacity(additional)?;

        self.try_reallocate(required)
    }

    /// Inserts `value` at `index`, shifting every element at or after `index`
    /// one position to the right.
    ///
    /// This is O(`len`); inserting near the end is cheaper than inserting near
    /// the front. Passing `index == len` is equivalent to
    /// [`push`](Self::push).
    ///
    /// # Panics
    ///
    /// Panics if `index > len`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_stl::collections::vector::Vector;
    ///
    /// let mut vector: Vector<i32> = (0..3).collect();
    /// vector.insert(1, 9);
    /// assert_eq!(vector.as_slice(), &[0, 9, 1, 2]);
    /// ```
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

    /// Moves every element of `other` to the back of this vector, leaving
    /// `other` empty.
    ///
    /// This is O(`other.len()`) plus any reallocation. No element is cloned or
    /// dropped: ownership moves from `other` to `self`, and `other` keeps its
    /// allocation so it can be reused.
    ///
    /// # Panics
    ///
    /// Panics if the combined length overflows `usize`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_stl::collections::vector::Vector;
    ///
    /// let mut head: Vector<i32> = (0..2).collect();
    /// let mut tail: Vector<i32> = (2..5).collect();
    /// head.append(&mut tail);
    /// assert_eq!(head.as_slice(), &[0, 1, 2, 3, 4]);
    /// assert!(tail.is_empty());
    /// ```
    pub fn append(&mut self, other: &mut Vector<T>) {
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
            ptr::copy_nonoverlapping(
                other.ptr.as_ptr(),
                self.ptr.as_ptr().add(self.len),
                other_len,
            );
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

    /// Appends all items produced by `iter`.
    ///
    /// This is equivalent to the [`Extend`] implementation, which is what makes
    /// `vector.extend(iter)` and `iter.collect::<Vector<_>>()` convenient. Both
    /// are available; the inherent method simply takes precedence in method
    /// resolution.
    pub fn extend(&mut self, iter: impl IntoIterator<Item = T>) {
        for item in iter {
            self.push(item);
        }
    }

    /// Appends clones of every element in `slice`.
    ///
    /// The source slice is left untouched. This is O(`slice.len()`) plus any
    /// reallocation. Requires `T: Clone`.
    pub fn extend_from_slice(&mut self, slice: &[T])
    where
        T: Clone,
    {
        self.reserve(slice.len());

        for item in slice {
            self.push(item.clone());
        }
    }

    /// Resizes the vector to `new_len` elements.
    ///
    /// If `new_len` is larger, the vector is filled with clones of `value`. If
    /// it is smaller, the extra elements are dropped. This is O(|`old_len` -
    /// `new_len`|) plus any reallocation. Requires `T: Clone`.
    pub fn resize(&mut self, new_len: usize, value: T)
    where
        T: Clone,
    {
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

    /// Resizes the vector to `new_len` elements, generating new values with `f`.
    ///
    /// Unlike [`resize`](Self::resize) this does not require `T: Clone`. `f` is
    /// called once per added element and not at all when the vector is being
    /// shortened. This is O(|`old_len` - `new_len`|) plus any reallocation.
    pub fn resize_with<F>(&mut self, new_len: usize, mut f: F)
    where
        F: FnMut() -> T,
    {
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

    /// Splits the vector in two at `at`, returning everything from `at` onwards
    /// as a new vector.
    ///
    /// `self` keeps the elements before `at`; the returned vector owns the rest.
    /// No element is cloned or dropped, though the returned vector performs its
    /// own allocation. `at == 0` returns a copy of the whole vector and empties
    /// `self`; `at == len` returns an empty vector and changes nothing. This is
    /// O(`len - at`).
    ///
    /// # Panics
    ///
    /// Panics if `at > len`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_stl::collections::vector::Vector;
    ///
    /// let mut vector: Vector<i32> = (0..5).collect();
    /// let tail = vector.split_off(3);
    /// assert_eq!(vector.as_slice(), &[0, 1, 2]);
    /// assert_eq!(tail.as_slice(), &[3, 4]);
    /// ```
    pub fn split_off(&mut self, at: usize) -> Vector<T> {
        assert!(at <= self.len, "index out of bounds");

        let mut new_vec = Vector::new();
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

    /// Returns an iterator over shared references to the elements.
    ///
    /// This is equivalent to `for element in &vector`. This is O(1).
    pub fn iter(&self) -> VectorIter<'_, T> {
        VectorIter {
            vec: self,
            index: 0,
        }
    }

    /// Returns an iterator over mutable references to the elements.
    ///
    /// This is equivalent to `for element in &mut vector`. This is O(1).
    pub fn iter_mut(&mut self) -> VectorIterMut<'_, T> {
        VectorIterMut {
            ptr: self.ptr.as_ptr(),
            len: self.len,
            index: 0,
            marker: std::marker::PhantomData,
        }
    }

    /// Reverses the order of the elements in place.
    ///
    /// This is O(`len`) and moves no allocations. Empty and single-element
    /// vectors are unaffected.
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

    /// Sorts the elements in ascending order.
    ///
    /// This is a stable sort: equal elements keep their relative order. This is
    /// O(`len` log `len`) in the worst case. Requires `T: Ord`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_stl::collections::vector::Vector;
    ///
    /// let mut vector: Vector<i32> = vec![3, 1, 2].into_iter().collect();
    /// vector.sort();
    /// assert_eq!(vector.as_slice(), &[1, 2, 3]);
    /// ```
    pub fn sort(&mut self)
    where
        T: Ord,
    {
        let slice = self.as_mut_slice();
        slice.sort();
    }
    /// Sorts the elements using `compare`.
    ///
    /// `compare(a, b)` must return the ordering of `a` relative to `b`. The sort
    /// is stable. This is O(`len` log `len`) in the worst case.
    pub fn sort_by<F>(&mut self, compare: F)
    where
        F: FnMut(&T, &T) -> std::cmp::Ordering,
    {
        let slice = self.as_mut_slice();
        slice.sort_by(compare);
    }

    /// Returns a raw immutable pointer to the first element's slot.
    ///
    /// The pointer is never null — an empty, never-allocated vector yields a
    /// dangling but correctly aligned pointer — and `end() - begin()` equals
    /// `len()`. It may only be dereferenced while the vector is alive and not
    /// reallocated; prefer [`as_slice`](Self::as_slice) unless raw pointers are
    /// genuinely needed. This is O(1).
    pub fn begin(&self) -> *const T {
        // Never null: an empty, never-allocated vector yields a dangling but
        // aligned pointer, which is what pointer arithmetic and comparisons
        // expect.
        self.ptr.as_ptr()
    }

    /// Returns a raw immutable pointer one past the last element.
    ///
    /// The pointer is never null and is valid to compare against
    /// [`begin`](Self::begin) and to use for pointer arithmetic; it must not be
    /// dereferenced. This is O(1).
    pub fn end(&self) -> *const T {
        // SAFETY: `len <= capacity`, so one-past-the-end is in bounds. For a
        // ZST the pointer never moves, which is also correct for `len` elements.
        unsafe { self.ptr.as_ptr().add(self.len) }
    }
    /// Sorts the half-open range `start..end` of elements in ascending order.
    ///
    /// The element at `end` is **not** included in the sort. The sort is stable.
    /// This is O(`n` log `n`) in the length of the range. Requires `T: Ord`.
    ///
    /// # Panics
    ///
    /// Panics if `start > end` or `end > len`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_stl::collections::vector::Vector;
    ///
    /// let mut vector: Vector<i32> = vec![9, 8, 3, 1, 2, 0].into_iter().collect();
    /// vector.sort_range(2, 5);
    /// assert_eq!(vector.as_slice(), &[9, 8, 1, 2, 3, 0]);
    /// ```
    pub fn sort_range(&mut self, start: usize, end: usize)
    where
        T: Ord,
    {
        assert!(start <= end, "invalid range");
        assert!(end <= self.len, "index out of bounds");

        self.as_mut_slice()[start..end].sort();
    }
    /// Binary-searches a sorted vector for `value`.
    ///
    /// Returns `Ok(index)` for a matching element — if several match, any one of
    /// them may be returned — or `Err(insertion_index)`, the position at which
    /// `value` could be inserted while keeping the vector sorted.
    ///
    /// **The elements must already be sorted** (for example with
    /// [`sort`](Self::sort)); the result is unspecified otherwise. This is
    /// O(log `len`) on each comparison.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_stl::collections::vector::Vector;
    ///
    /// let vector: Vector<i32> = (0..6).map(|n| n * 2).collect();
    /// assert_eq!(vector.binary_search(&4), Ok(2));
    /// assert_eq!(vector.binary_search(&5), Err(3));
    /// ```
    pub fn binary_search(&self, value: &T) -> Result<usize, usize>
    where
        T: Ord,
    {
        self.as_slice().binary_search(value)
    }
    /// Rotates the elements `mid` places to the left.
    ///
    /// `[a, b, c, d]` rotated left by 1 becomes `[b, c, d, a]`. This is O(`len`)
    /// and moves no allocations.
    ///
    /// # Panics
    ///
    /// Panics if `mid > len`.
    pub fn rotate_left(&mut self, mid: usize) {
        assert!(mid <= self.len, "mid out of bounds");

        self.as_mut_slice().rotate_left(mid);
    }
    /// Rotates the elements `mid` places to the right.
    ///
    /// `[a, b, c, d]` rotated right by 1 becomes `[d, a, b, c]`. This is O(`len`)
    /// and moves no allocations.
    ///
    /// # Panics
    ///
    /// Panics if `mid > len`.
    pub fn rotate_right(&mut self, mid: usize) {
        assert!(mid <= self.len, "mid out of bounds");

        self.as_mut_slice().rotate_right(mid);
    }
    /// Divides the vector into two slices at `mid`.
    ///
    /// Returns `(before, after)` where `before` holds `0..mid` and `after` holds
    /// `mid..len`. The two slices do not overlap and neither is a copy. This is
    /// O(1).
    ///
    /// # Panics
    ///
    /// Panics if `mid > len`.
    pub fn split_at(&self, mid: usize) -> (&[T], &[T]) {
        self.as_slice().split_at(mid)
    }
    /// Returns an iterator over all **overlapping** windows of `size` elements.
    ///
    /// Windows advance one element at a time, so they share elements with their
    /// neighbours. A `size` of 0 panics, and a `size` larger than the vector
    /// yields no windows. Use [`chunks`](Self::chunks) for non-overlapping
    /// slices. This is O(1) to create.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_stl::collections::vector::Vector;
    ///
    /// let vector: Vector<i32> = (0..4).collect();
    /// let windows: Vec<&[i32]> = vector.windows(2).collect();
    /// assert_eq!(windows, vec![&[0, 1][..], &[1, 2][..], &[2, 3][..]]);
    /// ```
    pub fn windows(&self, size: usize) -> std::slice::Windows<'_, T> {
        self.as_slice().windows(size)
    }
    /// Returns an iterator over **non-overlapping** chunks of at most `size`
    /// elements.
    ///
    /// Every element appears in exactly one chunk; the final chunk may be
    /// shorter. A `size` of 0 panics. Use [`windows`](Self::windows) for
    /// overlapping views. This is O(1) to create.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_stl::collections::vector::Vector;
    ///
    /// let vector: Vector<i32> = (0..5).collect();
    /// let chunks: Vec<&[i32]> = vector.chunks(2).collect();
    /// assert_eq!(chunks, vec![&[0, 1][..], &[2, 3][..], &[4][..]]);
    /// ```
    pub fn chunks(&self, size: usize) -> std::slice::Chunks<'_, T> {
        self.as_slice().chunks(size)
    }
    /// Binary-searches a sorted vector with a comparator.
    ///
    /// `compare` receives a reference to an element and returns its ordering
    /// relative to the value being searched for. This is the general form of
    /// [`binary_search`](Self::binary_search) for types that are not directly
    /// comparable.
    ///
    /// **The elements must already be sorted according to `compare`.** This is
    /// O(log `len`) calls to `compare`.
    pub fn binary_search_by<F>(&self, compare: F) -> Result<usize, usize>
    where
        F: FnMut(&T) -> std::cmp::Ordering,
    {
        self.as_slice().binary_search_by(compare)
    }

    /// Binary-searches a sorted vector by mapping each element to a key.
    ///
    /// `f` extracts the comparison key from an element and the result is used as
    /// in [`binary_search`](Self::binary_search). **The elements must already be
    /// sorted by the extracted key.** This is O(log `len`) calls to `f`.
    pub fn binary_search_by_key<B, F>(&self, key: &B, f: F) -> Result<usize, usize>
    where
        B: Ord,
        F: FnMut(&T) -> B,
    {
        self.as_slice().binary_search_by_key(key, f)
    }
}

pub struct VectorIter<'a, T> {
    vec: &'a Vector<T>,
    index: usize,
}
impl<'a, T> Iterator for VectorIter<'a, T> {
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

impl<T> ExactSizeIterator for VectorIter<'_, T> {}

impl<T> FusedIterator for VectorIter<'_, T> {}

pub struct VectorIterMut<'a, T> {
    ptr: *mut T,
    len: usize,
    index: usize,
    marker: std::marker::PhantomData<&'a mut T>,
}

impl<'a, T> Iterator for VectorIterMut<'a, T> {
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

impl<T> ExactSizeIterator for VectorIterMut<'_, T> {}

impl<T> FusedIterator for VectorIterMut<'_, T> {}

impl<'a, T> IntoIterator for &'a Vector<T> {
    type Item = &'a T;
    type IntoIter = VectorIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Vector<T> {
    type Item = &'a mut T;
    type IntoIter = VectorIterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// Owning iterator over a [`Vector`].
///
/// Created by [`Vector::into_iter`] (or, equivalently, `for value in vec`).
pub struct VectorIntoIter<T> {
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

impl<T> Iterator for VectorIntoIter<T> {
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

impl<T> DoubleEndedIterator for VectorIntoIter<T> {
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

impl<T> ExactSizeIterator for VectorIntoIter<T> {}

impl<T> FusedIterator for VectorIntoIter<T> {}

impl<T> Drop for VectorIntoIter<T> {
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
        Vector::<T>::deallocate(self.ptr, self.capacity);
    }
}

impl<T> IntoIterator for Vector<T> {
    type Item = T;
    type IntoIter = VectorIntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        // Wrapping in `ManuallyDrop` disables `Vector`'s destructor, so its
        // allocation and elements are transferred to the iterator without being
        // freed or dropped here.
        let vec = std::mem::ManuallyDrop::new(self);

        VectorIntoIter {
            ptr: vec.ptr,
            remaining: 0..vec.len,
            capacity: vec.capacity,
            marker: std::marker::PhantomData,
        }
    }
}
impl<T> Default for Vector<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for Vector<T> {
    fn drop(&mut self) {
        // `clear` drops every initialized element, then `deallocate` frees the
        // block. For a ZST both are no-ops with respect to the allocator: the
        // elements are still dropped by `clear`, but nothing is freed, because
        // there is nothing to free.
        self.clear();
        Self::deallocate(self.ptr, self.capacity);
    }
}

/// Indexing delegates to slice indexing, which gives the same `usize`, range and
/// step-range index types as `std::vec::Vec` and keeps the bounds check in one
/// place.
impl<T, I> Index<I> for Vector<T>
where
    I: std::slice::SliceIndex<[T]>,
{
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        &self.as_slice()[index]
    }
}

impl<T, I> IndexMut<I> for Vector<T>
where
    I: std::slice::SliceIndex<[T]>,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.as_mut_slice()[index]
    }
}

impl<T> Debug for Vector<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}
impl<T> Clone for Vector<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        let mut new_vec = Vector::new();
        new_vec.reserve(self.len);

        for item in self.iter() {
            new_vec.push(item.clone());
        }

        new_vec
    }
}

impl<T> PartialEq for Vector<T>
where
    T: PartialEq,
{
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

impl<T> Eq for Vector<T> where T: Eq {}

impl<T> Extend<T> for Vector<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.push(item);
        }
    }
}

impl<T> FromIterator<T> for Vector<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut vec = Vector::new();
        vec.extend(iter);
        vec
    }
}

impl<T> Deref for Vector<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T> DerefMut for Vector<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

/// Unit tests for the private invariants that the public API cannot reach.
#[cfg(test)]
mod tests {
    use super::*;

    /// A zero-sized element type used to check the ZST special-casing.
    #[derive(Debug, PartialEq, Eq)]
    struct Marker;

    #[test]
    fn new_vector_is_unallocated_with_a_dangling_pointer() {
        let vec: Vector<u32> = Vector::new();

        assert_eq!(vec.len, 0);
        assert_eq!(vec.capacity, 0);
        // A never-allocated vector must still hold a non-null, aligned pointer.
        // (`NonNull` already rules out null, so `dangling` is the strongest
        // assertion available here.)
        assert_eq!(vec.ptr, NonNull::dangling());
    }

    #[test]
    fn is_zst_detects_zero_sized_types() {
        assert!(Vector::<()>::is_zst());
        assert!(Vector::<Marker>::is_zst());
        assert!(!Vector::<u8>::is_zst());
        assert!(!Vector::<u64>::is_zst());
    }

    #[test]
    fn try_layout_reports_overflow_and_accepts_valid_capacities() {
        assert!(Vector::<u64>::try_layout(0).is_ok());
        assert!(Vector::<u64>::try_layout(1_000).is_ok());

        // `usize::MAX` elements of any non-ZST cannot fit in an address space.
        assert!(Vector::<u64>::try_layout(usize::MAX).is_err());
        assert!(Vector::<u8>::try_layout(usize::MAX).is_err());
    }

    #[test]
    fn try_layout_reports_overflow_for_both_len_and_additional() {
        let mut vec: Vector<u32> = Vector::new();
        vec.push(1);

        // `1 + usize::MAX` overflows the length computation itself.
        assert_eq!(vec.required_capacity(usize::MAX), Err("capacity overflow"));

        // `len + additional` fits, but the element layout does not.
        assert!(Vector::<u32>::try_layout(usize::MAX / 2).is_err());
    }

    #[test]
    fn amortized_capacity_never_rounds_down_a_request() {
        let mut vec: Vector<u32> = Vector::new();

        assert!(vec.amortized_capacity(100) >= 100);
        assert!(vec.amortized_capacity(1) >= 4);

        vec.reserve_exact(8);
        vec.extend(0..8);
        // Doubling applies to the *current* capacity once growth is needed.
        assert!(vec.amortized_capacity(9) >= 16);
    }

    #[test]
    fn zero_sized_type_reports_maximum_capacity_without_allocating() {
        let mut vec: Vector<Marker> = Vector::new();

        for _ in 0..10 {
            vec.push(Marker);
        }

        assert_eq!(vec.len, 10);
        assert_eq!(vec.capacity, usize::MAX);
        // No allocation happened, so the pointer is still dangling.
        assert_eq!(vec.ptr, NonNull::dangling());
    }

    #[test]
    fn deallocate_tolerates_never_allocated_and_zero_sized_vectors() {
        // These would abort if a zero-sized layout reached `dealloc`.
        Vector::<u32>::deallocate(NonNull::dangling(), 0);
        Vector::<()>::deallocate(NonNull::dangling(), usize::MAX);
    }

    #[test]
    fn grow_keeps_the_allocation_and_length_in_sync() {
        let mut vec: Vector<u32> = Vector::new();

        vec.grow();
        assert!(vec.capacity >= 4);
        assert_eq!(vec.len, 0);
        assert_ne!(vec.ptr, NonNull::dangling(), "growing must allocate");

        vec.push(1);
        vec.push(2);
        let capacity = vec.capacity;

        vec.grow();
        assert!(vec.capacity >= capacity);
        assert_eq!(vec.len, 2);
        assert_eq!(vec.as_slice(), &[1, 2]);
    }

    #[test]
    fn owned_iterator_range_tracks_both_ends() {
        let vec: Vector<i32> = (0..4).collect();
        let mut iter = vec.into_iter();

        assert_eq!(iter.remaining, 0..4);
        iter.next();
        assert_eq!(iter.remaining, 1..4);
        iter.next_back();
        assert_eq!(iter.remaining, 1..3);
        iter.next();
        iter.next();
        assert_eq!(iter.remaining.start, iter.remaining.end);
    }
}
