#![deny(missing_docs)]

//! Shrinks slice references
//!
//! `ResizeSlice` can be used to adjust the starting offset and length of a slice.
//!
//! ## Example
//!
//! ```
//! use resize_slice::ResizeSlice;
//!
//! let mut slice: &mut [_] = &mut [1, 2, 3, 4, 5];
//! slice.resize_from(2);
//!
//! assert_eq!(slice, &mut [3, 4, 5]);
//! ```

extern crate uninitialized;

use std::mem::replace;
use uninitialized::UNINITIALIZED;
use std::ptr::write_bytes;

/// Extension trait that allows you to resize mutable slice references
pub trait ResizeSlice {
    /// Resizes the slice to `start` offset and `end - start` len
    ///
    /// # Panics
    ///
    /// Panics on out of bounds resize (`start <= end <= self.len()`)
    fn resize(&mut self, start: usize, end: usize);

    /// Resize to a new beginning offset
    ///
    /// # Panics
    ///
    /// Panics if `start > self.len()`
    fn resize_from(&mut self, start: usize);

    /// Resize to a new length
    ///
    /// # Panics
    ///
    /// Panics if `end > self.len()`
    fn resize_to(&mut self, end: usize);
}

impl<'a, T> ResizeSlice for &'a mut [T] {
    #[inline]
    fn resize(&mut self, start: usize, end: usize) {
        assert!(start <= end && end <= self.len());
        let mut value = replace(self, &mut []);
        value = &mut unsafe { value.get_unchecked_mut(start..end) };
        replace(self, value);
    }

    #[inline]
    fn resize_from(&mut self, start: usize) {
        let len = self.len();
        self.resize(start, len);
    }

    #[inline]
    fn resize_to(&mut self, end: usize) {
        self.resize(0, end)
    }
}

impl<'a, T> ResizeSlice for &'a [T] {
    #[inline]
    fn resize(&mut self, start: usize, end: usize) {
        *self = &self[start..end];
    }

    #[inline]
    fn resize_from(&mut self, start: usize) {
        *self = &self[start..];
    }

    #[inline]
    fn resize_to(&mut self, end: usize) {
        *self = &self[..end];
    }
}

/// Extension methods for vector types
pub trait VecExt<T> {
    /// Unsafely resize a vector to the specified size, without initializing the memory.
    unsafe fn uninitialized_resize(&mut self, new_len: usize);

    /// Unsafely resize a vector to the specified size, zeroing the memory.
    unsafe fn zeroed_resize(&mut self, new_len: usize);
}

/// Extension methods for slices
pub trait SliceExt<T> {
    /// Copies the less of `self.len()` and `src.len()` from `src` into `self`,
    /// returning the amount of items copied.
    fn copy_from(&mut self, src: &[T]) -> usize where T: Copy;

    /// Copies elements to another location within the slice, which may overlap.
    fn copy_inner(&mut self, src: usize, dst: usize, len: usize) where T: Copy;
}

impl<T> SliceExt<T> for [T] {
    #[inline]
    fn copy_from(&mut self, src: &[T]) -> usize where T: Copy {
        use std::ptr::copy_nonoverlapping;
        use std::cmp::min;

        let len = min(self.len(), src.len());
        unsafe {
            copy_nonoverlapping(src.as_ptr(), self.as_mut_ptr(), len);
        }
        len
    }

    #[inline]
    fn copy_inner(&mut self, src: usize, dst: usize, len: usize) where T: Copy {
        use std::ptr::copy;
        assert!(self.len() - len >= src && self.len() - len >= dst);

        unsafe {
            copy(self.as_ptr().offset(src as isize), self.as_mut_ptr().offset(dst as isize), len);
        }
    }
}

impl<T> VecExt<T> for Vec<T> {
    #[inline]
    unsafe fn uninitialized_resize(&mut self, new_len: usize) {
        let len = self.len();
        if new_len > len {
            self.reserve_exact(new_len - len);
        }
        self.set_len(new_len);
    }

    #[inline]
    unsafe fn zeroed_resize(&mut self, new_len: usize) {
        self.uninitialized_resize(new_len);
        if !UNINITIALIZED {
            write_bytes(self.as_mut_ptr(), 0, new_len);
        }
    }
}

#[cfg(feature = "smallvec")]
mod smallvec_impl {
    extern crate smallvec;
    use self::smallvec::{SmallVec, Array};

    use std::ptr::write_bytes;
    use uninitialized::UNINITIALIZED;
    use super::VecExt;

    impl<T: Array> VecExt<T::Item> for SmallVec<T> {
        #[inline]
        unsafe fn uninitialized_resize(&mut self, new_len: usize) {
            let len = self.len();
            if new_len > len {
                self.reserve_exact(new_len - len);
            }
            self.set_len(new_len);
        }

        #[inline]
        unsafe fn zeroed_resize(&mut self, new_len: usize) {
            self.uninitialized_resize(new_len);
            if !UNINITIALIZED {
                write_bytes(self.as_mut_ptr(), 0, new_len);
            }
        }
    }
}

#[test]
fn resize() {
    let mut s: &mut [_] = &mut [1, 2, 3];
    assert_eq!(s.len(), 3);

    s.resize_from(1);
    assert_eq!(s.len(), 2);

    s.resize_to(1);
    assert_eq!(s.len(), 1);
    assert_eq!(s[0], 2);

    s.resize(1, 1);
    assert_eq!(s.len(), 0);
}

#[test]
#[should_panic]
fn resize_fail() {
    let mut s: &mut [_] = &mut [1, 2, 3];
    s.resize_to(4);
}
