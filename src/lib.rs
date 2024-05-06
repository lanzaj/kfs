#![feature(lang_items)]
#![no_std]
#![no_main]

mod vga_buffer;
mod interrupts;
mod gdt;

use core::panic::PanicInfo;

#[no_mangle]
pub extern fn k_main() {
    println!("Hello World{}", "!");
    gdt::init_gdt();
    loop{}
}

//panic!("Ca nous sera vachement utile pour debug");

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}