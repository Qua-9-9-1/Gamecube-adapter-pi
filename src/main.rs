#![no_std]
#![no_main]

mod joybus;
mod usb_hid;

use hal::pac;
use panic_halt as _;
use rp2040_hal as hal;

use joybus::Joybus;
use usb_device::class_prelude::*;
use usb_device::prelude::*;
use usb_hid::GamepadReport;
use usbd_hid::descriptor::SerializedDescriptor;
use usbd_hid::hid_class::HIDClass;

#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;


const USB_POLL_INTERVAL_US: u32 = 10_000;

const JOYBUS_GPIO_PIN: usize = 0;

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


    let _joybus_pin = pins.gpio0.into_pull_up_input();

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

    let joybus = Joybus::<JOYBUS_GPIO_PIN>::new();
    let mut report = GamepadReport::neutral();

    let timer = unsafe { &*pac::TIMER::PTR };
    let mut last_poll = timer.timerawl().read().bits();

    loop {

        usb_dev.poll(&mut [&mut hid]);

        let now = timer.timerawl().read().bits();
        if now.wrapping_sub(last_poll) >= USB_POLL_INTERVAL_US {
            last_poll = now;

            if let Some(data) = joybus.poll() {
                report.update_from_controller(&data);
            }

            if usb_dev.state() == UsbDeviceState::Configured {
                let _ = hid.push_input(&report);
            }
        }
    }
}
