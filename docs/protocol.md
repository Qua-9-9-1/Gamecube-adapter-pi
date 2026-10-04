# Protocols

## Joybus

The RP2040 uses the single-wire serial bus found in GameCube controllers. Each bit period is 4 microseconds:

- bit `1`: line low for approximately 1 microsecond, then high for 3 microseconds;
- bit `0`: line low for approximately 3 microseconds, then high for 1 microsecond.

The firmware sends this polling command:

```text
40 03 00
```

The expected response contains eight bytes:

| Byte | Contents |
| ---: | --- |
| 0 | main buttons |
| 1 | D-pad, Z, R, L, and signature bit |
| 2 | main stick X |
| 3 | main stick Y |
| 4 | C-Stick X |
| 5 | C-Stick Y |
| 6 | analog L trigger |
| 7 | analog R trigger |

Bit 7 of byte 1 is used as a controller signature and is not exposed as a button.

## WUP-028 report

The firmware exposes USB identity `VID:PID 057e:0337`. The input report contains 37 bytes:

```text
21 [port 1: 9 bytes] [port 2: 9 bytes] [port 3: 9 bytes] [port 4: 9 bytes]
```

Each port contains:

| Byte | Contents |
| ---: | --- |
| 0 | connection state, `0x10` when connected |
| 1 | first button group |
| 2 | additional buttons |
| 3 | main stick X |
| 4 | main stick Y |
| 5 | C-Stick X |
| 6 | C-Stick Y |
| 7 | analog L trigger |
| 8 | analog R trigger |

Reports are sent every 8 ms.

## USB initialization

The interface is exposed as a HID class 3 interface. The firmware responds to the embedded HID report descriptor and accepts the control request used by WUP-028-compatible adapters. The reserved output endpoint provides the endpoint numbering expected by the Switch.

Details that have not been confirmed against an official or Mayflash USB capture are documented as compatibility behavior, not as official Nintendo specifications.
