use bitflags::bitflags;

const CHANNEL_MAX_LENGTH: usize = 64;
const CHANNEL3_MAX_LENGTH: usize = 256;

pub struct APU {
    on: bool,
    channel1: bool,
    channel1_duty: u8,
    channel1_length: u8,
    channel2: bool,
    channel2_duty: u8,
    channel2_length: u8,
    channel3: bool,
    channel3_length: u8,
    channel4: bool,
    channel4_length: u8,
}

bitflags!(
    pub struct SoundOnOff: u8 {
        const All      = 0b_1000_0000;
        const Channel4 = 0b_0000_1000;
        const Channel3 = 0b_0000_0100;
        const Channel2 = 0b_0000_0010;
        const Channel1 = 0b_0000_0001;
    }
);

impl APU {
    pub fn new() -> APU {
        APU {
            on: false,
            channel1: false,
            channel1_duty: 0,
            channel1_length: 0,
            channel2: false,
            channel2_duty: 0,
            channel2_length: 0,
            channel3: false,
            channel3_length: 0,
            channel4: false,
            channel4_length: 0,
        }
    }

    pub fn read_byte(&mut self, addr: u16) -> u8 {
        match addr {
            0xff11 => {
                self.channel1_duty << 6
                    | (((CHANNEL_MAX_LENGTH - 1) as u8 - self.channel1_length) & 0b_0011_1111)
            }
            0xff16 => {
                self.channel2_duty << 6
                    | (((CHANNEL_MAX_LENGTH - 1) as u8 - self.channel2_length) & 0b_0011_1111)
            }
            0xff1b => (CHANNEL3_MAX_LENGTH - 1) as u8 - self.channel3_length,
            0xff20 => ((CHANNEL_MAX_LENGTH - 1) as u8 - self.channel4_length) & 0b_0011_1111,
            0xff26 => {
                let mut f = SoundOnOff::empty();
                f.set(SoundOnOff::All, self.on);
                f.set(SoundOnOff::Channel4, self.channel4);
                f.set(SoundOnOff::Channel3, self.channel3);
                f.set(SoundOnOff::Channel2, self.channel2);
                f.set(SoundOnOff::Channel1, self.channel1);
                f.bits()
            }
            _ => unimplemented!("read: Sound I/O {:04x}", addr),
        }
    }

    pub fn write_byte(&mut self, addr: u16, v: u8) {
        match addr {
            0xff11 => {
                self.channel1_duty = v >> 6;
                self.channel1_length = (CHANNEL_MAX_LENGTH - 1) as u8 - (v & 0b_0011_1111);
            }
            0xff16 => {
                self.channel2_duty = v >> 6;
                self.channel2_length = (CHANNEL_MAX_LENGTH - 1) as u8 - (v & 0b_0011_1111);
            }
            0xff1b => self.channel3_length = (CHANNEL3_MAX_LENGTH - 1) as u8 - v,
            0xff20 => self.channel4_length = (CHANNEL_MAX_LENGTH - 1) as u8 - (v & 0b_0011_1111),
            0xff26 => {
                let f = SoundOnOff::from_bits(v).unwrap();
                self.on = f.contains(SoundOnOff::All);
            }
            _ => unimplemented!("write: Sound I/O {:04x} {:02x}", addr, v),
        }
    }
}
