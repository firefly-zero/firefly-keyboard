extern crate alloc;

use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use firefly_rust::math::abs;
use firefly_rust::math::sqrt;
use firefly_rust::*;

enum KeyType {
    Char(char),
    Space,
    Backspace,
    Shift,
    Cancel,
    Ok,
}

struct Key {
    cells: u8, // amount of cells it takes up on grid
    r#type: KeyType,
}

impl From<char> for Key {
    fn from(value: char) -> Self {
        Key {
            cells: 1,
            r#type: KeyType::Char(value),
        }
    }
}

struct KeyRow {
    keys: Vec<Key>,
}

impl From<Vec<char>> for KeyRow {
    fn from(value: Vec<char>) -> Self {
        KeyRow {
            keys: value
                .iter()
                .map(|c| <char as Into<Key>>::into(*c))
                .collect(),
        }
    }
}

impl KeyRow {}

struct Board {
    rows: Vec<KeyRow>,
    shifted_rows: Vec<KeyRow>,
    shifted: bool,
}

impl Board {
    fn draw_highlight_in_key(
        &self,
        cell_width: u32,
        cell_height: u32,
        key_cells: u32,
        last_x: i32,
        last_y: i32,
        color: Color,
    ) {
        let mut x_modifier = 0;
        let mut y_modifier = 0;

        if last_x != 0 {
            x_modifier += 1
        }

        if last_y != 0 {
            y_modifier += 1
        }

        draw_rect(
            Point {
                x: last_x + x_modifier + 1,
                y: last_y + y_modifier + 1,
            },
            Size {
                width: (cell_width * key_cells) as i32 - 2 - x_modifier,
                height: cell_height as i32 - 2 - y_modifier,
            },
            Style::solid(color),
        );
    }

    fn draw(&self, luxboard: &LuxboardLite, font: &Font) {
        let rows = match self.shifted {
            true => &self.shifted_rows,
            false => &self.rows,
        };

        let cell_height = luxboard.height as i32 / (rows.len() - 1) as i32;
        let board_height = HEIGHT - (cell_height * (rows.len()) as i32);

        draw_rect(
            Point {
                x: 0,
                y: board_height,
            },
            Size {
                width: WIDTH,
                height: HEIGHT - board_height,
            },
            Style {
                fill_color: luxboard.bg_color,
                stroke_color: Color::None,
                stroke_width: 1,
            },
        );

        draw_line(
            Point {
                x: 0,
                y: board_height,
            },
            Point {
                x: WIDTH,
                y: board_height,
            },
            LineStyle {
                color: luxboard.line_color,
                width: 1,
            },
        );

        for i in 0..rows.len() + 1 {
            let i = i as i32;

            let cell_y = HEIGHT - (i * cell_height);

            draw_line(
                Point { x: 0, y: cell_y },
                Point {
                    x: WIDTH,
                    y: cell_y,
                },
                LineStyle {
                    color: luxboard.line_color,
                    width: 1,
                },
            );
        }

        for (row_idx, row) in rows.iter().rev().enumerate() {
            // draw from bottom up
            let mut this_row_cells = 0;

            for key in row.keys.iter() {
                this_row_cells += key.cells
            }

            let cell_width = WIDTH as u8 / this_row_cells;

            let mut current_x: u32 = 0;
            let mut top_y;

            for (col_idx, key) in row.keys.iter().enumerate() {
                let text = match key.r#type {
                    KeyType::Char(c) => &c.to_string(),
                    KeyType::Backspace => "BKSPC",
                    KeyType::Shift => "SHIFT",
                    KeyType::Space => "SPACE",
                    KeyType::Ok => "OK",
                    KeyType::Cancel => "CANCEL",
                };

                let text_draw_x = (current_x
                    + (((cell_width * key.cells) as u32 / 2) - (font.line_width(text) / 2)))
                    as i32
                    + 1;

                let last_x = current_x as i32;
                let last_y = HEIGHT - (cell_height * (row_idx + 1) as i32);

                current_x += (cell_width * key.cells) as u32;
                top_y = HEIGHT - (cell_height * row_idx as i32);

                if let KeyType::Shift = key.r#type
                    && self.shifted
                {
                    self.draw_highlight_in_key(
                        cell_width as u32,
                        cell_height as u32,
                        key.cells as u32,
                        last_x,
                        last_y,
                        luxboard.locked_key_color,
                    );
                }

                if self.rows.len() - row_idx - 1 == luxboard.ysel as usize
                    && col_idx == luxboard.xsel as usize
                {
                    self.draw_highlight_in_key(
                        cell_width as u32,
                        cell_height as u32,
                        key.cells as u32,
                        last_x,
                        last_y,
                        luxboard.highlight_color,
                    );
                }

                draw_line(
                    Point {
                        x: current_x as i32,
                        y: top_y,
                    },
                    Point {
                        x: current_x as i32,
                        y: HEIGHT - (cell_height * ((row_idx as i32) + 1)),
                    },
                    LineStyle {
                        color: luxboard.line_color,
                        width: 1,
                    },
                );

                draw_text(
                    text,
                    font,
                    Point {
                        x: text_draw_x,
                        y: top_y - (font.char_height() / 2) as i32 + 1,
                    },
                    luxboard.text_color,
                );
            }
        }
    }

    fn get(&self, x: usize, y: usize) -> Option<&Key> {
        let rows = match self.shifted {
            true => &self.shifted_rows,
            false => &self.rows,
        };

        let row = rows.get(y)?;
        row.keys.get(x)
    }
}

