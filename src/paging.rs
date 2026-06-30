// non-HLAT 4-level paging implementation for x86-64 long mode.
//
// # TODO: 
// - Add paging options
// - Maintain per-table #(present entries) using ignored bits, to add GC
// - Larger page sizes
// - Add direct-mapped region, needs support for sharing paging structures
// of direct-mapped region between different mappings.

#![allow(clippy::upper_case_acronyms)]

use core::{alloc::Layout, mem::MaybeUninit};
use ::alloc::alloc::alloc;
use crate::common::flags::{range_mask, write_byte_bitmask};

pub const PAGE_SIZE: usize = 4096;
pub const PAGE_MAP_NUM_ENTRIES: usize = 512;

trait PageMap<E: PageMapEntry>: Sized {
    fn alloc_and_init_new() -> Result<*mut [E; PAGE_MAP_NUM_ENTRIES], ()> {
        let layout = Layout::new::<Self>()
            .align_to(PAGE_SIZE)
            .unwrap();
        let pml4t_ptr = unsafe { alloc(layout) } as *mut [E; PAGE_MAP_NUM_ENTRIES];
        if pml4t_ptr.is_null() {
            return Err(());
        }
        unsafe { *pml4t_ptr = [E::new_empty(); PAGE_MAP_NUM_ENTRIES] };

        Ok(pml4t_ptr)
    }
}

trait RawEntry: Sized {
    fn from_bits(bits: u64) -> MaybeUninit<Self>;
    fn bits(&self) -> u64;
    fn bits_mut(&mut self) -> &mut u64;
}

trait PageMapEntry: RawEntry + Copy {
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

    fn is_present(&self) -> bool {
        (self.bits() & Self::PRESENT) != 0
    }

    fn set_present(&mut self, present: bool) {
        *self.bits_mut() = write_byte_bitmask(self.bits(), Self::PRESENT, present);
    }


    fn is_writable(&self) -> bool {
        (self.bits() & Self::RW) != 0
    }

    fn set_writable(&mut self, writable: bool) {
        *self.bits_mut() = write_byte_bitmask(self.bits(), Self::RW, writable);
    }

    fn is_accessed(&self) -> bool {
        (self.bits() & Self::ACCESSED) != 0
    }

    fn set_accessed(&mut self, accessed: bool) {
        *self.bits_mut() = write_byte_bitmask(self.bits(), Self::ACCESSED, accessed);
    }

    // TODO: Make trait generic over entry type, return entry pointer?
    fn get_addr(&self) -> *const u8 {
        (self.bits() & Self::ADDR_MASK) as *const u8
    }

    // TODO: Same as above for `addr` param
    fn set_addr(&mut self, addr: *const u8) {
        assert!((addr as u64) & !Self::ADDR_MASK == 0);

        *self.bits_mut() &= !Self::ADDR_MASK;
        *self.bits_mut() |= addr as u64;
    }
}

impl<T: RawEntry + Copy> PageMapEntry for T { }

/// For higher-level page maps (all paging structures except page-tables).
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



#[derive(Clone, Copy)]
struct PML4E(u64);
type PML4T = [PML4E; PAGE_MAP_NUM_ENTRIES];

impl PageMap<PML4E> for PML4T { }

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



#[derive(Clone, Copy)]
struct PDPTE(u64);
type PDPT = [PDPTE; PAGE_MAP_NUM_ENTRIES];

impl PageMap<PDPTE> for PDPT { }

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



#[derive(Clone, Copy)]
struct PDE(u64);
type PD = [PDE; PAGE_MAP_NUM_ENTRIES];

impl PageMap<PDE> for PD { }

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



#[derive(Clone, Copy)]
struct PTE(u64);
type PT = [PTE; PAGE_MAP_NUM_ENTRIES];

impl PageMap<PTE> for PT { }

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


pub struct MapOptions {
    writable: bool,
    user_accessible: bool,
}

pub struct PageMapping {
    inner: *mut PML4T,
}

impl PageMapping {
    const ADDR_PML4T_OFFSET_MASK: u64 = range_mask(47, 39);
    const ADDR_PDPT_OFFSET_MASK: u64 = range_mask(38, 30);
    const ADDR_PD_OFFSET_MASK: u64 = range_mask(29, 21);
    const ADDR_PT_OFFSET_MASK: u64 = range_mask(20, 12);
    const ADDR_PAGE_OFFSET_MASK: u64 = range_mask(11, 0);

    fn new() -> Option<Self> {
        let Ok(pml4t_ptr) = PML4T::alloc_and_init_new() else { return None; };
        Some(Self {
            inner: pml4t_ptr,
        })
    }

    fn map_pte(entry: &mut PTE, ppage: *const u8) {
        assert!((ppage as u64).is_multiple_of(PAGE_SIZE as u64));
        assert!(!entry.is_present());

        entry.set_addr(ppage);
        entry.set_present(true);
    }

