# Protocol and Data Specifications

This document explains the communication protocols used in this project: the low-level Nintendo GameCube **Joybus** protocol and the **USB HID Gamepad** report format, including the exact bit-level transformations between them.

---

## 1. Joybus Physical Layer Protocol

Joybus is a synchronous, single-wire serial protocol running at **250 kbit/s**. 

### Bit Timing and Waveforms

Every bit lasts exactly **4.0 microseconds (µs)**. The transmitting device signals a bit by pulling the line to 0V (LOW) for a specific duration, then releasing it so the pull-up resistor pulls it back to 3.3V (HIGH):

* **Bit 1:** Line is pulled **LOW for 1.0 µs**, then released **HIGH for 3.0 µs**.
* **Bit 0:** Line is pulled **LOW for 3.0 µs**, then released **HIGH for 1.0 µs**.
* **Stop Bit:** A 1.0 µs LOW pulse followed by the line remaining HIGH.

```
Joybus Bit 1 (4 µs period):
   3.3V ---+       +-------------------+
           |       | (Sample @ 2 µs: HIGH -> 1)
     0V    +-------+
             1 µs          3 µs

Joybus Bit 0 (4 µs period):
   3.3V ---+                           +---
           | (Sample @ 2 µs: LOW -> 0) |
     0V    +---------------------------+
                       3 µs            1 µs
```

### The 2 µs Sampling Window

When reading data from the controller, the adapter samples the line at **exactly 2.0 µs** after detecting the falling edge:
* If the line is **HIGH** at 2 µs: The controller held the line low for only 1 µs and released it $\rightarrow$ **Bit 1**.
* If the line is **LOW** at 2 µs: The controller is still pulling the line low until 3 µs $\rightarrow$ **Bit 0**.

---

## 2. Joybus Command and Response Cycle

Each polling cycle consists of a **host command phase** (transmitted by the RP2040) followed by a **controller response phase** (transmitted by the GameCube controller).

### 2.1 Host Command Phase (25 bits, ~100 µs)
The RP2040 sends the standard GameCube poll command:
1. `0x40` (8 bits: `01000000b`) — Poll buttons and analog axes.
2. `0x03` (8 bits: `00000011b`) — Data format specifier (standard format).
3. `0x00` (8 bits: `00000000b`) — Rumble command (`0` = motor off, `1` = motor on).
4. `Stop Bit` (1 bit: 1 µs LOW, released HIGH).

### 2.2 Controller Response Phase (65 bits, ~260 µs)
Within 4 to 10 µs after the console stop bit, the controller replies with 8 data bytes (64 bits) followed by its own stop bit:

```
+-------------------------------------------------------------------------------+
| Byte 0 | Byte 1 | Byte 2 | Byte 3 | Byte 4 | Byte 5 | Byte 6     | Byte 7     |
| Buttons| Buttons| Main X | Main Y | C-Stk X| C-Stk Y| Analog L   | Analog R   |
+-------------------------------------------------------------------------------+
```

#### Detailed Breakdown of the 8 Bytes

| Byte Index | Field Name | Bit Structure / Range | Description |
| :---: | :---: | :---: | :--- |
| **0** | `buttons_1` | `[0, 0, 0, Start, Y, X, B, A]` | Digital buttons. Bit 0 = A, Bit 1 = B, Bit 2 = X, Bit 3 = Y, Bit 4 = Start. |
| **1** | `buttons_2` | `[1, L, R, Z, D-Up, D-Dn, D-R, D-L]` | Digital buttons. Bits 0-3 = D-Pad, Bit 4 = Z, Bit 5 = R click, Bit 6 = L click, **Bit 7 = Always 1** (hardware ID). |
| **2** | `stick_x` | `0 .. 255` (Center ~128) | Main analog stick horizontal axis (0 = Left, 255 = Right). |
| **3** | `stick_y` | `0 .. 255` (Center ~128) | Main analog stick vertical axis (**~28 = Down, ~228 = Up**). |
| **4** | `c_stick_x` | `0 .. 255` (Center ~128) | C-Stick analog horizontal axis (0 = Left, 255 = Right). |
| **5** | `c_stick_y` | `0 .. 255` (Center ~128) | C-Stick analog vertical axis (**~28 = Down, ~228 = Up**). |
| **6** | `analog_l` | `0 .. 255` | Left analog trigger travel before digital click. |
| **7** | `analog_r` | `0 .. 255` | Right analog trigger travel before digital click. |

---

## 3. USB HID Gamepad Specification

The RP2040 enumerates as a standard USB HID Gamepad (`Usage: 0x05`, `Usage Page: 0x01 Generic Desktop`).

### 3.1 Report Descriptor Layout

