use core::{mem::MaybeUninit, ops::Index};

pub struct BufVec<T: Sized + Copy, const N: usize> {
    len: usize,
    buf: [MaybeUninit<T>; N]
}

pub struct BufVecIter<'a, T: Sized + Copy, const N: usize> {
    vec: &'a BufVec<T, N>,
    idx: usize,
}

impl<T: Sized + Copy, const N: usize> BufVec<T, N> {
    pub fn new() -> Self {
        Self {
            len: 0, 
            buf: [MaybeUninit::zeroed(); N],
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    /// Use result to force user to not ignore failure
    pub fn push(&mut self, item: T) -> Result<(), ()> {
        if self.len < N {
            self.buf[self.len] = MaybeUninit::new(item);
            self.len += 1;
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn extend(&mut self, items: &[T]) -> Result<(), ()> {
        for item in items {
            self.push(*item)?;
        }
        Ok(())
    }

    pub fn iter(&self) -> BufVecIter<'_, T, N>{
        BufVecIter {
            vec: self,
            idx: 0,
        }
    }
}

impl<'a, T: Sized + Copy, const N: usize> IntoIterator for &'a BufVec<T, N> {
    type Item = T;
    type IntoIter = BufVecIter<'a, T, N>;

    fn into_iter(self) -> Self::IntoIter {
        BufVecIter {
            vec: self,
            idx: 0,
        }
    }
}

impl<T: Sized + Copy, const N: usize> Index<usize> for BufVec<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.len);

        unsafe { self.buf[index].assume_init_ref() }
    }
}

impl<T: Sized + Copy, const N: usize> Iterator for BufVecIter<'_, T, N> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.vec.len() {
            let item = self.vec[self.idx];
            self.idx += 1;
            Some(item)
        } else {
            None
        }
    }
}
