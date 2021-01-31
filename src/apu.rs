use bitflags::bitflags;

const CHANNEL_MAX_LENGTH: usize = 64;
const CHANNEL3_MAX_LENGTH: usize = 256;

pub struct APU {
    enable: bool,
    channel1_enable: bool,
    channel1_duty: u8,
    channel1_new_length: u8,
    channel1_init_volume: u8,
    channel1_envelope_direction: bool,
    channel1_envelope_sweep: u8,
    channel1_frequency: u16,
    channel1_length: u8,
    channel1_length_enable: bool,
    channel2_enable: bool,
    channel2_duty: u8,
    channel2_new_length: u8,
    channel2_frequency: u16,
    channel2_length: u8,
    channel2_length_enable: bool,
    channel3_enable: bool,
    channel3_length: u8,
    channel4_enable: bool,
    channel4_length: u8,
    so1_terminal_channels: Channels,
    so2_terminal_channels: Channels,
    so1_vin: bool,
    so2_vin: bool,
    so1_volume: u8,
    so2_volume: u8,
}

bitflags!(
    struct Channels: u8 {
        const CHANNEL1 = 1 << 0;
        const CHANNEL2 = 1 << 1;
        const CHANNEL3 = 1 << 2;
        const CHANNEL4 = 1 << 3;
    }
);

impl APU {
    pub fn new() -> APU {
        APU {
            enable: false,
            channel1_enable: false,
            channel1_duty: 0,
            channel1_new_length: 0,
            channel1_init_volume: 0,
            channel1_envelope_direction: false,
            channel1_envelope_sweep: 0,
            channel1_frequency: 0,
            channel1_length: 0,
            channel1_length_enable: false,
            channel2_enable: false,
            channel2_duty: 0,
            channel2_new_length: 0,
            channel2_frequency: 0,
            channel2_length: 0,
            channel2_length_enable: false,
            channel3_enable: false,
            channel3_length: 0,
            channel4_enable: false,
            channel4_length: 0,
            so1_terminal_channels: Channels::empty(),
            so2_terminal_channels: Channels::empty(),
            so1_vin: false,
            so2_vin: false,
            so1_volume: 0,
            so2_volume: 0,
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0xff11 => {
                (self.channel1_duty << 6)
                    | (((CHANNEL_MAX_LENGTH - 1) as u8 - self.channel1_new_length) & 0b_0011_1111)
            }
            0xff12 => {
                (self.channel1_init_volume << 4)
                    | (if self.channel1_envelope_direction {
                        0b_0000_1000
                    } else {
                        0
                    })
                    | (self.channel1_envelope_sweep & 0b_0000_0111)
            }
            0xff13 => 0xff, // write only
            0xff14 => {
                if self.channel1_length_enable {
                    1 << 6
                } else {
                    0
                }
            }
            0xff16 => {
                (self.channel2_duty << 6)
                    | (((CHANNEL_MAX_LENGTH - 1) as u8 - self.channel2_new_length) & 0b_0011_1111)
            }
            0xff18 => 0xff, // write only
            0xff19 => {
                if self.channel2_length_enable {
                    1 << 6
                } else {
                    0
                }
            }
            0xff1b => (CHANNEL3_MAX_LENGTH - 1) as u8 - self.channel3_length,
            0xff20 => ((CHANNEL_MAX_LENGTH - 1) as u8 - self.channel4_length) & 0b_0011_1111,
            0xff24 => {
                (if self.so2_vin { 1 << 7 } else { 0 })
                    | self.so2_volume << 4
                    | (if self.so1_vin { 1 << 3 } else { 0 })
                    | self.so1_volume
            }
            0xff25 => self.so1_terminal_channels.bits | (self.so2_terminal_channels.bits << 4),
            0xff26 => {
                (if self.enable { 1 << 7 } else { 0 })
                    | (if self.channel4_enable { 1 << 3 } else { 0 })
                    | (if self.channel3_enable { 1 << 2 } else { 0 })
                    | (if self.channel2_enable { 1 << 1 } else { 0 })
                    | (if self.channel1_enable { 1 << 0 } else { 0 })
            }
            _ => unimplemented!("read: Sound I/O {:04x}", addr),
        }
    }

    pub fn write_byte(&mut self, addr: u16, v: u8) {
        match addr {
            0xff11 => {
                self.channel1_duty = v >> 6;
                self.channel1_new_length = (CHANNEL_MAX_LENGTH - 1) as u8 - (v & 0b_0011_1111);
            }
            0xff12 => {
                self.channel1_init_volume = v >> 4;
                self.channel1_envelope_direction = (v & 0b_0000_1000) == 0b_0000_1000;
                self.channel1_envelope_sweep = v & 0b_0000_0111;
            }
            0xff13 => {
                self.channel1_frequency = (self.channel1_frequency & 0b_0111_0000_0000) | v as u16;
                self.channel1_length = self.channel1_new_length;
                // period
            }
            0xff14 => {
                self.channel1_frequency = (self.channel1_frequency & 0b_0000_1111_1111)
                    | (((v & 0b_0000_0111) as u16) << 8);
                // period
                self.channel1_length_enable = (v & 0b_0100_0000) != 0;

                if v & 0b_1000_0000 != 0 {
                    self.enable = true;
                    self.channel1_length = self.channel1_new_length;
                    // sweep
                }
            }
            0xff16 => {
                self.channel2_duty = v >> 6;
                self.channel2_new_length = (CHANNEL_MAX_LENGTH - 1) as u8 - (v & 0b_0011_1111);
            }
            0xff18 => {
                self.channel2_frequency = (self.channel2_frequency & 0b_0111_0000_0000) | v as u16;
            }
            0xff19 => {
                self.channel2_frequency = (self.channel2_frequency & 0b_0000_1111_1111)
                    | (((v & 0b_0000_0111) as u16) << 8);
                // period
                self.channel2_length_enable = (v & 0b_0100_0000) != 0;

                if v & 0b_1000_0000 != 0 {
                    self.enable = true;
                    self.channel2_length = self.channel2_new_length;
                    // sweep
                }
            }
            0xff1b => self.channel3_length = (CHANNEL3_MAX_LENGTH - 1) as u8 - v,
            0xff20 => self.channel4_length = (CHANNEL_MAX_LENGTH - 1) as u8 - (v & 0b_0011_1111),
            0xff24 => {
                if self.enable {
                    self.so1_volume = v & 0b_0000_0111;
                    self.so1_vin = v & 0b_0000_1000 != 0;
                    self.so2_volume = (v & 0b_0111_0000) >> 4;
                    self.so2_vin = v & 0b_1000_0000 != 0;
                }
            }
            0xff25 => {
                if self.enable {
                    self.so1_terminal_channels = Channels::from_bits_truncate(v);
                    self.so2_terminal_channels = Channels::from_bits_truncate(v >> 4);
                }
            }
            0xff26 => {
                self.enable = v & (1 << 7) != 0;
                if !self.enable {
                    self.channel1_enable = false;
                    self.channel2_enable = false;
                    self.channel3_enable = false;
                    self.channel4_enable = false;
                    self.so1_terminal_channels = Channels::empty();
                    self.so2_terminal_channels = Channels::empty();
                }
            }
            _ => unimplemented!("write: Sound I/O {:04x} {:02x}", addr, v),
        }
    }
}
