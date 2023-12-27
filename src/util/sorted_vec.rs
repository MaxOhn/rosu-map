use std::{
    cmp::Ordering,
    fmt::{Debug, Formatter, Result as FmtResult},
    ops::{Deref, Index},
    slice::SliceIndex,
    vec::IntoIter,
};

/// A [`Vec`] whose elements are guaranteed to be unique and in order.
#[derive(Clone)]
pub struct SortedVec<T> {
    inner: Vec<T>,
}

impl<T> SortedVec<T> {
    /// Constructs a new, empty `SortedVec<T>`.
    pub const fn new() -> Self {
        Self { inner: Vec::new() }
    }

    /// Constructs a new, empty `SortedVec<T>` with at least the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
        }
    }

    /// Extracts the inner [`Vec`].
    pub fn into_inner(self) -> Vec<T> {
        self.inner
    }

    /// Returns a mutable reference to the underlying `Vec`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the items stay in order.
    pub unsafe fn as_inner_mut(&mut self) -> &mut Vec<T> {
        &mut self.inner
    }

    /// Removes the last element and returns it, or `None` if the vec is empty.
    pub fn pop(&mut self) -> Option<T> {
        self.inner.pop()
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all items `i` for which `f(&i)` returns `false`.
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.inner.retain(f);
    }
}

impl<T: Sortable> SortedVec<T> {
    /// Same as [`slice::binary_search_by`] with the function
    /// [`<T as Sortable>::cmp`](Sortable::cmp).
    pub fn find(&self, value: &T) -> Result<usize, usize> {
        self.inner
            .binary_search_by(|probe| <T as Sortable>::cmp(probe, value))
    }

    /// Push a new value into the sorted list based on [`<T as Sortable>::push`](Sortable::push).
    pub fn push(&mut self, value: T) {
        <T as Sortable>::push(value, self);
    }
}

impl<T> Deref for SortedVec<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T, I> Index<I> for SortedVec<T>
where
    I: SliceIndex<[T]>,
{
    type Output = <I as SliceIndex<[T]>>::Output;

    fn index(&self, index: I) -> &Self::Output {
        <Vec<T> as Index<I>>::index(&self.inner, index)
    }
}

impl<T: Debug> Debug for SortedVec<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        <Vec<T> as Debug>::fmt(&self.inner, f)
    }
}

impl<T: PartialEq> PartialEq for SortedVec<T> {
    fn eq(&self, other: &Self) -> bool {
        <Vec<T> as PartialEq>::eq(&self.inner, other)
    }
}

impl<T> Default for SortedVec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Sortable> From<Vec<T>> for SortedVec<T> {
    fn from(mut v: Vec<T>) -> Self {
        v.sort_by(<T as Sortable>::cmp);
        v.dedup_by(|a, b| {
            <T as Sortable>::cmp(a, b) == Ordering::Equal || <T as Sortable>::is_redundant(b, a)
        });

        Self { inner: v }
    }
}

impl<T: Sortable> FromIterator<T> for SortedVec<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::from(Vec::from_iter(iter))
    }
}

impl<T: Sortable> Extend<T> for SortedVec<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for value in iter {
            self.push(value);
        }
    }
}

/// Trait for types that can be sorted in a [`SortedVec`].
pub trait Sortable: Sized {
    /// An [`Ordering`] between `self` and `other`.
    fn cmp(&self, other: &Self) -> Ordering;

    /// Indicates whether `self` and `existing` are identical.
    #[allow(unused_variables)]
    fn is_redundant(&self, existing: &Self) -> bool {
        false
    }

    /// Pushes a value into the [`SortedVec`].
    fn push(self, sorted_vec: &mut SortedVec<Self>) {
        match sorted_vec.find(&self) {
            Ok(i) => sorted_vec.inner[i] = self,
            Err(i) if i == sorted_vec.len() => sorted_vec.inner.push(self),
            Err(i) => sorted_vec.inner.insert(i, self),
        }
    }
}

impl<T: Ord> Sortable for T {
    fn cmp(&self, other: &Self) -> Ordering {
        <Self as Ord>::cmp(self, other)
    }
}

impl<T> IntoIterator for SortedVec<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::SortedVec;

    #[test]
    fn sorts_on_push() {
        let mut v = SortedVec::with_capacity(4);

        v.push(42);
        v.push(13);
        v.push(20);
        v.push(0);

        assert_eq!(v.as_slice(), &[0_i32, 13, 20, 42]);
    }
}
