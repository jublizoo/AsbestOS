use crate::{arch::x86::utils::{read_port_byte, write_port_byte}, common::{SyncMutPtr, flags::write_byte_bitmask}};
use crate::spinlock::SpinLock;

#[derive(Clone, Copy)]
struct ExternalReg {
    read_port: u16,
    write_port: u16,
}

/// Graphics, CRT register ports
#[derive(Clone, Copy)]
struct SgcPort {
    addr_port: u16,
    data_port: u16,
}

/// VGA general info
static VGA_START: usize = 0xb8000;
const VGA_WIDTH: u8 = 80;
const VGA_HEIGHT: u8 = 25;

/// VGA register info
const EXT_MISC_REG: ExternalReg = ExternalReg { 
    read_port: 0x3cc, write_port: 0x3c2
};
const EXT_MISC_REG_IOAS_FLAG: u8 = 0;

const CURSOR_ADDR_REG: u16 = 0x4;
const CURSOR_DATA_REG: u16 = 0x5;
const CURSOR_LOC_HI_REG: u8 = 0xe;
const CURSOR_LOC_LO_REG: u8 = 0xf;
const CURSOR_START_REG: u8 = 0xa;
const CURSOR_START_REG_DISABLE_MASK: u8 = 1 << 5;

const GRAPHICS_ADDR_REG: u16 = 0x3ce;
const GRAPHICS_DATA_REG: u16 = 0x3cf;
const GRAPHICS_MISC_REG_IDX: u8 = 6;

const CRT_ADDR_REG: u16 = 0x3d4;
const CRT_DATA_REG: u16 = 0x3d5;

