use core::arch::asm;

use println;
use print;

use crate::{print_mem_area, tetris, vga_buffer::{self, Color, WRITER}};

const PS2_DATA_PORT: u16 = 0x60;
const PS2_STATUS_PORT: u16 = 0x64;

pub fn read_status() -> u8 {
    unsafe { inb(PS2_STATUS_PORT) }
}

pub fn read_data() -> u8 {
    while (read_status() & 0x01) == 0 {}
    unsafe { inb(PS2_DATA_PORT) }
}

pub fn try_read_data() -> u8 {
    if (read_status() & 0x01) != 0 {
        unsafe { return inb(PS2_DATA_PORT) }
    }
    return 0;
}

// Low-level I/O operations

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


pub fn handle_keyboard_input(scan_code: u8) {
    if scan_code == 72 {
        vga_buffer::WRITER.lock().scroll_up();
        return;
    }
    if scan_code == 80 {
        vga_buffer::WRITER.lock().scroll_down();
        return;
    }
    if scan_code == 75 {
        vga_buffer::WRITER.lock().move_cursor(-1);
    }
    if scan_code == 77 {
        vga_buffer::WRITER.lock().move_cursor(1);
    }
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
            let cmd = WRITER.lock().get_last_line();
            let mut tmp: [u8; 80] = [0; 80];
            for (i, char) in cmd.iter().enumerate() {
                tmp[i] = char.ascii;
            }
            let str = match core::str::from_utf8(&tmp) {
                Ok(s) => s,
                Err(_) => {
                    WRITER.lock().toggle_cmd(true);
                    println!("Unprintable characters spotted");
                    return;
                }
            };
            if let Some(cmd) = str.strip_prefix("$>") {
                if !cmd.trim().is_empty() {
                    call_function(str);
                } else {
                    WRITER.lock().toggle_cmd(true);
                    println!("");
                }
            }
        }
    }
}

fn call_function (input: &str) {
    WRITER.lock().toggle_cmd(false);
    let mut split = input[2..].split_whitespace();
    if let Some(cmd) = split.next() {
        match cmd {
            "color" => {
                ft_color(input);
            }
            "echo" => {
                ft_echo(input);
            }
            "stack" => {
                ft_dump_stack(input);
            }
            "help" => {
                ft_help();
            }
            "42" => {
                ft_42();
            }
            "reboot" => {
                ft_reboot();
            }
            "halt" => {
                ft_halt();
            }
            "tetris" => {
                tetris::ft_tetris();
            }
            "clear" => {
                ft_clear();
            }
            "gdt" => {
                ft_gdt();
            }
            "s" => {
                ft_switch_tab(0);
            }
            "1" => {
                ft_switch_tab(1);
            }
            "2" => {
                ft_switch_tab(2);
            }
            "3" => {
                ft_switch_tab(3);
            }
            _ => {
                WRITER.lock().toggle_cmd(true);
                println!("kfs: {}: command not found", cmd);
            }
        }
    }
}

fn ft_clear() {
    WRITER.lock().clear_terminal();
    WRITER.lock().toggle_cmd(true);
    println!("");
}

fn ft_42() {
    WRITER.lock().toggle_cmd(true);
    println!("Outstanding kfs1 project: 42");
}

fn ft_help() {
    println!("Welcome to our useless kernel !!!");
    println!("It can't do much for now and probably never will");
    println!("But here are a few commands you can use: ");
    println!("   ");
    println!("help : This is what you're seeing right now !");
    println!("echo : Prints something on the screen, WOW !");
    println!("stack   : Prints the stack");
    println!("reboot  : Reboots the machine");
    println!("halt    : Halts the CPU (Why would you do that?)");
    println!("color   : Changes the writing color <...arg : Color>");
    println!("42      : Prints 42 for kfs1's subject");
    println!("clear   : Clears the screen");
    println!("gdt     : Prints the Global Descriptor Table's memory space");
    println!("s/1/2/3 : Switch tab");
    WRITER.lock().toggle_cmd(true);
    println!("There might be other hidden features...");

}