/// Method of providing input from the touchpad to luxboard-lite.
#[derive(Default)]
pub enum LuxboardLiteInputMethod {
    #[default]
    /// Treat the touchpad as an 8-way d-pad.
    Dpad,
    /// Map a square area of the touchpad to each key on the keyboard.
    SquareMap,
}

/// Current state of the luxboard-lite instance.
pub enum LuxboardLiteState {
    /// Text was changed, returns the new text.
    TextChanged(String),
    /// Default state when the keyboard is open.
    Open,
    /// default state when the keyboard is closed.
    Closed,
    /// Luxboard was closed this update cycle. Returns the keyboard's text.
    JustClosed(String),
    /// Input was just cancelled by the user.
    JustCancelled,
}

/// Layout of the keyboard.
#[derive(Default)]
pub enum LuxboardLiteLayout {
    #[default]
    /// A QWERTY-similar layout.
    Qwertyish,
}

impl LuxboardLiteLayout {
    fn as_board(&self) -> Board {
        Board {
            rows: vec![
                vec!['1', '2', '3', '4', '5', '6', '7', '8', '9', '0'].into(),
                vec!['q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p'].into(),
                vec!['a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', '|'].into(),
                vec!['z', 'x', 'c', 'v', 'b', 'n', 'm', '.', ',', '?'].into(),
                KeyRow {
                    keys: vec![
                        Key {
                            cells: 2,
                            r#type: KeyType::Shift,
                        },
                        Key {
                            cells: 6,
                            r#type: KeyType::Space,
                        },
                        Key {
                            cells: 2,
                            r#type: KeyType::Backspace,
                        },
                    ],
                },
                KeyRow {
                    keys: vec![
                        '['.into(),
                        ']'.into(),
                        '-'.into(),
                        ':'.into(),
                        '\''.into(),
                        '`'.into(),
                        '='.into(),
                        Key {
                            cells: 2,
                            r#type: KeyType::Cancel,
                        },
                        Key {
                            cells: 1,
                            r#type: KeyType::Ok,
                        },
                    ],
                },
            ],
            shifted_rows: vec![
                vec!['!', '@', '#', '$', '%', '^', '&', '*', '(', ')'].into(),
                vec!['Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P'].into(),
                vec!['A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L', '\\'].into(),
                vec!['Z', 'X', 'C', 'V', 'B', 'N', 'M', '<', '>', '/'].into(),
                KeyRow {
                    keys: vec![
                        Key {
                            cells: 2,
                            r#type: KeyType::Shift,
                        },
                        Key {
                            cells: 6,
                            r#type: KeyType::Space,
                        },
                        Key {
                            cells: 2,
                            r#type: KeyType::Backspace,
                        },
                    ],
                },
                KeyRow {
                    keys: vec![
                        '{'.into(),
                        '}'.into(),
                        '_'.into(),
                        ';'.into(),
                        '"'.into(),
                        '~'.into(),
                        '+'.into(),
                        Key {
                            cells: 2,
                            r#type: KeyType::Cancel,
                        },
                        Key {
                            cells: 1,
                            r#type: KeyType::Ok,
                        },
                    ],
                },
            ],
            shifted: false,
        }
    }
}

