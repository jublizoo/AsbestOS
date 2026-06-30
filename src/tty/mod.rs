pub mod vga;

use core::fmt;
use core::ops::Deref;
use alloc::string::String;
use crate::common::try_vec::TryVec;
use crate::log::qemu_log;
use crate::vga::Vga;
use crate::common::string_buf::StringBuf;
use crate::spinlock::SpinLock;
use crate::klogln;

pub fn kfmt_internal(args: fmt::Arguments) -> Option<String> {
    let mut s = StringBuf::new();
    core::fmt::Write::write_fmt(&mut s, args)
        .is_ok()
        .then_some(Into::<String>::into(s))
}

#[macro_export]
macro_rules! kfmt {
    ( $fmt: expr ) => {{
        let args = format_args!($fmt);
        $crate::tty::kfmt_internal(args)
    }};
    ( $fmt: expr, $( $x: expr ),* ) => {{
        let args = format_args!($fmt, $( $x ),*);
        $crate::tty::kfmt_internal(args)
    }}
}

pub fn println_internal(tty: &mut Tty, args: fmt::Arguments) -> Result<(), ()> {
    let Some(s) = kfmt_internal(args) else { return Err(()) };
    tty.try_println(&s)?;
    Ok(())
}

#[macro_export]
macro_rules! kprintln_with_tty {
    ( $tty: expr, $fmt: expr, $( $x: expr ) ,*) => {{
        let tty: &mut Tty = $tty;
        let args = format_args!($fmt, $( $x ),*);
        println_internal(tty, args)
    }};
}

#[macro_export]
macro_rules! kprintln {
    () => {{
        let mut tty = $crate::tty::TTY.lock();
        let args = format_args!("");
        $crate::tty::println_internal(&mut *tty, args)
    }};
    ( $fmt: expr ) => {{
        let mut tty = $crate::tty::TTY.lock();
        let args = format_args!($fmt);
        $crate::tty::println_internal(&mut *tty, args)
    }};
    ( $fmt: expr, $( $x: expr ) ,*) => {{
        let mut tty = $crate::tty::TTY.lock();
        let args = format_args!($fmt, $( $x ),*);
        $crate::tty::println_internal(&mut *tty, args)
    }};
}

pub struct Tty {
    configured: bool,
    pub vga: Vga,
    /// (line idx, row idx within line) of first on-screen row.
    display_from: (usize, usize),
    lines: TryVec<StringBuf>,
    cur_input: StringBuf,
}

impl Tty {
    pub const fn new(vga: Vga) -> Self {
        Self {
            configured: false,
            vga,
            display_from: (0, 0),
            lines: TryVec::new(),
            cur_input: StringBuf::new(),
        }
    }

    pub fn configure(&mut self) {
        assert!(!self.configured);

        self.vga.configure();
        self.configured = true;
    }

    pub fn is_configured(&self) -> bool {
        self.configured
    }

    fn get_num_rows(&self, len: usize) -> usize {
        len.div_ceil(self.vga.get_width() as usize)
    }

    /// Returns correct value for `self.display_from`
    fn get_start_line(&self) -> (usize, usize) {
        let mut vga_row = self.vga.get_height() as i16;
        let mut start_line_idx = 0;
        for idx in self.lines.len()..0 {
            let line = &self.lines[idx];
            vga_row -= self.get_num_rows(line.len()) as i16;
            if vga_row <= 0 { 
                start_line_idx = idx;
                break; 
            } 
        }
        let line_row = (-vga_row) as usize;

        (start_line_idx, line_row)
    }

    // TODO
    pub fn redraw_from_row(&mut self, line_idx: usize, row_idx: usize) {
        assert!(self.configured);
        
    }

    #[allow(unused_assignments)]
    pub fn try_print(&mut self, s: &str) -> Result<(), ()> {
        assert!(self.configured);

        let (width, height) = (self.vga.get_width() as usize, self.vga.get_height() as usize);

        let mut num_displayed_rows = 0;
        for row in self.lines[self.display_from.0..].iter() {
            num_displayed_rows += self.get_num_rows(row.len());
        }
        num_displayed_rows -= self.display_from.1;
        let cur_input_num_rows = self.get_num_rows(self.cur_input.len());
        let next_row = num_displayed_rows + cur_input_num_rows;
        assert!(next_row <= height);

        let mut screen_row = next_row;
        let mut screen_col = self.cur_input.len() % height;

        for c in s.chars() {
            if c == '\n' {
                self.lines.try_push(self.cur_input.clone())?;
                self.cur_input = StringBuf::new();

                screen_row += 1;
                screen_col = 0;
            } else {
                self.cur_input.try_push(c as u8)?;
                self.vga.write_char_at(c as u8, screen_row as u8, screen_col as u8);
                screen_col += 1;

                if self.cur_input.len().is_multiple_of(width) {
                    screen_row += 1;
                    screen_col = 0;
                }
            }

            if screen_row >= height {
                assert!(screen_row == height);
                screen_row = height - 1;

                let first_line_len = self.lines[self.display_from.0].len();
                if self.display_from.1 + 1 < self.get_num_rows(first_line_len) {
                    self.display_from.1 += 1;
                } else {
                    self.display_from.0 += 1;
                    self.display_from.1 = 0;
                }

                self.vga.scroll_line();
            }
        }

        Ok(())
    }

    pub fn try_newline(&mut self) -> Result<(), ()> {
        assert!(self.configured);

        self.try_print("\n")
    }


    pub fn try_println(&mut self, s: &str) -> Result<(), ()> {
        self.try_print(s)?;
        self.try_newline()?;
        Ok(())
    }
}

pub static TTY: SpinLock<Tty> = SpinLock::new(Tty::new(Vga::new()));
