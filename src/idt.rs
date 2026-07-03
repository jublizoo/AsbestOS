use core::mem::MaybeUninit;
use core::ptr::{null, null_mut};
use core::alloc::Layout;
use alloc::alloc::alloc;

use crate::paging::PAGE_SIZE;
use crate::segmentation::PrivLevel;

const IDT_NUM_ENTRIES: u16 = 20;

const INTERRUPT_GATE_TYPE_ID: u8    = 0b1110;
const TRAP_GATE_TYPE_ID: u8         = 0b1111;

#[repr(u8)]
pub enum GateType {
    Interrupt = INTERRUPT_GATE_TYPE_ID,
    Trap =      TRAP_GATE_TYPE_ID,
}

impl From<u8> for GateType {
    fn from(value: u8) -> Self {
        match value {
            INTERRUPT_GATE_TYPE_ID => GateType::Interrupt,
            TRAP_GATE_TYPE_ID => GateType::Trap,
            _ => panic!("Invalid gate type")
        }
    }
}

#[repr(C)]
pub struct IdtEntry {
    offset_lo: u16,
    segsel: u16,
    /// 3 bits, rest are reserved.
    ist: u8,
    attrs: u8,
    offset_mid: u16,
    offset_hi: u32,
    reserved: u32,
}

// TODO: Don't ignore reserved bits.
impl IdtEntry {
    /// Offsets/masks within the `attrs` field 
    const GATE_TYPE_OFFSET: u8  = 0;
    const DPL_OFFSET: u8        = 5;
    const PRESENT_OFFSET: u8    = 7;
    const GATE_TYPE_MASK: u8    = 0xF;

    /// Masks for the offset itself, not gates
    const IDT_OFF_LO_MASK: u64  = 0x000000FF;
    const IDT_OFF_MID_MASK: u64 = 0x0000FF00;
    const IDT_OFF_HI_MASK: u64  = 0xFFFF0000;

    const CODE_SEGSEL: u16 = 0;

    // TODO: Make const.
    fn new_gate(offset: *const u8, dpl: PrivLevel, ist: u8, gate_type: GateType) -> Self {
        const PRESENT: bool = true;

        let attrs: u8 =
            (gate_type as u8) << Self::GATE_TYPE_OFFSET |
            (dpl as u8) << Self::DPL_OFFSET | 
            (PRESENT as u8) << Self::PRESENT_OFFSET;

        // TODO: Fix addr, no shift
        IdtEntry {
            offset_lo: ((offset as u64) & Self::IDT_OFF_LO_MASK) as u16,
            segsel: Self::CODE_SEGSEL,
            ist,
            attrs,
            offset_mid: ((offset as u64) & Self::IDT_OFF_MID_MASK) as u16,
            offset_hi: ((offset as u64) & Self::IDT_OFF_HI_MASK) as u32,
            reserved: 0,
        }

    }

    pub fn new_trap_gate(offset: *const u8, dpl: PrivLevel, ist: u8) -> Self {
        Self::new_gate(offset, dpl, ist, GateType::Trap)
    }

    pub fn get_gate_type(&self) -> GateType {
        GateType::from(self.attrs & Self::GATE_TYPE_MASK)
    }
}

type IdtInner = [IdtEntry; IDT_NUM_ENTRIES as usize];
struct Idt {
    inner: *mut IdtInner,
}

impl Idt {
    fn try_create_idt() -> Result<Self, ()> {
        let layout = Layout::new::<IdtInner>()
            .align_to(PAGE_SIZE)
            .unwrap();
        let idt_ptr = unsafe { alloc(layout) } as *mut IdtInner;
        if idt_ptr.is_null() { return Err(()); }

        Ok(Self {
            inner: idt_ptr
        })
    }

    /// Register handlers that can be only be called from kernel space.
    fn register_kernel_handler(&self, irqno: u16, handler: *const u8, ist: u8) {
        let gate = IdtEntry::new_trap_gate(handler, PrivLevel::Zero, ist);
        let idt = unsafe { &mut *self.inner };
        idt[irqno as usize] = gate;
    }

    /// Register handlers that can be called from user space using the `int` 
    /// instruction (e.g. syscall handlers).
    fn register_user_handler(&self, irqno: u16, handler: *const u8, ist: u8){
        let gate = IdtEntry::new_trap_gate(handler, PrivLevel::Three, ist);
        let idt = unsafe { &mut *self.inner };
        idt[irqno as usize] = gate;
    }

}

