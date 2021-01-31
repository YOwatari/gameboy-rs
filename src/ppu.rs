use bitflags::bitflags;
use std::fmt;

const VRAM_SIZE: usize = 8 * 1024;
const SCREEN_WIDTH: u8 = 160;
const SCREEN_HEIGHT: u8 = 144;

pub struct PPU {
    vram: [u8; VRAM_SIZE],
    mode: Mode,
    bgp: u8,
    clocks: u32,
    ly: u8,
    stat: Stat,
}

bitflags!(
    struct Stat: u8 {
        const LycInterrupt    = 0b_0100_0000;
        const OAMInterrupt    = 0b_0010_0000;
        const VBlankInterrupt = 0b_0001_0000;
        const HBlankInterrupt = 0b_0000_1000;
        const LycFlag         = 0b_0000_0100;
        const HBlankMode      = 0b_0000_0000;
        const VBlankMode      = 0b_0000_0001;
        const AccessOAMMode   = 0b_0000_0010;
        const AccessVRAMMode  = 0b_0000_0011;
    }
);

#[derive(Eq, PartialEq)]
enum Mode {
    HBlank,
    VBlank,
    AccessOAM,
    AccessVRAM,
}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            vram: [0; VRAM_SIZE],
            mode: Mode::HBlank,
            bgp: 0,
            clocks: 0,
            ly: 0,
            stat: Stat::empty(),
        }
    }

    pub fn run(&mut self, tick: u32) {
        self.clocks += tick;

        match self.mode {
            Mode::AccessOAM => {
                if self.clocks >= 80 {
                    self.clocks -= 80;
                    self.mode = Mode::AccessVRAM;
                    // render scanline
                }
            }
            Mode::AccessVRAM => {
                if self.clocks >= 172 {
                    self.clocks -= 172;
                    self.mode = Mode::HBlank;
                    // interrupt
                }
            }
            Mode::HBlank => {
                if self.clocks >= 204 {
                    self.clocks -= 204;
                    self.ly = self.ly.wrapping_add(1);

                    if self.ly >= SCREEN_HEIGHT {
                        self.mode = Mode::VBlank;
                    // interrupt
                    } else {
                        self.mode = Mode::AccessOAM;
                    }
                    // interrupt
                }
            }
            _ => {
                if self.clocks >= 456 {
                    self.clocks -= 456;
                    self.ly = self.ly.wrapping_add(1);

                    if self.ly >= SCREEN_HEIGHT + 10 {
                        self.mode = Mode::AccessOAM;
                        self.ly = 0;
                        // interrupt
                    }
                    // interrupt
                }
            }
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x8000..=0x9fff => {
                if self.mode == Mode::AccessVRAM {
                    return 0xff;
                }
                self.vram[(addr & (VRAM_SIZE - 1) as u16) as usize]
            }
            0xff44 => self.ly,
            0xff47 => self.bgp,
            _ => 0xff,
        }
    }

    pub fn write_byte(&mut self, addr: u16, v: u8) {
        match addr {
            0x8000..=0x9fff => {
                if self.mode == Mode::AccessVRAM {
                    return;
                }
                self.vram[(addr & (VRAM_SIZE - 1) as u16) as usize] = v;
            }
            0xff44 => (), // read only
            0xff47 => self.bgp = v,
            _ => unreachable!("write: not support address: 0x{:04x}", addr),
        }
    }
}

impl fmt::Debug for PPU {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PPU: {{ bgp: 0b{:08b} }}", self.bgp)
    }
}
