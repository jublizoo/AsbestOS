pub mod vga;

use core::fmt;
use core::ops::Deref;
use alloc::string::String;
use crate::common::try_vec::TryVec;
use crate::vga::Vga;
use crate::common::string_buf::StringBuf;
use crate::spinlock::SpinLock;

pub struct Tty {
    configured: bool,
    pub vga: Vga,
    display_from_line: usize,
    lines: TryVec<StringBuf>,
    cur_input: StringBuf,
}

impl Tty {
    pub const fn new(vga: Vga) -> Self {
        Self {
            configured: false,
            vga,
            display_from_line: 0,
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

    pub fn try_newline(&mut self) -> Result<(), ()> {
        assert!(self.configured);

        self.lines.try_push(self.cur_input.clone())?;
        self.cur_input = StringBuf::new();

        Ok(())
    }

    fn get_num_rows(&self, len: usize) -> usize {
        len.div_ceil(self.vga.get_width() as usize)
    }

    /// Returns (start_line_idx, line_row_idx) where line_row_idx is the index 
    /// of the physical row in the start line, on which to start.
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

    pub fn redraw_from_row(&mut self, line_idx: usize, row_idx: usize) {
        assert!(self.configured);
        
    }

    // TODO:
    // Get blank lines working
    // Add

    #[allow(unused_assignments)]
    pub fn try_print(&mut self, s: &str) -> Result<(), ()> {
        assert!(self.configured);

        let (width, height) = (self.vga.get_width() as usize, self.vga.get_height() as usize);

        let mut num_displayed_rows = 0;
        for row in self.lines[self.display_from_line..].iter() {
            num_displayed_rows += self.get_num_rows(row.len());
        }
        let cur_input_num_rows = self.get_num_rows(self.cur_input.len());
        let mut next_row = num_displayed_rows + cur_input_num_rows;

        if next_row >= height { 
            let (next_line_idx, start_row_idx) = self.get_start_line();
            self.redraw_from_row(next_line_idx, start_row_idx);
            let rows_from_start = self.lines[next_line_idx..].iter()
                .map(|line| self.get_num_rows(line.len()))
                .reduce(|x, y| x + y)
                .unwrap_or(0);
            next_row = rows_from_start - start_row_idx;
        }
        let mut row = next_row;
        let mut col = self.cur_input.len() % height;

        for c in s.chars() {
            if self.cur_input.len().is_multiple_of(width) {
                row += 1;
                col = 0;
            }
            if row >= height {
                assert!(row == height);
                self.vga.shift_by(1);
                row = height - 1;
            }

            if c == '\n' {
                self.lines.try_push(self.cur_input.clone())?;
                self.cur_input = StringBuf::new();
            } else {
                self.cur_input.try_push(c as u8)?;
                self.vga.write_char_at(c as u8, row as u8, col as u8);
                col += 1;
            }
        }

        Ok(())
    }

    pub fn try_println(&mut self, s: &str) -> Result<(), ()> {
        self.try_print(s)?;
        self.try_newline()?;
        Ok(())
    }
}

pub fn kfmt_internal(args: fmt::Arguments) -> Option<String> {
    let mut s = StringBuf::new();
    core::fmt::Write::write_fmt(&mut s, args)
        .is_ok()
        .then_some(Into::<String>::into(s))
}

#[macro_export]
macro_rules! kfmt {
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

pub static TTY: SpinLock<Tty> = SpinLock::new(Tty::new(Vga::new()));
