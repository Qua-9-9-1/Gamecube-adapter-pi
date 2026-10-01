# Engineering Journey and Post-Mortem

This document chronicles the complete development journey, technical hurdles, post-mortem analyses, and iterative solutions encountered while engineering a bare-metal GameCube controller to USB HID adapter on the Raspberry Pi RP2040 using Rust.

---

## 1. Physical Foundations and Electrical Challenges

Before writing any software, the physical communication circuit had to establish a reliable, noiseless electrical loop. Several hardware pitfalls prevented the hardware from operating correctly initially.

```
       RP2040 (3.3V Logic)                     GameCube Controller
   +-------------------------+               +---------------------+
   |                     3V3 |---------------> Pin 6 (3.3V Logic)   |
   |                         |   2.0 kΩ      |                     |
   |                         |  Pull-Up      |                     |
   |                         |  +---[===]---+|                     |
   |                         |  |            |                     |
   |                     GP0 |<-+------------> Pin 2 (Bidirectional|
   |                         |               |        Data Line)   |
   |                     GND |---------------+ Pin 3 (Logic GND)   |
   +-------------------------+               +---------------------+
```

### 1.1 The Floating Ground (GND) Failure

* **The Problem:** During the initial breadboard phase, the microcontroller could not detect any signal transitions from the controller, and oscilloscope/logic readings were erratic or completely flat.
* **Root Cause:** A cold solder joint on the ground pin resulted in a floating ground.
* **The "Plumbing" Analogy:** An electric circuit requires a closed loop. Transmitting data through a single wire without a solid ground reference is like pumping water into an intake pipe without connecting the drainage/return pipe: the water has nowhere to flow, and electrical potential (voltage) cannot establish itself.
* **The Solution:** Resoldering the ground connection directly to the RP2040's GND pin created an unambiguous common 0V reference potential between both devices.

---

### 1.2 The Missing Pull-Up Resistor

* **The Problem:** The data line floated unpredictably. When the controller or microcontroller pulled the line down to 0V, it failed to return to 3.3V fast enough to register subsequent bits, turning square wave pulses into smeared, sloping curves.
* **Root Cause:** Nintendo's Joybus protocol is an **open-drain** (open-collector) single-wire bus. Neither the console nor the controller actively drives the line to 3.3V; they only drive it to 0V (LOW) or release it into high impedance (HIGH-Z).
* **The "Mechanical Spring" Analogy:** Without a pull-up resistor, pulling the wire low is like depressing a spring that has no restorative force. It stays down or floats back up at an agonizingly slow pace. The pull-up resistor acts as a stiff mechanical spring: as soon as the line is released, it snaps back to 3.3V instantly.
* **The Solution:** Installing an external **2.0 kΩ (or 2.2 kΩ)** pull-up resistor between the DATA line (GP0) and the 3.3V power rail produced sharp, sub-microsecond rise times that met Joybus electrical standards.

---

## 2. The Timing Battles (Bare-Metal Software Hurdles)

The Nintendo GameCube Joybus protocol operates at **250 kbit/s**, which means each bit lasts exactly **4 microseconds (µs)**. In this protocol:
* A **Bit 1** is represented by: **1 µs LOW**, followed by **3 µs HIGH**.
* A **Bit 0** is represented by: **3 µs LOW**, followed by **1 µs HIGH**.

At this scale, a jitter of even half a microsecond will cause bit inversion or frame corruption.

```
Joybus Bit 1 (4 µs):
   3.3V ---+       +-------------------+
           |       | (Sample @ 2 µs: HIGH)
     0V    +-------+
             1 µs          3 µs

Joybus Bit 0 (4 µs):
   3.3V ---+                           +---
           | (Sample @ 2 µs: LOW)      |
     0V    +---------------------------+
                       3 µs            1 µs
```

---

### 2.1 The "Cloakroom" Failure (Standard Rust HAL Overhead)

