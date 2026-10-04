# Firmware architecture

## Active modules

- `main.rs` initializes clocks, GPIO, USB, and the polling schedule.
- `joybus.rs` sends the Joybus command and decodes the eight-byte response.
- `usb_switch.rs` exposes the WUP-028 descriptor, endpoints, and 37-byte report.

`usb_hid.rs` contains the legacy PC HID descriptor and is not used by the current Switch binary.

## Main loop

The USB report is updated every 8 ms at 125 Hz. Each cycle polls the four GPIO lines, marks absent controllers as disconnected, and sends a WUP report.

USB is serviced before the cycle, between Joybus transactions, and after the report is queued. This reduces the time a USB request waits for controller communication to finish.

## Joybus timing

Timing-critical functions are placed in SRAM with `link_section = ".data"`. Each transaction runs in a critical section so interrupts cannot alter the 1, 2, and 3 microsecond timing windows.

Receive waits use hardware-timer deadlines. A disconnected controller therefore cannot block the loop for an arbitrary number of iterations.

## Known limitations

- The four ports are polled sequentially.
- Bit-banging uses the CPU during each transaction.
- PIO and DMA are not currently used for Joybus.
- Rumble and USB output reports are not yet forwarded to controllers.
- The USB descriptor should not be changed without comparing it against a compatible adapter capture.
