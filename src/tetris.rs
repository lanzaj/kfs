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
    draw_str(2, 37, "Tetris", Color::White, Color::Black);
    draw_str(2, 16, "Stats", Color::White, Color::Black);
    draw_str(2, 60, "Next", Color::White, Color::Black);
    draw_str(9, 60, "Help", Color::White, Color::Black);
    draw_str(3, 11, "Score", Color::White, Color::Black);
    draw_str(5, 11, "Level", Color::White, Color::Black);
}

fn draw_bg() {
    for row in 3..23 {
        for col in (31..51).step_by(2) {
            draw_char(row, col, '.' as u8, Color::DarkGray, Color::Black);
        }
    }
}

fn draw_cell(x:usize, y: usize, color: Color) {
    let col = x * 2 + 30;
    let row = 22 - y;

    for i in 0..2 {
        draw_char(row, col + i, 32, color, color);
    }
}

fn draw_empty_cell(x:usize, y: usize, color: Color) {
    let col = x * 2 + 30;
    let row = 22 - y;
    draw_char(row, col, ' ' as u8, Color::Black, Color::Black);
    draw_char(row, col + 1, '.' as u8, color, Color::Black);
}

fn draw_next(tetraminos: char) {
    match tetraminos {
        'I' => {
            draw_cell(14, 18, Color::Cyan);
            draw_cell(15, 18, Color::Cyan);
            draw_cell(16, 18, Color::Cyan);
            draw_cell(17, 18, Color::Cyan);
        }
        'J' => {
            draw_cell(15, 18, Color::Blue);
            draw_cell(15, 17, Color::Blue);
            draw_cell(16, 17, Color::Blue);
            draw_cell(17, 17, Color::Blue);
        }
        'L' => {
            draw_cell(15, 17, Color::Brown);
            draw_cell(16, 17, Color::Brown);
            draw_cell(17, 17, Color::Brown);
            draw_cell(17, 18,Color::Brown);
        }
        'O' => {
            draw_cell(15, 17, Color::Yellow);
            draw_cell(15, 18, Color::Yellow);
            draw_cell(16, 17, Color::Yellow);
            draw_cell(16, 18,Color::Yellow);
        }
        'S' => {
            draw_cell(15, 17, Color::LightGreen);
            draw_cell(16, 17, Color::LightGreen);
            draw_cell(16, 18,Color::LightGreen);
            draw_cell(17, 18, Color::LightGreen);
        }
        'Z' => {
            draw_cell(15, 18, Color::Red);
            draw_cell(16, 17, Color::Red);
            draw_cell(16, 18,Color::Red);
            draw_cell(17, 17, Color::Red);
        }
        'T' => {
            draw_cell(15, 17, Color::Magenta);
            draw_cell(16, 17, Color::Magenta);
            draw_cell(16, 18,Color::Magenta);
            draw_cell(17, 17, Color::Magenta);
        }
        _ => panic!("not a tetraminos"),
    }
}

