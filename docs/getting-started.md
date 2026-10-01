# Getting Started: Build, Flash, and Test Guide

This guide covers prerequisites, compiling the firmware, flashing it onto the Raspberry Pi RP2040 (such as a Raspberry Pi Pico), and verifying controller functionality on Windows and Linux.

---

## 1. Prerequisites

### 1.1 Install Rust
Ensure you have the Rust toolchain installed. If not, install it via [rustup.rs](https://rustup.rs/):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 1.2 Install the RP2040 Compilation Target
The RP2040 uses an ARM Cortex-M0+ core (`thumbv6m-none-eabi`). Add the target with:
```bash
rustup target add thumbv6m-none-eabi
```

### 1.3 Install `elf2uf2-rs` (Optional but Recommended)
To automatically convert and flash the binary to the RP2040 in one step:
```bash
cargo install elf2uf2-rs
```

---

## 2. Compiling the Project

From the project root directory:

```bash
# Verify type-checking and syntax
cargo check

# Compile the optimized release build
cargo build --release
```

The compiled ELF binary will be located at:
```
target/thumbv6m-none-eabi/release/gc-adapter-rp2040
```

---

## 3. Flashing to the RP2040

### Method A: Automated Flashing with `cargo run` (Recommended)

1. Hold down the **BOOTSEL** button on your Raspberry Pi Pico / RP2040 board while plugging it into your computer via USB.
2. The board will appear as a USB mass storage drive named **`RPI-RP2`**.
3. Run:
   ```bash
   cargo run --release
   ```
   `elf2uf2-rs` will automatically convert the binary to a `.uf2` file, copy it to the board, and reboot the RP2040.

### Method B: Manual UF2 Conversion and Drag-and-Drop

1. Convert the compiled ELF binary to a `.uf2` image:
   ```bash
   elf2uf2-rs target/thumbv6m-none-eabi/release/gc-adapter-rp2040 firmware.uf2
   ```
2. Hold down the **BOOTSEL** button while connecting the RP2040 via USB to enter bootloader mode.
3. Drag and drop `firmware.uf2` into the **`RPI-RP2`** drive. The board will reboot automatically.

---

## 4. Verification and Testing

### 4.1 On Windows

1. Press `Win + R`, type **`joy.cpl`**, and press Enter.
2. In the *Game Controllers* window, verify that **"GameCube Adapter PC"** is listed.
3. Select the adapter and click **Properties**:
   * **Main Stick (Left Stick):** Moving the gray stick moves the crosshair inside the 2D box. The stick is centered at rest, and pushing UP moves the crosshair UP.
   * **C-Stick (Right Stick):** Moving the yellow C-stick deflects the `Rx` and `Ry` rotation gauges.
   * **Buttons:** Pressing A, B, X, Y, Start, D-Pad (Up, Down, Left, Right), Z, L, and R lights up the respective numbered circles.
   * **Ghost Button Check:** Ensure no buttons (specifically Button 16) remain permanently illuminated when the controller is untouched.

### 4.2 On Linux

1. Install `joystick` testing utilities if needed:
   ```bash
   sudo apt-get install joystick
   ```
2. Check for the newly created device node:
   ```bash
   ls -l /dev/input/js*
   ```
3. Run `jstest`:
   ```bash
   jstest /dev/input/js0
   ```
   * Axes 0 and 1 correspond to the Main Stick (`X` and `Y`).
   * Axes 2 and 3 correspond to the C-Stick (`Rx` and `Ry`).
   * Buttons 0 through 11 register presses cleanly with 0 bouncing.

---

## 5. Troubleshooting Guide

| Symptom | Probable Cause | Corrective Action |
| :--- | :--- | :--- |
| **Pico does not appear as `RPI-RP2`** | Faulty USB cable or BOOTSEL button not held firmly. | Use a data-capable USB cable and ensure BOOTSEL is held while inserting the USB plug. |
| **Windows shows "Code 43 / Unknown USB Device"** | Disconnected controller causing CPU timeout loop. | Verify you have compiled the latest code with microsecond bounded timeouts (< 50 µs). |
| **Controller inputs are completely unresponsive** | Missing pull-up resistor or faulty GND. | Ensure a **2.0 kΩ resistor** connects DATA (`GP0`) to 3.3V, and verify GND continuity with a multimeter. |
| **Vertical stick movement is inverted** | Missing Y-axis normalization. | Verify that `255 - stick_y` is applied in `update_from_controller()`. |
| **Button 16 is constantly pressed** | Unmasked Nintendo signature bit. | Verify that `buttons_2 & 0x7F` is applied in `update_from_controller()`. |
| **Sticks drift or are not centered at rest** | Worn GameCube potentiometers. | Clean the joystick potentiometer box with contact cleaner or adjust the game's deadzone. |
