extern crate alloc;

use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::vec;

use firefly_rust::math::abs;
use firefly_rust::math::sqrt;
use firefly_rust::*;


fn pow(n: f32, pow: i32) -> f32 {
    let mut tmp = n;

    for i in 0..pow {
        tmp *= n;
    }

    tmp
}

enum KeyType {
    Char(char),
    Space,
    Backspace,
    Shift,
    Cancel,
    Ok
}

struct Key {
    cells: u8, // amount of cells it takes up on grid
    r#type: KeyType
}

impl From<char> for Key {
    fn from(value: char) -> Self {
        Key { cells: 1, r#type: KeyType::Char(value) }
    }
}

struct KeyRow {
    keys: Vec<Key>
}

impl From<Vec<char>> for KeyRow {
    fn from(value: Vec<char>) -> Self {
        KeyRow { keys: value.iter().map(|c| <char as Into<Key>>::into(*c)).collect() }
    }
}

impl KeyRow {
}

struct Board {
    rows: Vec<KeyRow>
}

impl Board {
    fn draw(&self, height: i32, font: &Font, xsel: Option<u32>, ysel: Option<u32>) {
        let cell_height = height / (self.rows.len() - 1) as i32;
        let board_height = HEIGHT - (cell_height * (self.rows.len() - 1) as i32);

        draw_line(
            Point { x: 0, y: board_height },
            Point { x: WIDTH, y: board_height },
            LineStyle {
                color: Color::Black,
                width: 1
            }
        );

        for i in 0..self.rows.len() + 1 {
            let i = i as i32;

            let cell_y = HEIGHT - (i * cell_height as i32);

            draw_line(
                Point { x: 0, y: cell_y },
                Point { x: WIDTH, y: cell_y },
                LineStyle {
                    color: Color::Black,
                    width: 1
                }
            );
        }

        for (row_idx, row) in self.rows.iter().rev().enumerate() { // draw from bottom up
            let mut this_row_cells = 0;

            for key in row.keys.iter() {
                this_row_cells += key.cells
            }

            let cell_width = WIDTH as u8 / this_row_cells;

            let mut current_x: u32 = 0;
            let mut top_y: i32 = 0;

            for (col_idx, key) in row.keys.iter().enumerate() {
                let text = match key.r#type {
                    KeyType::Char(c) => {
                        &c.to_string()
                    },
                    KeyType::Backspace => "BKSPC",
                    KeyType::Shift => "SHIFT",
                    KeyType::Space => "SPACE",
                    KeyType::Ok => "OK",
                    KeyType::Cancel => "CANCEL"
                };

                let text_draw_x = (
                    current_x + (((cell_width * key.cells) as u32 / 2) - (font.line_width(text) / 2))
                ) as i32 + 1;

                let mut last_x = current_x as i32;
                let mut last_y = HEIGHT - (cell_height * (row_idx + 1) as i32);

                current_x += (cell_width * key.cells) as u32;
                top_y = HEIGHT - (cell_height * row_idx as i32);

                if let Some(c) = xsel && let Some(r) = ysel {
                    if self.rows.len() - row_idx - 1 == r as usize && col_idx == c as usize {
                        let mut x_modifier = 0;
                        let mut y_modifier = 0;

                        if last_x != 0 {
                            x_modifier += 1
                        }

                        if last_y != 0 {
                            y_modifier += 1
                        }
                        
                        draw_rect(
                            Point { x: last_x + x_modifier + 1, y: last_y + y_modifier + 1 },
                            Size { width: (cell_width * key.cells) as i32 - 2 - x_modifier, height: cell_height - 2 - y_modifier },
                            Style::solid(Color::Yellow)
                        );
                    }
                }

                draw_line(
                    Point { x: current_x as i32, y: top_y },
                    Point { x: current_x as i32, y: HEIGHT - (cell_height * ((row_idx as i32) + 1)) },
                    LineStyle {
                        color: Color::Black,
                        width: 1
                    }
                );

                draw_text(
                    text,
                    font,
                    Point {
                        x: text_draw_x,
                        y: top_y - (font.char_height() / 2) as i32 + 2
                    },
                    Color::Black
                );
            }
        }
    }
}


