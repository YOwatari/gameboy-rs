use bitflags::bitflags;

pub struct JoyPad {
    register: u8,
    input: KeyInput,
}

bitflags!(
    pub struct KeyInput: u8 {
        const DOWN   = 0b_1000_0000;
        const UP     = 0b_0100_0000;
        const LEFT   = 0b_0010_0000;
        const RIGHT  = 0b_0001_0000;
        const START  = 0b_0000_1000;
        const SELECT = 0b_0000_0100;
        const B      = 0b_0000_0010;
        const A      = 0b_0000_0001;
    }
);

impl JoyPad {
    pub fn new() -> JoyPad {
        JoyPad {
            register: 0xff,
            input: KeyInput::from_bits_truncate(0xff),
        }
    }

    pub fn key_down(&mut self, key: KeyInput) {
        self.input.set(key, false);
    }

    pub fn key_up(&mut self, key: KeyInput) {
        self.input.set(key, true);
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0xff00 => {
                match self.register & 0x30 {
                    // select button keys
                    0x10 => (self.register & 0xf0) | (self.input.bits & 0x0f),
                    // select direction keys
                    0x20 => (self.register & 0xf0) | ((self.input.bits >> 4) & 0x0f),
                    _ => 0xff,
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
