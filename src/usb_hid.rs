use usbd_hid::descriptor::generator_prelude::*;

#[gen_hid_descriptor(
    (collection = APPLICATION, usage_page = GENERIC_DESKTOP, usage = GAMEPAD) = {
        (collection = PHYSICAL, usage = POINTER) = {
            (usage_page = GENERIC_DESKTOP, usage = 0x30) = {
                #[item_settings data, variable, absolute] x=input;
            };
            (usage_page = GENERIC_DESKTOP, usage = 0x31) = {
                #[item_settings data, variable, absolute] y=input;
            };
            (usage_page = GENERIC_DESKTOP, usage = 0x33) = {
                #[item_settings data, variable, absolute] rx=input;
            };
            (usage_page = GENERIC_DESKTOP, usage = 0x34) = {
                #[item_settings data, variable, absolute] ry=input;
            };
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
