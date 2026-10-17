//! A last-in, first-out (LIFO) stack.
//!
//! [`Stack<T>`] is an adaptor over
//! [`Vector<T>`](crate::collections::vector::Vector): it keeps its elements in a
//! growable array but exposes only the end of that array, so the most recently
//! pushed element is the first one to come back out.
//!
//! ```
//! use rust_stl::collections::stack::Stack;
//!
//! let mut stack = Stack::new();
//! stack.push(1);
//! stack.push(2);
//!
//! assert_eq!(stack.top(), Some(&2));
//! assert_eq!(stack.pop(), Some(2));
//! assert_eq!(stack.pop(), Some(1));
//! assert_eq!(stack.pop(), None);
//! ```

use crate::collections::vector::Vector;
use crate::collections::vector::VectorIntoIter;
use crate::collections::vector::VectorIter;
use crate::collections::vector::VectorIterMut;
use std::fmt::Debug;
use std::iter::Extend;
use std::iter::FromIterator;

/// A growable last-in, first-out (LIFO) stack.
///
/// `Stack<T>` stores its elements in a [`Vector<T>`](crate::collections::vector::Vector)
/// and only ever exposes the **top** of that vector — the most recently pushed
/// element. [`push`](Stack::push) adds to the top, [`pop`](Stack::pop) removes
/// from the top, and [`top`](Stack::top) / [`top_mut`](Stack::top_mut) inspect
/// it.
///
/// # LIFO ordering
///
/// Elements come back out in the reverse of the order they went in:
///
/// ```
/// use rust_stl::collections::stack::Stack;
///
/// let mut stack = Stack::new();
/// for n in 1..=3 {
///     stack.push(n);
/// }
///
/// let mut popped = Vec::new();
/// while let Some(value) = stack.pop() {
///     popped.push(value);
/// }
/// assert_eq!(popped, [3, 2, 1]);
/// ```
///
/// An empty stack is a valid state: [`pop`](Stack::pop) returns `None` and
/// [`top`](Stack::top) / [`top_mut`](Stack::top_mut) return `None` rather than
/// panicking.
///
/// # Complexity
///
/// | Operation | Average | Worst case |
/// |---|---|---|
/// | [`push`](Stack::push) | O(1) | O(`len`) when the capacity is exceeded |
/// | [`pop`](Stack::pop) | O(1) | O(1) |
/// | [`top`](Stack::top) / [`top_mut`](Stack::top_mut) | O(1) | O(1) |
/// | [`clear`](Stack::clear) | O(`len`) | O(`len`) |
///
/// Reallocation and element-management details live in the backing
/// [`Vector<T>`](crate::collections::vector::Vector); the stack adds only
/// constant-time forwarding.
///
/// # Relationship to `Vector`
///
/// `Stack<T>` does **not** duplicate any of `Vector`'s memory-management code;
/// every method forwards to the backing vector. Because the stack is only an
/// adaptor, it deliberately does not offer the whole of `Vector`'s API — there
/// is no indexing operator, no insertion in the middle and no iteration over
/// arbitrary positions. The [`get`](Stack::get) and [`get_mut`](Stack::get_mut)
/// accessors are the deliberate exception; see their documentation.
///
/// # Examples
///
/// The three accessors on a one-element stack:
///
/// ```
/// use rust_stl::collections::stack::Stack;
///
/// let mut stack = Stack::new();
/// stack.push("first".to_string());
///
/// assert_eq!(stack.len(), 1);
/// assert_eq!(stack.top(), Some(&"first".to_string()));
///
/// // Mutating the top does not remove it.
/// stack.top_mut().unwrap().push_str("!");
/// assert_eq!(stack.top(), Some(&"first!".to_string()));
///
/// assert_eq!(stack.pop(), Some("first!".to_string()));
/// assert!(stack.is_empty());
/// ```
///
/// Building a stack from an iterator:
///
/// ```
/// use rust_stl::collections::stack::Stack;
///
/// let stack: Stack<i32> = (0..4).collect();
/// assert_eq!(stack.len(), 4);
/// assert_eq!(stack.top(), Some(&3));
/// ```
pub struct Stack<T> {
    data: Vector<T>,
}

