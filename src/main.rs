#![no_std]
#![no_main]

mod joybus;
mod usb_hid;

use hal::pac;
use panic_halt as _;
use rp2040_hal as hal;

use usb_device::class_prelude::*;
use usb_device::prelude::*;
use usb_hid::GamepadReport;
use usbd_hid::descriptor::SerializedDescriptor;
use usbd_hid::hid_class::HIDClass;

#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

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
    let _pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let usb_bus = hal::usb::UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    );
    let bus_allocator = UsbBusAllocator::new(usb_bus);

    let mut hid = HIDClass::new(&bus_allocator, GamepadReport::desc(), 10);
    let mut usb_dev = UsbDeviceBuilder::new(&bus_allocator, UsbVidPid(0x1209, 0x0001))
        .strings(&[StringDescriptors::default()
            .manufacturer("Nivo")
            .product("GameCube Adapter PC")])
        .unwrap()
        .device_class(0x00)
        .build();

    let _gp0 = _pins.gpio0.into_pull_up_input();

    let mut report = GamepadReport {
        x: 128,
        y: 128,
        rx: 128,
        ry: 128,
        buttons_1: 0,
        buttons_2: 0,
    };

    let timer = unsafe { &*rp2040_hal::pac::TIMER::PTR };
    let mut last_poll = timer.timerawl().read().bits();

    loop {
        let now = timer.timerawl().read().bits();

        if now.wrapping_sub(last_poll) >= 10_000 {
            last_poll = now;

            if let Some(data) = joybus::poll_controller() {
                report.buttons_1 = data.buttons_1;
                report.buttons_2 = data.buttons_2 & 0x7F;

                report.x = data.stick_x;
                report.y = 255_u8.saturating_sub(data.stick_y);
                report.rx = data.c_stick_x;
                report.ry = 255_u8.saturating_sub(data.c_stick_y);
            }

            let _ = hid.push_input(&report);
        }

        if usb_dev.poll(&mut [&mut hid]) {
            let _ = hid.push_input(&report);
        }
    }
}
