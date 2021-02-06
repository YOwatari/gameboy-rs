use std::fmt;

pub struct Noise {
    pub status: bool,
    length: u8,
}

impl Noise {
    pub fn new() -> Noise {
        Noise {
            status: false,
            length: 0,
        }
    }

    pub fn read_nr_x1(&self) -> u8 {
        self.length
    }

    pub fn write_nr_x1(&mut self, v: u8) {
        self.length = v & 0x3f;
    }
}

impl fmt::Debug for Noise {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Noise: {{ 1: {:02x}, 2: {:02x}, 3: {:02x} }}",
            self.length, 0, 0
        )
    }
}
