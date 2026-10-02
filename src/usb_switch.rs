#[repr(C, packed)]
pub struct WupReport {
    pub instruction: u8,
    pub port_1: [u8; 9],
    pub port_2: [u8; 9],
    pub port_3: [u8; 9],
    pub port_4: [u8; 9],
}

impl WupReport {
    pub fn new() -> Self {
        Self {
            instruction: 0x21,
            port_1: [0x00, 0x00, 0x00, 128, 128, 128, 128, 0x00, 0x00],
            port_2: [0x00, 0x00, 0x00, 128, 128, 128, 128, 0x00, 0x00],
            port_3: [0x00, 0x00, 0x00, 128, 128, 128, 128, 0x00, 0x00],
            port_4: [0x00, 0x00, 0x00, 128, 128, 128, 128, 0x00, 0x00],
        }
    }

pub fn update_port_1(&mut self, data: &crate::joybus::ControllerData) {
        self.port_1[0] = 0x10;
        let mut wup_b1 = data.buttons_1 & 0x0F; 
        
        wup_b1 |= (data.buttons_2 & 0x0F) << 4; 
        
        
        let mut wup_b2 = 0;
        
        if (data.buttons_1 & 0x10) != 0 { wup_b2 |= 0x01; }
        if (data.buttons_2 & 0x10) != 0 { wup_b2 |= 0x02; }
        if (data.buttons_2 & 0x20) != 0 { wup_b2 |= 0x04; }
        if (data.buttons_2 & 0x40) != 0 { wup_b2 |= 0x08; }

        self.port_1[1] = wup_b1;
        self.port_1[2] = wup_b2;
        self.port_1[3] = data.stick_x;
        self.port_1[4] = data.stick_y;
        self.port_1[5] = data.c_stick_x;
        self.port_1[6] = data.c_stick_y;
        self.port_1[7] = data.l_analog;
        self.port_1[8] = data.r_analog;
    }

    pub fn disconnect_port_1(&mut self) {
        self.port_1[0] = 0x00;

        self.port_1[1] = 0x00;
        self.port_1[2] = 0x00;
        self.port_1[3] = 128;
        self.port_1[4] = 128;
        self.port_1[5] = 128;
        self.port_1[6] = 128;
        self.port_1[7] = 0x00;
        self.port_1[8] = 0x00;
    }
}

use usb_device::class_prelude::*;
use usb_device::Result;

pub struct WupClass<'a, B: UsbBus> {
    iface: InterfaceNumber,
    ep_in: EndpointIn<'a, B>,
    ep_out: EndpointOut<'a, B>,
}

impl<'a, B: UsbBus> WupClass<'a, B> {
    pub fn new(alloc: &'a UsbBusAllocator<B>) -> Self {
        Self {
            iface: alloc.interface(),
            ep_in: alloc.interrupt(37, 1), 
            ep_out: alloc.interrupt(5, 1),
        }
    }

    pub fn write_report(&mut self, report: &WupReport) -> Result<usize> {
        let bytes = unsafe {
            core::slice::from_raw_parts(report as *const _ as *const u8, 37)
        };
        self.ep_in.write(bytes)
    }

    pub fn poll(&mut self) {
        let mut buf = [0u8; 5];
        let _ = self.ep_out.read(&mut buf);
    }
}

impl<B: UsbBus> UsbClass<B> for WupClass<'_, B> {
    fn get_configuration_descriptors(&self, writer: &mut DescriptorWriter) -> Result<()> {
        writer.interface(self.iface, 0xFF, 0xFF, 0xFF)?;
        writer.endpoint(&self.ep_in)?;
        writer.endpoint(&self.ep_out)?;
        Ok(())
    }
}