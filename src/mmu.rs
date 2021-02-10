use bitflags::bitflags;
use std::fmt;

use crate::apu::APU;
use crate::cartridge::Cartridge;
use crate::joypad::JoyPad;
use crate::ppu::PPU;
use crate::serial::Serial;
use crate::timer::Timer;

const WORKING_RAM_SIZE: usize = 8 * 1024;
const HIGH_RAM_SIZE: usize = 128;

pub struct MMU {
    cartridge: Cartridge,
    wram: [u8; WORKING_RAM_SIZE],
    hram: [u8; HIGH_RAM_SIZE],
    pub ppu: PPU,
    apu: APU,
    bios_disable: bool,
    interrupt_enable: IE,
    interrupt_flag: IF,
    serial: Serial,
    timer: Timer,
    joypad: JoyPad,
}

bitflags!(
    struct IE: u8 {
        const VBLANK   = 0b_0000_0001;
        const LCD_STAT = 0b_0000_0010;
        const TIMER    = 0b_0000_0100;
        const SERIAL   = 0b_0000_1000;
        const JOYPAD   = 0b_0001_0000;
    }
);

bitflags!(
    struct IF: u8 {
        const VBLANK   = 0b_0000_0001;
        const LCD_STAT = 0b_0000_0010;
        const TIMER    = 0b_0000_0100;
        const SERIAL   = 0b_0000_1000;
        const JOYPAD   = 0b_0001_0000;
    }
);

impl MMU {
    pub fn new(bios: Vec<u8>, rom: Vec<u8>) -> MMU {
        MMU {
            cartridge: Cartridge::new(bios, rom),
            wram: [0; WORKING_RAM_SIZE],
            hram: [0; HIGH_RAM_SIZE],
            ppu: PPU::new(),
            apu: APU::new(),
            bios_disable: false,
            interrupt_enable: IE::empty(),
            interrupt_flag: IF::empty(),
            serial: Serial::new(),
            timer: Timer::new(),
            joypad: JoyPad::new(),
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
            0xa000..=0xbfff => self.cartridge.read_byte(addr),
            0xc000..=0xdfff => self.wram[(addr & (WORKING_RAM_SIZE as u16 - 1)) as usize],
            0xe000..=0xfdff => {
                self.wram[((addr - 0x2000) & (WORKING_RAM_SIZE as u16 - 1)) as usize]
            }
            0xfe00..=0xfe9f => self.ppu.read_byte(addr),
            0xfea0..=0xfeff => 0xff, // unused
            0xff00 => self.joypad.read_byte(addr),
            0xff01..=0xff02 => self.serial.read_byte(addr),
            0xff04..=0xff07 => self.timer.read_byte(addr),
            0xff0f => self.interrupt_flag.bits,
            0xff03 | 0xff08..=0xff0e | 0xff4c..=0xff4f | 0xff51..=0xff7e => {
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
            0xff7f => 0xff, // unused
            0xff80..=0xfffe => self.hram[(addr & (HIGH_RAM_SIZE as u16 - 1)) as usize],
            0xffff..=0xffff => self.interrupt_enable.bits,
            _ => 0xff,
        }
    }

    pub fn write_byte(&mut self, addr: u16, v: u8) {
        match addr {
            0x0000..=0x7fff => self.cartridge.write_byte(addr, v),
            0x8000..=0x9fff => self.ppu.write_byte(addr, v),
            0xa000..=0xbfff => self.cartridge.write_byte(addr, v),
            0xc000..=0xdfff => self.wram[(addr & (WORKING_RAM_SIZE as u16 - 1)) as usize] = v,
            0xe000..=0xfdff => {
                self.wram[((addr - 0x2000) & (WORKING_RAM_SIZE as u16 - 1)) as usize] = v;
            }
            0xfe00..=0xfe9f => self.ppu.write_byte(addr, v),
            0xfea0..=0xfeff => (), // unused
            0xff00 => self.joypad.write_byte(addr, v),
            0xff01..=0xff02 => self.serial.write_byte(addr, v),
            0xff04..=0xff07 => self.timer.write_byte(addr, v),
            0xff0f => self.interrupt_flag = IF::from_bits_truncate(v),
            0xff03 | 0xff08..=0xff0e | 0xff4c..=0xff4f | 0xff51..=0xff7e => {
                unimplemented!("write: I/O register: {:04x} {:02x}", addr, v)
            }
            0xff10..=0xff3f => self.apu.write_byte(addr, v),
            0xff40..=0xff45 | 0xff47..=0xff4b => self.ppu.write_byte(addr, v),
            0xff46 => unimplemented!("write: I/O register: {:04x} {:02x}", addr, v),
            0xff50 => self.bios_disable = v != 0,
            0xff7f => (), // unused
            0xff80..=0xfffe => self.hram[(addr & (HIGH_RAM_SIZE as u16 - 1)) as usize] = v,
            0xffff..=0xffff => self.interrupt_enable = IE::from_bits_truncate(v),
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
        write!(f, "MMU {{ ppu: {:?}, apu: {:?} }}", self.ppu, self.apu)
    }
}
