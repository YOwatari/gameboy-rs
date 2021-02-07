use bitflags::bitflags;

pub struct Serial {
    data: u8,
    control: Control,
}

bitflags!(
    struct Control: u8 {
        const START            = 0b_1000_0000;
        const FAST_CLOCK_SPEED = 0b_0000_0010;
        const INTERNAL_CLOCK   = 0b_0000_0001;
    }
);

impl Serial {
    pub fn new() -> Serial {
        Serial {
            data: 0,
            control: Control::empty(),
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0xff01 => self.data,
            0xff02 => self.control.bits,
            _ => unimplemented!("read: Serial I/O: {:04x}", addr),
        }
    }

    pub fn write_byte(&mut self, addr: u16, v: u8) {
        match addr {
            0xff01 => self.data = v,
            0xff02 => self.control = Control::from_bits_truncate(v),
            _ => unimplemented!("write: Serial I/O: {:04x} {:02x}", addr, v),
        }
    }
}
