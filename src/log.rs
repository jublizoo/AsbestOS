use crate::arch::x86::utils::write_port_byte;
use crate::tty::{Tty, kfmt_internal};
use core::fmt;

const COM1_PORT: u16 = 0x3f8;

pub fn qemu_log(s: &[u8]) {
    for byte in s {
        unsafe { write_port_byte(COM1_PORT, *byte) };
    }
}

pub fn klog_internal(args: fmt::Arguments) -> Result<(), ()> {
    let Some(s) = kfmt_internal(args) else { return Err(()) };
    qemu_log(s.as_bytes());
    Ok(())
}

#[macro_export]
macro_rules! klog {
    ( $fmt: expr ) => {{
        let args = format_args!($fmt);
        $crate::log::klog_internal(args)
    }};
    ( $fmt: expr, $( $x: expr ) ,*) => {{
        let args = format_args!($fmt, $( $x ),*);
        $crate::log::klog_internal(args)
    }};
}

pub fn klogln_internal(args: fmt::Arguments) -> Result<(), ()> {
    let Some(s) = kfmt_internal(args) else { return Err(()) };
    qemu_log(s.as_bytes());
    qemu_log(b"\n");
    Ok(())
}


#[macro_export]
macro_rules! klogln {
    ( $fmt: expr ) => {{
        let args = format_args!($fmt);
        $crate::log::klogln_internal(args)
    }};
    ( $fmt: expr, $( $x: expr ) ,*) => {{
        let args = format_args!($fmt, $( $x ),*);
        $crate::log::klogln_internal(args)
    }};
}
