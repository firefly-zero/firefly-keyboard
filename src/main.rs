#![no_std]
#![no_main]
use core::cell::OnceCell;

use firefly_rust::*;

use crate::luxboard_lite::{LuxboardLite, LuxboardLiteOptions};

mod luxboard_lite;

static mut STATE: OnceCell<State> = OnceCell::new();

struct State {
    font: FileBuf,
    luxboard_lite: LuxboardLite,
    buttons: Buttons,
}

fn get_state() -> &'static mut State {
    #[allow(static_mut_refs)]
    unsafe { STATE.get_mut() }.unwrap()
}

#[unsafe(no_mangle)]
extern "C" fn boot() {
    let state = State {
        font: load_file_buf("font").expect("could not load font!"),
        luxboard_lite: LuxboardLite::new(LuxboardLiteOptions {
            layout: luxboard_lite::LuxboardLiteLayout::Qwertyish,
            height: 75,
            bg_color: Some(Color::Black),
            text_color: Some(Color::White),
            line_color: Some(Color::Gray),
            highlight_color: Some(Color::Blue),
            locked_key_color: Some(Color::Purple),
            ..Default::default()
        }),
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

    if state.luxboard_lite.is_open() {
        state.luxboard_lite.update();
    } else {
        if pressed.e {
            state.luxboard_lite.open();
        }
    }

    state.buttons = buttons;
}

#[unsafe(no_mangle)]
extern "C" fn render() {
    let state = get_state();

    clear_screen(Color::White);

    if state.luxboard_lite.is_open() {
        state.luxboard_lite.render(&state.font.as_font());

        let mut tmp = state.luxboard_lite.text.clone();
        tmp.push('_');

        draw_text(
            &tmp,
            &state.font.as_font(),
            Point { x: 4, y: 8 },
            Color::Black,
        );
    }
}
