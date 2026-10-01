use usbd_hid::descriptor::generator_prelude::*;
use crate::joybus::ControllerData;

#[gen_hid_descriptor(
    (collection = APPLICATION, usage_page = GENERIC_DESKTOP, usage = GAMEPAD) = {
        (usage_page = GENERIC_DESKTOP, usage = X) = {
            #[item_settings data, variable, absolute] x=input;
        };
        (usage_page = GENERIC_DESKTOP, usage = Y) = {
            #[item_settings data, variable, absolute] y=input;
        };
        (usage_page = GENERIC_DESKTOP, usage = 0x33) = {
            #[item_settings data, variable, absolute] rx=input;
        };
        (usage_page = GENERIC_DESKTOP, usage = 0x34) = {
            #[item_settings data, variable, absolute] ry=input;
        };
        (usage_page = BUTTON, usage_min = 1, usage_max = 8) = {
            #[packed_bits 8] #[item_settings data, variable, absolute] buttons_1=input;
        };
        (usage_page = BUTTON, usage_min = 9, usage_max = 16) = {
            #[packed_bits 8] #[item_settings data, variable, absolute] buttons_2=input;
        };
    }
)]
pub struct GamepadReport {
    pub x: u8,
    pub y: u8,
    pub rx: u8,
    pub ry: u8,
    pub buttons_1: u8,
    pub buttons_2: u8,
}

impl GamepadReport {
    pub const fn neutral() -> Self {
        Self {
            x: 128,
            y: 128,
            rx: 128,
            ry: 128,
            buttons_1: 0,
            buttons_2: 0,
        }
    }

    pub fn update_from_controller(&mut self, data: &ControllerData) {
        self.buttons_1 = data.buttons_1;

        self.buttons_2 = data.buttons_2 & 0x7F;

        self.x = data.stick_x;
        self.y = 255_u8.saturating_sub(data.stick_y);

        self.rx = data.c_stick_x;
        self.ry = 255_u8.saturating_sub(data.c_stick_y);
    }
}
