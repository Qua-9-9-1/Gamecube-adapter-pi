# GameCube Adapter RP2040

Bare-metal Rust firmware for using up to four Nintendo GameCube controllers with an RP2040.

The firmware exposes the device as a WUP-028-compatible adapter for Wii U and Nintendo Switch. Controllers are read over Joybus and converted into a 37-byte USB report.

## Project status

- Four Joybus ports on GPIO 0 through 3
- Four-port WUP-028 USB report
- Tested with Windows, Linux, Dolphin, and Ryujinx
- Nintendo Switch mode tested with simulated and physical controller data
- Deterministic bit-banged Joybus acquisition
- PIO, DMA, rumble, and controller hot-plug handling remain future work

The firmware currently uses `usb-device` and `rp2040-hal`. Embassy is not required for the current implementation.

## Hardware

- RP2040 board with USB device support, such as a Raspberry Pi Pico
- One Joybus data line per controller on GPIO 0, 1, 2, and 3
- Common ground between the board and the controllers
- 3.3 V logic power
- One external pull-up resistor per DATA line, typically 2.0 kOhm to 3.3 V

Rumble motors require an appropriate power supply. Do not connect their power line to the 3.3 V logic rail without checking the wiring and current requirements.

## Build and flash

Install the Rust target and UF2 runner:

```bash
rustup target add thumbv6m-none-eabi
cargo install elf2uf2-rs
```

Check and build:

```bash
cargo fmt --check
cargo check --target thumbv6m-none-eabi
cargo build --release --target thumbv6m-none-eabi
```

To flash with `elf2uf2-rs`:

1. Hold the BOOTSEL button while connecting the board.
2. Run `cargo run --release`.
3. Disconnect and reconnect the board normally.

## Testing

On Nintendo Switch, open the controller-order screen. The device should be detected as a GameCube controller adapter.

On Windows, use `joy.cpl` to verify enumeration and axes. Dolphin can use the matching DirectInput or SDL device. Ryujinx may require selecting the SDL backend in its controller settings.

On Linux:

```bash
sudo apt install joystick
jstest /dev/input/js0
```

## Documentation

- [Getting started](docs/getting-started.md)
- [Firmware architecture](docs/firmware.md)
- [Protocols](docs/protocol.md)
- [Hardware and wiring](docs/hardware.md)
- [Technical history](docs/development-journey.md)

## License

No license has been defined for this repository yet.