fn ft_color(input: &str) {
    if let Some(i) = input.find("color ") { 
        let color_str = input[i + 6..].trim_start();
        if let Some(color_word) = color_str.split_whitespace().next() {
            match color_word {
                "blue" => {WRITER.lock().change_color(Color::Blue, Color::Black);},
                "green" => {WRITER.lock().change_color(Color::Green, Color::Black);},
                "cyan" => {WRITER.lock().change_color(Color::Cyan, Color::Black);},
                "red" => {WRITER.lock().change_color(Color::Red, Color::Black);},
                "magenta" => {WRITER.lock().change_color(Color::Magenta, Color::Black);},
                "brown" => {WRITER.lock().change_color(Color::Brown, Color::Black);},
                "lightgray" => {WRITER.lock().change_color(Color::LightGray, Color::Black);},
                "darkgray" => {WRITER.lock().change_color(Color::DarkGray, Color::Black);},
                "lightblue" => {WRITER.lock().change_color(Color::LightBlue, Color::Black);},
                "lightgreen" => {WRITER.lock().change_color(Color::LightGreen, Color::Black);},
                "lightcyan" => {WRITER.lock().change_color(Color::LightCyan, Color::Black);},
                "lightred" => {WRITER.lock().change_color(Color::LightRed, Color::Black);},
                "pink" => {WRITER.lock().change_color(Color::Pink, Color::Black);},
                "yellow" => {WRITER.lock().change_color(Color::Yellow, Color::Black);},
                "white" => {WRITER.lock().change_color(Color::White, Color::Black);},
                _ => {
                    WRITER.lock().toggle_cmd(true);
                    println!("Invalid color: {}", color_word);
                    return;
                }
            }
            WRITER.lock().toggle_cmd(true);
            println!("Now writing in {}", color_word);
        } else {
            println!("Please provide a color among those :");
            println!("    blue, green, cyan, red, magenta, brown, lightgray");
            println!("    darkgray, lightblue, lightgreen, lightcyan, lightcyan");
            WRITER.lock().toggle_cmd(true);
            println!("    lightred, pink, yellow or white.");
        }
    }
}

fn ft_halt() {
    unsafe {
		asm!("hlt", options(nomem, nostack, preserves_flags));
	}
}

fn ft_reboot() {
    println!("rebooting ... \n");
    unsafe {
        outb(0x64, 0xfe);
        loop {}
    }
}

fn ft_echo(input: &str) {
    WRITER.lock().toggle_cmd(true);
    if let Some(i) = input.find("echo ") {
        println!("{}", &input[i + 5..].trim_start());
    } else {
        println!("{}", input);
    }
}

fn ft_gdt() {
    println!("Global Descriptor Table (located at 0x800)");
    print_mem_area(0x800 as *mut i32, 16);
    WRITER.lock().toggle_cmd(true);
    println!("-----end of gdt at 0x838------");
}

fn ft_switch_tab(n: usize) {
    WRITER.lock().switch_tab(n);
    WRITER.lock().toggle_cmd(true);
    println!("");

}

extern "C" {
    static stack_bottom: u8;
    static stack_top: u8;
}

fn ft_dump_stack(input: &str) {
    if let Some(i) = input.find("stack ") { 
        let size_str = input[i + 6..].trim_start();
        if let Some(size_word) = size_str.split_whitespace().next() {
            let size = atousize(size_word);
            match size {
                Some(num) => {
                    unsafe {
                        let bottom = &stack_bottom as *const u8 as usize;
                        let top = &stack_top as *const u8 as usize;
                        if num > top - bottom {
                            WRITER.lock().toggle_cmd(true);
                            println!("Value given bigger than kernel stack...");
                            return ;
                        }
                        println!("Stack from {:#x} to {:#x}", bottom, bottom + num);
                        print_mem_area(bottom as *mut i32, num);
                        WRITER.lock().toggle_cmd(true);
                        println!("-----end of stack segment------");
                    }
                }
                None => {
                    WRITER.lock().toggle_cmd(true);
                    println!("Please provide a numeric value corresponding to the size you want to read.");
                    return ;
                }
            }
        } else {
            WRITER.lock().toggle_cmd(true);
            println!("Please provide a numeric value corresponding to the size you want to read.");
        }
    }
}

fn atousize(s: &str) -> Option<usize> {
    let mut result: usize = 0;
    let chars = s.chars().peekable();

    for c in chars {
        if let Some(digit) = c.to_digit(10) {
            result = result.checked_mul(10)?.checked_add(digit as usize)?;
        } else {
            return None;
        }
    }
    Some(result)
}
