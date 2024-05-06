extern crate lazy_static; 

use core::arch::asm;

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
            base_middle: ((base & 0xff0000) >> 16) as u8,
            access: access,
            granularity: (((limit & 0xf0000) >> 16) | (other & 0xf0) as u32)  as u8,
            base_high: ((base & 0xff000000) >> 24) as u8,
        }
    }
}

use self::lazy_static::lazy_static;

lazy_static! {
    #[link_section = ".gdt"]
    pub static ref GDT: [GdtEntry; 7] = [
        GdtEntry::new(0x0, 0x0, 0x0, 0x0),          // NULL
        GdtEntry::new(0x0, 0xFFFFF, 0x9B, 0x0D),    // Kernel Code
        GdtEntry::new(0x0, 0xFFFFF, 0x93, 0x0D),    // Kernel Data
        GdtEntry::new(0x0, 0x0, 0x97, 0x0D),        // Kernel Stack
        GdtEntry::new(0x0, 0xFFFFF, 0xFF, 0x0D),    // User Code
        GdtEntry::new(0x0, 0xFFFFF, 0xF3, 0x0D),    // User Data
        GdtEntry::new(0x0, 0x0, 0xF7, 0x0D),        // User Stack
    ];
}

pub fn init_gdt() {
    let gdtr = GdtR {size: GDT.len() as u16 * core::mem::size_of::<GdtEntry>() as u16, addr: GDT.as_ptr() as u32};
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
            // "mov ss, ax",  CETTE MERDE FAIT TOUT PETER
            options(preserves_flags)
        );
    }
}