impl<T> Stack<T> {
    /// Creates an empty stack.
    ///
    /// No allocation is performed; the first [`push`](Stack::push) triggers the
    /// first allocation. Use [`with_capacity`](Stack::with_capacity) when the
    /// final size is known in advance.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_stl::collections::stack::Stack;
    ///
    /// let stack: Stack<i32> = Stack::new();
    /// assert!(stack.is_empty());
    /// assert_eq!(stack.len(), 0);
    /// ```
    pub fn new() -> Self {
        Self {
            data: Vector::new(),
        }
    }

    /// Pushes `value` onto the top of the stack.
    ///
    /// `value` is moved into the stack and becomes the new
    /// [`top`](Stack::top). This is O(1) amortized: an individual push that
    /// exceeds the capacity reallocates, which is O(`len`), but the capacity
    /// doubles so the cost averages out.
    ///
    /// # Panics
    ///
    /// Panics if the additional element would overflow the stack's maximum
    /// length.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_stl::collections::stack::Stack;
    ///
    /// let mut stack = Stack::new();
    /// stack.push(10);
    /// stack.push(20);
    ///
    /// assert_eq!(stack.top(), Some(&20));
    /// assert_eq!(stack.len(), 2);
    /// ```
    pub fn push(&mut self, value: T) {
        self.data.push(value);
    }

    /// Removes and returns the element on top of the stack.
    ///
    /// Returns `None` if the stack is empty. Ownership of the returned value
    /// passes to the caller; the stack no longer holds it and will not drop it.
    /// This is O(1) and does not release the backing allocation.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_stl::collections::stack::Stack;
    ///
    /// let mut stack: Stack<i32> = (0..2).collect();
    /// assert_eq!(stack.pop(), Some(1));
    /// assert_eq!(stack.pop(), Some(0));
    /// assert_eq!(stack.pop(), None);
    /// ```
    pub fn pop(&mut self) -> Option<T> {
        self.data.pop()
    }

    /// Returns a shared reference to the element on top of the stack, or `None`
    /// if it is empty.
    ///
    /// The top is the most recently pushed element that has not been popped
    /// since. The borrow keeps the stack available for further reads but not for
    /// mutation until it ends. This is O(1).
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_stl::collections::stack::Stack;
    ///
    /// let mut stack = Stack::new();
    /// assert_eq!(stack.top(), None);
    ///
    /// stack.push(7);
    /// assert_eq!(stack.top(), Some(&7));
    /// ```
    pub fn top(&self) -> Option<&T> {
        self.data.last()
    }

    /// Returns a mutable reference to the element on top of the stack, or `None`
    /// if it is empty.
    ///
    /// This is the way to update the top element in place without removing and
    /// re-pushing it. The stack is borrowed mutably for as long as the returned
    /// reference is alive. This is O(1).
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_stl::collections::stack::Stack;
    ///
    /// let mut stack: Stack<i32> = (0..3).collect();
    /// assert_eq!(stack.top_mut(), Some(&mut 2));
    ///
    /// *stack.top_mut().unwrap() = 20;
    /// assert_eq!(stack.pop(), Some(20));
    /// ```
    pub fn top_mut(&mut self) -> Option<&mut T> {
        self.data.last_mut()
    }

