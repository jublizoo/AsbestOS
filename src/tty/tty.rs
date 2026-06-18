use core::fmt::Write;
use core::str::from_utf8;
use core::{fmt, fmt::Error};

use super::vga::Vga;
use alloc::collections::TryReserveError;
use alloc::vec::Vec;
use alloc::string::{String, ToString};

struct Tty {
    configured: bool,
    vga: Vga,
    lines: Option<Vec<String>>,
}

impl Tty {
    pub const fn new(vga: Vga) -> Self {
        Self {
            configured: false,
            vga,
            lines: None,
        }
    }

    pub fn configure(&mut self) {
        self.vga.configure();
    }

    pub fn configured(&self) -> bool {
        self.configured
    }

    pub fn println(_s: &str) {
        

    }
}

pub struct StringBuf {
    buf: Vec<u8>,
}

impl StringBuf {
    pub fn new() -> Self {
        Self {
            buf: Vec::new(),
        }
    }
}

impl Into<String> for StringBuf {
    fn into(self) -> String {
        alloc::string::String::from_utf8(self.buf).unwrap()
    }
}

impl fmt::Write for StringBuf {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let bytes = s.as_bytes();
        let res: Result<(), TryReserveError> = self.buf.try_reserve(bytes.len());
        if res.is_err() {
            return Err(Error {});
        }
        self.buf.extend_from_slice(bytes);

        Ok(())
    }
}

fn test() {
    let mut s = crate::tty::tty::StringBuf::new();
    core::fmt::Write::write_fmt(&mut s, format_args!(""))
        .is_ok()
        .then_some(Into::<String>::into(s));
}

#[macro_export]
macro_rules! kfmt {
    ( $fmt: expr, $( $x: expr ),* ) => {{
        let mut s = $crate::tty::tty::StringBuf::new();
        core::fmt::Write::write_fmt(&mut s, format_args!($fmt, $($x),*))
            .is_ok()
            .then_some(Into::<String>::into(s))
    }}
}

macro_rules! println {
    ( $tty: expr, $fmt: expr, $( $x: expr ) ,*) => {
        let tty: Tty = $tty;
        tty
    };
}


// fn x(tty: Tty) -> Result<(), TryReserveError> {
//     let mut s = String::new();
//     String::try_reserve(&mut s, 100)?;
//     fmt::write(&s, format_args!())
//     println!(tty, 0, 1, 2);
//     Ok(())
// }
