use crate::*;
use alloc::string::String;
use alloc::vec::Vec;
use firefly_rust::Theme;
use firefly_rust::get_me;
use firefly_rust::get_settings;
use firefly_rust::{Buttons, Font, Pad, Peer, read_buttons, read_pad};

/// Current state of the luxboard-lite instance.
pub enum State {
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

/// Luxboard initialization options.
pub struct Options {
    pub layout: QwertyLayout,
    pub height: u32,
    pub theme: Option<Theme>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            layout: QwertyLayout::default(),
            height: 75,
            theme: None,
        }
    }
}

/// LuxboardLite virtual keyboard.
///
/// Please use [`new()`](fn@Self::new) or [`default()`](fn@Self::default) to create a new instance!
pub struct Keyboard {
    is_open_state: bool,
    board: QwertyLayout,
    /// Height of the keyboard. Defaults to `75`.
    pub height: u32,
    pub(crate) xsel: u32,
    pub(crate) ysel: u32,
    last_pad: Option<Pad>,
    last_buttons: Buttons,
    /// Current keyboard text.
    pub text: String,
    /// Colors to use.
    pub theme: Theme,
}

impl Keyboard {
    /// Creates a new LuxboardLite instance.
    pub fn new(options: Options) -> Keyboard {
        Keyboard {
            board: options.layout,
            height: options.height,
            is_open_state: false,
            xsel: 0,
            ysel: 0,
            last_pad: read_pad(Peer::COMBINED),
            last_buttons: read_buttons(Peer::COMBINED),
            text: String::default(),
            theme: options
                .theme
                .unwrap_or_else(|| get_settings(get_me()).theme),
        }
    }

    /// Updates keyboard input.
    pub fn update(&mut self) -> State {
        let mut ret_state = State::Open;

        if !self.is_open_state {
            return State::Closed;
        }

        let buttons = read_buttons(Peer::COMBINED);
        let pressed = buttons.just_pressed(&self.last_buttons);

        let pad = read_pad(Peer::COMBINED);

        if let Some(pad) = pad {
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
                self.ysel = 0;
            } else if (self.ysel as i32) + ychg < 0 {
                self.ysel = self.board.rows.len() as u32 - 1;
            } else {
                self.ysel += ychg as u32;
            }

            let row_len = self.board.rows.get(self.ysel as usize).unwrap().keys.len() as i32 - 1;

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
                self.xsel = 0;
            } else if (self.xsel as i32) + xchg < 0 {
                self.xsel = row_len as u32;
            } else {
                self.xsel += xchg as u32;
            }
        }

        if pressed.e {
            if let Some(key) = self.board.get(self.xsel as usize, self.ysel as usize) {
                match key.key_type {
                    KeyType::Char(c) => {
                        self.text.push(c);
                        ret_state = State::TextChanged(self.text.clone());
                    }
                    KeyType::Space => {
                        self.text.push(' ');
                        ret_state = State::TextChanged(self.text.clone());
                    }
                    KeyType::Backspace => {
                        self.text.pop();
                        ret_state = State::TextChanged(self.text.clone());
                    }
                    KeyType::Shift => self.board.shifted = !self.board.shifted,
                    KeyType::Ok => {
                        ret_state = State::JustClosed(self.text.clone());
                    }
                    KeyType::Cancel => {
                        ret_state = State::JustCancelled;
                    }
                }
            }
        } else if pressed.w {
            if self.text.is_empty() {
                // NOTE: remove?
                ret_state = State::JustCancelled
            } else {
                self.text.pop();
                ret_state = State::TextChanged(self.text.clone())
            }
        }

        self.last_pad = pad;
        self.last_buttons = buttons;

        match ret_state {
            State::JustCancelled | State::JustClosed(_) => self.is_open_state = false,
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
}

impl Default for Keyboard {
    fn default() -> Self {
        Keyboard::new(Options::default())
    }
}