    /// Returns the number of elements on the stack.
    ///
    /// This is O(1).
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_stl::collections::stack::Stack;
    ///
    /// let mut stack = Stack::new();
    /// assert_eq!(stack.len(), 0);
    /// stack.push(1);
    /// assert_eq!(stack.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns `true` if the stack holds no elements.
    ///
    /// This is O(1) and equivalent to `len() == 0`.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Removes every element, dropping each one exactly once.
    ///
    /// The backing allocation is retained, so the stack can be refilled without
    /// reallocating. Call [`shrink_to_fit`](Stack::shrink_to_fit) afterwards to
    /// release it. This is O(`len`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_stl::collections::stack::Stack;
    ///
    /// let mut stack: Stack<i32> = (0..4).collect();
    /// stack.clear();
    ///
    /// assert!(stack.is_empty());
    /// assert_eq!(stack.pop(), None);
    ///
    /// // The stack is reusable after clearing.
    /// stack.push(1);
    /// assert_eq!(stack.top(), Some(&1));
    /// ```
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Creates an empty stack with room for at least `capacity` elements.
    ///
    /// Reserving the expected final size up front avoids the intermediate
    /// reallocations that repeated [`push`](Stack::push)es would otherwise
    /// cause. This is O(1) apart from a single allocation.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_stl::collections::stack::Stack;
    ///
    /// let stack: Stack<i32> = Stack::with_capacity(8);
    /// assert!(stack.is_empty());
    /// assert!(stack.capacity() >= 8);
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vector::with_capacity(capacity),
        }
    }

    /// Swaps the contents of `self` and `other` in O(1).
    ///
    /// The two stacks exchange elements *and* capacity wholesale; individual
    /// elements are not moved or cloned. Note that this differs from
    /// [`Vector::swap`](crate::collections::vector::Vector::swap), which
    /// exchanges two indexed elements of one vector.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_stl::collections::stack::Stack;
    ///
    /// let mut a: Stack<i32> = (0..3).collect();
    /// let mut b: Stack<i32> = (10..12).collect();
    ///
    /// a.swap(&mut b);
    ///
    /// assert_eq!(a.top(), Some(&11));
    /// assert_eq!(b.top(), Some(&2));
    /// ```
    pub fn swap(&mut self, other: &mut Self) {
        std::mem::swap(self, other);
    }

    /// Returns the number of elements the stack can hold without reallocating.
    ///
    /// This is O(1) and is always at least [`len`](Stack::len). The exact value
    /// is an implementation detail of the backing vector and is not part of the
    /// contract. A zero-sized element type reports [`usize::MAX`], because such a
    /// stack never needs to allocate.
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    /// Reserves capacity for at least `additional` more elements.
    ///
    /// The stack may allocate more than requested; `capacity()` afterwards is
    /// guaranteed to be at least `len() + additional`. If the current capacity
    /// already fits, nothing happens. Existing elements are preserved. This is
    /// O(`len`) when it reallocates and O(1) otherwise.
    ///
    /// # Panics
    ///
    /// Panics if `len() + additional` overflows `usize`, or if the resulting
    /// element layout cannot be represented.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_stl::collections::stack::Stack;
    ///
    /// let mut stack: Stack<i32> = Stack::new();
    /// stack.reserve(16);
    /// assert!(stack.capacity() >= 16);
    ///
    /// stack.push(1);
    /// assert_eq!(stack.pop(), Some(1));
    /// ```
    pub fn reserve(&mut self, additional: usize) {
        self.data.reserve(additional);
    }

    /// Reserves capacity for exactly `additional` more elements.
    ///
    /// Unlike [`reserve`](Stack::reserve) this does not over-allocate, which is
    /// useful when the final size is known precisely. Existing elements are
    /// preserved. This is O(`len`) when it reallocates and O(1) otherwise.
    ///
    /// # Panics
    ///
    /// Panics if `len() + additional` or the resulting element layout overflows.
    pub fn reserve_exact(&mut self, additional: usize) {
        self.data.reserve_exact(additional);
    }

    /// Reduces the capacity to the current length.
    ///
    /// For an empty stack this releases the allocation entirely; for a
    /// non-empty stack it keeps every element. This is O(`len`) when it
    /// reallocates and O(1) otherwise, and it is a no-op for zero-sized element
    /// types.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_stl::collections::stack::Stack;
    ///
    /// let mut stack: Stack<i32> = Stack::with_capacity(64);
    /// stack.push(1);
    /// stack.shrink_to_fit();
    ///
    /// assert_eq!(stack.len(), 1);
    /// assert_eq!(stack.top(), Some(&1));
    /// ```
    pub fn shrink_to_fit(&mut self) {
        self.data.shrink_to_fit();
    }

    /// Returns a shared reference to the element at `index`, or `None` if
    /// `index` is out of bounds.
    ///
    /// The stack is ordered from the bottom up: index `0` is the oldest element
    /// and index [`len()`](Stack::len)` - 1` is the
    /// [`top`](Stack::top). This accessor is an escape hatch for inspecting
    /// interior elements; ordinary stack use goes through [`top`](Stack::top)
    /// and [`pop`](Stack::pop). It is O(1).
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_stl::collections::stack::Stack;
    ///
    /// let stack: Stack<i32> = (0..3).collect();
    /// assert_eq!(stack.get(0), Some(&0)); // bottom
    /// assert_eq!(stack.get(2), Some(&2)); // top
    /// assert_eq!(stack.get(3), None);
    /// ```
    pub fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index)
    }

    /// Returns a mutable reference to the element at `index`, or `None` if
    /// `index` is out of bounds.
    ///
    /// As with [`get`](Stack::get), index `0` is the bottom of the stack and
    /// index [`len()`](Stack::len)` - 1` is the [`top`](Stack::top). This is the
    /// mutable form of the escape hatch; prefer
    /// [`top_mut`](Stack::top_mut) for updating the top element. It is O(1).
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_stl::collections::stack::Stack;
    ///
    /// let mut stack: Stack<i32> = (0..3).collect();
    /// *stack.get_mut(0).unwrap() = 100;
    /// assert_eq!(stack.get(0), Some(&100));
    /// ```
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.data.get_mut(index)
    }
}

