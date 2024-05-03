#![feature(lang_items)]
#![no_std]
#![no_main]

mod vga_buffer;

use core::panic::PanicInfo;

#[no_mangle]
pub extern fn k_main() {
    let hello = b"Hello World!";
    let color_byte = 0x1f;

    let clear = b"               ";
    let clear_color = 0x00;

    let mut clear_screen = [clear_color; 30];
    for (i, char_byte) in clear.into_iter().enumerate() {
        clear_screen[i * 2] = *char_byte;
    }
    let bootingkfs = 0xb8000 as *mut _;
    unsafe {*bootingkfs  = clear_screen};

    let mut hello_colored = [color_byte; 24];
    for (i, char_byte) in hello.into_iter().enumerate() {
        hello_colored[i * 2] = *char_byte;
    }
    let buffer_ptr = (0xb8000 + 1988) as *mut _;
    unsafe { *buffer_ptr = hello_colored };
    loop{}
}

#[lang = "eh_personality"] #[no_mangle] pub extern fn eh_personality() {}

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}