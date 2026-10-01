use rp2040_hal::pac;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ControllerData {
    pub buttons_1: u8,
    pub buttons_2: u8,
    pub stick_x: u8,
    pub stick_y: u8,
    pub c_stick_x: u8,
    pub c_stick_y: u8,
    pub analog_l: u8,
    pub analog_r: u8,
}

#[allow(dead_code)]
impl ControllerData {
    pub fn button_a(&self) -> bool { (self.buttons_1 & (1 << 0)) != 0 }
    pub fn button_b(&self) -> bool { (self.buttons_1 & (1 << 1)) != 0 }
    pub fn button_x(&self) -> bool { (self.buttons_1 & (1 << 2)) != 0 }
    pub fn button_y(&self) -> bool { (self.buttons_1 & (1 << 3)) != 0 }
    pub fn button_start(&self) -> bool { (self.buttons_1 & (1 << 4)) != 0 }

    pub fn dpad_left(&self) -> bool { (self.buttons_2 & (1 << 0)) != 0 }
    pub fn dpad_right(&self) -> bool { (self.buttons_2 & (1 << 1)) != 0 }
    pub fn dpad_down(&self) -> bool { (self.buttons_2 & (1 << 2)) != 0 }
    pub fn dpad_up(&self) -> bool { (self.buttons_2 & (1 << 3)) != 0 }
    pub fn button_z(&self) -> bool { (self.buttons_2 & (1 << 4)) != 0 }
    pub fn button_r(&self) -> bool { (self.buttons_2 & (1 << 5)) != 0 }
    pub fn button_l(&self) -> bool { (self.buttons_2 & (1 << 6)) != 0 }
}

pub struct Joybus<const PIN: usize>;

impl<const PIN: usize> Joybus<PIN> {
    const PIN_MASK: u32 = 1 << PIN;

    pub const fn new() -> Self {
        Self
    }

    #[link_section = ".data"]
    #[inline(always)]
    fn wait_us(timer: &pac::timer::RegisterBlock, us: u32) {
        let start = timer.timerawl().read().bits();
        while timer.timerawl().read().bits().wrapping_sub(start) < us {}
    }

    #[inline(always)]
    fn pull_low(&self, sio: &pac::sio::RegisterBlock) {
        unsafe {
            sio.gpio_out_clr().write(|w| w.bits(Self::PIN_MASK));
            sio.gpio_oe_set().write(|w| w.bits(Self::PIN_MASK));
        }
    }

    #[inline(always)]
    fn float_high(&self, sio: &pac::sio::RegisterBlock) {
        unsafe {
            sio.gpio_oe_clr().write(|w| w.bits(Self::PIN_MASK));
        }
    }

    #[inline(always)]
    fn is_high(&self, sio: &pac::sio::RegisterBlock) -> bool {
        (sio.gpio_in().read().bits() & Self::PIN_MASK) != 0
    }

    #[link_section = ".data"]
    #[inline(never)]
    fn send_bit(&self, sio: &pac::sio::RegisterBlock, timer: &pac::timer::RegisterBlock, bit: bool) {
        self.pull_low(sio);
        if bit {
            Self::wait_us(timer, 1);
            self.float_high(sio);
            Self::wait_us(timer, 3);
        } else {
            Self::wait_us(timer, 3);
            self.float_high(sio);
            Self::wait_us(timer, 1);
        }
    }

    #[link_section = ".data"]
    #[inline(never)]
    fn send_byte(&self, sio: &pac::sio::RegisterBlock, timer: &pac::timer::RegisterBlock, mut byte: u8) {
        for _ in 0..8 {
            self.send_bit(sio, timer, (byte & 0x80) != 0);
            byte <<= 1;
        }
    }


    #[link_section = ".data"]
    pub fn poll(&self) -> Option<ControllerData> {
        cortex_m::interrupt::free(|_| {
            let sio = unsafe { &*pac::SIO::PTR };
            let timer = unsafe { &*pac::TIMER::PTR };

            self.send_byte(sio, timer, 0x40);
            self.send_byte(sio, timer, 0x03);
            self.send_byte(sio, timer, 0x00);
            self.send_bit(sio, timer, true);

            self.float_high(sio);

            let mut bytes = [0u8; 8];

            for byte in bytes.iter_mut() {
                let mut current_byte = 0u8;
                for _ in 0..8 {
                    let start = timer.timerawl().read().bits();
                    while self.is_high(sio) {
                        if timer.timerawl().read().bits().wrapping_sub(start) > 50 {
                            return None;
                        }
                    }

                    Self::wait_us(timer, 2);
                    let bit_value = if self.is_high(sio) { 1 } else { 0 };
                    current_byte = (current_byte << 1) | bit_value;

                    let start_return = timer.timerawl().read().bits();
                    while !self.is_high(sio) {
                        if timer.timerawl().read().bits().wrapping_sub(start_return) > 15 {
                            return None;
                        }
                    }
                }
                *byte = current_byte;
            }

            Some(ControllerData {
                buttons_1: bytes[0],
                buttons_2: bytes[1],
                stick_x: bytes[2],
                stick_y: bytes[3],
                c_stick_x: bytes[4],
                c_stick_y: bytes[5],
                analog_l: bytes[6],
                analog_r: bytes[7],
            })
        })
    }
}
