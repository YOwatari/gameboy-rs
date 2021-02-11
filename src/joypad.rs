pub struct JoyPad {
    register: u8,
    input: u8,
}

impl JoyPad {
    pub fn new() -> JoyPad {
        JoyPad {
            register: 0xff,
            input: 0xff,
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0xff00 => {
                match self.register & 0x30 {
                    // select direction keys
                    0x10 => (self.register & 0xf0) | ((self.input & 0x0f) >> 4),
                    // select button keys
                    0x20 => (self.register & 0xf0) | (self.input & 0x0f),
                    _ => 0xff, // TODO unimplemented!("unknown key selection: {:02x}", self.register),
                }
            }
            _ => unimplemented!("read: JoyPad I/O {:04x}", addr),
        }
    }

    pub fn write_byte(&mut self, addr: u16, v: u8) {
        match addr {
            0xff00 => self.register = (self.register & 0xcf) | (v & 0x30),
            _ => unimplemented!("write: JoyPad I/O {:04x}", addr),
        }
    }
}
