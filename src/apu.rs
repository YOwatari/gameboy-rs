use bitflags::bitflags;

pub struct APU {
    on: bool,
    channel1: bool,
    channel2: bool,
    channel3: bool,
    channel4: bool,
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
            channel2: false,
            channel3: false,
            channel4: false,
        }
    }

    pub fn read_byte(&mut self, addr: u16) -> u8 {
        match addr {
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
            0xff26 => {
                let f = SoundOnOff::from_bits(v).unwrap();
                self.on = f.contains(SoundOnOff::All);
            }
            _ => unimplemented!("write: Sound I/O {:04x} {:02x}", addr, v),
        }
    }
}
