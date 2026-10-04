#![no_std]
#![no_main]

mod joybus;
mod usb_switch;

use hal::pac;
use panic_halt as _;
use rp2040_hal as hal;
use usb_device::class_prelude::*;
use usb_device::prelude::*;
use usb_switch::{WupClass, WupReport};

#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;
const PORT_PINS: [u32; 4] = [1 << 0, 1 << 1, 1 << 2, 1 << 3];

#[hal::entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let _core = pac::CorePeripherals::take().unwrap();

    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);
    let clocks = hal::clocks::init_clocks_and_plls(
        12_000_000,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let sio = hal::Sio::new(pac.SIO);
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let _gp0 = pins.gpio0.into_pull_up_input();
    let _gp1 = pins.gpio1.into_pull_up_input();
    let _gp2 = pins.gpio2.into_pull_up_input();
    let _gp3 = pins.gpio3.into_pull_up_input();

    let usb_bus = hal::usb::UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    );
    let bus_allocator = UsbBusAllocator::new(usb_bus);
    let mut nintendo_class = WupClass::new(&bus_allocator);
    let mut usb_dev = UsbDeviceBuilder::new(&bus_allocator, UsbVidPid(0x057e, 0x0337))
        .strings(&[StringDescriptors::default()
            .manufacturer("Nintendo")
            .product("GameCube For Switch")])
        .unwrap()
        .device_class(0x00)
        .device_release(0x0100)
        .max_packet_size_0(64)
        .unwrap()
        .max_power(500)
        .unwrap()
        .build();

    let mut report = WupReport::new();
    let timer = unsafe { &*rp2040_hal::pac::TIMER::PTR };
    let mut last_poll = timer.timerawl().read().bits();

    loop {
        let now = timer.timerawl().read().bits();

        if now.wrapping_sub(last_poll) >= 8_000 {
            last_poll = now;

            for i in 0..4 {
                if let Some(data) = joybus::poll_controller(PORT_PINS[i]) {
                    report.update_port(i, &data);
                } else {
                    report.disconnect_port(i);
                }
            }

            let _ = nintendo_class.write_report(&report);
        }

        usb_dev.poll(&mut [&mut nintendo_class]);
    }
}
