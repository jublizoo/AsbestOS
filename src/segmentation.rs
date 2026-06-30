#[repr(u8)]
pub enum PrivLevel {
    Zero = 0,
    One,
    Two,
    Three,
}

struct TssDescriptor {

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
struct SegmentDescriptor {
    seg_limit_lo: u16,
    base_addr_lo: u16,
    base_addr_mid: u8,
    segment_type: u8,
    attrs_1: u8,
    seg_limit_hi: u8,
    attrs_2: u8,
    base_addr_hi: u8,
}

impl SegmentDescriptor {

}
