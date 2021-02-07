use bitflags::bitflags;
use std::fmt;

mod noise;
mod square;
mod wave;

use noise::Noise;
use square::Square;
use wave::Wave;

pub struct APU {
    channel1: Square,
    channel2: Square,
    channel3: Wave,
    channel4: Noise,
    left_volume: MasterVolume, // so2
    left_output: SelectChannels,
    right_volume: MasterVolume, // so1
    right_output: SelectChannels,
    enable: bool,
}

struct MasterVolume {
    vin: bool,
    level: u8,
}

impl MasterVolume {
    pub fn new() -> MasterVolume {
        MasterVolume {
            vin: false,
            level: 0,
        }
    }
}

bitflags!(
    struct SelectChannels: u8 {
        const CHANNEL1 = 0b_0001;
        const CHANNEL2 = 0b_0010;
        const CHANNEL3 = 0b_0100;
        const CHANNEL4 = 0b_1000;
    }
);

impl APU {
    pub fn new() -> APU {
        APU {
            channel1: Square::new(),
            channel2: Square::new(),
            channel3: Wave::new(),
            channel4: Noise::new(),
            left_volume: MasterVolume::new(),
            left_output: SelectChannels::empty(),
            right_volume: MasterVolume::new(),
            right_output: SelectChannels::empty(),
            enable: false,
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0xff10 => self.channel1.read_nr_x0(),
            0xff11 => self.channel1.read_nr_x1(),
            0xff12 => self.channel1.read_nr_x2(),
            0xff13 => 0xff, // write only
            0xff14 => self.channel1.read_nr_x4(),

            0xff16 => self.channel2.read_nr_x1(),
            0xff17 => self.channel2.read_nr_x2(),
            0xff18 => 0xff, // write only
            0xff19 => self.channel2.read_nr_x4(),

            0xff1a => self.channel3.read_nr_x0(),
            0xff1b => self.channel3.read_nr_x1(),

            0xff20 => self.channel4.read_nr_x1(),
            0xff21 => self.channel4.read_nr_x2(),
            0xff22 => self.channel4.read_nr_x3(),
            0xff23 => self.channel4.read_nr_x4(),

            0xff24 => {
                (if self.left_volume.vin { 0x80 } else { 0x00 })
                    | self.left_volume.level << 4
                    | if self.right_volume.vin { 0x08 } else { 0x00 }
                    | self.right_volume.level
            }
            0xff25 => (self.left_output.bits << 4) | self.right_output.bits,
            0xff26 => {
                (if self.enable { 0x80 } else { 0 })
                    | (if self.channel4.status { 0b_1000 } else { 0 })
                    | (if self.channel3.status { 0b_0100 } else { 0 })
                    | (if self.channel2.status { 0b_0010 } else { 0 })
                    | (if self.channel1.status { 0b_0001 } else { 0 })
            }
            _ => unimplemented!("read: Sound I/O {:04x}", addr),
        }
    }

    pub fn write_byte(&mut self, addr: u16, v: u8) {
        match addr {
            0xff10 => self.channel1.write_nr_x0(v),
            0xff11 => self.channel1.write_nr_x1(v),
            0xff12 => self.channel1.write_nr_x2(v),
            0xff13 => self.channel1.write_nr_x3(v),
            0xff14 => self.channel1.write_nr_x4(v),

            0xff16 => self.channel2.write_nr_x1(v),
            0xff17 => self.channel2.write_nr_x2(v),
            0xff18 => self.channel2.write_nr_x3(v),
            0xff19 => self.channel2.write_nr_x4(v),

            0xff1a => self.channel3.write_nr_x0(v),
            0xff1b => self.channel3.write_nr_x1(v),

            0xff20 => self.channel4.write_nr_x1(v),
            0xff21 => self.channel4.write_nr_x2(v),
            0xff22 => self.channel4.write_nr_x3(v),
            0xff23 => self.channel4.write_nr_x4(v),

            0xff24 => {
                self.left_volume.vin = v & 0x80 != 0;
                self.left_volume.level = (v & 0x70) >> 4;
                self.right_volume.vin = v & 0x08 != 0;
                self.right_volume.level = v & 0x07;
            }
            0xff25 => {
                self.left_output = SelectChannels::from_bits_truncate(v >> 4);
                self.right_output = SelectChannels::from_bits_truncate(v);
            }
            0xff26 => {
                self.enable = v & 0x80 != 0;
                if !self.enable {
                    self.channel4.status = v & 0b_1000 != 0;
                    self.channel3.status = v & 0b_0100 != 0;
                    self.channel2.status = v & 0b_0010 != 0;
                    self.channel1.status = v & 0b_0001 != 0;
                    self.right_output = SelectChannels::empty();
                    self.left_output = SelectChannels::empty();
                }
            }
            _ => unimplemented!("write: Sound I/O {:04x} {:02x}", addr, v),
        }
    }
}

impl fmt::Debug for APU {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "APU {{ nr1: {:?}, nr2: {:?}, nr3: {:?}, nr4: {:?} }}",
            self.channel1, self.channel2, self.channel3, self.channel4,
        )
    }
}