/// Attr bit flags
const FG_OFFSET: u8 = 0;
const BG_OFFSET: u8 = 4;
const BRIGHT: u8 = 1 << 3;
const BLINK: u8 = 1 << 7;

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct VgaAttr(u8);

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum BgColor {
    Black,
    Blue,
    Green,
    Cyan,
    Red,
    Magenta,
    Brown,
    LightGray,
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum FgColor {
    Black,
    Blue,
    Green,
    Cyan,
    Red,
    Magenta,
    Brown,
    LightGray,
    DarkGray,
    BrightBlue,
    BrightGreen,
    BrighnCyan,
    BrightRed,
    BrightMagenta,
    Yellow,
    White,
}

impl VgaAttr {
    pub fn from_fg(col: FgColor) -> Self {
        Self((col as u8) << FG_OFFSET)
    }

    pub fn with_bright(mut self) -> Self {
        self.0 |= BRIGHT;
        self
    }

    pub fn with_blink(mut self) -> Self {
        self.0 |= BLINK;
        self
    }

    pub fn with_bg(mut self, bg: BgColor) -> Self {
        self.0 |= (bg as u8) << BG_OFFSET;
        self
    }

    fn as_byte(&self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
struct VgaCell {
    char: u8,
    attr: u8,
}

impl VgaCell {
    fn new(char: u8, attr: VgaAttr) -> Self {
        Self {
            char,
            attr: attr.as_byte(),
        }
    }
}

type VgaBuf = [[VgaCell; VGA_WIDTH as usize]; VGA_HEIGHT as usize];
type VgaBufLinear = [u8; (VGA_WIDTH as usize) * (VGA_HEIGHT as usize)];

/// Used to modify/query VGA state, caches some VGA state.
/// For use with VGA text mode, exposes only select functionality.
///
/// The safety of the methods exposed by this struct rely on the assumption
/// that there is at most one instance of the struct. This is enforced through
/// a private `new` method, and a single global instance, `VGA`.
pub struct Vga {
    buf: SyncMutPtr<VgaBuf>,
    configured: bool,
    cursor_visible: bool,
    cursor_row: u8,
    cursor_col: u8,
}

/// API boundary around Vga exposes just the most basic writing functionality,
/// but exposing a few higher-level functions like `scroll_by`, which allow for
/// compilers to use instrinsics (`memcpy`, etc), which would otherwise be more
/// difficult to optimize from repeated read/set calls.
impl Vga {
    pub(super) const fn new() -> Self {
        Self {
            buf: SyncMutPtr::new(VGA_START as *mut VgaBuf),
            configured: false,
            cursor_visible: false,
            cursor_row: 0,
            cursor_col: 0,
        }
    }

    pub fn is_configured(&self) -> bool {
        self.configured
    }

    pub fn configure(&mut self) {
        assert!(!self.configured);

        self.configured = true;
        self.set_crt_port();
        self.move_cursor(self.cursor_row, self.cursor_col);
        self.enable_cursor();
    }

    pub fn get_width(&self) -> u16 {
        VGA_WIDTH as u16
    }

    pub fn get_height(&self) -> u16 {
        VGA_HEIGHT as u16
    }

    fn buf(&mut self) -> &mut VgaBuf {
        unsafe { &mut *self.buf.get() }
    }

    fn buf_linear(&mut self) -> &mut VgaBufLinear {
        unsafe{ &mut *(self.buf.get() as *mut VgaBufLinear) }
    }

    unsafe fn write_external_reg(
        &self, 
        reg: ExternalReg,
        f: impl Fn(u8) -> u8,
    ) -> u8 {
        let prev_data = unsafe { read_port_byte(reg.read_port) };
        let data_to_write = f(prev_data);
        if data_to_write != prev_data {
            unsafe { write_port_byte(reg.write_port, data_to_write) };
        }
        prev_data
    }

    /// Writes to a sequencer, graphics, or CRT controller register
    /// Returns previous register value
    unsafe fn write_sgc_reg(
        &self, 
        reg_idx: u8, 
        addr_reg: u16,
        data_reg: u16,
        f: impl Fn(u8) -> u8
    ) -> u8 {
        let prev_reg_idx = unsafe { read_port_byte(addr_reg) };
        unsafe { write_port_byte(addr_reg, reg_idx); }
        let prev_data = unsafe { read_port_byte(data_reg) };
        let data_to_write = f(prev_data);
        if data_to_write != prev_data {
            unsafe { write_port_byte(data_reg, data_to_write); }
        }
        // Restore addr reg
        if prev_reg_idx != reg_idx {
            unsafe { write_port_byte(addr_reg, prev_reg_idx); }
        }

        prev_data
    }

    unsafe fn write_graphics_reg(&self, reg_idx: u8, f: impl Fn(u8) -> u8) -> u8 {
        unsafe {
            self.write_sgc_reg(reg_idx, GRAPHICS_ADDR_REG, GRAPHICS_DATA_REG, f)
        }
    }

    unsafe fn write_crt_reg(&self, reg_idx: u8, f: impl Fn(u8) -> u8) -> u8 {
        assert!(self.configured);

        unsafe {
            self.write_sgc_reg(reg_idx, CRT_ADDR_REG, CRT_DATA_REG, f)
        }
    }

    fn set_crt_port(&self) {
        unsafe { self.write_external_reg(EXT_MISC_REG, |x| x | EXT_MISC_REG_IOAS_FLAG) };
    }

    fn coords_to_idx(row: u8, col: u8) -> u16 {
        assert!(row < VGA_HEIGHT);
        assert!(col < VGA_WIDTH);

        (row as u16) * (VGA_WIDTH as u16) + (col as u16)
    }

    fn idx_to_coords(idx: u16) -> (u8, u8) {
        ((idx / (VGA_WIDTH as u16)) as u8, (idx % VGA_WIDTH as u16) as u8)
    }

    fn move_cursor_raw(&mut self, idx: u16) {
        let idx_lo = (idx & 0xff) as u8;
        let idx_hi = ((idx >> 8) & 0xff) as u8;

        unsafe { 
            self.write_crt_reg(CURSOR_LOC_LO_REG, |_| idx_lo);
            self.write_crt_reg(CURSOR_LOC_HI_REG, |_| idx_hi);
        }
    }

    pub fn move_cursor(&mut self, row: u8, col: u8) {
        assert!(self.configured);

        let idx = Self::coords_to_idx(row, col);
        self.cursor_row = row;
        self.cursor_col = col;
        self.move_cursor_raw(idx);
    }

    /// Returns previous cursor visibility (true if visible)
    fn set_cursor_visibility(&mut self, visible: bool) -> bool {
        let prev_val = unsafe { self.write_crt_reg(CURSOR_START_REG, |reg| {
            write_byte_bitmask(reg, CURSOR_START_REG_DISABLE_MASK, !visible)
        })};

        (prev_val & CURSOR_START_REG_DISABLE_MASK) == 0
    }

    pub fn enable_cursor(&mut self) {
        assert!(self.configured);
        assert!(!self.cursor_visible);

        self.set_cursor_visibility(true);
        self.cursor_visible = true;
    }

    pub fn disable_cursor(&mut self) {
        assert!(self.configured);
        assert!(self.cursor_visible);

        self.set_cursor_visibility(false);
        self.cursor_visible = false;
    }

    fn get_graphics_misc(&self) -> u8 {
        unsafe { self.write_graphics_reg(GRAPHICS_MISC_REG_IDX, |x| x) }
    }

    pub fn write_at(&mut self, ch: u8, attr: VgaAttr, row: u8, col: u8) {
        self.buf()[row as usize][col as usize] = VgaCell::new(ch, attr);
    }

    pub fn write_char_at(&mut self, ch: u8, row: u8, col: u8) {
        self.buf()[row as usize][col as usize].char = ch;
    }

    pub fn write_attr_at(&mut self, attr: VgaAttr, row: u8, col: u8) {
        self.buf()[row as usize][col as usize].attr = attr.as_byte();
    }

    pub fn write_at_idx(&mut self, ch: u8, attr: VgaAttr, idx: u16) {
        let (row, col) = Self::idx_to_coords(idx);
        self.write_at(ch, attr, row, col);
    }

    pub fn write_bytes(&mut self, text: &[u8], attr: VgaAttr, start_row: u8, start_col: u8) {
        assert!(self.configured);

        let start_idx = Self::coords_to_idx(start_row, start_col);

        for (i, &ch) in text.iter().enumerate() {
            self.write_at_idx(ch, attr, start_idx + i as u16);
        }
    }

    pub fn write_bytes_default(&mut self, text: &[u8], start_row: u8, start_col: u8) {
        assert!(self.configured);

        let attr = VgaAttr::from_fg(FgColor::White);
        self.write_bytes(text, attr, start_row, start_col);
    }

    pub fn shift_by(&mut self, shift_by: u16) {
        let buf = self.buf_linear();

        for i in 0..(buf.len() - shift_by as usize) {
            buf[i] = buf[i + shift_by as usize];
        }

    }

    pub fn scroll_by(&mut self, scroll_by: u8) {
        let buf = self.buf();
        for i in 0..((VGA_HEIGHT - scroll_by) as usize) {
            buf[i] = buf[i + scroll_by as usize]
        }
    }

    pub fn scroll_line(&mut self) {
        self.scroll_by(1);
    }
}

pub static VGA: SpinLock<Vga> = SpinLock::new(Vga::new());
