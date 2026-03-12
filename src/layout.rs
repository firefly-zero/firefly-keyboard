use crate::*;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use firefly_rust::*;

pub(crate) enum KeyType {
    Char(char),
    Space,
    Backspace,
    Shift,
    Cancel,
    Ok,
}

pub(crate) struct Key {
    pub cells: u8, // amount of cells it takes up on grid
    pub key_type: KeyType,
}

impl From<char> for Key {
    fn from(value: char) -> Self {
        Key {
            cells: 1,
            key_type: KeyType::Char(value),
        }
    }
}

pub(crate) struct KeyRow {
    pub keys: Vec<Key>,
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

pub struct QwertyLayout {
    pub(crate) rows: Vec<KeyRow>,
    pub(crate) shifted_rows: Vec<KeyRow>,
    pub(crate) shifted: bool,
}

impl Default for QwertyLayout {
    fn default() -> Self {
        Self {
            rows: vec![
                vec!['1', '2', '3', '4', '5', '6', '7', '8', '9', '0'].into(),
                vec!['q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p'].into(),
                vec!['a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', '|'].into(),
                vec!['z', 'x', 'c', 'v', 'b', 'n', 'm', '.', ',', '?'].into(),
                KeyRow {
                    keys: vec![
                        Key {
                            cells: 2,
                            key_type: KeyType::Shift,
                        },
                        Key {
                            cells: 6,
                            key_type: KeyType::Space,
                        },
                        Key {
                            cells: 2,
                            key_type: KeyType::Backspace,
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
                            key_type: KeyType::Cancel,
                        },
                        Key {
                            cells: 1,
                            key_type: KeyType::Ok,
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
                            key_type: KeyType::Shift,
                        },
                        Key {
                            cells: 6,
                            key_type: KeyType::Space,
                        },
                        Key {
                            cells: 2,
                            key_type: KeyType::Backspace,
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
                            key_type: KeyType::Cancel,
                        },
                        Key {
                            cells: 1,
                            key_type: KeyType::Ok,
                        },
                    ],
                },
            ],
            shifted: false,
        }
    }
}

impl QwertyLayout {
    pub fn new() -> Self {
        Self::default()
    }

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

    pub(crate) fn draw(&self, kbd: &Keyboard, font: &Font) {
        let rows = match self.shifted {
            true => &self.shifted_rows,
            false => &self.rows,
        };

        let cell_height = kbd.height as i32 / (rows.len() - 1) as i32;
        let board_height = HEIGHT - (cell_height * (rows.len()) as i32);
        let theme = kbd.theme;

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
                fill_color: theme.bg,
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
                color: theme.primary,
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
                    color: theme.primary,
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
                let text = match key.key_type {
                    KeyType::Char(c) => &c.to_string(),
                    KeyType::Backspace => "BKSPC",
                    KeyType::Shift => "SHIFT",
                    KeyType::Space => "SPACE",
                    KeyType::Ok => "OK",
                    KeyType::Cancel => "CANCEL",
                };

                let half_line = font.line_width_ascii(text) / 2;
                let text_draw_x =
                    (current_x + (((cell_width * key.cells) as u32 / 2) - half_line)) as i32 + 1;

                let last_x = current_x as i32;
                let last_y = HEIGHT - (cell_height * (row_idx + 1) as i32);

                current_x += (cell_width * key.cells) as u32;
                top_y = HEIGHT - (cell_height * row_idx as i32);

                if let KeyType::Shift = key.key_type
                    && self.shifted
                {
                    self.draw_highlight_in_key(
                        cell_width as u32,
                        cell_height as u32,
                        key.cells as u32,
                        last_x,
                        last_y,
                        theme.secondary,
                    );
                }

                if self.rows.len() - row_idx - 1 == kbd.ysel as usize
                    && col_idx == kbd.xsel as usize
                {
                    self.draw_highlight_in_key(
                        cell_width as u32,
                        cell_height as u32,
                        key.cells as u32,
                        last_x,
                        last_y,
                        theme.accent,
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
                        color: theme.primary,
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
                    theme.primary,
                );
            }
        }
    }

    pub(crate) fn get(&self, x: usize, y: usize) -> Option<&Key> {
        let rows = match self.shifted {
            true => &self.shifted_rows,
            false => &self.rows,
        };

        let row = rows.get(y)?;
        row.keys.get(x)
    }
}
