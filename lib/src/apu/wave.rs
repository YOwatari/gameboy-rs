use std::fmt;

pub struct Wave {
    pub status: bool,
    length: u8,
}

impl Wave {
    pub fn new() -> Wave {
        Wave {
            status: false,
            length: 0,
        }
    }

    pub fn read_nr_x0(&self) -> u8 {
        if self.status {
            0x80
        } else {
            0x00
        }
    }

    pub fn write_nr_x0(&mut self, v: u8) {
        self.status = v & 0x80 != 0;
    }

    pub fn read_nr_x1(&self) -> u8 {
        self.length
    }

    pub fn write_nr_x1(&mut self, v: u8) {
        self.length = v;
    }
}

impl fmt::Debug for Wave {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Wave: {{ 0: {:02x}, 1: {:02x}, 2: {:02x}, 3: {:02x} }}",
            if self.status { 0x80 } else { 0 },
            self.length,
            0,
            0
        )
    }
}
