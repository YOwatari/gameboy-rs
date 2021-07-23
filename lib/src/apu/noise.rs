use std::fmt;

pub struct Noise {
    // NR44
    pub status: bool,
    use_length: bool,

    // NR43
    polynomial_frequency: u8,
    polynomial_width: bool,
    polynomial_ratio: u8,

    // NR42
    volume: u8,
    increase: bool,
    length: u8,

    // NR41
    sound_length: u8,
}

impl Noise {
    pub fn new() -> Noise {
        Noise {
            status: false,
            use_length: false,
            volume: 0,
            increase: false,
            length: 0,
            sound_length: 0,
            polynomial_frequency: 0,
            polynomial_width: false,
            polynomial_ratio: 0,
        }
    }

    pub fn read_nr_x1(&self) -> u8 {
        self.sound_length
    }

    pub fn write_nr_x1(&mut self, v: u8) {
        self.sound_length = v & 0x3f;
    }

    pub fn read_nr_x2(&self) -> u8 {
        (self.volume << 4) | if self.increase { 0x08 } else { 0 } | self.length
    }

    pub fn write_nr_x2(&mut self, v: u8) {
        self.volume = (v & 0xf0) >> 4;
        self.increase = v & 0x08 != 0;
        self.length = v & 0x07;
    }

    pub fn read_nr_x3(&self) -> u8 {
        (self.polynomial_frequency << 4)
            | if self.polynomial_width { 0x08 } else { 0 }
            | self.polynomial_ratio
    }

    pub fn write_nr_x3(&mut self, v: u8) {
        self.polynomial_frequency = (v & 0xf0) >> 4;
        self.polynomial_width = v & 0x08 != 0;
        self.polynomial_ratio = v & 0x07;
    }

    pub fn read_nr_x4(&self) -> u8 {
        0b_1011_1111 | if self.use_length { 0x40 } else { 0 }
    }

    pub fn write_nr_x4(&mut self, v: u8) {
        self.status = v & 0x80 != 0;
        self.use_length = v & 0x40 != 0;
    }
}

impl fmt::Debug for Noise {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Noise: {{ 1: {:02x}, 2: {:02x}, 3: {:02x} }}",
            self.sound_length, 0, 0
        )
    }
}
