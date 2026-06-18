pub mod flags;
pub mod buf_vec;
pub mod region;

pub fn magic_break() { }

/// A sync ptr is a wrapper around a const ptr, which is Send + Sync. This is 
/// done safely by making it unsafe to get the inner field.
/// 
/// This struct is best used as an "opaque ptr", which is never dereferenced.
pub struct SyncPtr<T>(*const T);

impl<T> SyncPtr<T> {
    pub const fn new(ptr: *const T) -> Self {
        Self(ptr)
    }

    /// SAFETY: Must ensure no race conditions when accessing data stored at ptr.
    /// This function is always safe if the returned pointer is never dereferenced.
    pub unsafe fn get(&self) -> *const T {
        self.0
    }
}

unsafe impl<T> Send for SyncPtr<T> {}
unsafe impl<T> Sync for SyncPtr<T> {}

pub struct SyncMutPtr<T>(*mut T);

impl<T> SyncMutPtr<T> {
    pub const fn new(ptr: *mut T) -> Self {
        Self(ptr)
    }

    /// SAFETY: Must ensure no race conditions when accessing data stored at ptr.
    /// This function is always safe if the returned pointer is never dereferenced.
    pub unsafe fn get(&self) -> *mut T {
        self.0
    }
}

unsafe impl<T> Send for SyncMutPtr<T> {}
unsafe impl<T> Sync for SyncMutPtr<T> {}
