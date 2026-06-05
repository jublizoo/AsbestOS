use core::ptr::NonNull;
use alloc::string::{String, ToString};
use volatile::VolatilePtr;

static VGA_START: usize = 0xb8000;

#[repr(C, packed)]
struct VgaAttr {
    fg0: bool,
    fg1: bool,
    fg2: bool,
    fg3: bool,
    bg0: bool,
    bg1: bool,
    bg2: bool,
    blink: bool,
}

struct FgColor {
    fg0: bool,
    fg1: bool,
    fg2: bool,
}

struct BgColor {
    bg0: bool,
    bg1: bool,
    bg2: bool,
}

impl BgColor {
    fn default() -> Self {
        Self {
            bg0: false,
            bg1: false,
            bg2: false,
        }
    }
}

impl VgaAttr {
    fn from_fg(col: FgColor) -> Self {
        let bg = BgColor::default();
        Self {
            fg0: col.fg0,
            fg1: col.fg1,
            fg2: col.fg2,
            fg3: false,
            bg0: bg.bg0,
            bg1: bg.bg1,
            bg2: bg.bg2,
            blink: false,
        }
    }

    fn with_bright(mut self) -> Self {
        self.fg3 = true;
        self
    }

    fn with_blink(mut self) -> Self {
        self.blink = true;
        self
    }

    fn with_bg(mut self, bg: BgColor) -> Self {
        self.bg0 = bg.bg0;
        self.bg1 = bg.bg1;
        self.bg2 = bg.bg2;
        self
    }

    fn as_byte(&self) -> u8 {
        unsafe { *(self as *const VgaAttr as *const u8) }
    }
}

pub fn write_bytes(text: &[u8]) {
    for (i, &ch) in text.iter().enumerate() {
        write_at_index(ch, i);
    }

    // let fg = FgColor { fg0: true, fg1: true, fg2: false };
    // let col = VgaAttr::from_fg(fg).with_bright();
    // let byte = col.as_byte();
    // let s = u8::to_string(&byte);
    // for (i, &ch) in s.as_bytes().iter().enumerate() {
    //     write_at_index(ch, i);
    // }
}

fn write_at_index(ch: u8, idx: usize) {
    let vga_buf = VGA_START as *mut u8;

    unsafe {
        write_byte(vga_buf.offset(idx as isize * 2), ch);
        write_byte(vga_buf.offset(idx as isize * 2 + 1), 0xb);
    }
}

fn write_byte(ptr: *mut u8, ch: u8) {
    let ptr = unsafe { VolatilePtr::new(NonNull::new(ptr).unwrap()) };
    ptr.write(ch);
}