pub enum LuxboardLiteState {
    Open,
    Closed,
    TextChanged(String),
    JustClosed(String)
}


pub enum LuxboardLiteLayout {
    Qwertyish
}

impl LuxboardLiteLayout {
    fn as_board(&self) -> Board {
        Board {
            rows: vec![
                vec!['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'].into(),
                vec!['q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p'].into(),
                vec!['a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', '|'].into(),
                vec!['z', 'x', 'c', 'v', 'b', 'n', 'm', '.', ',', '?'].into(),
                KeyRow {
                    keys: vec![
                        Key {cells: 2, r#type: KeyType::Shift},
                        Key {cells: 6, r#type: KeyType::Space},
                        Key {cells: 2, r#type: KeyType::Backspace}
                    ]
                },
                KeyRow {
                    keys: vec![
                        '['.into(), ']'.into(), '-'.into(), ':'.into(), '\''.into(), '`'.into(),
                        Key {cells: 2, r#type: KeyType::Cancel},
                        Key {cells: 2, r#type: KeyType::Ok}
                    ]
                }
            ],
        }
    }
}


pub struct LuxboardLiteOptions {
    pub layout: LuxboardLiteLayout,
    pub height: u32
}

pub struct LuxboardLite {
    is_open_state: bool,
    board: Board,
    height: u32,
    xsel: Option<u32>,
    ysel: Option<u32>
}

impl LuxboardLite {
    pub fn new(options: LuxboardLiteOptions) -> LuxboardLite {
        LuxboardLite {
            board: options.layout.as_board(),
            height: options.height,
            is_open_state: false,
            xsel: None,
            ysel: None
        }
    }

    pub fn update(&mut self) -> LuxboardLiteState {
        let pad = read_pad(Peer::COMBINED);
        if let Some(pad) = pad {
            let u = pad.azimuth().cos() / pad.radius();
            let v = pad.azimuth().sin() / pad.radius();

            let u2: f32 = u * u;
            let v2: f32 = v * v;
            let twosqrt2: f32 = 2.0 * sqrt(2.0);
            let subtermx: f32 = 2.0 + u2 - v2;
            let subtermy: f32 = 2.0 - u2 + v2;

            let termx1 = abs(subtermx + u * twosqrt2);
            let termx2 = abs(subtermx - u * twosqrt2);
            let termy1 = abs(subtermy + v * twosqrt2);
            let termy2 = abs(subtermy - v * twosqrt2);

            let x = 0.5 * sqrt(termx1) - 0.5 * sqrt(termx2);
            let y = 0.5 * sqrt(termy1) - 0.5 * sqrt(termy2);

            // incredibly sorry for this magic number, but it does indeed work-
            let x = x * 100.0 * 8.892360466414977;
            let y = y * 100.0 * 8.892360466414977;

            let x = (x + 1.0) / 2.0;
            let y = (y - 1.0).abs() / 2.0;

            let y = (y * self.board.rows.len() as f32 - 1.0) as u32;
            let x = (x * self.board.rows.get(y as usize).unwrap().keys.len() as f32 - 1.0) as u32;

            // self.xsel = Some(x);
            // self.ysel = Some(y);
        } else {
            self.xsel = None;
            self.ysel = None;
        }

        let ret_state = LuxboardLiteState::Closed;

        if self.is_open_state {
            return ret_state;
        }

        ret_state
    }

    pub fn render(&mut self, font: &Font) {
        self.board.draw(self.height as i32, font, self.xsel, self.ysel);
    }

    pub fn open(&mut self) {
        self.is_open_state = true;
    }

    pub fn is_open(&mut self) -> bool {
        self.is_open_state
    }
}
