#![allow(clippy::upper_case_acronyms)]

use core::mem::MaybeUninit;

use crate::common::flags::{range_mask, write_byte_bitmask, write_byte_flag};

// TODO:
// For large mapped regions, use larger page sizes
// Keep count of pages

const PAGE_SIZE: usize = 4096;
const PAGE_MAP_NUM_ENTIRES: usize = 512;

trait RawEntry: Sized {
    fn from_bits(bits: u64) -> MaybeUninit<Self>;
    fn bits(&self) -> u64;
    fn bits_mut(&mut self) -> &mut u64;
}

trait PageMapEntry: RawEntry {
    const PRESENT: u64 =            1 << 0;
    const RW: u64 =                 1 << 1;
    const USER_SUPER: u64 =         1 << 2;
    const WRITE_THROUGH: u64 =      1 << 3;
    const CACHE_DISABLE: u64 =      1 << 4;
    const ACCESSED: u64 =           1 << 5;

    const ADDR_MASK: u64 = range_mask(12, 50);

    fn new_empty() -> Self {
        unsafe { Self::from_bits(0).assume_init() }
    }

    fn new_raw(
        writable: bool,
        user: bool,
        addr: *const u8,
    ) -> u64 {
        const ENTRY_PRESENT: bool =         true;
        const ENTRY_WRITE_THROUGH: bool =   false;
        const ENTRY_CACHE_DISABLE: bool =   false;
        const ENTRY_ACCESSED: bool =        false;

        let mut entry = 0;
        if writable { entry |= Self::RW; }
        if user { entry |= Self::USER_SUPER; }
        if ENTRY_PRESENT { entry |= Self::PRESENT; }
        if ENTRY_WRITE_THROUGH { entry |= Self::WRITE_THROUGH; }
        if ENTRY_CACHE_DISABLE { entry |= Self::CACHE_DISABLE; }
        if ENTRY_ACCESSED { entry |= Self::ACCESSED; }

        assert!(addr as u64 & Self::ADDR_MASK == 0);
        entry |= addr as u64;

        entry
    }

    fn is_writable(&self) -> bool {
        (self.bits() & Self::RW) != 0
    }

    fn set_writable(&self, writable: bool) {
        *self.bits_mut() = write_byte_bitmask(self.bits(), Self::RW, writable);
    }

    fn is_accessed(&self) -> bool {
        (self.bits() & Self::ACCESSED) != 0
    }

    fn get_addr(&self) -> u64 {
        self.bits() & Self::ADDR_MASK
    }

    fn set_addr(&mut self, addr: *const u8) {
        assert!((addr as u64) & !Self::ADDR_MASK == 0);

        *self.bits_mut() &= !Self::ADDR_MASK;
        *self.bits_mut() |= addr as u64;
    }
}

/// For higher-level page maps (all but page-tables)
///
/// Reuse code for ignored bits, as all higher-level page maps share the same
/// ignored bit ranges.
trait HigherPageMapEntry: RawEntry {
    const IGNORED_BITS: (u8, u8) = (52, 62);
    const IGNORED_MASK: u64 = range_mask(52, 62);
    const IGNORED_OFFSET: u8 = 52;

    fn get_ignored_bits(&self) -> u64 {
        (self.bits() & Self::IGNORED_MASK) << Self::IGNORED_OFFSET
    }

    fn set_ignored_bits(&mut self, bits: u16) {
        let positioned_bits = (bits as u64) << Self::IGNORED_OFFSET;
        assert!(positioned_bits & Self::IGNORED_MASK == 0);
        *self.bits_mut() &= !(Self::IGNORED_MASK);
        *self.bits_mut() |= positioned_bits;
    }
}

impl<T: RawEntry> PageMapEntry for T { }



struct PML4E(u64);
type PML4T = [PML4E; PAGE_MAP_NUM_ENTIRES];

impl RawEntry for PML4E {
    fn from_bits(bits: u64) -> MaybeUninit<Self> { MaybeUninit::new(Self(bits)) }
    fn bits(&self) -> u64 { self.0 }
    fn bits_mut(&mut self) -> &mut u64 { &mut self.0 }
}

