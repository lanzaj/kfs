#![feature(lang_items)]
#![no_std]
#![no_main]

mod vga_buffer;

use core::panic::PanicInfo;

#[no_mangle]
pub extern fn k_main() {
    println!("Hello World{}", "!");
    println!("Hello World{}", "!");
    println!("Hello World{}", "!");
    println!("Hello World{}", "!");
    println!("Hello World{}", "!");
    panic!("Ca nous sera vachement utile pour debug");
    loop{}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}