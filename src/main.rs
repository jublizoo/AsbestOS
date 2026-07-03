#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(formatting_options)]
#![allow(dead_code)]

extern crate alloc;

mod panic;
mod simple_alloc;
mod spinlock;
mod math;
mod common;
mod arch;
mod tty;
mod paging;
mod idt;
mod log;
mod tss;
mod segmentation;

use crate::tty::vga;
use crate::common::region::Region;
use crate::common::buf_vec::BufVec;
use multiboot2::{BootInformation, BootInformationHeader, MemoryAreaType, MemoryArea};
use alloc::string::String;

unsafe extern "C" {
    static __kernel_start: u8;
    static __kernel_end: u8;
}

// Hardcode for now, do something nicer later...
// Maybe walk PTs during alloc setup?
const PT_START: *const u8 = 0x1000 as *const u8;
const PT_END: *const u8 = 0x20000 as *const u8;

fn print_regions(bootinfo: &BootInformation) {
    let mem_map = bootinfo.memory_map_tag()
        .unwrap()
        .memory_areas();

    for region in mem_map.iter()
        .filter(|r| r.typ() == MemoryAreaType::Available)
    {
        let region: &MemoryArea = region;
        kprintln!("{:X} -> {:X} ({:?})", 
            region.start_address(), 
            region.end_address(), 
            region.typ()
        ).unwrap();
    }

    for region in mem_map.iter()
        .filter(|r| r.typ() != MemoryAreaType::Available)
    {
        let region: &MemoryArea = region;
        kprintln!("{:X} -> {:X} ({:?})", 
            region.start_address(), 
            region.end_address(), 
            region.typ()
        ).unwrap();
    }
}

fn print_sections(bootinfo: &BootInformation) {
    if let Some(elf_sections) = bootinfo.elf_sections_tag() {
        for section in elf_sections.sections() {
            let section_name = section.name().unwrap_or("[no name]");
            kprintln!("section {}: {:X} -> {:X}", 
                section_name,
                section.start_address(), 
                section.end_address()
            ).unwrap();
        }
    } else {
        kprintln!("No section").unwrap();
    }
}

fn print_static_regions() {
    kprintln!("pt: {:X} -> {:X}", (PT_START as u64), (PT_END as u64)).unwrap();
    let (kstart, kend) = unsafe { (&__kernel_start as *const u8, &__kernel_end as *const u8) };
    kprintln!("kern: {:X} -> {:X}", (kstart as u64), (kend as u64)).unwrap();

}

// Not meant to be a robust solution.
fn setup_alloc_or_panic(bootinfo: &BootInformation) {
    let mem_map = bootinfo.memory_map_tag()
        .unwrap()
        .memory_areas();

    let mut used_regions = BufVec::<_, 64>::new();
    if let Some(elf_sections) = bootinfo.elf_sections_tag() {
        for section in elf_sections.sections() {
            used_regions.push(Region::from(&section)).unwrap();
        }
    }
    used_regions.extend(&[
        Region::new(PT_START, PT_END),
        Region::new(bootinfo.start_address() as *const u8, bootinfo.end_address() as *const u8),
    ]).unwrap();

    let mut avail_regions = BufVec::<_, 128>::new();
    for region in mem_map.iter()
            .filter(|area| area.typ() == MemoryAreaType::Available)
            .map(Region::from) 
    { avail_regions.push(region).unwrap(); }

    for used in &used_regions {
        let mut buf = BufVec::<_, 128>::new();
        for avail in &avail_regions {
            for free in avail.subtract(&used).iter().flatten() {
                buf.push(*free).unwrap();
            }
        }
        avail_regions = buf;
    }

    assert!(avail_regions.iter().all(|avail| used_regions.iter().all(|used| !used.intersects(&avail))));

    let max_region = avail_regions.iter()
        .reduce(|r1, r2| if r1.size() > r2.size() {r1} else {r2})
        .unwrap();

    unsafe { 
        simple_alloc::ALLOCATOR.configure(max_region.start(), max_region.end()); 
    }
}

// Remove: kernel code, stack, PTS, GDT, Multiboot struct
// 
// VGA fns
// Setup IDT + enable interrupts
// VMem
//
// Load executable?
// Userspace
// Syscalls

#[unsafe(no_mangle)]
fn rust_main(mb_magic: u32, mbi_ptr: u32) -> ! {
    if mb_magic != multiboot2::MAGIC {
        panic!("Wrong mb_magic");
    }
    let bootinfo = unsafe {  
        BootInformation::load(mbi_ptr as *const BootInformationHeader)
            .unwrap() 
    };

    let mut vga = vga::VGA.lock();
    vga.configure();
    vga.move_cursor(10, 10);
    drop(vga);

    setup_alloc_or_panic(&bootinfo);

    let mut tty = tty::TTY.lock();
    tty.configure();
    drop(tty);

    let mut s = alloc::string::String::new();
    for i in 0..10 {
        s = kfmt!("{s}, current iter: {i}").unwrap();
    }
    kprintln!("{s}").unwrap();

    loop { }
}
