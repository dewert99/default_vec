use crate::default_vec::DefaultVec;
use core::fmt::{Debug, Formatter};
use core::iter;
use core::marker::PhantomData;
use core::ops::{BitAndAssign, BitOrAssign, BitXorAssign, SubAssign};

type Elt = u32;

/// A simple unbounded bitset that fits in 2 `usize`s worth of memory
///
/// It resizes its heap allocation whenever a number that wouldn't otherwise fit in memory is added
/// and doesn't ever shrink its memory so it could end of wasting memory if a very large element
/// is added and then removed
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct BitSet<I = usize>(DefaultVec<Elt>, PhantomData<I>);

impl<I> Default for BitSet<I> {
    fn default() -> Self {
        BitSet(DefaultVec::default(), PhantomData)
    }
}

impl<I> Clone for BitSet<I> {
    fn clone(&self) -> Self {
        BitSet(self.0.clone(), PhantomData)
    }

    fn clone_from(&mut self, source: &Self) {
        self.0.clone_from(&source.0)
    }
}

impl<I> PartialEq<Self> for BitSet<I> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<I> Eq for BitSet<I> {}

#[inline]
fn split(x: usize) -> (usize, Elt, u32) {
    let offset = (x % Elt::BITS as usize) as u32;
    (x / Elt::BITS as usize, 1 << offset, offset)
}

impl<I: Into<usize>> BitSet<I> {
    /// Adds an element to the set
    ///
    /// If the set did not have this value present, true is returned.
    ///
    /// If the set did have this value present, false is returned.
    ///
    /// ```rust
    /// use default_vec2::BitSet;
    /// let mut s: BitSet<usize> = BitSet::default();
    /// assert!(s.insert(0));
    /// assert!(!s.insert(0));
    /// ```
    pub fn insert(&mut self, x: I) -> bool {
        let (chunk_idx, mask, _) = split(x.into());
        let chunk = self.0.get_mut(chunk_idx);
        let res = (*chunk & mask) == 0;
        *chunk |= mask;
        res
    }

    /// Removes an element form the set.
    ///
    /// Returns whether the value was present in the set.
    ///
    /// ```rust
    /// use default_vec2::BitSet;
    /// let mut s: BitSet<usize> = BitSet::default();
    /// assert!(!s.remove(0));
    /// s.insert(0);
    /// assert!(s.remove(0));
    /// assert!(!s.remove(0))
    /// ```
    pub fn remove(&mut self, x: I) -> bool {
        let (chunk_idx, mask, _) = split(x.into());
        let chunk = self.0.get_mut(chunk_idx);
        let res = (*chunk & mask) != 0;
        *chunk &= !mask;
        res
    }

    /// Inserts `x` if `v` is true, or removes it otherwise.
    ///
    /// Returns whether `self` used to contain `x`
    ///
    /// ```rust
    /// use default_vec2::BitSet;
    /// let mut s: BitSet<usize> = BitSet::default();
    /// assert!(!s.set(0, false));
    /// assert!(!s.set(0, true));
    /// assert!(s.set(0, true));
    /// assert!(s.set(0, false));
    /// ```
    pub fn set(&mut self, x: I, v: bool) -> bool {
        let (chunk_idx, mask, shift) = split(x.into());
        let chunk = self.0.get_mut(chunk_idx);
        let res = (*chunk & mask) != 0;
        *chunk &= !mask;
        *chunk |= (v as Elt) << shift;
        res
    }

    /// Checks if the set contains an element
    pub fn contains(&self, x: I) -> bool {
        let (chunk_idx, mask, _) = split(x.into());
        let chunk = self.0.get(chunk_idx);
        (chunk & mask) != 0
    }

    /// Same as contains but already reserves space for `x`
    pub fn contains_mut(&mut self, x: I) -> bool {
        let (chunk_idx, mask, _) = split(x.into());
        let chunk = *self.0.get_mut(chunk_idx);
        (chunk & mask) != 0
    }

    /// Removes all elements from the set
    pub fn clear(&mut self) {
        self.0.clear()
    }
}

