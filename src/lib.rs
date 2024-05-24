#![feature(lang_items, asm, naked_functions, core_intrinsics)]
#![no_std]
#![no_main]

mod vga_buffer;
mod gdt;
mod io;
mod tetris;

use core::panic::PanicInfo;

#[no_mangle]
pub extern fn k_main() {
    vga_buffer::print_welcome_screen();
    gdt::init_gdt();
    // dump_stack();
    // print_mem_area(0x800 as *mut i32, 10);
    loop{
        let scan_code = read_data();
        handle_keyboard_input(scan_code);
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

fn print_mem_area(addr: *mut i32, size: usize) {
    let mut addr_value = addr as i32;
    addr_value = addr_value - addr_value % 4;
    let addr_value = addr_value as *mut i32;
    for number in 0..(size / 4) {
        print_mem_line(addr_value.wrapping_add(number * 4));
    }
}

fn print_mem_line(addr: *mut i32) {
    print!("{:010?}   |   ", addr);

    unsafe {
        print!("{:08x}  ", *(addr.wrapping_add(0)) as u32);
        print!("{:08x}  ", *(addr.wrapping_add(1)) as u32);
        print!("{:08x}  ", *(addr.wrapping_add(2)) as u32);
        print!("{:08x}  ", *(addr.wrapping_add(3)) as u32);
        println!("");
    }
}

use core::arch::asm;

use io::{handle_keyboard_input, read_data};

use crate::vga_buffer::{disable_cursor, WRITER};
fn dump_stack() {
    let esp: usize;
	unsafe {
        asm!("mov {}, esp", out(reg) esp, options(nomem, nostack));
	}
	println!("esp: {:08x}", esp);
    print_mem_area(esp as *mut i32, 56);
}