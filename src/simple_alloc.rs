use core::alloc::{GlobalAlloc, Layout};
use core::ptr::{null, null_mut};
use core::sync::atomic::AtomicBool;
use crate::spinlock::{SpinLock};
use crate::math::{Alignable};
use crate::common::SyncPtr;

/// A simple test allocator.
///
/// Allocates by incrementing a pointer, never frees allocations.
///
/// Invariant: data at limit is never accessed directly via this struct. It could
/// still be accessed by the caller of `alloc`.
struct PushAllocatorInternal {
    configured: bool,
    /// Next available byte
    current_byte: *const u8,
    end: *const u8,
}

impl PushAllocatorInternal {
    const fn new() -> Self {
        Self {
            configured: false,
            current_byte: core::ptr::null(),
            end: core::ptr::null(),
        }
    }

    unsafe fn configure(&mut self, start: *const u8, end: *const u8) {
        assert!(!self.configured);
        self.current_byte = start;
        self.end = end;
        self.configured = true;
    }

    unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        assert!(self.configured);

        let alignment = layout.align() as *const u8;
        let region_start = self.current_byte.align_up(alignment);
        let region_end = unsafe { region_start.add(layout.size()) };

        if region_end > self.end {
            // Failure
            return null_mut();
        }

        self.current_byte = region_end;
        region_start as *mut u8
    }
}

pub struct PushAllocator(SpinLock<PushAllocatorInternal>);

unsafe impl Send for PushAllocator {}
unsafe impl Sync for PushAllocator {}

impl PushAllocator {
    const fn new() -> Self {
        Self(SpinLock::new(PushAllocatorInternal::new()))
    }

    /// SAFETY: 
    pub unsafe fn configure(&self, start: *const u8, end: *const u8) {
        let mut allocator = self.0.lock();
        unsafe { allocator.configure(start, end) }
    }
}

unsafe impl GlobalAlloc for PushAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { self.0.lock().alloc(layout) }
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
pub static ALLOCATOR: PushAllocator = PushAllocator::new();

#[alloc_error_handler]
fn alloc_error(_layout: core::alloc::Layout) -> ! {
    panic!("OOM");
}