/// Luxboard initialization options.
/// See [LuxboardLite] for variable definitions.
#[derive(Default)]
pub struct LuxboardLiteOptions {
    pub layout: LuxboardLiteLayout,
    pub height: Option<u32>,
    pub line_color: Option<Color>,
    pub text_color: Option<Color>,
    pub highlight_color: Option<Color>,
    pub locked_key_color: Option<Color>,
    pub bg_color: Option<Color>,
    pub input_method: LuxboardLiteInputMethod,
    pub wrap_around: Option<bool>
}

/// LuxboardLite virtual keyboard.
///
/// Please use [`new()`](fn@Self::new) or [`default()`](fn@Self::default) to create a new instance!
pub struct LuxboardLite {
    is_open_state: bool,
    board: Board,
    /// Height of the keyboard. Defaults to `75`.
    pub height: u32,
    xsel: u32,
    ysel: u32,
    last_pad: Option<Pad>,
    last_buttons: Buttons,
    /// Current keyboard text.
    pub text: String,
    /// Color of the lines separating keyboard keys. Defaults to [`Color::Black`].
    pub line_color: Color,
    /// Text color. Defaults to [`Color::Black`].
    pub text_color: Color,
    /// Highlighted key background color. Defaults to [`Color::Yellow`].
    pub highlight_color: Color,
    /// Locked key background color (e.g. SHIFT is locked). Defaults to [`Color::Cyan`].
    pub locked_key_color: Color,
    /// Keyboard background color. Defaults to [`Color::White`].
    pub bg_color: Color,
    /// Method of recieving input.
    pub input_method: LuxboardLiteInputMethod,
    /// Input into a keyboard edge will wrap around to the other side. Defaults to `true`.
    pub wrap_around: bool
}

impl LuxboardLite {
    /// Creates a new LuxboardLite instance.
    pub fn new(options: LuxboardLiteOptions) -> LuxboardLite {
        LuxboardLite {
            board: options.layout.as_board(),
            height: options.height.unwrap_or(75),
            is_open_state: false,
            xsel: 0,
            ysel: 0,
            last_pad: read_pad(Peer::COMBINED),
            last_buttons: read_buttons(Peer::COMBINED),
            text: String::default(),
            line_color: options.line_color.unwrap_or(Color::Black),
            text_color: options.text_color.unwrap_or(Color::Black),
            highlight_color: options.highlight_color.unwrap_or(Color::Yellow),
            locked_key_color: options.locked_key_color.unwrap_or(Color::Cyan),
            bg_color: options.bg_color.unwrap_or(Color::White),
            input_method: options.input_method,
            wrap_around: options.wrap_around.unwrap_or(true)
        }
    }

