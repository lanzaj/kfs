extern crate volatile;
extern crate lazy_static;
extern crate spin;


#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black      = 0,
    Blue       = 1,
    Green      = 2,
    Cyan       = 3,
    Red        = 4,
    Magenta    = 5,
    Brown      = 6,
    LightGray  = 7,
    DarkGray   = 8,
    LightBlue  = 9,
    LightGreen = 10,
    LightCyan  = 11,
    LightRed   = 12,
    Pink       = 13,
    Yellow     = 14,
    White      = 15,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;
const LINE_NB: usize = 1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)] // to guarantee field order is kept
struct ScreenChar {
    ascii: u8,
    color: ColorCode,
}

// since we only write to a buffer and never read from it, we need to
// make sure these writes won't be optimized by future versions of rust
// compiler. Making the screenChars volatile tells the compiler that they
// are absolutely necessary
use self::volatile::Volatile; 

struct Vec {
    buffer: [[ScreenChar; BUFFER_WIDTH]; LINE_NB],
    oldest: usize,
    newest: usize,
    size: usize,
}

impl Vec {
    fn new() -> Self {
        Vec {
            buffer: [[ScreenChar {
                ascii: b' ',
                color: ColorCode((Color::Black as u8) << 4 | (Color::White as u8)),
            }; BUFFER_WIDTH]; LINE_NB],
            oldest: 0,
            newest: 0,
            size: 0,
        }
    }

    fn push_new_line(&mut self, line: [ScreenChar; BUFFER_WIDTH]) {
        if self.size == self.buffer.len() {
            self.pop_oldest_line();
        }
        self.buffer[self.newest] = line;
        self.newest = (self.newest + 1) % self.buffer.len();
        self.size += 1
    }

    fn pop_oldest_line(&mut self) -> Option<[ScreenChar; BUFFER_WIDTH]> {
        if self.size == 0 {
            None
        } else {
            let line = self.buffer[self.oldest];
            self.oldest = (self.oldest + 1) % self.buffer.len();
            self.size -= 1;
            Some(line)
        }
    }
}

#[repr(transparent)]
struct Vgabuffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    vga_buffer: &'static mut Vgabuffer,
    lines: Vec,
    scroll: usize,
}

impl Writer {
    
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }
                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;
                let color_code = self.color_code;
                self.vga_buffer.chars[row][col].write(ScreenChar{
                    ascii: byte,
                    color: color_code,
                });
                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self) {
        let mut line = [ScreenChar {
            ascii: b' ',
            color: self.color_code,
        }; BUFFER_WIDTH];
        let row = BUFFER_HEIGHT - 1;
        for col in 0..BUFFER_WIDTH {
            line[col] = self.vga_buffer.chars[row][col].read();
        }
        self.lines.push_new_line(line);
        self.column_position = 0;
        self.clear_row(BUFFER_HEIGHT - 1);
        self.scroll = 0;
        self.update_vga_buffer();
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii: b' ',
            color: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.vga_buffer.chars[row][col].write(blank);
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // unprintable -> prints a ■
                _ => self.write_byte(0xfe),
            }
        }
    }

    pub fn change_color(&mut self, foreground: Color, background: Color) {
        self.color_code = ColorCode::new(foreground, background);
    }

    pub fn scroll_up(&mut self) {
        if self.lines.size > BUFFER_HEIGHT - 2 && self.scroll < self.lines.size - (BUFFER_HEIGHT - 2) /* && self.scroll < LINE_NB - (BUFFER_HEIGHT - 2) */{
            self.scroll += 1;
            self.update_vga_buffer();
        }
    }

    pub fn scroll_down(&mut self) {
        if self.scroll > 0 {
            self.scroll -= 1;
            self.update_vga_buffer();
        }
    }

    fn update_vga_buffer(&mut self) {
        // On pourra mettre ca ailleurs vu qu'il y a pas besoin de redessiner a chaque fois
        // la ligne de demarcation de la cmd line
        for col in 0..(BUFFER_WIDTH) {
            self.vga_buffer.chars[BUFFER_HEIGHT - 2][col].write(ScreenChar {
                ascii: b'_',
                color: self.color_code,
            });
        }
        for row in 0..(BUFFER_HEIGHT-2) {
            self.clear_row(row);
            for col in 0..(BUFFER_WIDTH) {
                self.vga_buffer.chars[row][col].write(self.lines.buffer[(LINE_NB - (BUFFER_HEIGHT - 2) + row + self.lines.newest - self.scroll) % LINE_NB][col]);
            }
        }
    }
}

use self::lazy_static::lazy_static;
use self::spin::Mutex;

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Blue),
        vga_buffer: unsafe { &mut *(0xb8000 as *mut Vgabuffer) },
        lines: Vec::new(),
        scroll: 0,
    });
}

use core::arch::asm;
use core::fmt;

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

const VGA_COMMAND_PORT: u16 = 0x3D4;
const VGA_DATA_PORT: u16 = 0x3D5;

pub fn disable_cursor() {
    unsafe {
        // Send command to VGA controller to disable cursor
        asm!(
            "out dx, al", 
            in("dx") VGA_COMMAND_PORT, 
            in("al") 0x0Au8
        );

        // Send value 0x20 to the data port to disable cursor
        asm!(
            "out dx, al", 
            in("dx") VGA_DATA_PORT, 
            in("al") 0x20u8
        );
    }
}

pub fn print_welcome_screen() {
    disable_cursor();
    print!(
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
/* ************************************************************************** */");
    WRITER.lock().change_color(Color::White, Color::Black);
    println!("");
}