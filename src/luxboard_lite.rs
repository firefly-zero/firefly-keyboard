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

                let last_x = current_x as i32;
                let last_y = HEIGHT - (cell_height * (row_idx + 1) as i32);

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
                        y: top_y - (font.char_height() / 2) as i32 + 1
                    },
                    Color::Black
                );
            }
        }
    }

    fn get(&self, x: usize, y: usize) -> Option<&Key> {
        let row = self.rows.get(y)?;
        row.keys.get(x)
    }
}


pub enum LuxboardLiteState {
    Open,
    Closed,
    TextChanged(String),
    JustClosed(String),
    JustCancelled
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
    xsel: u32,
    ysel: u32,
    last_pad: Option<Pad>,
    last_buttons: Buttons,
    text: String
}

impl LuxboardLite {
    pub fn new(options: LuxboardLiteOptions) -> LuxboardLite {
        LuxboardLite {
            board: options.layout.as_board(),
            height: options.height,
            is_open_state: false,
            xsel: 0,
            ysel: 0,
            last_pad: read_pad(Peer::COMBINED),
            last_buttons: read_buttons(Peer::COMBINED),
            text: String::default()
        }
    }

    pub fn update(&mut self) -> LuxboardLiteState {
        let mut ret_state = LuxboardLiteState::Open;

        if !self.is_open_state {
            return LuxboardLiteState::Closed;
        }

        let buttons = read_buttons(Peer::COMBINED);
        let pressed = buttons.just_pressed(&self.last_buttons);

        let pad = read_pad(Peer::COMBINED);

        if let Some(pad) = pad {
            let dpad = pad.as_dpad8();
            let pressed = dpad.just_pressed(&self.last_pad.unwrap_or(Pad::default()).as_dpad8());

            let last_row_len = self.board.rows.get(self.ysel as usize).unwrap().keys.len() as i32 - 1;

            let mut xchg: i32 = 0;
            let mut ychg: i32 = 0;

            if pressed.up {
                ychg -= 1;
            } else if pressed.down {
                ychg += 1;
            }

            if pressed.right {
                xchg += 1;
            } else if pressed.left {
                xchg -= 1;
            }

            if (self.ysel as i32) + ychg > self.board.rows.len() as i32 - 1 {
                self.ysel = self.board.rows.len() as u32 - 1;
            } else if (self.ysel as i32) + ychg < 0 {
                self.ysel = 0
            } else {
                self.ysel += ychg as u32;
            }

            let row_len = self.board.rows.get(self.ysel as usize).unwrap().keys.len() as i32 - 1;

            if last_row_len > row_len {
                let mut cells = Vec::with_capacity(row_len as usize);

                for (idx, keys) in self.board.rows.get(self.ysel as usize).unwrap().keys.iter().enumerate() {
                    for _ in 0..keys.cells {
                        cells.push(idx as u32);
                    }
                }

                self.xsel = *cells.get(self.xsel as usize).unwrap();
            } else if last_row_len < row_len {
                // TODO: better logic

                self.xsel += 1;
            }

            if (self.xsel as i32) + xchg > row_len {
                self.xsel = row_len as u32;
            } else if (self.xsel as i32) + xchg < 0 {
                self.xsel = 0
            } else {
                self.xsel += xchg as u32;
            }
        }

        if pressed.e {
            if let Some(key) = self.board.get(self.xsel as usize, self.ysel as usize) {
                match key.r#type {
                    KeyType::Char(c) => {
                        self.text.push(c);
                        ret_state = LuxboardLiteState::TextChanged(self.text.clone());
                    },
                    KeyType::Ok => {
                        ret_state = LuxboardLiteState::JustClosed(self.text.clone());
                    },
                    KeyType::Cancel => {
                        ret_state = LuxboardLiteState::JustCancelled;
                    },
                    _ => {}
                }
            }
        }

        self.last_pad = pad;
        self.last_buttons = buttons;

        if self.is_open_state {
            return ret_state;
        }

        ret_state
    }

    pub fn render(&mut self, font: &Font) {
        self.board.draw(self.height as i32, font, Some(self.xsel), Some(self.ysel));
    }

    pub fn open(&mut self) {
        self.is_open_state = true;
    }

    pub fn is_open(&mut self) -> bool {
        self.is_open_state
    }
}