    fn alloc_and_map_pde(entry: &mut PDE, vpage: *const u8, ppage: *const u8) -> Result<(), ()> {
        assert!((ppage as u64).is_multiple_of(PAGE_SIZE as u64));
        assert!(!entry.is_present());

        let pt_ptr = PT::alloc_and_init_new()?;
        let pt = unsafe { &mut *pt_ptr };
        let pt_offset = vpage as u64 & Self::ADDR_PT_OFFSET_MASK;
        let pte = &mut pt[pt_offset as usize];

        Self::map_pte(pte, ppage);
        entry.set_addr(pt_ptr as *const u8);
        entry.set_present(true);

        Ok(())
    }

    fn alloc_and_map_pdpte(entry: &mut PDPTE, vpage: *const u8, ppage: *const u8) -> Result<(), ()> {
        assert!((ppage as u64).is_multiple_of(PAGE_SIZE as u64));
        assert!(!entry.is_present());

        let pd_ptr = PD::alloc_and_init_new()?;
        let pd = unsafe { &mut *pd_ptr };
        let pd_offset = vpage as u64 & Self::ADDR_PT_OFFSET_MASK;
        let pde = &mut pd[pd_offset as usize];

        Self::alloc_and_map_pde(pde, vpage, ppage)?;
        entry.set_addr(pd_ptr as *const u8);
        entry.set_present(true);

        Ok(())
    }

    fn alloc_and_map_pml4e(entry: &mut PML4E, vpage: *const u8, ppage: *const u8) -> Result<(), ()> {
        assert!((ppage as u64).is_multiple_of(PAGE_SIZE as u64));
        assert!(!entry.is_present());

        let pdpt_ptr = PDPT::alloc_and_init_new()?;
        let pdpt = unsafe { &mut *pdpt_ptr };
        let pdpt_offset = vpage as u64 & Self::ADDR_PT_OFFSET_MASK;
        let pdpte = &mut pdpt[pdpt_offset as usize];

        Self::alloc_and_map_pdpte(pdpte, vpage, ppage)?;
        entry.set_addr(pdpt_ptr as *const u8);
        entry.set_present(true);

        Ok(())
    }

    /// Returns if the page was previously mapped
    pub unsafe fn map_page(&mut self, vpage: *const u8, ppage: *const u8) -> Result<bool, ()> {
        let pml4t_offset = vpage as u64 & Self::ADDR_PML4T_OFFSET_MASK;
        let pdpt_offset = vpage as u64 & Self::ADDR_PDPT_OFFSET_MASK;
        let pd_offset = vpage as u64 & Self::ADDR_PD_OFFSET_MASK;
        let pt_offset = vpage as u64 & Self::ADDR_PT_OFFSET_MASK;
        let _page_offset = vpage as u64 & Self::ADDR_PAGE_OFFSET_MASK;

        let pml4t = unsafe { &mut *self.inner };
        let pml4e = &mut pml4t[pml4t_offset as usize];
        if !pml4e.is_present() {
            Self::alloc_and_map_pml4e(pml4e, vpage, ppage)?;
            return Ok(true);
        }

        let pdpt = unsafe { &mut *(pml4e.get_addr() as *mut PDPT) };
        let pdpte = &mut pdpt[pdpt_offset as usize];
        if !pdpte.is_present() {
            Self::alloc_and_map_pdpte(pdpte, vpage, ppage)?;
            return Ok(true);
        }

        let pd = unsafe { &mut *(pdpte.get_addr() as *mut PD) };
        let pde = &mut pd[pd_offset as usize];
        if !pde.is_present() {
            Self::alloc_and_map_pde(pde, vpage, ppage)?;
            return Ok(true);
        }

        let pt = unsafe { &mut *(pde.get_addr() as *mut PT) };
        let pte = &mut pt[pt_offset as usize];
        assert!(!pte.is_present());
        pte.set_addr(ppage);

        Ok(false)
    }

    /// Returns Err if the page is already unmapped
    pub unsafe fn try_unmap_page(&mut self, vpage: *const u8) -> Result<(), ()> {
        let pml4t_offset = vpage as u64 & Self::ADDR_PML4T_OFFSET_MASK;
        let pdpt_offset = vpage as u64 & Self::ADDR_PDPT_OFFSET_MASK;
        let pd_offset = vpage as u64 & Self::ADDR_PD_OFFSET_MASK;
        let pt_offset = vpage as u64 & Self::ADDR_PT_OFFSET_MASK;
        let _page_offset = vpage as u64 & Self::ADDR_PAGE_OFFSET_MASK;

        let pml4t = unsafe { &mut *self.inner };
        let pml4e = &mut pml4t[pml4t_offset as usize];
        if !pml4e.is_present() { return Err(()) }

        let pdpt = unsafe { &mut *(pml4e.get_addr() as *mut PDPT) };
        let pdpte = &mut pdpt[pdpt_offset as usize];
        if !pdpte.is_present() { return Err(()) }

        let pd = unsafe { &mut *(pdpte.get_addr() as *mut PD) };
        let pde = &mut pd[pd_offset as usize];
        if !pde.is_present() { return Err(()) }

        let pt = unsafe { &mut *(pde.get_addr() as *mut PT) };
        let pte = &mut pt[pt_offset as usize];
        if !pde.is_present() { return Err(()) }
        pte.set_present(false);

        Ok(())
    }

    pub unsafe fn unmap_page(&mut self, vpage: *const u8) {
        unsafe { self.try_unmap_page(vpage) }
            .unwrap();
    }
}
