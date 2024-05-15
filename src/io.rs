use println;

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
    // Here you can process the scan code to determine which key was pressed
    // For simplicity, let's just print the scan code to the serial port
    println!("Scan code: {}", scan_code);
}