impl HigherPageMapEntry for PML4E { }

impl PML4E {
    fn new(
        writable: bool,
        user: bool,
        addr: *const u8,
    ) -> Self {
        Self(Self::new_raw(writable, user, addr))
    }
}



struct PDPTE(u64);
type PDPT = [PDPTE; PAGE_MAP_NUM_ENTIRES];

impl RawEntry for PDPTE {
    fn from_bits(bits: u64) -> MaybeUninit<Self> { MaybeUninit::new(Self(bits)) }
    fn bits(&self) -> u64 { self.0 }
    fn bits_mut(&mut self) -> &mut u64 { &mut self.0 }
}

impl HigherPageMapEntry for PDPTE { }

impl PDPTE {
    fn new(
        writable: bool,
        user: bool,
        addr: *const u8,
    ) -> Self {
        Self(Self::new_raw(writable, user, addr))
    }
}



struct PDE(u64);
type PD = [PDE; PAGE_MAP_NUM_ENTIRES];

impl RawEntry for PDE {
    fn from_bits(bits: u64) -> MaybeUninit<Self> { MaybeUninit::new(Self(bits)) }
    fn bits(&self) -> u64 { self.0 }
    fn bits_mut(&mut self) -> &mut u64 { &mut self.0 }
}

impl HigherPageMapEntry for PDE { }

impl PDE {
    fn new(
        writable: bool,
        user: bool,
        addr: *const u8,
    ) -> Self {
        Self(Self::new_raw(writable, user, addr))
    }
}



struct PTE(u64);
type PT = [PTE; PAGE_MAP_NUM_ENTIRES];

impl RawEntry for PTE {
    fn from_bits(bits: u64) -> MaybeUninit<Self> { MaybeUninit::new(Self(bits)) }
    fn bits(&self) -> u64 { self.0 }
    fn bits_mut(&mut self) -> &mut u64 { &mut self.0 }
}

impl PTE {
    const DIRTY: u64 = 1 << 6;
    const GLOBAL: u64 = 1 << 8;

    /// Mask for first range of ignored bits
    const IGNORED_MASK_1: u64 = range_mask(52, 58);
    /// Offset for first range of ignored bits
    const IGNORED_OFFSET_1: u8 = 52;
    /// Mask for second range of ignored bits
    const IGNORED_MASK_2: u64 = range_mask(9, 11);
    /// Offset for second range of ignored bits, relative to offset in combined u16
    const IGNORED_RELATIVE_OFFSET_2: u8 = 9;


    fn new(
        writable: bool,
        user: bool,
        addr: *const u8,
        global: bool,
    ) -> Self {
        const ENTRY_DIRTY: bool = false;
        static X: bool = false;

        let mut entry = Self::new_raw(writable, user, addr);
        if global { entry |= Self::GLOBAL; }
        if ENTRY_DIRTY { entry |= Self::DIRTY; }

        Self(entry)
    }

    /// This method is named with an 'all', because unlike the other page maps, we need to combine
    /// multiple ranges of bits in order to use 9+ ignored bits (needed to keep track of the up to 
    /// 2^9=512 potential present entries in a page map). In other page maps, a single range has at
    /// least 9 bits, so we don't use the other ignored bits.
    fn get_all_ignored_bits(&self) -> u16 {
        let bits_1 = Self::IGNORED_MASK_1 & self.0;
        let bits_2 = Self::IGNORED_MASK_2 & self.0;

        ((bits_1 >> Self::IGNORED_OFFSET_1) | (bits_2 >> Self::IGNORED_RELATIVE_OFFSET_2)) as u16
    }

    fn set_all_ignored_bits(&mut self, bits: u16) {
        let positioned_bits = ((bits as u64) << Self::IGNORED_OFFSET_1) | 
            ((bits as u64) << Self::IGNORED_RELATIVE_OFFSET_2);
        assert!(positioned_bits & (Self::IGNORED_MASK_1 | Self::IGNORED_MASK_2) == 0);
        self.0 &= !(Self::IGNORED_MASK_1 | Self::IGNORED_MASK_2);
        self.0 |= positioned_bits;
    }
}
