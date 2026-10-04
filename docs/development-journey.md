# Technical history

## Current design choices

The project started as a PC HID adapter and later added WUP-028 compatibility for Wii U and Nintendo Switch. The current binary uses the WUP-028 path by default.

Switch compatibility depends on several details being correct at the same time:

- USB identity `057e:0337`;
- HID interface and report descriptor;
- endpoints and polling intervals;
- control requests;
- 37-byte reports sent at 125 Hz.

These parts should remain stable during Joybus performance work.

## Implemented changes

- four GPIO controller ports;
- multi-port WUP report;
- hardware-timer-based Joybus timeouts;
- USB servicing between transactions;
- conversion of GameCube buttons and axes to WUP-028 format.

## Remaining work

- measure cycle duration with a logic analyzer;
- incrementally replace bit-banging with a PIO state machine;
- evaluate DMA after PIO timing is validated;
- implement rumble and USB output reports;
- add host-side tests for report conversion.

Compatibility claims should be backed by hardware testing or a USB capture from the target adapter.
