# luxboard lite

Virtual keyboard for [Firefly Zero](https://fireflyzero.com/) apps using [the Rust SDK](https://github.com/firefly-zero/firefly-rust).

![a screenshot taken from Firefly Zero of a dark gray, QWERTY layout keyboard with a light gray outline around each key. the selected key, 'OK', is highlighted in blue. above the keyboard is the text, "hello, world!"](assets/demo.png)

## Installation

```bash
cargo add firefly-keyboard
```

## Usage

```rust
// fn boot()
let opts = firefly_keyboard::Options::default();
let kbd = firefly_keyboard::Keyboard::new(opts);

// fn update()
kbd.update();
if !kbd.is_open() {
    let text = kbd.text.clone();
    // ...
}

// fn render()
kbd.render();
```

See [examples/keyboard/main.rs](./examples/keyboard/main.rs) for a complete example.

## License

[MIT License](./LICENSE). Feel free to use the package in any apps and games and modify it for your needs.

Initially developed by [Nanobot567](https://codeberg.org/Nanobot567) and is currently officially maintained by the Firefly Zero core team.
