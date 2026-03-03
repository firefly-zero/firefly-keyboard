#![no_std]
#![no_main]
use core::cell::OnceCell;

use firefly_rust::*;

use crate::luxboard_lite::{LuxboardLite, LuxboardLiteOptions};

mod luxboard_lite;

static mut STATE: OnceCell<State> = OnceCell::new();

struct State {
    font: FileBuf,
    luxboard_lite: LuxboardLite
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
            height: 70
        })
    };

    #[allow(static_mut_refs)]
    unsafe { STATE.set(state) }.ok().unwrap();
}

#[unsafe(no_mangle)]
extern "C" fn update() {
    let state = get_state();

    state.luxboard_lite.update();
}

#[unsafe(no_mangle)]
extern "C" fn render() {
    let state = get_state();

    clear_screen(Color::White);
    
    state.luxboard_lite.render(&state.font.as_font());
}