* **The Problem:** Attempting to toggle GPIO pins using standard high-level Rust Hardware Abstraction Layer (HAL) methods resulted in pulses stretching beyond 2.5 µs instead of the commanded 1.0 µs. The GameCube controller completely ignored commands.
* **Root Cause:** Standard HAL primitives enforce type safety, runtime peripheral borrowing checks, and multi-layered function abstractions. A pin write routine took upwards of 40 to 50 CPU cycles instead of 1 cycle.
* **The Solution (Direct SIO Register Access):** We bypassed high-level wrappers for time-critical operations and manipulated the RP2040's **Single-cycle IO (SIO)** peripheral directly:
  - To pull the line LOW: atomically set the output register bit and enable output driver in 1 cycle (`gpio_out_clr` + `gpio_oe_set`).
  - To release the line HIGH: atomically clear the output driver enable bit in 1 cycle (`gpio_oe_clr`), allowing the pull-up resistor to restore 3.3V.

---

### 2.2 The "Corridor" Failure (Flash Memory and XIP Cache Latency)

* **The Problem:** Even with direct register writes, pulse lengths fluctuated randomly by 200 ns to 800 ns. The timing was nondeterministic.
* **Root Cause:** By default, compiled code resides in external SPI Flash memory. The RP2040 fetches instructions via an Execute-In-Place (XIP) cache. When an instruction or loop causes a cache miss, the Cortex-M0+ core stalls while the external QSPI bus fetches the missing bytes.
* **The "Corridor" Analogy:** A chef preparing a recipe that requires split-second timing cannot afford to walk down a long hallway to a storeroom to read the next sentence of the recipe. The recipe must be placed directly on the kitchen counter.
* **The Solution (`#[link_section = ".data"]`):** We instructed the linker to load all timing-critical routines (`wait_us`, `send_bit`, `send_byte`, `poll`) directly into the RP2040's internal fast SRAM (`.data` section) during boot. Zero cache misses occur, resulting in cycle-accurate timing.

---

### 2.3 The "Phone Call" Failure (USB & Interrupt Collisions)

* **The Problem:** During communication bursts with the controller, a byte would occasionally be dropped or corrupted, causing the controller to disconnect or glitch for a frame.
* **Root Cause:** The host PC sent USB token packets or peripheral interrupts fired while the RP2040 was in the middle of sampling a 4 µs Joybus bit. Servicing the interrupt stalled execution, blowing past the microsecond sampling window.
* **The "Phone Call" Analogy:** Trying to measure an athletic race with a microsecond stopwatch while answering phone calls at random intervals. Answering the phone means the measurement is lost.
* **The Solution (`cortex_m::interrupt::free`):** The entire Joybus transaction (~400 µs) is enclosed in an atomic critical section:
  ```rust
  cortex_m::interrupt::free(|_| {
      // Send 24-bit command + stop bit
      // Receive 64-bit controller payload + stop bit
  });
  ```
  The processor ignores outside interruptions during the dialogue, guaranteeing deterministic timing.

---

## 3. Protocol Translation and Operating System Logistics

Once clean microsecond pulses were successfully transmitted and captured, the raw bitstream had to be parsed and translated into compliant USB HID reports for the operating system.

### 3.1 Logical Polarity Inversion (The Mirror Effect)

* **The Problem:** The adapter initially interpreted incoming data as completely invalid or rejected packets.
* **Root Cause:** The bit-sampling logic initially inverted polarity: confusing a line held low for 3 µs as a `1` and 1 µs as a `0`.
* **The Solution:** Standardizing the sampling instant:
  1. Detect the falling edge initiated by the controller (beginning of the bit).
  2. Wait exactly **2 µs**.
  3. Sample the line:
     - If the line is already **HIGH** (3.3V) at 2 µs $\rightarrow$ It returned high at 1 µs $\rightarrow$ **Bit 1**.
     - If the line is still **LOW** (0V) at 2 µs $\rightarrow$ It remains low until 3 µs $\rightarrow$ **Bit 0**.

---

### 3.2 The "Suffocation" Failure (Over-Polling the Controller)

* **The Problem:** The controller would reply once or twice, then return all zeros, float the line, or glitch out.
* **Root Cause:** Running an unthrottled polling loop at 125 MHz hammered the controller with new commands every few microseconds. Real GameCube controller microcontrollers require recovery time between interrogations.
* **The Solution (The Hardware Metronome):** We implemented a strict 10 millisecond (100 Hz) pacing mechanism using the RP2040 64-bit hardware timer (`pac::TIMER`). The adapter queries the controller once every 10,000 µs, matching both the controller's internal update cycle and the USB HID polling rate (`bInterval = 10`).

---

### 3.3 The "Single Box" Failure (Axis Collisions in Linux)