    /// Updates keyboard input.
    pub fn update(&mut self) -> LuxboardLiteState {
        let mut ret_state = LuxboardLiteState::Open;

        if !self.is_open_state {
            return LuxboardLiteState::Closed;
        }

        let buttons = read_buttons(Peer::COMBINED);
        let pressed = buttons.just_pressed(&self.last_buttons);

        let pad = read_pad(Peer::COMBINED);

        if let Some(pad) = pad {
            match self.input_method {
                LuxboardLiteInputMethod::Dpad => {
                    let dpad = pad.as_dpad8();
                    let pressed = dpad.just_pressed(&self.last_pad.unwrap_or_default().as_dpad8());

                    let last_row_len =
                        self.board.rows.get(self.ysel as usize).unwrap().keys.len() as i32 - 1;

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
                        if self.wrap_around {
                            self.ysel = 0;
                        } else {
                            self.ysel = self.board.rows.len() as u32 - 1;
                        }
                    } else if (self.ysel as i32) + ychg < 0 {
                        if self.wrap_around {
                            self.ysel = self.board.rows.len() as u32 - 1;
                        } else {
                            self.ysel = 0
                        }
                    } else {
                        self.ysel += ychg as u32;
                    }

                    let row_len =
                        self.board.rows.get(self.ysel as usize).unwrap().keys.len() as i32 - 1;

                    if last_row_len > row_len {
                        let mut cells = Vec::with_capacity(row_len as usize);

                        for (idx, keys) in self
                            .board
                            .rows
                            .get(self.ysel as usize)
                            .unwrap()
                            .keys
                            .iter()
                            .enumerate()
                        {
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
                        if self.wrap_around {
                            self.xsel = 0;
                        } else {
                            self.xsel = row_len as u32;
                        }
                    } else if (self.xsel as i32) + xchg < 0 {
                        if self.wrap_around {
                            self.xsel = row_len as u32;
                        } else {
                            self.xsel = 0
                        }
                    } else {
                        self.xsel += xchg as u32;
                    }
                },
                LuxboardLiteInputMethod::SquareMap => {
                    let sqrt22 = sqrt(2.0) / 2.0;

                    let x = pad.x as f32 / 1000.0; // to unit circle
                    let y = pad.y as f32 / 1000.0;

                    let x = x.min(sqrt22).max(-sqrt22); // limit to square
                    let y = y.min(sqrt22).max(-sqrt22);

                    let x = x * sqrt22; // center
                    let y = y * sqrt22;

                    let x = x * 2.0;
                    let y = y * 2.0;

                    let x = x + 0.5;

                    let y = y * (self.board.rows.len() as f32 - 1.0);

                    let y = abs((y - self.board.rows.len() as f32) / 2.0);

                    let row_len = (self.board.rows.get(y as usize).unwrap().keys.len() - 1) as u32;

                    let x = x * (row_len as f32);

                    let mut x = x as u32;
                    let mut y = y as u32;

                    if x > row_len as u32 {
                        x = row_len as u32;
                    }

                    if y > self.board.rows.len() as u32 - 1 {
                        y = self.board.rows.len() as u32 - 1;
                    }

                    self.xsel = x;
                    self.ysel = y;
                }
            }
        }

        if pressed.e {
            if let Some(key) = self.board.get(self.xsel as usize, self.ysel as usize) {
                match key.r#type {
                    KeyType::Char(c) => {
                        self.text.push(c);
                        ret_state = LuxboardLiteState::TextChanged(self.text.clone());
                    }
                    KeyType::Space => {
                        self.text.push(' ');
                        ret_state = LuxboardLiteState::TextChanged(self.text.clone());
                    }
                    KeyType::Backspace => {
                        self.text.pop();
                        ret_state = LuxboardLiteState::TextChanged(self.text.clone());
                    }
                    KeyType::Shift => self.board.shifted = !self.board.shifted,
                    KeyType::Ok => {
                        ret_state = LuxboardLiteState::JustClosed(self.text.clone());
                    }
                    KeyType::Cancel => {
                        ret_state = LuxboardLiteState::JustCancelled;
                    }
                    _ => {}
                }
            }
        } else if pressed.w {
            if self.text.is_empty() {
                // NOTE: remove?
                ret_state = LuxboardLiteState::JustCancelled
            } else {
                self.text.pop();
                ret_state = LuxboardLiteState::TextChanged(self.text.clone())
            }
        }

        self.last_pad = pad;
        self.last_buttons = buttons;

        match ret_state {
            LuxboardLiteState::JustCancelled | LuxboardLiteState::JustClosed(_) => {
                self.is_open_state = false
            }
            _ => {}
        }

        ret_state
    }

    /// Renders the keyboard.
    pub fn render(&self, font: &Font) {
        if self.is_open_state {
            self.board.draw(self, font);
        }
    }

    /// Opens the keyboard.
    pub fn open(&mut self) {
        self.is_open_state = true;
        self.last_buttons = read_buttons(Peer::COMBINED);
        self.xsel = 0;
        self.ysel = 0;
    }

    /// Returns if the keyboard is currently open.
    pub fn is_open(&mut self) -> bool {
        self.is_open_state
    }

    /// Clears the keyboard's text.
    pub fn clear(&mut self) {
        self.text.clear();
    }

    /// Sets the keyboard's text.
    pub fn set_text(&mut self, text: &str) {
        self.text.clear();
        self.text.push_str(text);
    }

    /// Sets the keyboard layout.
    pub fn set_layout(&mut self, layout: LuxboardLiteLayout) {
        self.board = layout.as_board();
    }
}

impl Default for LuxboardLite {
    fn default() -> Self {
        LuxboardLite::new(LuxboardLiteOptions::default())
    }
}
