use std::fmt;

pub struct Square {
    // NR13-14/NR23-24
    pub status: bool,
    use_length: bool,
    frequency: u16,

    // NR12/NR22
    volume: u8,
    increase: bool,
    length: u8,

    // NR11/NR21
    wave_duty: u8,
    sound_length: u8,

    // NR0
    sweep_time: u8,
    sweep_increase: bool,
    sweep_shift: u8,
}

impl Square {
    pub fn new() -> Square {
        Square {
            status: false,
            use_length: false,
            frequency: 0,
            volume: 0,
            increase: false,
            length: 0,
            wave_duty: 0,
            sound_length: 0,
            sweep_time: 0,
            sweep_increase: false,
            sweep_shift: 0,
        }
    }

    pub fn read_nr_x0(&self) -> u8 {
        (self.sweep_time << 4) | if self.sweep_increase { 0 } else { 0x08 } | self.sweep_shift
    }

    pub fn write_nr_x0(&mut self, v: u8) {
        self.sweep_time = (v & 0x70) >> 4;
        self.sweep_increase = v & 0x08 == 0;
        self.sweep_shift = v & 0x07;
    }

    pub fn read_nr_x1(&self) -> u8 {
        (self.wave_duty << 6) | 0x3f
    }

    pub fn write_nr_x1(&mut self, v: u8) {
        self.wave_duty = (v & 0xc0) >> 6;
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

    pub fn write_nr_x3(&mut self, v: u8) {
        self.frequency = (self.frequency & 0x0700) | (v as u16);
    }

    pub fn read_nr_x4(&self) -> u8 {
        0b_1011_1111 | if self.use_length { 0x40 } else { 0 }
    }

    pub fn write_nr_x4(&mut self, v: u8) {
        self.status = v & 0x80 != 0;
        self.use_length = v & 0x40 != 0;
        self.frequency = (self.frequency & 0x00ff) | ((v as u16) << 8);
    }
}

impl fmt::Debug for Square {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Square {{ 0: {:02x}, 1: {:02x}, 2: {:02x}, 4: {:02x} }}",
            (self.sweep_time << 4) | if self.sweep_increase { 0 } else { 0x08 } | self.sweep_shift,
            (self.wave_duty << 6) | 0x3f,
            (self.volume << 4) | if self.increase { 0x08 } else { 0 } | self.length,
            0b_1011_1111 | if self.use_length { 0x40 } else { 0 }
        )
    }
}
