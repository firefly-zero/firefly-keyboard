#![no_std]
#![no_main]

extern crate alloc;

use alloc::format;
use core::cell::OnceCell;
use firefly_keyboard::{Keyboard, Options};
use firefly_rust::*;

static mut STATE: OnceCell<State> = OnceCell::new();

struct State {
    font: FontBuf,
    keyboard: Keyboard,
    buttons: Buttons,
}

fn get_state() -> &'static mut State {
    #[allow(static_mut_refs)]
    unsafe { STATE.get_mut() }.unwrap()
}

#[unsafe(no_mangle)]
extern "C" fn boot() {
    let state = State {
        font: load_file_buf("font").unwrap().into(),
        keyboard: Keyboard::new(Options::default()),
        buttons: read_buttons(Peer::COMBINED),
    };

    #[allow(static_mut_refs)]
    unsafe { STATE.set(state) }.ok().unwrap();
}

#[unsafe(no_mangle)]
extern "C" fn update() {
    let state = get_state();
    let buttons = read_buttons(Peer::COMBINED);
    let pressed = buttons.just_pressed(&state.buttons);
    if state.keyboard.is_open() {
        state.keyboard.update();
    } else if pressed.e {
        state.keyboard.open();
    }
    state.buttons = buttons;
}

#[unsafe(no_mangle)]
extern "C" fn render() {
    let state = get_state();
    clear_screen(Color::Black);
    let font = &state.font;
    if state.keyboard.is_open() {
        state.keyboard.render(font);
        let mut tmp = state.keyboard.text.clone();
        tmp.push('_');
        draw_text(&tmp, font, Point::new(4, 8), Color::White);
    } else {
        let text = format!(
            "current text: {}\n\npress E to open keyboard",
            state.keyboard.text,
        );
        draw_text(&text, font, Point::new(4, 8), Color::White);
    }
}
