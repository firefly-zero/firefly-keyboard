id := "nanobot567.luxboard-lite"

default:
  firefly_cli build
  firefly_cli emulator -- --id {{id}}

emu: 
  firefly_cli emulator

clippy:
  cargo +nightly clippy --target wasm32-unknown-unknown -Zbuild-std

clippy-fix:
  cargo +nightly clippy --target wasm32-unknown-unknown -Zbuild-std --fix --bin "luxboard-lite" -p luxboard-lite --allow-dirty
