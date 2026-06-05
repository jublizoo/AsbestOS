use core::alloc::{GlobalAlloc, Layout};
use core::ptr::{null, null_mut};
use crate::spinlock::{SpinLock};
use crate::math::{Alignable};

/// A simple test allocator.
///
/// Allocates by incrementing a pointer, never frees allocations.
///
/// Invariant: data at limit is never accessed directly via this struct. It could
/// still be accessed by the caller of `alloc`.
struct PushAllocator {
    /// Next available
    current: SpinLock<*const u8>,
    /// Last available byte
    end: *const u8,
}

/// SAFETY: Data at `limit` is never directly accessed, and `current` is logically `Send + Sync`
unsafe impl Send for PushAllocator {}
unsafe impl Sync for PushAllocator {}

impl PushAllocator {
    const fn new(start: *const u8, end: *const u8) -> Self {
        Self {
            current: SpinLock::new(start),
            end,
        }
    }
}

unsafe impl GlobalAlloc for PushAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut current = self.current.lock();
        let alignment = layout.align() as *const u8;
        let start = current.align_up(alignment);
        let lim = unsafe { start.add(layout.size()) };
        let end = unsafe { lim.sub(1) };

        // Failure
        if end > self.end {
            return null_mut();
        }

        *current = lim;
        start as *mut u8

    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) { }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let start = unsafe { self.alloc(layout) };
        if start.is_null() { return null_mut(); }
        unsafe { core::ptr::write_bytes(start, 0, layout.size()) };
        start
    }
}


#[global_allocator]
static ALLOCATOR: PushAllocator = PushAllocator::new(null(), 10000 as *const u8);

#[alloc_error_handler]
fn alloc_error(_layout: core::alloc::Layout) -> ! {
    panic!("OOM");
}
