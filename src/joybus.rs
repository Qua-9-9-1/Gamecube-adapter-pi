use rp2040_hal::pac;

pub struct ControllerData {
    pub buttons_1: u8,
    pub buttons_2: u8,
    pub stick_x: u8,
    pub stick_y: u8,
    pub c_stick_x: u8,
    pub c_stick_y: u8,
    pub l_analog: u8,
    pub r_analog: u8,
}

#[link_section = ".data"]
#[inline(always)]
fn wait_us(us: u32) {
    unsafe {
        let timer = &*pac::TIMER::PTR;
        let start = timer.timerawl().read().bits();
        while timer.timerawl().read().bits().wrapping_sub(start) < us {}
    }
}

macro_rules! pull_low {
    () => {
        unsafe {
            let sio = &*pac::SIO::PTR;
            sio.gpio_out_clr().write(|w| w.bits(1 << 0));
            sio.gpio_oe_set().write(|w| w.bits(1 << 0));
        }
    };
}

macro_rules! float_high {
    () => {
        unsafe {
            let sio = &*pac::SIO::PTR;
            sio.gpio_oe_clr().write(|w| w.bits(1 << 0));
        }
    };
}

#[link_section = ".data"]
#[inline(never)]
fn send_bit(bit: bool) {
    pull_low!();
    if bit {
        wait_us(1);
        float_high!();
        wait_us(3);
    } else {
        wait_us(3);
        float_high!();
        wait_us(1);
    }
}

#[link_section = ".data"]
#[inline(never)]
fn send_byte(mut byte: u8) {
    for _ in 0..8 {
        send_bit((byte & 0x80) != 0);
        byte <<= 1;
    }
}

#[link_section = ".data"]
pub fn poll_controller() -> Option<ControllerData> {
    cortex_m::interrupt::free(|_| {
        send_byte(0x40);
        send_byte(0x03);
        send_byte(0x00);
        send_bit(true);

        float_high!();

        let sio = unsafe { &*pac::SIO::PTR };

        let mut bytes = [0u8; 8];
        for byte_idx in 0..8 {
            let mut current_byte = 0u8;
            for _ in 0..8 {
                let mut timeout = 0;

                while (sio.gpio_in().read().bits() & (1 << 0)) != 0 {
                    timeout += 1;
                    if timeout > 100_000 {
                        return None;
                    }
                }

                wait_us(2);

                let is_low = (sio.gpio_in().read().bits() & (1 << 0)) == 0;
                let bit_value = if is_low { 0 } else { 1 };

                current_byte = (current_byte << 1) | bit_value;

                while (sio.gpio_in().read().bits() & (1 << 0)) == 0 {
                    timeout += 1;
                    if timeout > 100_000 {
                        return None;
                    }
                }
            }
            bytes[byte_idx] = current_byte;
        }

        if bytes[1] & 0x80 == 0 {
            return None;
        }

        Some(ControllerData {
            buttons_1: bytes[0],
            buttons_2: bytes[1],
            stick_x: bytes[2],
            stick_y: bytes[3],
            c_stick_x: bytes[4],
            c_stick_y: bytes[5],
            l_analog: bytes[6],
            r_analog: bytes[7],
        })
    })
}
