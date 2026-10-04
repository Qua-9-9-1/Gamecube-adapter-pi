use usb_device::class_prelude::*;
use usb_device::Result;

const WUP_REPORT_SIZE: usize = 37;
const HID_REPORT_DESCRIPTOR_SIZE: u16 = 28;
const CONNECTED_PORT: u8 = 0x10;

#[repr(C, packed)]
pub struct WupReport {
    pub instruction: u8,
    pub ports: [[u8; 9]; 4],
}

impl WupReport {
    pub fn new() -> Self {
        Self {
            instruction: 0x21,
            ports: [[0x00, 0x00, 0x00, 128, 128, 128, 128, 0x00, 0x00]; 4],
        }
    }

    pub fn update_port(&mut self, port_idx: usize, data: &crate::joybus::ControllerData) {
        let port = &mut self.ports[port_idx];
        port[0] = CONNECTED_PORT;

        let mut wup_b1 = data.buttons_1 & 0x0F;
        wup_b1 |= (data.buttons_2 & 0x0F) << 4;

        let mut wup_b2 = 0;
        if (data.buttons_1 & 0x10) != 0 {
            wup_b2 |= 0x01;
        }
        if (data.buttons_2 & 0x10) != 0 {
            wup_b2 |= 0x02;
        }
        if (data.buttons_2 & 0x20) != 0 {
            wup_b2 |= 0x04;
        }
        if (data.buttons_2 & 0x40) != 0 {
            wup_b2 |= 0x08;
        }

        port[1] = wup_b1;
        port[2] = wup_b2;
        port[3] = data.stick_x;
        port[4] = data.stick_y;
        port[5] = data.c_stick_x;
        port[6] = data.c_stick_y;
        port[7] = data.l_analog;
        port[8] = data.r_analog;
    }

    pub fn disconnect_port(&mut self, port_idx: usize) {
        let port = &mut self.ports[port_idx];
        port[0] = 0x00;
        port[1] = 0x00;
        port[2] = 0x00;
        port[3] = 128;
        port[4] = 128;
        port[5] = 128;
        port[6] = 128;
        port[7] = 0x00;
        port[8] = 0x00;
    }
}

pub struct WupClass<'a, B: UsbBus> {
    iface: InterfaceNumber,
    ep_in: EndpointIn<'a, B>,
    _dummy_out: EndpointOut<'a, B>,
    ep_out: EndpointOut<'a, B>,
}

impl<'a, B: UsbBus> WupClass<'a, B> {
    pub fn new(alloc: &'a UsbBusAllocator<B>) -> Self {
        Self {
            iface: alloc.interface(),
            ep_in: alloc.interrupt(37, 8),
            _dummy_out: alloc.interrupt(1, 8),
            ep_out: alloc.interrupt(5, 8),
        }
    }

    pub fn write_report(&mut self, report: &WupReport) -> Result<usize> {
        let bytes = unsafe {
            core::slice::from_raw_parts(report as *const _ as *const u8, WUP_REPORT_SIZE)
        };
        self.ep_in.write(bytes)
    }
}

static HID_REPORT_DESCRIPTOR: [u8; HID_REPORT_DESCRIPTOR_SIZE as usize] = [
    0x05, 0x01, 0x09, 0x05, 0xA1, 0x01, 0x09, 0x01, 0xA1, 0x00, 0x05, 0x09, 0x19, 0x01, 0x29, 0x08,
    0x15, 0x00, 0x25, 0x01, 0x95, 0x08, 0x75, 0x01, 0x81, 0x02, 0xC0, 0xC0,
];

impl<B: UsbBus> UsbClass<B> for WupClass<'_, B> {
    fn get_configuration_descriptors(&self, writer: &mut DescriptorWriter) -> Result<()> {
        writer.interface(self.iface, 3, 0, 0)?;

        let hid_descriptor = [
            0x10,
            0x01,
            0x00,
            0x01,
            0x22,
            HID_REPORT_DESCRIPTOR_SIZE as u8,
            (HID_REPORT_DESCRIPTOR_SIZE >> 8) as u8,
        ];
        writer.write(0x21, &hid_descriptor)?;

        writer.endpoint(&self.ep_in)?;
        writer.endpoint(&self.ep_out)?;
        Ok(())
    }

    fn control_in(&mut self, xfer: ControlIn<B>) {
        let req = xfer.request();

        if req.request_type == usb_device::control::RequestType::Standard
            && req.request == usb_device::control::Request::GET_DESCRIPTOR
        {
            let desc_type = (req.value >> 8) as u8;
            if desc_type == 0x22 {
                let _ = xfer.accept_with_static(&HID_REPORT_DESCRIPTOR);
                return;
            }
        }

        if req.request_type == usb_device::control::RequestType::Class && req.request == 11 {
            let _ = xfer.accept_with(&[]);
        }
    }

    fn control_out(&mut self, xfer: ControlOut<B>) {
        let req = xfer.request();
        if req.request_type == usb_device::control::RequestType::Class && req.request == 11 {
            let _ = xfer.accept();
        }
    }
}