The report is defined via the `#[gen_hid_descriptor]` macro in `src/usb_hid.rs`:

* **4 Analog Axes (`axes: [u8; 4]`, 8-bit unsigned integers, range `0` to `255`):**
  * `axes[0]` (`0x30`): Primary Left Stick Horizontal (0 = Left, 255 = Right)
  * `axes[1]` (`0x31`): Primary Left Stick Vertical
  * `axes[2]` (`0x32`): C-Stick Horizontal
  * `axes[3]` (`0x33`): C-Stick Vertical
* **16 Logical Buttons (1-bit each, packed into 2 bytes):**
  * Buttons 1 to 8: A, B, X, Y, Start, Unused (3 bits)
  * Buttons 9 to 16: D-Left, D-Right, D-Down, D-Up, Z, R, L, Unused (1 bit)

```rust
pub struct GamepadReport {
    pub axes: [u8; 4],
    pub buttons_1: u8,
    pub buttons_2: u8,
}
```

---

## 4. Normalization and Protocol Conversion Rules

The raw GameCube readings cannot be passed directly to the PC without two essential mathematical transformations:

### 4.1 Vertical Axis Inversion (`255 - y`)
* **Nintendo convention:** Joybus outputs ~228 when the stick is pushed UP, and ~28 when pushed DOWN.
* **USB HID convention:** The USB Generic Desktop standard (and Windows `joy.cpl`) defines coordinate `0` as UP (forward) and `255` as DOWN (backward).
* **The Transformation:**
  ```rust
  self.y = 255_u8.saturating_sub(data.stick_y);
  self.ry = 255_u8.saturating_sub(data.c_stick_y);
  ```
  This ensures pushing up on either stick moves the cursor up on PC without requiring per-game in-game axis inversions.

### 4.2 Nintendo Signature Bit Masking (`& 0x7F`)
* **The Issue:** Bit 7 of Byte 1 in the Joybus response is hardcoded to `1` by Nintendo hardware.
* **The Consequence:** Without masking, Bit 7 maps to USB Button 16, causing Button 16 to be permanently active on Windows and Linux.
* **The Transformation:**
  ```rust
  self.buttons_2 = data.buttons_2 & 0x7F;
  ```
  This strips bit 7 while preserving D-Pad, Z, R, and L states.

---

## 5. End-to-End Data Mapping Summary

| Controller Action | Raw Joybus Location | USB HID Target Field | Target Value / Action |
| :--- | :--- | :--- | :--- |
| **Button A** | Byte 0, Bit 0 | `buttons_1`, Bit 0 | USB Button 1 |
| **Button B** | Byte 0, Bit 1 | `buttons_1`, Bit 1 | USB Button 2 |
| **Button X** | Byte 0, Bit 2 | `buttons_1`, Bit 2 | USB Button 3 |
| **Button Y** | Byte 0, Bit 3 | `buttons_1`, Bit 3 | USB Button 4 |
| **Button Start** | Byte 0, Bit 4 | `buttons_1`, Bit 4 | USB Button 5 |
| **D-Pad Left** | Byte 1, Bit 0 | `buttons_2`, Bit 0 | USB Button 9 |
| **D-Pad Right** | Byte 1, Bit 1 | `buttons_2`, Bit 1 | USB Button 10 |
| **D-Pad Down** | Byte 1, Bit 2 | `buttons_2`, Bit 2 | USB Button 11 |
| **D-Pad Up** | Byte 1, Bit 3 | `buttons_2`, Bit 3 | USB Button 12 |
| **Button Z** | Byte 1, Bit 4 | `buttons_2`, Bit 4 | USB Button 13 |
| **Button R** (Digital) | Byte 1, Bit 5 | `buttons_2`, Bit 5 | USB Button 14 |
| **Button L** (Digital) | Byte 1, Bit 6 | `buttons_2`, Bit 6 | USB Button 15 |
| **Nintendo Signature** | Byte 1, Bit 7 | *Masked out* | Cleared to 0 |
| **Main Stick X** | Byte 2 (`0..255`) | `report.axes[0]` (`0..255`) | Direct mapping (Left = 0, Right = 255) |
| **Main Stick Y** | Byte 3 (`0..255`) | `report.axes[1]` (`0..255`) | **PC Inverted:** `255 - raw_y` (Up = 0, Down = 255) |
| **C-Stick X** | Byte 4 (`0..255`) | `report.axes[2]` (`0..255`) | Direct mapping (Left = 0, Right = 255) |
| **C-Stick Y** | Byte 5 (`0..255`) | `report.axes[3]` (`0..255`) | **PC Inverted:** `255 - raw_y` (Up = 0, Down = 255) |

