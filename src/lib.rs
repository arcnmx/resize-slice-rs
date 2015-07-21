#![deny(missing_docs)]
#![cfg_attr(feature = "unstable", feature(raw))]

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

use std::mem::transmute;

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

fn slice_resize<T>(slice: &mut Slice<T>, start: usize, end: usize) {
    assert!(start <= end && end <= slice.len);

    slice.data = unsafe { slice.data.offset(start as isize) };
    slice.len = end - start;
}

impl<'a, T> ResizeSlice for &'a mut [T] {
    fn resize(&mut self, start: usize, end: usize) {
        unsafe {
            slice_resize::<T>(transmute(self), start, end);
        }
    }

    fn resize_from(&mut self, start: usize) {
        let len = self.len();
        self.resize(start, len);
    }

    fn resize_to(&mut self, end: usize) {
        self.resize(0, end)
    }
}

impl<'a, T> ResizeSlice for &'a [T] {
    fn resize(&mut self, start: usize, end: usize) {
        unsafe {
            slice_resize::<T>(transmute(self), start, end);
        }
    }

    fn resize_from(&mut self, start: usize) {
        let len = self.len();
        self.resize(start, len);
    }

    fn resize_to(&mut self, end: usize) {
        self.resize(0, end)
    }
}

#[cfg(feature = "unstable")]
use std::raw::Slice;

#[cfg(not(feature = "unstable"))]
struct Slice<T> {
    data: *const T,
    len: usize,
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
