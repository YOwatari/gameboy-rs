pub struct Timer {
    divider: u8,
    counter: u8,
    modulo: u8,
    control: u8,
    interrupt: u8,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            divider: 0,
            counter: 0,
            modulo: 0,
            control: 0,
            interrupt: 0,
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0xff06 => self.modulo,
            0xff07 => self.control,
            _ => unimplemented!("read: Timer I/O {:04x}", addr),
        }
    }

    pub fn write_byte(&mut self, addr: u16, v: u8) {
        match addr {
            0xff06 => self.modulo = v,
            0xff07 => self.control = v,
            _ => unimplemented!("write: Timer I/O {:04x} {:02x}", addr, v),
        };
    }
}
