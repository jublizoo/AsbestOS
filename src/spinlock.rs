use core::ops::{Deref, DerefMut};
use core::sync::atomic::{Ordering::Acquire, AtomicBool};
use core::cell::UnsafeCell;

// TODO: Check cache consistency validity of Spinlock

pub struct SpinLock<T> {
    taken: AtomicBool,
    inner: UnsafeCell<T>,
}

pub struct SpinGuard<'a, T> {
    lock: &'a SpinLock<T>,
}

impl<T> SpinLock<T> {
    pub const fn new(inner: T) -> Self {
        Self {
            taken: AtomicBool::new(false),
            inner: UnsafeCell::new(inner),
        }
    }

    pub fn try_lock(&self) -> Option<SpinGuard<'_, T>> {
        if self.taken.compare_exchange(false, true, Acquire, Acquire).is_err() {
            return None;
        }

        Some(SpinGuard {
            lock: self
        })
    }

    pub fn lock(&self) -> SpinGuard<'_, T> {
        while self.taken.compare_exchange(false, true, Acquire, Acquire)
            .is_err() { }

        SpinGuard {
            lock: self
        }
    }

    /// VERY VERY UNSAFE. IF YOU NEED THIS, IT IS PROBABLY UB. ONLY USE FOR PANICS
    pub unsafe fn force_lock(&self) -> SpinGuard<'_, T> {
        SpinGuard {
            lock: self
        }
    }

    fn expect_lock(&self) -> SpinGuard<'_, T> {
        self.taken.compare_exchange(false, true, Acquire, Acquire)
            .unwrap();

        SpinGuard {
            lock: self
        }
    }

    unsafe fn unlock(&self) {
        self.taken.compare_exchange(true, false, Acquire, Acquire)
            .unwrap();
    }

}

impl<'a, T> Drop for SpinGuard<'a, T> {
    fn drop(&mut self) {
        unsafe { self.lock.unlock(); }
    }
}

impl<'a, T> Deref for SpinGuard<'a ,T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.inner.get() }
    }
}

impl<'a, T> DerefMut for SpinGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.inner.get() }
    }
}

unsafe impl<T: Send> Send for SpinLock<T> {}
unsafe impl<T: Send> Sync for SpinLock<T> {}
