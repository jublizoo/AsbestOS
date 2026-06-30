use core::mem;

/// We use `RspN` with a 64-bit addr broken into 2 32-bit fields, because using a
/// u64 requires 64-bit alignment, but the RspN's in the TSS are only 32-bit aligned.
/// This could be solved with `#[repr(packed)]`, but taking references to unaligned
/// fields can cause UB. By breaking `RspN` into two u32's, the struct's alignment 
/// becomes 64 bits.
#[repr(C)]
#[derive(Clone, Copy)]
struct UnalignedU64 {
    lo: u32,
    hi: u32,
}

#[repr(C)]
struct IoBitmap {

}

impl IoBitmap {
    fn new() -> Self {
        Self {}
    }
}

impl UnalignedU64 {
    fn from_u64(val: u64) -> Self {
        Self {
            lo: (val & 0xFF) as u32,
            hi: (val >> 32) as u32,
        }
    }

    fn to_u64(self) -> u64 {
        (self.lo as u64) | ((self.hi as u64) << 32)
    }
}

#[repr(C)]
struct Tss {
    reserved_0: [u8; 4],
    rsps: [UnalignedU64; 3],
    reserved_1: [u8; 8],
    ists: [UnalignedU64; 7],
    reserved_2: [u8; 12],
    io_bitmap_offset: u16,
    io_bitmap: IoBitmap,
}

impl Tss {
    fn new() -> Self {
        Self {
            reserved_0: [0; 4],
            rsps: [UnalignedU64::from_u64(0); 3],
            reserved_1: [0; 8],
            ists: [UnalignedU64::from_u64(0); 7],
            reserved_2: [0; 12],
            io_bitmap_offset: mem::offset_of!(Tss, io_bitmap) as u16,
            io_bitmap: IoBitmap::new(),
        }
    }
}
