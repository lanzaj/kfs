#![feature(naked_functions)]
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
    let mut previous_segment: *mut i32 = addr_value;
    let mut pass = false;
    for number in 0..(size / 4) {
        let addr = addr_value.wrapping_add(number * 4);
        let identical = identical_segments(previous_segment, addr, number);
        if identical && pass == false {
            println!("[...]");
            pass = true;
        } else if !identical {
            print_mem_line(addr);
            pass = false;
        }
        previous_segment = addr
    }
}

fn identical_segments(previous: *mut i32, new: *mut i32, n: usize) -> bool {
    if n == 0 {
        return false;
    }
    unsafe {
        let previous_a = *(previous.wrapping_add(0));
        let previous_b = *(previous.wrapping_add(1));
        let previous_c = *(previous.wrapping_add(2));
        let previous_d = *(previous.wrapping_add(3));
        let new_a = *(new.wrapping_add(0));
        let new_b = *(new.wrapping_add(1));
        let new_c = *(new.wrapping_add(2));
        let new_d = *(new.wrapping_add(3));
        if previous_a == new_a && previous_b == new_b && previous_c == new_c && previous_d == new_d {
            return true
        }
        false
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

use io::{handle_keyboard_input, read_data};