fn draw_game_ui() {
    draw_rectangle(2, 23, 29, 50);
    draw_rectangle(2, 6, 10, 26);
    draw_rectangle(2, 7, 53, 71);
    draw_rectangle(9, 21, 53, 71);
    draw_text();
    draw_bg();

    // draw_scale();
    
}
fn  draw_scale() {
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

fn  draw_board(data: Data) {
    for x in 0..10 {
        for y in 0..20 {
            match data.board[x][y] {
                1 => draw_cell(x, y, Color::Cyan),
                2 => draw_cell(x, y, Color::Blue),
                3 => draw_cell(x, y, Color::Brown),
                4 => draw_cell(x, y, Color::Yellow),
                5 => draw_cell(x, y, Color::LightGreen),
                6 => draw_cell(x, y, Color::Red),
                7 => draw_cell(x, y, Color::Magenta),
                _ => draw_empty_cell(x, y, Color::DarkGray),
            }
        }
    }
}

fn  name_to_index(c: char) -> usize {
    match c {
        'I' => 0,
        'J' => 1,
        'L' => 2,
        'O' => 3,
        'S' => 4,
        'Z' => 5,
        'T' => 6,
        _ => 7,
    }
}
fn  draw_current_tetrominos(data: Data) {
    for y in 0..4 {
        for x in 0..4 {
            if rot_array[name_to_index(data.current)][data.rot][y][x] != 0 {
                draw_cell((data.pos.x + x as i32)as usize, (data.pos.y + 4 - y as i32) as usize, data.color)
            }
        }
    }
}

fn  display_game(data: Data) {
    draw_board(data);
    draw_nbr(3, 24, data.score, Color::White, Color::Black);
    draw_nbr(5, 24, data.level, Color::White, Color::Black);
    draw_next(data.next);
    draw_current_tetrominos(data);
}

#[derive(Debug, Clone, Copy)]
struct Coord {
    x: i32,
    y: i32,
}

#[derive(Debug, Clone, Copy)]
struct Data {
    board: [[u8; 22]; 10],
    end: bool,
    level: u32,
    score: u32,
    scan_code: u8,
    next: char,
    current: char,
    pos: Coord,
    rot: usize,
    color: Color,
    tick: usize,
    input_cooldown: [usize; 255],
}

impl Data {
    pub fn new() -> Self {
        Data {
            board: [[0; 22]; 10],
            end: false,
            level: 1,
            score: 0,
            scan_code: 0,
            next: 'T',
            current: 'T',
            pos: Coord { x: 3, y: 15 },
            rot: 0,
            color: Color::Magenta,
            tick: 0,
            input_cooldown: [0; 255],
        }
    }
}


fn  check_cell(data: &mut Data) -> bool {
    for y in 0..4 {
        for x in 0..4 {
            if (rot_array[name_to_index(data.current)][data.rot][y][x] != 0) &&
                (((data.pos.x + x as i32) < 0) || ((data.pos.x + x as i32) >= 10) ||
                ((data.pos.y + 4 - y as i32) < 0) || ((data.pos.y + 4 - y as i32) >= 22) ||
                data.board[(data.pos.x + x as i32) as usize][(data.pos.y + 4 - y as i32) as usize] != 0) {
                
                    return false;
            }
        }
    }
    true
}

fn  handle_keyboard_input(data: &mut Data) {
    if data.scan_code == 1 {
        data.end = true;
        return;
    }
    if data.scan_code == 37 {
        data.rot = (data.rot + 1) % 4;
        if !check_cell(data) {
            data.rot = (data.rot + 3) % 4;
        }
    }
    if data.scan_code == 36 {
        data.rot = (data.rot + 3) % 4;
        if !check_cell(data) {
            data.rot = (data.rot + 1) % 4;
        }
    }
    if data.scan_code == 30 {
        data.pos.x = data.pos.x - 1;
        if !check_cell(data) {
            data.pos.x = data.pos.x + 1;
        }
    }
    if data.scan_code == 32 {
        data.pos.x = data.pos.x + 1;
        if !check_cell(data) {
            data.pos.x = data.pos.x - 1;
        }
    }
}

fn  update_tick(data: &mut Data) {
    data.tick = data.tick + 1;
    if data.tick > 10 {
        data.tick = 0;
        data.pos.y = data.pos.y - 1;
        if !check_cell(data) {
            data.pos.y = data.pos.y + 1;
        }
    }
}

fn  fill_fake_board(data: &mut Data) {
    data.board[0][0] = 1;
    data.board[1][0] = 1;
    data.board[2][0] = 1;
    data.board[3][0] = 1;

    data.board[4][0] = 7;
    data.board[5][0] = 7;
    data.board[6][0] = 7;
    data.board[5][1] = 7;

    data.board[8][19] = 7;
}

fn  exit_tetris() {
    WRITER.lock().toggle_cmd(true);
    println!("See you soon!");       
}

use crate::io::try_read_data;

fn read_input(data: &mut Data) {
    let new_scan_code = try_read_data();
    if new_scan_code != data.scan_code
    || data.input_cooldown[new_scan_code as usize] > 16000 {
        data.scan_code = new_scan_code;
        data.input_cooldown[new_scan_code as usize] = 0;
    }
    else {
        data.scan_code = 0;
        data.input_cooldown[new_scan_code as usize] += 1;
    }
}

pub fn ft_tetris() {
    let mut data: Data = Data::new();
    clear_window();
    draw_game_ui();
    //fill_fake_board(&mut data);
    loop {
        if data.end {
            exit_tetris();
            break;
        }
        read_input(&mut data);
        handle_keyboard_input(&mut data);
        update_tick(&mut data);
        display_game(data);
    }
}

const rot_array: [[[[u8; 4]; 4]; 4]; 7] = [
        [ // I
            [
                [0, 0, 0, 0],
                [1, 1, 1, 1],
                [0, 0, 0, 0],
                [0, 0, 0, 0]
            ],
            [
                [0, 0, 1, 0],
                [0, 0, 1, 0],
                [0, 0, 1, 0],
                [0, 0, 1, 0]
            ],
            [
                [0, 0, 0, 0],
                [0, 0, 0, 0],
                [1, 1, 1, 1],
                [0, 0, 0, 0]
            ],
            [
                [0, 1, 0, 0],
                [0, 1, 0, 0],
                [0, 1, 0, 0],
                [0, 1, 0, 0]
            ]
        ],
        [ // J
            [
                [2, 0, 0, 0],
                [2, 2, 2, 0],
                [0, 0, 0, 0],
                [0, 0, 0, 0]
            ],
            [
                [0, 2, 2, 0],
                [0, 2, 0, 0],
                [0, 2, 0, 0],
                [0, 0, 0, 0]
            ],
            [
                [0, 0, 0, 0],
                [2, 2, 2, 0],
                [0, 0, 2, 0],
                [0, 0, 0, 0]
            ],
            [
                [0, 2, 0, 0],
                [0, 2, 0, 0],
                [2, 2, 0, 0],
                [0, 0, 0, 0]
            ]
        ],
        [ // L
            [
                [0, 0, 3, 0],
                [3, 3, 3, 0],
                [0, 0, 0, 0],
                [0, 0, 0, 0]
            ],
            [
                [0, 3, 0, 0],
                [0, 3, 0, 0],
                [0, 3, 3, 0],
                [0, 0, 0, 0]
            ],
            [
                [0, 0, 0, 0],
                [3, 3, 3, 0],
                [3, 0, 0, 0],
                [0, 0, 0, 0]
            ],
            [
                [3, 3, 0, 0],
                [0, 3, 0, 0],
                [0, 3, 0, 0],
                [0, 0, 0, 0]
            ]
        ],
        [ // O
            [
                [0, 4, 4, 0],
                [0, 4, 4, 0],
                [0, 0, 0, 0],
                [0, 0, 0, 0]
            ],
            [
                [0, 4, 4, 0],
                [0, 4, 4, 0],
                [0, 0, 0, 0],
                [0, 0, 0, 0]
            ],
            [
                [0, 4, 4, 0],
                [0, 4, 4, 0],
                [0, 0, 0, 0],
                [0, 0, 0, 0]
            ],
            [
                [0, 4, 4, 0],
                [0, 4, 4, 0],
                [0, 0, 0, 0],
                [0, 0, 0, 0]
            ]
        ],
        [ // S
            [
                [0, 5, 5, 0],
                [5, 5, 0, 0],
                [0, 0, 0, 0],
                [0, 0, 0, 0]
            ],
            [
                [0, 5, 0, 0],
                [0, 5, 5, 0],
                [0, 0, 5, 0],
                [0, 0, 0, 0]
            ],
            [
                [0, 0, 0, 0],
                [0, 5, 5, 0],
                [5, 5, 0, 0],
                [0, 0, 0, 0]
            ],
            [
                [5, 0, 0, 0],
                [5, 5, 0, 0],
                [0, 5, 0, 0],
                [0, 0, 0, 0]
            ]
        ],
        [ // Z
            [
                [6, 6, 0, 0],
                [0, 6, 6, 0],
                [0, 0, 0, 0],
                [0, 0, 0, 0]
            ],
            [
                [0, 0, 6, 0],
                [0, 6, 6, 0],
                [0, 6, 0, 0],
                [0, 0, 0, 0]
            ],
            [
                [0, 0, 0, 0],
                [6, 6, 0, 0],
                [0, 6, 6, 0],
                [0, 0, 0, 0]
            ],
            [
                [0, 6, 0, 0],
                [6, 6, 0, 0],
                [6, 0, 0, 0],
                [0, 0, 0, 0]
            ]
        ],
        [ // T
            [
                [0, 7, 0, 0],
                [7, 7, 7, 0],
                [0, 0, 0, 0],
                [0, 0, 0, 0]
            ],
            [
                [0, 7, 0, 0],
                [0, 7, 7, 0],
                [0, 7, 0, 0],
                [0, 0, 0, 0]
            ],
            [
                [0, 0, 0, 0],
                [7, 7, 7, 0],
                [0, 7, 0, 0],
                [0, 0, 0, 0]
            ],
            [
                [0, 7, 0, 0],
                [7, 7, 0, 0],
                [0, 7, 0, 0],
                [0, 0, 0, 0]
            ]
        ]
    ];