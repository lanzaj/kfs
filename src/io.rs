use println;
use print;

use crate::{dump_stack, print_mem_area, vga_buffer::{Color, WRITER}};

const PS2_DATA_PORT: u16 = 0x60;
const PS2_STATUS_PORT: u16 = 0x64;

pub fn read_status() -> u8 {
    unsafe { io::inb(PS2_STATUS_PORT) }
}

pub fn read_data() -> u8 {
    while (read_status() & 0x01) == 0 {}
    unsafe { io::inb(PS2_DATA_PORT) }
}

// Low-level I/O operations
mod io {
    use core::arch::asm;

    pub unsafe fn inb(port: u16) -> u8 {
        let result: u8;
        // Inline assembly to read from the specified port
        asm!(
            "in al, dx",
            in("dx") port,
            out("al") result,
        );
        result
    }

    pub unsafe fn outb(port: u16, value: u8) {
        // Inline assembly to write to the specified port
        asm!(
            "out dx, al",
            in("dx") port,
            in("al") value,
        );
    }
}

pub fn handle_keyboard_input(scan_code: u8) {
    if scan_code == 26 {
        vga_buffer::WRITER.lock().scroll_up();
        return;
    }
    if scan_code == 27 {
        vga_buffer::WRITER.lock().scroll_down();
        return;
    }
    
    //println!("Scan code: {}", scan_code);
    static mut SHIFT : u8 = 0;
    static mut CAPS : u8 = 0;
    if scan_code == 42 || scan_code == 54 {
        unsafe { SHIFT = SHIFT + 1 };
        return;
    }
    if scan_code == 170 || scan_code == 182 {
        unsafe { SHIFT = SHIFT - 1 };
        return;
    }
    if scan_code == 58 {
        unsafe {
            if CAPS == 0 {
                CAPS = 1;
            }
            else {
                CAPS = 0;
            }
        };
        return;
    }
    const KBD_US: [&str; 59] = [
        "\0", // 0
        "\x1b", // 1 - echap
        "1", // 2
        "2", // 3
        "3", // 4
        "4", // 5
        "5", // 6
        "6", // 7
        "7", // 8
        "8", // 9
        "9", // 10
        "0", // 11
        "-", // 12
        "=", // 13
        "\x08", // 14 - delete key
        "\t", // 15
        "q", // 16
        "w", // 17
        "e", // 18
        "r", // 19
        "t", // 20
        "y", // 21
        "u", // 22
        "i", // 23
        "o", // 24
        "p", // 25
        "[", // 26
        "]", // 27
        "\n", // 28
        "\0", // 29 - Control key
        "a", // 30
        "s", // 31
        "d", // 32
        "f", // 33
        "g", // 34
        "h", // 35
        "j", // 36
        "k", // 37
        "l", // 38
        ";", // 39
        "'", // 40
        "`", // 41
        "\0", // 42 - Shift key
        "\\", // 43
        "z", // 44
        "x", // 45
        "c", // 46
        "v", // 47
        "b", // 48
        "n", // 49
        "m", // 50
        ",", // 51
        ".", // 52
        "/", // 53
        "\0", // 54 - Right Shift key
        "\0", // 55 - *
        "\0", // 56 - Alt key
        " ", // 57 - Space
        "\0", // 58 - Caps Lock
    ];
    const KBD_US_MAJ: [&str; 59] = [
        "\0", // 0
        "\x1b", // 1 - echap
        "!", // 2
        "@", // 3
        "#", // 4
        "$", // 5
        "%", // 6
        "^", // 7
        "&", // 8
        "*", // 9
        "(", // 10
        ")", // 11
        "_", // 12
        "+", // 13
        "\x08", // 14 - delete key
        "\t", // 15
        "Q", // 16
        "W", // 17
        "E", // 18
        "R", // 19
        "T", // 20
        "Y", // 21
        "U", // 22
        "I", // 23
        "O", // 24
        "P", // 25
        "{", // 26
        "}", // 27
        "\n", // 28
        "\0", // 29 - Control key
        "A", // 30
        "S", // 31
        "D", // 32
        "F", // 33
        "G", // 34
        "H", // 35
        "J", // 36
        "K", // 37
        "L", // 38
        ":", // 39
        "\"", // 40
        "~", // 41
        "\0", // 42 - Shift key
        "|", // 43
        "Z", // 44
        "X", // 45
        "C", // 46
        "V", // 47
        "B", // 48
        "N", // 49
        "M", // 50
        "<", // 51
        ">", // 52
        "?", // 53
        "\0", // 54 - Right Shift key
        "\0", // 55 - *
        "\0", // 56 - Alt key
        " ", // 57 - Space
        "\0", // 58 - Caps Lock
    ];
    if (scan_code as usize) < KBD_US.len() && scan_code != '\0' as u8 {
        if unsafe {SHIFT == 0 && CAPS == 0} {
            print!("{}",KBD_US[scan_code as usize]);
        }
        else {
            print!("{}",KBD_US_MAJ[scan_code as usize]);
        }
        if scan_code == 28 {
            call_function("bg_color \n");
        }
    }
}

fn call_function (input: &str) {
    if input.starts_with("color ") {
        WRITER.lock().change_color(Color::White, Color::Black);
    }
    if input.starts_with("bg_color ") {
        WRITER.lock().change_color(Color::LightBlue, Color::Black);
    }
    if input.starts_with("echo ") {
        ft_echo();
        return;
    }
    if input.starts_with("print_memory ") {
        print_mem_area(0x800 as *mut i32,100);
        return;
    }
    match input {
        "dumpstack\n" => dump_stack(),
        "help\n" => ft_help(),
        "ls\n" => ft_ls(),
        "reboot\n" => ft_reboot(),
        _ => return,
    }
}

fn ft_help() {
    println!("dir    foo    bar    .    ..");
}

fn ft_reboot() {
    println!("rebooting ... \n");
}

fn ft_shutdown() {
    println!("powering off... \n");
}

fn ft_nyan_cat() {
    println!("nyan_cat \n");
}

fn ft_ls() {
    println!("dir    foo    bar    .    ..");
}

fn ft_echo() {
    println!("Hello World\n");
}
