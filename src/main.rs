#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![allow(dead_code)]

use crate::vga::write_bytes;
// use bootloader::{BootInfo, bootinfo, entry_point};
use multiboot2::{BootInformation, BootInformationHeader};

mod vga;
mod panic;
mod simple_alloc;
mod spinlock;
mod utils;
mod math;

extern crate alloc;

static HELLO: &[u8] = b"Hello World!";


// entry_point!(entry);
//
// fn entry(boot_info: &'static BootInfo) -> ! {
//
//     loop { }
// }

#[unsafe(no_mangle)]
fn rust_main(mb_magic: u32, mbi_ptr: u32) -> ! {
    if mb_magic != multiboot2::MAGIC {
        panic!("Wrong mb_magic");
    }
    let bootinfo = unsafe {  
        BootInformation::load(mbi_ptr as *const BootInformationHeader)
            .unwrap() 
    };
    let _cmd = bootinfo.command_line_tag();

    loop {
        write_bytes(HELLO);
    }
}