impl<T> Default for Stack<T> {
    /// Creates an empty stack; equivalent to [`Stack::new`].
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Debug for Stack<T>
where
    T: Debug,
{
    /// Formats the stack as a list from the bottom to the top.
    ///
    /// The order matches [`iter`](IntoIterator) over `&Stack<T>`, so the
    /// [`top`](Stack::top) element is printed last.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.data.iter()).finish()
    }
}

impl<T> Clone for Stack<T>
where
    T: Clone,
{
    /// Returns a deep copy of the stack.
    ///
    /// Every element is cloned, so the clone owns independent values and its own
    /// allocation; mutating one stack does not affect the other.
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}

impl<T> PartialEq for Stack<T>
where
    T: PartialEq,
{
    /// Two stacks are equal when they have the same length and equal elements
    /// in the same bottom-to-top order.
    ///
    /// Capacity is ignored: two stacks with identical contents are equal even if
    /// one reserved more space.
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl<T> Eq for Stack<T> where T: Eq {}

impl<T> Extend<T> for Stack<T> {
    /// Pushes every item produced by `iter` onto the stack, in order.
    ///
    /// The last item yielded ends up on top. See the [`FromIterator`]
    /// implementation to build a stack directly from an iterator.
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.data.extend(iter);
    }
}

impl<T> FromIterator<T> for Stack<T> {
    /// Collects the items of `iter` into a new stack, in order.
    ///
    /// The first item becomes the bottom and the last becomes the top. This is
    /// what allows `(0..4).collect::<Stack<_>>()`.
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut stack = Stack::new();
        stack.extend(iter);
        stack
    }
}

impl<T> IntoIterator for Stack<T> {
    type Item = T;
    type IntoIter = VectorIntoIter<T>;

    /// Consumes the stack, yielding its elements by value from the bottom to the
    /// top.
    ///
    /// The iteration order is the reverse of the pop order: the first value
    /// yielded is the element at the bottom. Because the owned iterator is
    /// double-ended, calling [`rev`](Iterator::rev) yields the elements in pop
    /// order instead.
    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a Stack<T> {
    type Item = &'a T;
    type IntoIter = VectorIter<'a, T>;

    /// Iterates over shared references to the elements from the bottom to the
    /// top, leaving the stack usable afterwards.
    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Stack<T> {
    type Item = &'a mut T;
    type IntoIter = VectorIterMut<'a, T>;

    /// Iterates over mutable references to the elements from the bottom to the
    /// top, allowing in-place mutation.
    fn into_iter(self) -> Self::IntoIter {
        self.data.iter_mut()
    }
}
