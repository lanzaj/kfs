extern crate lazy_static; 

use core::arch::asm;

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct IdtEntry {
    pub offset_0: u16,
    pub segment_selector: u16,
    pub reserved: u8,
    pub p_dpl_0_gatetype: u8,
    pub offset_48: u16,
}

impl IdtEntry {
    pub fn new(offset:u32) -> IdtEntry {
        IdtEntry {
            offset_0: (offset & 0xffff) as u16,
            segment_selector: 0x08,
            reserved: 0,
            p_dpl_0_gatetype: 0x8e,
            offset_48: ((offset >> 16) &0xffff) as u16,
        }
    }
    pub fn missing() -> IdtEntry {
        IdtEntry {
            offset_0: 0,
            segment_selector:0,
            reserved: 0,
            p_dpl_0_gatetype: 0,
            offset_48: 0,
        }
    }
}

#[repr(C, packed)]
pub struct IdtR {
    pub size:u16,
    pub addr:u32,
}

use crate::print_mem_area;

use self::lazy_static::lazy_static;

lazy_static! {
    #[link_section = ".idt"]
    static ref IDT: [IdtEntry; 256] = {
        let mut idt = [IdtEntry::missing(); 256];
        idt[0] = IdtEntry::new(divide_by_zero_handler as u32);
        idt
    };
}

pub fn init_idt() {
    let idtr = IdtR {size: IDT.len() as u16 * core::mem::size_of::<IdtEntry>() as u16, addr: IDT.as_ptr() as u32};
    unsafe {
        asm!("lidt [{}]", in(reg) &idtr, options(readonly, nostack, preserves_flags));
    }
}

use println;
extern "C" fn divide_by_zero_handler(stack_frame: &ExceptionStackFrame) -> ! {
    let mut stack_frame: u32;
	unsafe {
        asm!("mov {}, esp", out(reg) stack_frame, options(nomem, nostack));
        let stack_frame_ptr: *const ExceptionStackFrame = stack_frame as *const _;
        let stack_frame_ref = &*stack_frame_ptr;
        println!("EXCEPTION: DIVIDE BY ZERO\n{:#?}", stack_frame_ref);
	}
    loop {}
}

#[derive(Debug)]
#[repr(C, packed)]
struct ExceptionStackFrame {
    instruction_pointer: u32,
    code_segment: u32,
    cpu_flags: u32,
    stack_pointer: u32,
    stack_segment: u32,
}