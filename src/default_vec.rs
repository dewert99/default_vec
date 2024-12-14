use alloc::boxed::Box;
use core::fmt::Debug;
use core::marker::PhantomData;
use core::ops::{Index, IndexMut};
use core::{mem, slice};
#[cfg(feature = "serde-1")]
use serde::{Deserialize, Serialize};

/// A mapping from indexes to values where all indexes initially map to [`Default::default`]
///
/// It is stored in 2 `usize`s worth of memory since it doesn't need to store the length.
///
/// It resizes its heap allocation whenever an element that wouldn't otherwise fit in memory is added
/// and doesn't ever shrink its memory so it could end of wasting memory if an element is added with
/// a large index and then removed
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde-1", derive(Serialize, Deserialize))]
pub struct DefaultVec<T, I: Into<usize> = usize>(Box<[T]>, PhantomData<I>);

pub trait ConstDefault: Default + 'static {
    /// Constant version of default value
    const DEFAULT: &'static Self;
}

impl<T: Default, I: Into<usize>> DefaultVec<T, I> {
    #[cold]
    #[inline(never)]
    pub(super) fn reserve(&mut self, i: usize) {
        let mut v = mem::take(&mut self.0).into_vec();
        v.reserve(i + 1 - v.len());
        v.resize_with(v.capacity(), T::default);
        self.0 = v.into_boxed_slice();
        assert!(i < self.0.len())
    }

    /// Returns mutable access to the element at `i`
    pub fn get_mut(&mut self, i: I) -> &mut T {
        let i: usize = i.into();
        if i < self.0.len() {
            &mut self.0[i]
        } else {
            self.reserve(i);
            &mut self.0[i]
        }
    }

    /// Returns shared access to the element at `i`
    pub fn get(&self, i: I) -> T
    where
        T: Copy,
    {
        let i: usize = i.into();
        self.0.get(i).copied().unwrap_or_default()
    }

    /// Resets all elements to there default value
    pub fn clear(&mut self) {
        self.0.fill_with(Default::default)
    }

    pub fn capacity(&self) -> usize {
        self.0.len()
    }

    /// Returns an iterator over the elements of this list
    /// the iterator will have `capacity` elements
    pub fn iter(&self) -> slice::Iter<T> {
        self.0.iter()
    }

    /// Returns a mutable iterator over the elements of this list
    /// the iterator will have `capacity` elements
    pub fn iter_mut(&mut self) -> slice::IterMut<T> {
        self.0.iter_mut()
    }
}
impl<T: ConstDefault, I: Into<usize>> Index<I> for DefaultVec<T, I> {
    type Output = T;

    fn index(&self, index: I) -> &Self::Output {
        self.0.get(index.into()).unwrap_or(T::DEFAULT)
    }
}

impl<T: ConstDefault, I: Into<usize>> IndexMut<I> for DefaultVec<T, I> {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.get_mut(index)
    }
}
