const CMOS_ADDRESS: u16 = 0x70;
const CMOS_DATA: u16 = 0x71;

use core::str::EncodeUtf16;

use crate::{print, println};

pub fn bcd_to_binary(bcd: u8) -> u8 {
    ((bcd & 0xf0) >> 4) * 10 + (bcd & 0x0f)
}

fn read_cmos(register: u8) -> u8 {
    unsafe {
        use crate::io::{inb, outb};
        outb(CMOS_ADDRESS, register);
        inb(CMOS_DATA)
    }
}

fn get_rtc_time() -> (u8, u8, u8, u16) {
    let seconds = bcd_to_binary(read_cmos(0x00));
    let minutes = bcd_to_binary(read_cmos(0x02));
    let hours = bcd_to_binary(read_cmos(0x04));
    let milliseconds = bcd_to_binary(read_cmos(0x00)) as u16 * 1000 / 60;

    (hours, minutes, seconds, milliseconds)
}

fn get_rtc_date() -> (u8, u8, u8) {
    let year = bcd_to_binary(read_cmos(0x09));
    let month = bcd_to_binary(read_cmos(0x08));
    let day = bcd_to_binary(read_cmos(0x07));

    (year, month, day)
}

fn time() {
    let (hours, minutes, seconds, milliseconds) = get_rtc_time();
    println!("{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, milliseconds);
}

pub fn date() {
    let (hours, minutes, seconds, milliseconds) = get_rtc_time();
    let (year, month, day) = get_rtc_date();

    let full_year = 2000 + year as u16;

    println!(
        "{:02}/{:02}/{:04} {:02}:{:02}:{:02}.{:03}",
        day, month, full_year, hours, minutes, seconds, milliseconds
    );
}

///////////////////

use crate::vga_buffer::{ColorCode, Color, WRITER};

fn  draw_char(row: usize, col:usize, char: u8, foreground: Color, background: Color) {
    WRITER.lock().set_vga_buffer(row, col, char, ColorCode::new(foreground, background));
}

fn  draw_str(row: usize, col:usize, s: &str, foreground: Color, background: Color) {
    for (index, c) in s.chars().enumerate() {
        draw_char(row, col + index, c as u8, foreground, background);
    }
}

fn  draw_digit(row:usize, col:usize, n:u8, foreground: Color, background: Color) {
    draw_char(row, col, ('0' as u8).wrapping_add(n), foreground, background)
}

fn  draw_nbr(row:usize, col:usize, n:u32, foreground: Color, background: Color) {
    if n >= 10 {
        draw_nbr(row, col - 1, n/10, foreground, background);
    }
    draw_digit(row, col, (n%10) as u8, foreground, background);
}

fn  clear_window() {
    for row in 0..25 {
        for col in 0..80 {
            draw_char(row, col, 32, Color::White, Color::Black);
        }
    }
}

fn  draw_rectangle(row_up: usize, row_down: usize, col_left:usize, col_right:usize) {
    for row in (row_up + 1)..row_down {
        draw_char(row, col_left, 0xB3, Color::White, Color::Black)
    }
    for row in (row_up + 1)..row_down {
        draw_char(row, col_right, 0xB3, Color::White, Color::Black)
    }
    for col in (col_left + 1)..col_right {
        draw_char(row_up, col, 0xC4, Color::White, Color::Black)
    }
    for col in (col_left + 1)..col_right {
        draw_char(row_down, col, 0xC4, Color::White, Color::Black)
    }
    draw_char(row_up, col_left, 0xDA, Color::White, Color::Black);
    draw_char(row_up, col_right, 0xBF, Color::White, Color::Black);
    draw_char(row_down, col_left, 0xC0, Color::White, Color::Black);
    draw_char(row_down, col_right, 0xD9, Color::White, Color::Black);
}

fn draw_text() {
    draw_str(1, 37, "Tetris", Color::White, Color::Black);
    draw_str(1, 16, "Stats", Color::White, Color::Black);
    draw_str(1, 60, "Next", Color::White, Color::Black);
    draw_str(8, 60, "Help", Color::White, Color::Black);
    draw_str(2, 11, "Score", Color::White, Color::Black);
    draw_str(4, 11, "Level", Color::White, Color::Black);
}

fn draw_bg() {
    for row in 2..24 {
        for col in (31..51).step_by(2) {
            draw_char(row, col, '.' as u8, Color::DarkGray, Color::Black);
        }
    }
}

fn draw_cell(x:usize, y: usize, color: Color) {
    let col = x * 2 + 30;
    let row = 23 - y;

    for i in 0..2 {
        draw_char(row, col + i, 32, color, color);
    }
}
fn draw_next(tetraminos: char) {
    match tetraminos {
        'I' => {
            draw_cell(14, 19, Color::Cyan);
            draw_cell(15, 19, Color::Cyan);
            draw_cell(16, 19, Color::Cyan);
            draw_cell(17, 19, Color::Cyan);
        }
        'J' => {
            draw_cell(15, 20, Color::Blue);
            draw_cell(15, 19, Color::Blue);
            draw_cell(16, 19, Color::Blue);
            draw_cell(17, 19, Color::Blue);
        }
        'L' => {
            draw_cell(15, 19, Color::Brown);
            draw_cell(16, 19, Color::Brown);
            draw_cell(17, 19, Color::Brown);
            draw_cell(17, 20,Color::Brown);
        }
        'O' => {
            draw_cell(15, 19, Color::Yellow);
            draw_cell(15, 20, Color::Yellow);
            draw_cell(16, 19, Color::Yellow);
            draw_cell(16, 20,Color::Yellow);
        }
        'S' => {
            draw_cell(15, 19, Color::LightGreen);
            draw_cell(16, 19, Color::LightGreen);
            draw_cell(16, 20,Color::LightGreen);
            draw_cell(17, 20, Color::LightGreen);
        }
        'Z' => {
            draw_cell(15, 20, Color::Red);
            draw_cell(16, 19, Color::Red);
            draw_cell(16, 20,Color::Red);
            draw_cell(17, 19, Color::Red);
        }
        'T' => {
            draw_cell(15, 19, Color::Magenta);
            draw_cell(16, 19, Color::Magenta);
            draw_cell(16, 20,Color::Magenta);
            draw_cell(17, 19, Color::Magenta);
        }
        _ => panic!("not a tetraminos"),
    }
}

fn draw_game_board() {
    draw_rectangle(1, 24, 29, 50);
    draw_rectangle(1, 5, 10, 26);
    draw_rectangle(1, 6, 53, 71);
    draw_rectangle(8, 20, 53, 71);
    draw_text();
    draw_bg();

    for i in 0..10 {
        if i % 2 == 0 {
            draw_cell(i, 0, Color::Cyan);
        }
        else {
            draw_cell(i, 0, Color::LightGreen)
        }
    }
    for i in 0..22 {
        if i % 2 == 0 {
            draw_cell(0, i, Color::Cyan);
        }
        else {
            draw_cell(0, i, Color::LightGreen)
        }
    }
}

fn display_game(data: Data) {
    draw_nbr(2, 24, 42, Color::White, Color::Black);
    draw_nbr(4, 24, 0, Color::White, Color::Black);
    draw_next('T');
}

#[derive(Debug, Clone, Copy)]
struct Data {
    board: [[u8; 22]; 10],
    end: bool,
    level: u8,
    score: u32,
    scan_code: u8,
}

impl Data {
    pub fn new() -> Self {
        Data {
            board: [[0; 22]; 10],
            end: false,
            level: 1,
            score: 0,
            scan_code: 0,
        }
    }
}

use crate::io::read_data;

pub fn handle_keyboard_input(data: &mut Data) {
    if data.scan_code == 1 {
        data.end = true;
        return;
    }
}


pub fn ft_tetris() {
    let mut data: Data = Data::new();
    clear_window();
    draw_game_board();
    loop {
        if data.end {
            WRITER.lock().toggle_cmd(true);
            println!("See you soon!                                                            ");
            break;
        }
        data.scan_code = read_data();
        handle_keyboard_input(&mut data);
        display_game(data);
    }
}