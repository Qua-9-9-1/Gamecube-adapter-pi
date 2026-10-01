use usbd_hid::descriptor::generator_prelude::*;

#[gen_hid_descriptor(
    (collection = APPLICATION, usage_page = GENERIC_DESKTOP, usage = GAMEPAD) = {
        (usage_page = GENERIC_DESKTOP, usage_min = 0x30, usage_max = 0x33) = {
            #[item_settings data, variable, absolute] axes=input;
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
    pub axes: [u8; 4],
    pub buttons_1: u8,
    pub buttons_2: u8,
}
