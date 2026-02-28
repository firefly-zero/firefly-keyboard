extern crate alloc;

use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::vec;

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
    fn draw(&self, height: i32, font: &Font) {
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

        for (idx, row) in self.rows.iter().rev().enumerate() {
            let mut this_row_cells = 0;

            for key in row.keys.iter() {
                this_row_cells += key.cells
            }

            let cell_width = WIDTH as u8 / this_row_cells;

            let mut current_x: u32 = 0;

            for key in row.keys.iter() {
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


                current_x += (cell_width * key.cells) as u32;

                let top_y = HEIGHT - (cell_height * idx as i32);

                draw_line(
                    Point { x: current_x as i32, y: top_y },
                    Point { x: current_x as i32, y: HEIGHT - (cell_height * ((idx as i32) + 1)) },
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
    height: u32
}

impl LuxboardLite {
    pub fn new(options: LuxboardLiteOptions) -> LuxboardLite {
        LuxboardLite {
            board: options.layout.as_board(),
            height: options.height,
            is_open_state: false    
        }
    }

    pub fn update(&mut self) -> LuxboardLiteState {
        let ret_state = LuxboardLiteState::Closed;

        if self.is_open_state {
            return ret_state;
        }

        ret_state
    }

    pub fn render(&mut self, font: &Font) {
        self.board.draw(self.height as i32, font);
    }

    pub fn open(&mut self) {
        self.is_open_state = true;
    }

    pub fn is_open(&mut self) -> bool {
        self.is_open_state
    }
}
