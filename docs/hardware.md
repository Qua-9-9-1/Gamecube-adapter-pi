# Hardware and wiring

## Controller signals

Each GameCube controller uses a single-wire 3.3 V DATA signal and a common ground.

| Port | RP2040 GPIO |
| --- | ---: |
| 1 | GPIO 0 |
| 2 | GPIO 1 |
| 3 | GPIO 2 |
| 4 | GPIO 3 |

The GPIOs are used as high-impedance inputs or low outputs. The firmware must never actively drive the DATA line high.

## Pull-up resistors

Install an external resistor of approximately 2.0 kOhm between each DATA line and 3.3 V. The RP2040 internal pull-ups are too weak to guarantee the rise time required by Joybus.

## Power

Use a stable 3.3 V supply for controller logic and connect all grounds. Rumble motors require a separate supply suitable for their current.

Before connecting a controller, verify ground continuity and ensure that no 5 V signal reaches the logic DATA line.
