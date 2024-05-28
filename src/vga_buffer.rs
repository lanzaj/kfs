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
const LINE_NB: usize = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    pub fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)] // to guarantee field order is kept
pub struct ScreenChar {
    pub ascii: u8,
    pub color: ColorCode,
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
    column_position: [usize; 3],
    color_code: ColorCode,
    vga_buffer: &'static mut Vgabuffer,
    lines: [Vec; 3],
    scroll: [usize; 3],
    cmd: bool,
    active_tab: usize,
    behind_cursor: ScreenChar
}

impl Writer {
    
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            b'\x08' => {
                if self.column_position[self.active_tab] > 2 {
                    self.column_position[self.active_tab] -= 1;
                    let row = BUFFER_HEIGHT - 1;
                    let col = self.column_position[self.active_tab];
                    if self.vga_buffer.chars[row][0].read().ascii == b'$' {
                        for i in col..(BUFFER_WIDTH - 1) {
                            self.vga_buffer.chars[row][i].write(ScreenChar{
                                ascii: self.vga_buffer.chars[row][i + 1].read().ascii as u8,
                                color: self.color_code,
                            });
                        }
                    }
                    self.vga_buffer.chars[row][BUFFER_WIDTH - 1].write(ScreenChar{
                        ascii: ' ' as u8,
                        color: self.color_code,
                    });
                    self.update_cursor(col);
                }
            },
            byte => {
                if self.column_position[self.active_tab] >= BUFFER_WIDTH {
                    return;
                }
                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position[self.active_tab];
                let color_code = self.color_code;
                if self.vga_buffer.chars[row][0].read().ascii == b'$' {
                    for i in ((col + 1)..(BUFFER_WIDTH - 1)).rev() {
                        self.vga_buffer.chars[row][i].write(ScreenChar{
                            ascii: self.vga_buffer.chars[row][i - 1].read().ascii as u8,
                            color: self.color_code,
                        });
                    }
                }
                self.vga_buffer.chars[row][col].write(ScreenChar{
                    ascii: byte,
                    color: color_code,
                });
                self.column_position[self.active_tab] += 1;
                if self.column_position[self.active_tab] < BUFFER_WIDTH {
                    self.update_cursor(self.column_position[self.active_tab]);
                }
            }
        }
    }

    fn update_cursor(&mut self, position: usize) {
        let row = BUFFER_HEIGHT - 1;
        self.behind_cursor = self.vga_buffer.chars[row][position].read();
        let cursor = match self.active_tab {
            0 => {ScreenChar { ascii: self.behind_cursor.ascii, color: ColorCode((Color::LightBlue as u8) << 4 | (Color::Black as u8)), }},
            1 => {ScreenChar { ascii: self.behind_cursor.ascii, color: ColorCode((Color::Red as u8) << 4 | (Color::Black as u8)), }},
            2 => {ScreenChar { ascii: self.behind_cursor.ascii, color: ColorCode((Color::Green as u8) << 4 | (Color::Black as u8)), }},
            _ => {ScreenChar { ascii: self.behind_cursor.ascii, color: ColorCode((Color::Black as u8) << 4 | (Color::Black as u8)), }}
        };
        self.vga_buffer.chars[row][position].write(cursor);
    }

    pub fn move_cursor(&mut self, offset: i8) {
        let row = BUFFER_HEIGHT - 1;
        let col = self.column_position[self.active_tab];
        if col > BUFFER_WIDTH - 2 && offset > 0 {
            return;
        }
        if col < BUFFER_WIDTH {
            self.vga_buffer.chars[row][col].write(self.behind_cursor);
        }
        if offset > 0 && col < BUFFER_WIDTH {
            self.column_position[self.active_tab] += 1;
        }
        if offset < 0 && col > 2 && col <= BUFFER_WIDTH {
            self.column_position[self.active_tab] -= 1;
        }
        if self.column_position[self.active_tab] < BUFFER_WIDTH {
            self.update_cursor(self.column_position[self.active_tab]);
        }
    }

    fn new_line(&mut self) {
        let mut line = [ScreenChar {
            ascii: b' ',
            color: self.color_code,
        }; BUFFER_WIDTH];
        let row = BUFFER_HEIGHT - 1;
        for col in 0..BUFFER_WIDTH {
            let mut char = self.vga_buffer.chars[row][col].read();
            if char.color != self.color_code && (col > 1 || ( char.ascii != b'$' && char.ascii != b'>')) {
                char.color = self.color_code;
            }
            line[col] = char;
        }
        self.lines[self.active_tab].push_new_line(line);
        self.clear_row(BUFFER_HEIGHT - 1);
        if self.cmd == true {
            self.column_position[self.active_tab] = 2;
        } else {
            self.column_position[self.active_tab] = 0;
        }
        self.scroll[self.active_tab] = 0;
        self.update_vga_buffer();
        if self.cmd == true {
            self.cmd = false;
        }
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
                0x20..=0x7e | b'\n' | b'\x08' => self.write_byte(byte),
                // tab
                b'\t' => self.write_string("    "),
                // unprintable -> does nothing
                _ => {},
            }
        }
    }

    pub fn change_color(&mut self, foreground: Color, background: Color) {
        self.color_code = ColorCode::new(foreground, background);
    }

    pub fn scroll_up(&mut self) {
        if self.lines[self.active_tab].size > BUFFER_HEIGHT - 2 && self.scroll[self.active_tab] < self.lines[self.active_tab].size - (BUFFER_HEIGHT - 2) {
            self.scroll[self.active_tab] += 1;
            self.update_vga_buffer();
        }
    }

    pub fn scroll_down(&mut self) {
        if self.scroll[self.active_tab] > 0 {
            self.scroll[self.active_tab] -= 1;
            self.update_vga_buffer();
        }
    }

    pub fn update_vga_buffer(&mut self) {
        self.print_interface();
        for row in 0..(BUFFER_HEIGHT-2) {
            self.clear_row(row);
            for col in 0..(BUFFER_WIDTH) {
                self.vga_buffer.chars[row][col].write(self.lines[self.active_tab].buffer[(LINE_NB - (BUFFER_HEIGHT - 2) + row + self.lines[self.active_tab].newest - self.scroll[self.active_tab]) % LINE_NB][col]);
            }
        }
    }

    fn print_interface(&mut self) {
        let mut color = Color::LightBlue;
        if self.active_tab == 1 {
            color = Color::Red;
        } else if self.active_tab == 2 {
            color = Color::Green;
        }
        for col in 0..(BUFFER_WIDTH) {
            self.vga_buffer.chars[BUFFER_HEIGHT - 2][col].write(ScreenChar {
                ascii: 0xc4,
                color: ColorCode((Color::Black as u8) << 4 | (color as u8)),
            });
        }
        self.vga_buffer.chars[BUFFER_HEIGHT - 2][68].write(ScreenChar {
            ascii: '/' as u8,
            color: ColorCode((Color::Black as u8) << 4 | (color as u8)),
        });
        self.vga_buffer.chars[BUFFER_HEIGHT - 2][69].write(ScreenChar {
            ascii: ' ' as u8,
            color: ColorCode((Color::Black as u8) << 4 | (color as u8)),
        });
        if self.active_tab == 0 {
            self.vga_buffer.chars[BUFFER_HEIGHT - 2][70].write(ScreenChar {
                ascii: '1' as u8,
                color: ColorCode((color as u8) << 4 | (Color::Black as u8)),
            });
        } else {
            self.vga_buffer.chars[BUFFER_HEIGHT - 2][70].write(ScreenChar {
                ascii: '1' as u8,
                color: ColorCode((Color::Black as u8) << 4 | (color as u8)),
            });
        }
        self.vga_buffer.chars[BUFFER_HEIGHT - 2][71].write(ScreenChar {
            ascii: ' ' as u8,
            color: ColorCode((Color::Black as u8) << 4 | (color as u8)),
        });
        if self.active_tab == 1 {
            self.vga_buffer.chars[BUFFER_HEIGHT - 2][72].write(ScreenChar {
                ascii: '2' as u8,
                color: ColorCode((color as u8) << 4 | (Color::Black as u8)),
            });
        } else {
            self.vga_buffer.chars[BUFFER_HEIGHT - 2][72].write(ScreenChar {
                ascii: '2' as u8,
                color: ColorCode((Color::Black as u8) << 4 | (color as u8)),
            });
        }
        self.vga_buffer.chars[BUFFER_HEIGHT - 2][73].write(ScreenChar {
            ascii: ' ' as u8,
            color: ColorCode((Color::Black as u8) << 4 | (color as u8)),
        });
        if self.active_tab == 2 {
            self.vga_buffer.chars[BUFFER_HEIGHT - 2][74].write(ScreenChar {
                ascii: '3' as u8,
                color: ColorCode((color as u8) << 4 | (Color::Black as u8)),
            });
        } else {
            self.vga_buffer.chars[BUFFER_HEIGHT - 2][74].write(ScreenChar {
                ascii: '3' as u8,
                color: ColorCode((Color::Black as u8) << 4 | (color as u8)),
            });
        }
        self.vga_buffer.chars[BUFFER_HEIGHT - 2][75].write(ScreenChar {
            ascii: ' ' as u8,
            color: ColorCode((Color::Black as u8) << 4 | (color as u8)),
        });
        self.vga_buffer.chars[BUFFER_HEIGHT - 2][76].write(ScreenChar {
            ascii: '/' as u8,
            color: ColorCode((Color::Black as u8) << 4 | (color as u8)),
        });
        if self.cmd == true {
            self.vga_buffer.chars[BUFFER_HEIGHT - 1][0].write(ScreenChar {
                ascii: b'$',
                color: ColorCode((Color::Black as u8) << 4 | (color as u8)),
            });
            self.vga_buffer.chars[BUFFER_HEIGHT - 1][1].write(ScreenChar {
                ascii: b'>',
                color: ColorCode((Color::Black as u8) << 4 | (color as u8)),
            });
            self.vga_buffer.chars[BUFFER_HEIGHT - 1][2].write(ScreenChar {
                ascii: b' ',
                color: ColorCode((color as u8) << 4 | (Color::Black as u8)),
            });
        }
    }

    pub fn set_vga_buffer(&mut self, row:usize, col: usize, byte: u8, color_code: ColorCode) {
        self.vga_buffer.chars[row][col].write(ScreenChar{
            ascii: byte,
            color: color_code,
        });
    }

    pub fn get_last_line(&mut self) -> [ScreenChar; 80] {
        if self.lines[self.active_tab].newest == 0 {
            return self.lines[self.active_tab].buffer[LINE_NB - 1];
        }
        self.lines[self.active_tab].buffer[(self.lines[self.active_tab].newest - 1) % LINE_NB]
    }

    pub fn toggle_cmd(&mut self, state: bool) {
        self.cmd = state;
    }

    pub fn clear_terminal(&mut self) {
        let empty_line = [ScreenChar {
            ascii: b' ',
            color: self.color_code,
        }; BUFFER_WIDTH];
        for i in 0..LINE_NB {
            self.lines[self.active_tab].buffer[i] = empty_line;
        }
        self.lines[self.active_tab].newest = 1;
        self.lines[self.active_tab].size = 1;
    }

    pub fn switch_tab(&mut self, n: usize) {
        if n == 0 {
            if self.active_tab < 2 {
                self.active_tab += 1;
            } else {
                self.active_tab = 0;
            }
        } else {
            self.active_tab = n - 1;
        }
    }
}

use self::lazy_static::lazy_static;
use self::spin::Mutex;

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: [0; 3],
        color_code: ColorCode::new(Color::Yellow, Color::Blue),
        vga_buffer: unsafe { &mut *(0xb8000 as *mut Vgabuffer) },
        lines: [Vec::new(), Vec::new(), Vec::new()],
        scroll: [0; 3],
        cmd: false,
        active_tab: 0,
        behind_cursor: ScreenChar{
            ascii: b' ',
            color: ColorCode::new(Color::White, Color::Black),
        }
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
    println!(
"/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*      Kernel from Scratch                             :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: tgrasset and jlanza                        +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*                                                     #+#    #+#             */
/*                                                    ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */");
    // println!("");
    WRITER.lock().change_color(Color::White, Color::Black);
    println!(
"Welcome to our useless kernel !!!
It can't do much for now and probably never will
But here are a few commands you can use: 
 
help    : This is more or less what you're seeing right now !
echo    : Prints something on the screen, WOW ! <...args>
stack   : Prints the stack
reboot  : Reboots the machine
halt    : Halts the CPU (Why would you do that?)
color   : Changes the writing color <...arg : Color>
There might be other hidden features...");
    WRITER.lock().toggle_cmd(true);
    println!("");
}