* **The Problem:** In initial tests with `jstest` under Linux, moving the C-Stick overwrote the Main Stick values, or axes stuck at `-32767`.
* **Root Cause:** Defining generic, unordered axis fields without standard USB HID usage tags caused the Linux `evdev` driver to merge or misassign the incoming coordinates.
* **The Solution:** A structured HID descriptor with explicit usage tags and structured byte packing:
  - Main Stick: `X` (`0x30`) and `Y` (`0x31`)
  - C-Stick: `Rx` (`0x33`) and `Ry` (`0x34`)
  - Discrete bit-packed button fields with defined logical ranges.

---

### 3.4 The Ghost Button 15/16 (Nintendo Signature Bit)

* **The Problem:** Button 15 (or 16 depending on indexing) was detected as constantly held down on the PC.
* **Root Cause:** In the standard Nintendo GameCube Joybus protocol, the controller's 8-byte status response has a fixed signature:
  - Byte 1: `[ Bit 7: ALWAYS 1 | L | R | Z | D-Up | D-Down | D-Right | D-Left ]`
  - Bit 7 of byte 1 is hardwired to `1` by Nintendo hardware as a device identification flag.
* **The Solution:** Masking out bit 7 before packing the report:
  ```rust
  self.buttons_2 = data.buttons_2 & 0x7F;
  ```

---

## 4. The Windows Compatibility & Hardening Phase

While the initial implementation worked under Linux (`jstest`), running it on Windows revealed critical flaws that led to adapter freezes, device descriptor errors, and inverted controls.

| Issue on Windows | Root Cause | Implemented Solution |
| :--- | :--- | :--- |
| **USB Host Starvation (Freezing)** | `push_input()` was only executed when `usb_dev.poll()` returned `true`. Windows does not emit bus events when waiting on an interrupt pipe, causing a permanent deadlock. | `usb_dev.poll()` is now called unconditionally on every loop iteration, while `push_input()` is called on every 10 ms timer tick when `Configured`. |
| **USB Descriptor Failure (Code 43)** | A disconnected controller caused the `while` loop to spin for 100,000 iterations inside `interrupt::free`, blocking CPU interrupts for 8 ms every 10 ms. Windows xHCI timed out during enumeration. | Timeouts are now bounded by microsecond hardware timestamps: 50 µs for the initial falling edge, 15 µs for bit recovery. Total disconnect latency is < 50 µs. |
| **Inverted Vertical Axes** | GameCube controllers output ~228 for UP and ~28 for DOWN. USB HID specifies 0 for UP and 255 for DOWN. | `y = 255_u8.saturating_sub(data.stick_y)` is now applied to both Main Y and C-Stick Y. |
| **Missing C-Stick in PC Games** | Usages `0x30..=0x33` mapped the C-Stick to Z (Throttle) and Rx (Rudder). DirectInput games failed to detect a right stick. | Usages were updated to `X` (0x30), `Y` (0x31) for Main Stick, and `Rx` (0x33), `Ry` (0x34) for C-Stick. |

---

## 5. Summary of the Architectural Progression

```mermaid
timeline
    title Evolution of the GameCube RP2040 Adapter
    section Phase 1 : Electrical
        Cold solder joint on GND : Fixed floating ground
        Signal floating / slow rise : Added 2.0 kΩ external pull-up
    section Phase 2 : Real-Time Core
        Slow Rust HAL pin writes : Migrated to single-cycle SIO
        XIP flash cache jitter : Linked critical routines into .data (SRAM)
        Interrupt interference : Protected transactions with interrupt::free
    section Phase 3 : Protocol Translation
        Inverted line polarity : Corrected 2 µs sampling window
        Controller over-polling : Enforced 10 ms hardware timer metronome
        Ghost Button 16 : Bitmasked byte 1 with 0x7F
    section Phase 4 : Windows & Hardening
        USB polling deadlock : Made usb_dev.poll() unconditional
        8 ms CPU lockup on unplug : Replaced loop counters with 50 µs timer limits
        Inverted sticks on Windows : Normalized Y-axes (255 - y)
        Unrecognized right stick : Adopted standard DirectInput X, Y, Rx, Ry usages
```

Today, the adapter functions deterministically: zero timing variance is left to chance, electrical specifications match Nintendo's requirements, and USB reports strictly adhere to USB HID class specifications on both Windows and Linux.
