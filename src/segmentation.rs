use alloc::task::Wake;

use crate::common::flags::{bits_in_range, bits_in_range_2};

#[repr(u8)]
pub enum PrivLevel {
    Zero = 0,
    One,
    Two,
    Three,
}

#[repr(transparent)]
struct SegSel {
    inner: u16,
}

#[repr(u8)]
enum TableIndicator {
    Gdt = 0,
    Ldt = 1,
}

impl SegSel {
    const RPL_OFFSET: u8                = 0;
    const IDX_OFFSET: u8                = 2;
    const TABLE_INDICATOR_OFFSET: u8    = 3;

    const fn new(idx: u16, rpl: PrivLevel, table: TableIndicator) -> Self {
        SegSel {
            inner: 
                (rpl as u16) << Self::RPL_OFFSET |
                idx << Self::IDX_OFFSET |
                (table as u16) << Self::TABLE_INDICATOR_OFFSET,
        }
    }
}

#[repr(C)]
struct TssDescriptor {
    seg_limit_lo: u16,
    base_addr_lo: u16,
    base_addr_mid1: u8,
    attrs_1: u8,
    attrs_2: u8,
    base_addr_mid2: u8,
    base_addr_hi: u32,
    reserved_or_zero: u32,
}

impl TssDescriptor {
    // TODO: Replace masks with get_bits_in_range function.
    const LIMIT_LO_RANGE:   (u8, u8) = (0, 15);
    const LIMIT_HI_RANGE:   (u8, u8) = (16, 19);
    const BASE_LO_RANGE:    (u8, u8) = (0, 15);
    const BASE_MID1_RANGE:  (u8, u8) = (16, 23);
    const BASE_MID2_RANGE:  (u8, u8) = (24, 31);
    const BASE_HI_RANGE:    (u8, u8) = (32, 63);

    /// Offsets in attrs_1
    const TYPE_OFFSET: u8       = 0;
    const DPL_OFFSET: u8        = 5;
    const PRESENT_OFFSET: u8    = 7;

    /// Offsets in attrs_2
    const LIMIT_OFFSET: u8          = 0;
    const AVL_OFFSET: u8            = 4;
    const GRANULARITY_OFFSET: u8    = 7;

    const TYPE_BUSY: u8     = 0b1011;

    fn new(base: *const u8, limit: u32, dpl: PrivLevel) -> Self {
        const PRESENT: bool = true;
        /// Limit interpreted in 4KiB increments
        const GRANULARITY: bool = true;

        let base_addr_lo    = bits_in_range_2(base as u64, Self::BASE_LO_RANGE) as u16;
        let base_addr_mid1  = bits_in_range_2(base as u64, Self::BASE_MID1_RANGE) as u8;
        let base_addr_mid2  = bits_in_range_2(base as u64, Self::BASE_MID2_RANGE) as u8;
        let base_addr_hi    = bits_in_range_2(base as u64, Self::BASE_HI_RANGE) as u32;
        let seg_limit_lo    = bits_in_range_2(limit as u64, Self::LIMIT_LO_RANGE) as u16;
        let seg_limit_hi    = bits_in_range_2(limit as u64, Self::LIMIT_HI_RANGE) as u8;

        Self {
            base_addr_lo,
            base_addr_mid1,
            base_addr_mid2,
            base_addr_hi,
            seg_limit_lo,
            attrs_1: 
                Self::TYPE_OFFSET   << Self::TYPE_OFFSET    | 
                (dpl as u8)         << Self::DPL_OFFSET     |
                (PRESENT as u8)     << Self::PRESENT_OFFSET,
            attrs_2:
                seg_limit_hi        << Self::LIMIT_OFFSET   |
                (GRANULARITY as u8) << Self::GRANULARITY_OFFSET,
            reserved_or_zero: 0,
        }
    }

}
