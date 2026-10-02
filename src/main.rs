#![no_std]
#![no_main]

mod joybus;
// mod usb_hid;
mod usb_switch;

use hal::pac;
use panic_halt as _;
use rp2040_hal as hal;

use usb_device::class_prelude::*;
use usb_device::prelude::*;
use usb_switch::{WupClass, WupReport};
// use usb_hid::GamepadReport;
// use usbd_hid::descriptor::SerializedDescriptor;
// use usbd_hid::hid_class::HIDClass;

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

    // let mut hid = HIDClass::new(&bus_allocator, GamepadReport::desc(), 10);
    // let mut usb_dev = UsbDeviceBuilder::new(&bus_allocator, UsbVidPid(0x1209, 0x0001))
    // .strings(&[StringDescriptors::default()
    //     .manufacturer("Nivo")
    //     .product("GameCube Adapter PC")])
    // .unwrap()
    // .device_class(0x00)
    // .build();

    let mut nintendo_class = WupClass::new(&bus_allocator);
    let mut usb_dev = UsbDeviceBuilder::new(&bus_allocator, UsbVidPid(0x057e, 0x0337))
        .strings(&[StringDescriptors::default()
            .manufacturer("Nintendo")
            .product("WUP-028")])
        .unwrap()
        .device_class(0xFF)
        .build();

    let _gp0 = _pins.gpio0.into_pull_up_input();

    let mut report = WupReport::new();
    let timer = unsafe { &*rp2040_hal::pac::TIMER::PTR };
    let mut last_poll = timer.timerawl().read().bits();
    let mut timeout_counter = 0;

    loop {
        let now = timer.timerawl().read().bits();

        if now.wrapping_sub(last_poll) >= 8_000 {
            last_poll = now;

            if let Some(data) = joybus::poll_controller() {
                timeout_counter = 0;
                report.update_port_1(&data);
            } else {
                timeout_counter += 1;
                if timeout_counter > 10 {
                    report.disconnect_port_1();
                }
            }

            let _ = nintendo_class.write_report(&report);
        }

        if usb_dev.poll(&mut [&mut nintendo_class]) {
            nintendo_class.poll();
        }
    }
}
