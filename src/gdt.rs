extern crate lazy_static; 

use core::arch::asm;

use println;

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct GdtEntry {
    pub limit_low: u16,
    pub base_low: u16,
    pub base_middle: u8,
    pub access: u8,
    pub granularity: u8,
    pub base_high: u8,
}

#[repr(C, packed)]
pub struct GdtR {
    pub size:u16,
    pub addr:u32,
}

impl GdtEntry {
    pub fn new(base: u32 , limit: u32, access: u8, other:u8) -> GdtEntry {
        GdtEntry {
            limit_low: (limit & 0xffff) as u16,
            base_low: (base & 0xffff) as u16,
            base_middle: ((base >> 16) & 0xff) as u8,
            access: access,
            granularity: (other & 0xf0) | (((limit >> 16) & 0x0f) as u8),
            base_high: ((base >> 24) & 0xff) as u8,
        }
    }
}

use crate::{print_mem_area, print_mem_line};

pub fn init_gdt() {
    let gdt: [GdtEntry; 7] = [
        GdtEntry::new(0x0, 0x0, 0x0, 0x0),          // NULL
        GdtEntry::new(0x0, 0xFFFFF, 0x9A, 0xCF),    // Kernel Code
        GdtEntry::new(0x0, 0xFFFFF, 0x92, 0xCF),    // Kernel Data
        GdtEntry::new(0x0, 0x0, 0x98, 0xCF),        // Kernel Stack
        GdtEntry::new(0x0, 0xFFFFF, 0xFA, 0xCF),    // User Code
        GdtEntry::new(0x0, 0xFFFFF, 0xF2, 0xCF),    // User Data
        GdtEntry::new(0x0, 0x0, 0xF8, 0xCF),        // User Stack
    ];
    let dest_addr = 0x800 as *mut GdtEntry;
    for (index, entry) in gdt.iter().enumerate() {
        unsafe {
            let dest_entry = dest_addr.offset(index as isize);
            core::ptr::write(dest_entry, *entry);
        }
    }
    let gdtr = GdtR {size: gdt.len() as u16 * core::mem::size_of::<GdtEntry>() as u16, addr: dest_addr as u32};
    unsafe { 
        asm!(
            "lgdt [{0}]",
            in(reg) &gdtr,
            options(readonly, nostack, preserves_flags)
        );
        asm!(
            "mov ax, 0x10",
            "mov ds, ax",
            "mov es, ax",
            "mov fs, ax",
            "mov gs, ax",
            "mov ss, ax", 
            options(preserves_flags)
        );
    }
    // println!("{:x}", GDT.as_ptr() as u32);
    // print_mem_area(0x00158014 as *mut i32, 92);
}

