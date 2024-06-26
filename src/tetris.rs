const CMOS_ADDRESS: u16 = 0x70;
const CMOS_DATA: u16 = 0x71;

use crate::println;

////RAND
use core::cell::Cell;

pub struct SimpleRng {
    state: Cell<u32>,
}

impl SimpleRng {
    pub fn new(seed: u32) -> Self {
        SimpleRng {
            state: Cell::new(seed),
        }
    }

    pub fn next_u32(&self) -> u32 {
        let a: u32 = 1664525;
        let c: u32 = 1013904223;

        let state = self.state.get();
        let next_state = state.wrapping_mul(a).wrapping_add(c);
        self.state.set(next_state);

        next_state
    }
}

//// TIME

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

fn get_rtc_time() -> (u8, u8, u8) {
    let seconds = bcd_to_binary(read_cmos(0x00));
    let minutes = bcd_to_binary(read_cmos(0x02));
    let hours = bcd_to_binary(read_cmos(0x04));
    (hours, minutes, seconds)
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

    draw_str(12, 55, "Left         A", Color::White, Color::Black);
    draw_str(13, 55, "Right        D", Color::White, Color::Black);
    draw_str(14, 55, "Down         S", Color::White, Color::Black);
    draw_str(15, 55, "Drop     Space", Color::White, Color::Black);
    draw_str(16, 55, "Rot.L        J", Color::White, Color::Black);
    draw_str(17, 55, "Rot.R        K", Color::White, Color::Black);
    draw_str(18, 55, "Quit       Esc", Color::White, Color::Black);
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


fn draw_ghost_cell(x:usize, y: usize, color: Color) {
    let col = x * 2 + 30;
    let row = 22 - y;

    for i in 0..2 {
        draw_char(row, col + i, 0xb0, color, Color::Black);
    }
}

fn draw_empty_cell(x:usize, y: usize, color: Color) {
    let col = x * 2 + 30;
    let row = 22 - y;
    draw_char(row, col, ' ' as u8, Color::Black, Color::Black);
    draw_char(row, col + 1, '.' as u8, color, Color::Black);
}

fn draw_next(tetraminos: char) {
    for x in 14..18 {
        for y in 17..19 {
            draw_cell(x, y, Color::Black);
        }
    }
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
            draw_cell(15, 17, Color::Green);
            draw_cell(16, 17, Color::Green);
            draw_cell(16, 18,Color::Green);
            draw_cell(17, 18, Color::Green);
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
    draw_rectangle(2, 6, 9, 26);
    draw_rectangle(2, 7, 53, 71);
    draw_rectangle(9, 21, 53, 71);
    draw_text();
    draw_bg();
}

fn  game_over(data: &mut Data) {
    draw_rectangle(7, 17, 25, 54);
    for row in 8..17 {
        for col in 26..54 {
            draw_char(row, col, ' ' as u8, Color::Black, Color::Black);
        }
    }
    draw_str(9, 36, "GAME OVER", Color::Red, Color::Black);
    draw_str(11, 31, "Level............", Color::White, Color::Black);
    draw_str(12, 31, "Lines............", Color::White, Color::Black);
    draw_str(13, 31, "Score............", Color::White, Color::Black);
    draw_str(15, 31, "Press Esc to quit", Color::White, Color::Black);
    draw_nbr(11, 48, data.level, Color::White, Color::Black);
    draw_nbr(12, 48, data.total_line_cleared, Color::White, Color::Black);
    draw_nbr(13, 48, data.score, Color::White, Color::Black);
}

fn  draw_board(data: Data) {
    for x in 0..10 {
        for y in 0..20 {
            match data.board[x][y] {
                1 => draw_cell(x, y, Color::Cyan),
                2 => draw_cell(x, y, Color::Blue),
                3 => draw_cell(x, y, Color::Brown),
                4 => draw_cell(x, y, Color::Yellow),
                5 => draw_cell(x, y, Color::Green),
                6 => draw_cell(x, y, Color::Red),
                7 => draw_cell(x, y, Color::Magenta),
                _ => {
                    if data.current_board[x][y] != 0 {
                        if data.current_board[x][y] > 10 {
                            draw_ghost_cell(x, y, data.color);
                        }
                        else {
                            draw_cell(x, y, data.color);
                        }
                    }
                    else {
                        draw_empty_cell(x, y, Color::DarkGray);
                    }
                },
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
fn  place_current_tetrominos(data: &mut Data) {
    let save_pos_y = data.pos.y;

    while check_cell(data) {
        data.pos.y = data.pos.y - 1;
    }
    data.pos.y = data.pos.y + 1;

    let pos_y_down = data.pos.y;
    data.pos.y = save_pos_y;

    data.current_board = [[0; 22]; 10];
    for y in 0..4 {
        for x in 0..4 {
            if ROT_ARRAY[name_to_index(data.current)][data.rot][y][x] != 0 {
                data.current_board[(data.pos.x + x as i32)as usize][(pos_y_down + 4 - y as i32) as usize] = ROT_ARRAY[name_to_index(data.current)][data.rot][y][x] + 10;
                data.current_board[(data.pos.x + x as i32)as usize][(data.pos.y + 4 - y as i32) as usize] = ROT_ARRAY[name_to_index(data.current)][data.rot][y][x];
            }
        }
    }
}

fn  display_game(data: Data) {
    draw_board(data);
    draw_nbr(3, 24, data.score, Color::White, Color::Black);
    draw_nbr(5, 24, data.level, Color::White, Color::Black);
}


#[derive(Debug, Clone, Copy)]
struct Coord {
    x: i32,
    y: i32,
}

#[derive(Debug, Clone, Copy)]
struct Data {
    board: [[u8; 22]; 10],
    current_board: [[u8; 22]; 10],
    exit: bool,
    game_over: bool,
    level: u32,
    score: u32,
    total_line_cleared: u32,
    scan_code: u8,
    next: char,
    current: char,
    pos: Coord,
    rot: usize,
    color: Color,
    tick: usize,
    counting_ticks: usize,
    ticks_per_seconds: usize,
    time: (u8, u8, u8),
}

impl Data {
    pub fn new() -> Self {
        Data {
            board: [[0; 22]; 10],
            current_board: [[0; 22]; 10],
            exit: false,
            game_over: false,
            level: 1,
            score: 0,
            total_line_cleared: 0,
            scan_code: 0,
            next: 'I',
            current: 'O',
            pos: Coord { x: 3, y: 15 },
            rot: 0,
            color: Color::Yellow,
            tick: 0,
            counting_ticks: 0,
            ticks_per_seconds: 300,
            time: (0, 0, 0),
        }
    }
}


fn  check_cell(data: &mut Data) -> bool {
    for y in 0..4 {
        for x in 0..4 {
            if (ROT_ARRAY[name_to_index(data.current)][data.rot][y][x] != 0) &&
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
    match data.scan_code {
        1 => {
            data.exit = true;
        },
        37 => { //37-165
            data.rot = (data.rot + 1) % 4;
            if !check_cell(data) {
                data.rot = (data.rot + 3) % 4;
            }
        },
        36 => { //36-164
            data.rot = (data.rot + 3) % 4;
            if !check_cell(data) {
                data.rot = (data.rot + 1) % 4;
            }
        },
        30 => { //30-158
            data.pos.x = data.pos.x - 1;
            if !check_cell(data) {
                data.pos.x = data.pos.x + 1;
            }
        },
        31 => { //31-159
            data.pos.y = data.pos.y - 1;
            if !check_cell(data) {
                data.pos.y = data.pos.y + 1;
            }
        },
        32 => { //32-160
            data.pos.x = data.pos.x + 1;
            if !check_cell(data) {
                data.pos.x = data.pos.x - 1;
            }
        },
        57 | 17 => { //57-185
            while check_cell(data) {
                data.pos.y = data.pos.y - 1;
            }
            data.pos.y = data.pos.y + 1;
        },
        _ => {},
    }
    data.scan_code = 0;
}

fn  clear_line(data: &mut Data, y_stop: usize) {
    for y in y_stop..20 {
        for x in 0..10 {
            data.board[x][y] = data.board[x][y+1];
        }
    }
}

fn  clear_lines(data: &mut Data) {
    let mut n_line_cleared = 0;
    let mut y: i32 = 0;
    while y < 22 {
        for x in 0..10 {
            if data.board[x][y as usize] == 0 {
                break;
            }
            if x == 9 {
                clear_line(data, y as usize);
                n_line_cleared += 1;
                data.total_line_cleared += 1;
                y -= 1;
            }
        }
        y += 1;
    }
    data.level = data.total_line_cleared / 10 + 1;
    match n_line_cleared {
        1 => data.score += 100 * data.level,
        2 => data.score += 300 * data.level,
        3 => data.score += 500 * data.level,
        4 => data.score += 800 * data.level,
        _ => return,
    }
}

fn  finish_tetraminos(data: &mut Data, rng: &SimpleRng) {
    for x in 0..10 {
        for y in 0..20 {
            if data.current_board[x][y] != 0 {
                data.board[x][y] = data.current_board[x][y];
            }
        }
    }
    let tetraminos: [char; 7] = ['I', 'J', 'L', 'O', 'S', 'Z', 'T'];
    data.current = data.next;
    data.next = tetraminos[(rng.next_u32() % 7) as usize];
    data.pos = Coord { x: 3, y: 15 };
    data.rot = 0;
    draw_next(data.next);
    if !check_cell(data) {
        data.game_over = true;
        game_over(data);
        return;
    }
    clear_lines(data);
    match data.current {
        'I' => data.color = Color::Cyan,
        'J' => data.color = Color::Blue,
        'L' => data.color = Color::Brown,
        'O' => data.color = Color::Yellow,
        'S' => data.color = Color::Green, 
        'Z' => data.color = Color::Red, 
        'T' => data.color = Color::Magenta, 
        _ => {},
    }
}

fn  init_game(data: &mut Data, rng: &SimpleRng) {
    let tetraminos: [char; 7] = ['I', 'J', 'L', 'O', 'S', 'Z', 'T'];
    data.current = tetraminos[(rng.next_u32() % 7) as usize];
    data.next = tetraminos[(rng.next_u32() % 7) as usize];
    match data.current {
        'I' => data.color = Color::Cyan,
        'J' => data.color = Color::Blue,
        'L' => data.color = Color::Brown,
        'O' => data.color = Color::Yellow,
        'S' => data.color = Color::Green, 
        'Z' => data.color = Color::Red, 
        'T' => data.color = Color::Magenta, 
        _ => {},
    }
    draw_next(data.next);
}

fn  update_tick(data: &mut Data, rng: &SimpleRng) {
    if data.time == get_rtc_time() {
        data.counting_ticks += 1;
    }
    else {
        data.ticks_per_seconds = data.counting_ticks;
        data.counting_ticks = 0;
        data.time = get_rtc_time();
    }
    data.tick += 1;
    let speed: usize;
    if data.level >= 15 {
        speed = data.ticks_per_seconds - (29 * data.ticks_per_seconds as u32 / 30) as usize;
    }
    else {
        speed = data.ticks_per_seconds - ((data.level + 14) * data.ticks_per_seconds as u32 / 30) as usize;
    }
    if data.tick > speed {
        data.tick = 0;
        data.pos.y = data.pos.y - 1;
        if !check_cell(data) {
            data.pos.y = data.pos.y + 1;
            place_current_tetrominos(data);
            finish_tetraminos(data, rng);
        }
    }
}

fn  exit_tetris() {
    WRITER.lock().toggle_cmd(true);
    println!("See you soon!");       
}

use crate::io::try_read_data;

fn read_input(data: &mut Data) {
    data.scan_code = try_read_data();
}

fn  wait_start_of_second(data: &mut Data) {
    let (a, b, c) = get_rtc_time();
    while (a, b, c) == get_rtc_time() {}
    data.time = get_rtc_time();
}

pub fn ft_tetris() {
    let mut data: Data = Data::new();
    clear_window();
    draw_game_ui();
    let (_ , minutes, seconds) = get_rtc_time();
    let rng = SimpleRng::new(minutes as u32* 100 + seconds as u32);
    init_game(&mut data, &rng);
    wait_start_of_second(&mut data);
    loop {
        if data.exit {
            exit_tetris();
            break;
        }
        read_input(&mut data);
        if data.game_over {
            if data.scan_code == 1 {
                data.exit = true;
            }
            continue;
        }
        handle_keyboard_input(&mut data);
        update_tick(&mut data, &rng);
        if !data.game_over {
            place_current_tetrominos(&mut data);
            display_game(data);
        }
    }
}

const ROT_ARRAY: [[[[u8; 4]; 4]; 4]; 7] = [
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