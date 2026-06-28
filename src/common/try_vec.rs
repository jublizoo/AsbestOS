use core::{ops::Index, slice::SliceIndex};

use alloc::vec::Vec;
use crate::no_alloc;

pub struct TryVec<T> {
    buf: Vec<T>
}

fn test() {
    let mut x = Vec::new();
    x.push(0);
}

type Result = core::result::Result<(), ()>;

impl<T> TryVec<T> {
    pub const fn new() -> Self {
        Self {
            // Will not allocate
            buf: Vec::new()
        }
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn try_push(&mut self, value: T) -> Result {
        if self.buf.try_reserve(1).is_err() {
            return Err(())
        }
        no_alloc!(self.buf.push(value));
        Ok(())
    }

    pub fn iter(&self) -> TryVecIter<T> {
        TryVecIter {
            idx: 0,
            try_vec: self,
        }
    }
}

pub struct TryVecIter<'a, T> {
    idx: usize,
    try_vec: &'a TryVec<T>
}

impl<'a, T> IntoIterator for &'a TryVec<T> {
    type Item = &'a T;
    type IntoIter = TryVecIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> Iterator for TryVecIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.try_vec.len() {
            let item = &self.try_vec.buf[self.idx];
            self.idx += 1;
            Some(item)
        } else {
            None
        }
    }
}

impl<T, I: SliceIndex<[T]>> Index<I> for TryVec<T> {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        &self.buf[index]
    }
}
