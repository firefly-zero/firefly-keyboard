use crate::*;
use alloc::string::String;
use alloc::vec::Vec;
use firefly_rust::DPad8;
use firefly_rust::Theme;
use firefly_rust::get_me;
use firefly_rust::get_settings;
use firefly_rust::{Buttons, Font, Peer, read_buttons, read_pad};

/// The current state of the keyboard.
#[derive(Clone, Copy)]
pub enum State {
    /// Text was changed, returns the new text.
    TextChanged,
    /// Default state when the keyboard is open.
    Open,
    /// Default state when the keyboard is closed.
    Closed,
    /// Keyboard was closed this update cycle. Returns the keyboard's text.
    JustClosed,
    /// Input was just cancelled by the user.
    JustCancelled,
}

/// [`Keyboard`] initialization options.
pub struct Options {
    /// The keyboard layout. Currently only QWERTY is supported.
    pub layout: QwertyLayout,
    /// The keyboard height. Default: 75px.
    pub height: u32,
    /// The keyboard color scheme. Defaults to the theme set in system settings.
    pub theme: Theme,
    /// The peer to read input from. Defaults to the combined input.
    pub peer: Peer,
    /// If true, the keyboard will be open by default.
    pub open: bool,
    /// The default text. Empty by default.
    pub text: String,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            layout: QwertyLayout::default(),
            height: 75,
            theme: get_settings(get_me()).theme,
            peer: Peer::COMBINED,
            open: true,
            text: String::default(),
        }
    }
}

/// The virtual keyboard.
pub struct Keyboard {
    pub height: u32,
    pub text: String,
    pub theme: Theme,
    pub peer: Peer,

    pub(crate) xsel: u32,
    pub(crate) ysel: u32,

    is_open: bool,
    held_for: u32,
    board: QwertyLayout,
    last_pad: DPad8,
    last_buttons: Buttons,
}

impl Keyboard {
    #[must_use]
    pub fn new(options: Options) -> Keyboard {
        Keyboard {
            board: options.layout,
            height: options.height,
            is_open: options.open,
            xsel: 0,
            ysel: 0,
            held_for: 0,
            last_pad: DPad8::default(),
            last_buttons: Buttons::default(),
            text: options.text,
            theme: options.theme,
            peer: options.peer,
        }
    }

    /// Updates keyboard input.
    pub fn update(&mut self) -> State {
        if !self.is_open {
            return State::Closed;
        }

        let pad = read_pad(self.peer).unwrap_or_default();
        let dpad = pad.as_dpad8();
        self.handle_pad(dpad);

        let buttons = read_buttons(self.peer);
        let state = self.handle_buttons(buttons);
        if matches!(state, State::JustCancelled | State::JustClosed) {
            self.is_open = false;
        }

        self.last_pad = dpad;
        self.last_buttons = buttons;
        state
    }

    fn handle_pad(&mut self, dpad: DPad8) {
        self.held_for = if dpad.any() {
            self.held_for.wrapping_add(1)
        } else {
            0
        };
        let pressed = if self.held_for > 30 && self.held_for.is_multiple_of(5) {
            dpad
        } else {
            dpad.just_pressed(&self.last_pad)
        };

        let last_row = self.board.rows.get(self.ysel as usize).unwrap();
        let last_row_len = last_row.keys.len() as i32 - 1;
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

        let row = self.board.rows.get(self.ysel as usize).unwrap();
        let row_len = row.keys.len() as i32 - 1;

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

    fn handle_buttons(&mut self, buttons: Buttons) -> State {
        let mut state = State::Open;

        let pressed = buttons.just_pressed(&self.last_buttons);
        if pressed.s || pressed.e {
            let maybe_state = self.handle_pressed();
            if let Some(new_state) = maybe_state {
                state = new_state;
            }
        }

        let released = buttons.just_released(&self.last_buttons);
        if released.s || released.e {
            let maybe_state = self.handle_released();
            if let Some(new_state) = maybe_state {
                state = new_state;
            }
        }

        if pressed.w {
            state = if self.text.is_empty() {
                // NOTE: remove?
                State::JustCancelled
            } else {
                self.text.pop();
                State::TextChanged
            }
        }
        if pressed.n {
            self.board.shifted = !self.board.shifted;
        }
        state
    }

    fn handle_pressed(&mut self) -> Option<State> {
        let key = self.board.get(self.xsel as usize, self.ysel as usize)?;
        match key.key_type {
            KeyType::Char(c) => {
                self.text.push(c);
                Some(State::TextChanged)
            }
            KeyType::Space => {
                self.text.push(' ');
                Some(State::TextChanged)
            }
            KeyType::Backspace => {
                self.text.pop();
                Some(State::TextChanged)
            }
            KeyType::Shift => {
                self.board.shifted = !self.board.shifted;
                None
            }
            KeyType::Ok | KeyType::Cancel => None,
        }
    }

    fn handle_released(&mut self) -> Option<State> {
        let key = self.board.get(self.xsel as usize, self.ysel as usize)?;
        match key.key_type {
            KeyType::Ok => Some(State::JustClosed),
            KeyType::Cancel => Some(State::JustCancelled),
            _ => None,
        }
    }

    /// Renders the keyboard.
    pub fn render(&self, font: &Font) {
        if self.is_open {
            self.board.draw(self, font);
        }
    }

    /// Opens the keyboard.
    pub fn open(&mut self) {
        self.is_open = true;
        self.last_buttons = read_buttons(self.peer);
        self.xsel = 0;
        self.ysel = 0;
    }

    /// Returns if the keyboard is currently open.
    pub fn is_open(&mut self) -> bool {
        self.is_open
    }
}

impl Default for Keyboard {
    fn default() -> Self {
        Keyboard::new(Options::default())
    }
}