impl<I: From<usize> + Into<usize> + Copy> BitSet<I> {
    /// Iterate over all elements in the set
    ///
    /// Run time is proportional to the largest element that has ever been in the set
    pub fn iter(&self) -> impl Iterator<Item = I> + '_ {
        let max = self.0.capacity() * (Elt::BITS as usize);
        (0..max).map(I::from).filter(|x| self.contains(*x))
    }
}

impl<'a, I> BitAndAssign<&'a BitSet> for BitSet<I> {
    /// Sets `*self` to the intersection of `self` and `other`
    ///
    /// ### Example:
    /// ```
    /// use default_vec2::BitSet;
    /// let mut s1: BitSet<usize> = BitSet::from_iter([0, 1]);
    /// let s2 = BitSet::from_iter([0, 42]);
    /// s1 &= &s2;
    ///
    /// assert_eq!(vec![0], s1.iter().collect::<Vec<_>>());
    /// ```
    fn bitand_assign(&mut self, rhs: &'a BitSet) {
        for (this, other) in self
            .0
            .iter_mut()
            .zip(rhs.0.iter().copied().chain(iter::repeat(0)))
        {
            *this &= other
        }
    }
}

impl<'a, I> BitOrAssign<&'a BitSet> for BitSet<I> {
    /// Sets `*self` to the intersection of `self` and `other`
    ///
    /// ### Example:
    /// ```
    /// use default_vec2::BitSet;
    /// let mut s1: BitSet<usize> = BitSet::from_iter([0, 1]);
    /// let s2 = BitSet::from_iter([0, 42]);
    /// s1 |= &s2;
    ///
    /// assert_eq!(vec![0, 1, 42], s1.iter().collect::<Vec<_>>());
    /// ```
    fn bitor_assign(&mut self, rhs: &'a BitSet) {
        if rhs.0.capacity() > self.0.capacity() {
            self.0.reserve(rhs.0.capacity())
        }
        for (this, other) in self.0.iter_mut().zip(rhs.0.iter().copied()) {
            *this |= other
        }
    }
}

impl<'a, I> SubAssign<&'a BitSet> for BitSet<I> {
    /// Sets `*self` to the set difference of `self` and `other`
    ///
    /// ### Example:
    /// ```
    /// use default_vec2::BitSet;
    /// let mut s1: BitSet<usize> = BitSet::from_iter([0, 1]);
    /// let s2 = BitSet::from_iter([0, 42]);
    /// s1 -= &s2;
    ///
    /// assert_eq!(vec![1], s1.iter().collect::<Vec<_>>());
    /// ```
    fn sub_assign(&mut self, rhs: &'a BitSet) {
        for (this, other) in self.0.iter_mut().zip(rhs.0.iter().copied()) {
            *this &= !other
        }
    }
}

impl<'a, I> BitXorAssign<&'a BitSet> for BitSet<I> {
    /// Sets `*self` to the set symmetric difference of `self` and `other`
    ///
    /// ### Example:
    /// ```
    /// use default_vec2::BitSet;
    /// let mut s1: BitSet<usize> = BitSet::from_iter([0, 1]);
    /// let s2 = BitSet::from_iter([0, 42]);
    /// s1 ^= &s2;
    ///
    /// assert_eq!(vec![1, 42], s1.iter().collect::<Vec<_>>());
    /// ```
    fn bitxor_assign(&mut self, rhs: &'a BitSet) {
        if rhs.0.capacity() > self.0.capacity() {
            self.0.reserve(rhs.0.capacity())
        }
        for (this, other) in self.0.iter_mut().zip(rhs.0.iter().copied()) {
            *this ^= other
        }
    }
}

impl<I: Into<usize>> Extend<I> for BitSet<I> {
    fn extend<T: IntoIterator<Item = I>>(&mut self, iter: T) {
        iter.into_iter().for_each(|x| {
            self.insert(x);
        })
    }
}

impl<I: Into<usize>> FromIterator<I> for BitSet<I> {
    fn from_iter<T: IntoIterator<Item = I>>(iter: T) -> Self {
        let mut res = Self::default();
        res.extend(iter);
        res
    }
}

impl<I: From<usize> + Into<usize> + Copy + Debug> Debug for BitSet<I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}
