#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![allow(dead_code)]

use crate::tty::{vga, vga::VgaAttr};
use crate::common::region::Region;
use crate::common::buf_vec::BufVec;
use multiboot2::{BootInformation, BootInformationHeader, MemoryAreaType, MemoryArea};
use alloc::{fmt::format, format, string::{String, ToString}, vec::Vec};

mod panic;
mod simple_alloc;
mod spinlock;
mod math;
mod common;
mod arch;
mod tty;

extern crate alloc;

unsafe extern "C" {
    static __kernel_start: u8;
    static __kernel_end: u8;
}


const PT_START: *const u8 = 0x1000 as *const u8;
const PT_END: *const u8 = 0x20000 as *const u8;

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
    used_regions.extend(&unsafe {[
        Region::new(&__kernel_start, &__kernel_end),
        Region::new(PT_START, PT_END),
        Region::new(bootinfo.start_address() as *const u8, bootinfo.end_address() as *const u8),
    ]}).unwrap();

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
        // let start = max_region.start();
        // simple_alloc::ALLOCATOR.configure(start, start.add(8)); 
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

    let mut vga = vga::VGA.lock();

    let s = kfmt!("magic pointer is: {} (and in hex): {:X}", mb_magic, mb_magic);
    if let Some(s) = s {
        vga.write_bytes_default(s.as_bytes(), 0, 0);
    } else {
        vga.write_bytes_default(b"No memory :()", 0, 0);
    }




    // let mem_map = bootinfo.memory_map_tag()
    //     .unwrap()
    //     .memory_areas();
    // let attr = VgaAttr::from_fg(vga::FgColor::Green)
    //     .with_bg(vga::BgColor::Brown);
    //
    // for (row, region) in mem_map.iter()
    //     .filter(|r| r.typ() == MemoryAreaType::Available)
    //     .enumerate() 
    // {
    //     let region: &MemoryArea = region;
    //     let region_str: String = format!("{:X} -> {:X} ({:?})", region.start_address(), region.end_address(), region.typ());
    //     vga.write_bytes(region_str.as_bytes(), attr, row as u8, 0);
    // }
    //
    // let attr = VgaAttr::from_fg(vga::FgColor::Magenta)
    //     .with_bg(vga::BgColor::LightGray);
    // for (row, region) in mem_map.iter()
    //     .filter(|r| r.typ() != MemoryAreaType::Available)
    //     .enumerate() 
    // {
    //     let region: &MemoryArea = region;
    //     let region_str: String = format!("{:X} -> {:X} ({:?})", 
    //         region.start_address(), region.end_address(), region.typ());
    //     vga.write_bytes(region_str.as_bytes(), attr, row as u8 + 4, 0);
    // }
    //
    // if let Some(elf_sections) = bootinfo.elf_sections_tag() {
    //     for (row, section) in elf_sections.sections().enumerate() {
    //         let section_name = section.name().unwrap_or("[no name]");
    //         let section_str: String = format!("section {}: {:X} -> {:X}", 
    //             section_name,
    //             section.start_address(), 
    //             section.end_address());
    //         vga.write_bytes(section_str.as_bytes(), attr, row as u8, 0);
    //     }
    // } else {
    //     vga.write_bytes(b"none", attr, 10, 0);
    // }
    //
    // let start = 20;
    // let attr = VgaAttr::from_fg(vga::FgColor::BrightRed)
    //     .with_bg(vga::BgColor::Green)
    //     .with_blink();
    // let pt_str: String = format!("pt: {:X} -> {:X}", (PT_START as u64), (PT_END as u64));
    // vga.write_bytes(pt_str.as_bytes(), attr, start + 1, 0);
    // let (kstart, kend) = unsafe { (&__kernel_start as *const u8, &__kernel_end as *const u8) };
    // let pt_str: String = format!("kern: {:X} -> {:X}", (kstart as u64), (kend as u64));
    // vga.write_bytes(pt_str.as_bytes(), attr, start + 2, 0);

    loop { }
}
