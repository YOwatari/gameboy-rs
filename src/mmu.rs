use crate::apu::APU;
use crate::cartridge::Cartridge;
use crate::ppu::PPU;

use std::fmt;

const WORKING_RAM_SIZE: usize = 8 * 1024;
const HIGH_RAM_SIZE: usize = 128;

pub struct MMU {
    cartridge: Cartridge,
    wram: [u8; WORKING_RAM_SIZE],
    hram: [u8; HIGH_RAM_SIZE],
    pub ppu: PPU,
    apu: APU,
    bios_disable: bool,
}

impl MMU {
    pub fn new(bios: Vec<u8>, rom: Vec<u8>) -> MMU {
        MMU {
            cartridge: Cartridge::new(bios, rom),
            wram: [0; WORKING_RAM_SIZE],
            hram: [0; HIGH_RAM_SIZE],
            ppu: PPU::new(),
            apu: APU::new(),
            bios_disable: false,
        }
    }

    pub fn run(&mut self, tick: u32) {
        self.ppu.run(tick);
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x00ff => {
                if self.bios_disable {
                    0xff
                } else {
                    self.cartridge.read_byte(addr)
                }
            }
            0x0100..=0x7fff => self.cartridge.read_byte(addr),
            0x8000..=0x9fff => self.ppu.read_byte(addr),
            0xa000..=0xbeff => unimplemented!("read: Cartridge RAM: {:x}", addr),
            0xc000..=0xdfff => self.wram[(addr & (WORKING_RAM_SIZE as u16 - 1)) as usize],
            0xe000..=0xfdff => {
                self.wram[((addr - 0x2000) & (WORKING_RAM_SIZE as u16 - 1)) as usize]
            }
            0xfe00..=0xfe9f => unimplemented!("read: OAM: {:x}", addr),
            0xfea0..=0xfeff => 0xff, // unused
            0xff00..=0xff0f | 0xff4c..=0xff4f | 0xff51..=0xff7f => {
                unimplemented!("read: I/O register: {:x}", addr)
            }
            0xff10..=0xff3f => self.apu.read_byte(addr),
            0xff40..=0xff45 | 0xff47..=0xff4b => self.ppu.read_byte(addr),
            0xff46 => unimplemented!("read: I/O register: {:x}", addr),
            0xff50 => {
                if self.bios_disable {
                    1
                } else {
                    0
                }
            }
            0xff80..=0xfffe => self.hram[(addr & (HIGH_RAM_SIZE as u16 - 1)) as usize],
            0xffff..=0xffff => unimplemented!("read: Interrupt Enable Register: {:x}", addr),
            _ => 0xff,
        }
    }

    pub fn write_byte(&mut self, addr: u16, v: u8) {
        match addr {
            0x0000..=0x7fff => (),
            0x8000..=0x9fff => self.ppu.write_byte(addr, v),
            0xa000..=0xbeff => unimplemented!("write: Cartridge RAM: {:04x} {:02x}", addr, v),
            0xc000..=0xdfff => self.wram[(addr & (WORKING_RAM_SIZE as u16 - 1)) as usize] = v,
            0xe000..=0xfdff => {
                self.wram[((addr - 0x2000) & (WORKING_RAM_SIZE as u16 - 1)) as usize] = v;
            }
            0xfe00..=0xfe9f => unimplemented!("write: OAM: {:04x} {:02x}", addr, v),
            0xfea0..=0xfeff => (),
            0xff00..=0xff0f | 0xff4c..=0xff4f | 0xff51..=0xff7f => {
                unimplemented!("write: I/O register: {:04x} {:02x}", addr, v)
            }
            0xff10..=0xff3f => self.apu.write_byte(addr, v),
            0xff40..=0xff45 | 0xff47..=0xff4b => self.ppu.write_byte(addr, v),
            0xff46 => unimplemented!("write: I/O register: {:04x} {:02x}", addr, v),
            0xff50 => self.bios_disable = v != 0,
            0xff80..=0xfffe => self.hram[(addr & (HIGH_RAM_SIZE as u16 - 1)) as usize] = v,
            0xffff..=0xffff => {
                unimplemented!("write: Interrupt Enable Register: {:04x} {:02x}", addr, v)
            }
            _ => unreachable!("write: not support the address: {:04x} {:02x}", addr, v),
        }
    }

    pub fn read_word(&self, addr: u16) -> u16 {
        self.read_byte(addr) as u16 | (self.read_byte(addr + 1) as u16) << 8
    }

    pub fn write_word(&mut self, addr: u16, v: u16) {
        self.write_byte(addr, v as u8);
        self.write_byte(addr + 1, (v >> 8) as u8);
    }
}

impl fmt::Debug for MMU {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MMU {{ ppu: {:?} }}", self.ppu)
    }
}
