#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![allow(dead_code)]

use crate::vga::{write_bytes, write_col};
use bootloader::{BootInfo, bootinfo, entry_point};
use multiboot2::{BootInformation, BootInformationHeader, MemoryAreaType, MemoryArea};
use alloc::vec::Vec;
use common::SyncPtr;

mod vga;
mod panic;
mod simple_alloc;
mod spinlock;
mod math;
mod common;

extern crate alloc;

static HELLO: &[u8] = b"Hello World!";


// entry_point!(entry);
//
// fn entry(boot_info: &'static BootInfo) -> ! {
//
//     loop { }
// }

static PT_END: SyncPtr<u8> = SyncPtr::new(0x20000 as *const u8);

// Remove: kernel code, stack, PTS, GDT, Multiboot struct
// 
// VGA fns
// Setup IDT + enable interrupts
// 

#[unsafe(no_mangle)]
fn rust_main(mb_magic: u32, mbi_ptr: u32) -> ! {
    if mb_magic != multiboot2::MAGIC {
        panic!("Wrong mb_magic");
    }
    let bootinfo = unsafe {  
        BootInformation::load(mbi_ptr as *const BootInformationHeader)
            .unwrap() 
    };
    let mem_map = bootinfo.memory_map_tag()
        .unwrap()
        .memory_areas();

    let max_region = mem_map.iter()
        .filter(|area| area.typ() == MemoryAreaType::Available)
        .reduce(|r1, r2| if r1.size() > r2.size() {r1} else {r2})
        .expect("No free memory");

    let raw_start = max_region.start_address() as *const u8;
    let start = raw_start.max(unsafe { PT_END.get() });
    let end = max_region.end_address() as *const u8;
    unsafe { simple_alloc::ALLOCATOR.configure(start, end); }

    let region_sizes: Vec<u64> = mem_map.iter()
        .filter(|area| area.typ() == MemoryAreaType::Available)
        .map(|r| r.size())
        .collect();

    // let _cmd = bootinfo.command_line_tag();

    write_bytes(HELLO);
    write_col();

    loop { }
}
