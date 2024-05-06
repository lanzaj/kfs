#![feature(lang_items)]
#![no_std]
#![no_main]

mod vga_buffer;
mod interrupts;
mod gdt;

use core::panic::PanicInfo;

#[no_mangle]
pub extern fn k_main() {
    println!(
"/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*      kfs                                             :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: tgrasset and jlanza                        +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*                                                     #+#    #+#             */
/*                                                    ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */
\n\n\n\n\n\n\n\n\n\n\n\n");
    gdt::init_gdt();
    loop{}
}

//panic!("Ca nous sera vachement utile pour debug");

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}