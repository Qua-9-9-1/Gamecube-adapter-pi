# Getting started

## Prerequisites

- Rust and Cargo
- The `thumbv6m-none-eabi` target
- `elf2uf2-rs` for automatic flashing
- An RP2040 board in USB device mode

```bash
rustup target add thumbv6m-none-eabi
cargo install elf2uf2-rs
```

## Build

```bash
cargo fmt --check
cargo check --target thumbv6m-none-eabi
cargo build --release --target thumbv6m-none-eabi
```

The release binary is written to `target/thumbv6m-none-eabi/release/`.

## Flash

1. Hold BOOTSEL while connecting the board.
2. Run:

```bash
cargo run --release
```

3. Reconnect the board without holding BOOTSEL.

The Cargo configuration uses `elf2uf2-rs -d` as its runner.

## Hardware checks

Before the first test, verify:

- common ground between the RP2040 and controllers;
- each controller DATA line is connected to the expected GPIO;
- logic power is 3.3 V;
- every DATA line has an external pull-up resistor;
- the rumble power line is not connected directly to a GPIO output.

## Switch verification

Open the controller-order screen. The firmware uses USB identity `057e:0337` and the WUP-028 protocol.

## PC verification

On Windows, open `joy.cpl`, then configure the device in Dolphin or Ryujinx.

On Linux:

```bash
ls -l /dev/input/js*
jstest /dev/input/js0
```

`WGInput`, `SDL`, and `DInput` may represent the same Windows device through different APIs. In Dolphin, select one backend for each controller.
