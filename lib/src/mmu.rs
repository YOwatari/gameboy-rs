use bitflags::bitflags;
use std::fmt;

use crate::apu::APU;
use crate::cartridge::Cartridge;
use crate::joypad::JoyPad;
use crate::ppu::{OAM_SIZE, PPU};
use crate::serial::Serial;
use crate::timer::Timer;

const WORKING_RAM_SIZE: usize = 8 * 1024;
const HIGH_RAM_SIZE: usize = 128;

pub struct MMU {
    pub cartridge: Cartridge,
    wram: [u8; WORKING_RAM_SIZE],
    hram: [u8; HIGH_RAM_SIZE],
    pub ppu: PPU,
    apu: APU,
    pub interrupt_enable: Interrupt,
    pub interrupt_flag: Interrupt,
    serial: Serial,
    timer: Timer,
    pub joypad: JoyPad,
}

bitflags!(
    pub struct Interrupt: u8 {
        const VBLANK   = 0b_0000_0001;
        const LCD_STAT = 0b_0000_0010;
        const TIMER    = 0b_0000_0100;
        const SERIAL   = 0b_0000_1000;
        const JOYPAD   = 0b_0001_0000;
    }
);

impl MMU {
    pub fn new(rom: Vec<u8>) -> MMU {
        MMU {
            cartridge: Cartridge::new(rom),
            wram: [0; WORKING_RAM_SIZE],
            hram: [0; HIGH_RAM_SIZE],
            ppu: PPU::new(),
            apu: APU::new(),
            interrupt_enable: Interrupt::empty(),
            interrupt_flag: Interrupt::empty(),
            serial: Serial::new(),
            timer: Timer::new(),
            joypad: JoyPad::new(),
        }
    }

    pub fn init(&mut self) {
        self.write_byte(0xff05, 0x00);
        self.write_byte(0xff06, 0x00);
        self.write_byte(0xff07, 0x00);
        self.write_byte(0xff10, 0x80);
        self.write_byte(0xff11, 0xbf);
        self.write_byte(0xff12, 0xf3);
        self.write_byte(0xff14, 0xbf);
        self.write_byte(0xff16, 0x3f);
        self.write_byte(0xff17, 0x00);
        self.write_byte(0xff19, 0xbf);
        self.write_byte(0xff1a, 0x7f);
        self.write_byte(0xff1b, 0xff);
        self.write_byte(0xff1c, 0x9f);
        self.write_byte(0xff1e, 0xbf);
        self.write_byte(0xff20, 0xff);
        self.write_byte(0xff21, 0x00);
        self.write_byte(0xff22, 0x00);
        self.write_byte(0xff23, 0xbf);
        self.write_byte(0xff24, 0x77);
        self.write_byte(0xff25, 0xf3);
        self.write_byte(0xff26, 0xf1);
        self.write_byte(0xff40, 0x91);
        self.write_byte(0xff42, 0x00);
        self.write_byte(0xff43, 0x00);
        self.write_byte(0xff45, 0x00);
        self.write_byte(0xff47, 0xfc);
        self.write_byte(0xff48, 0xff);
        self.write_byte(0xff49, 0xff);
        self.write_byte(0xff4a, 0x00);
        self.write_byte(0xff4b, 0x00);
        self.write_byte(0xffff, 0x00);
    }

    pub fn run(&mut self, ticks: u32) {
        self.ppu.run(ticks);
        self.timer.run(ticks);

        if self.ppu.interrupt_vblank {
            self.interrupt_flag.set(Interrupt::VBLANK, true);
            self.ppu.interrupt_vblank = false;
        }

        if self.ppu.interrupt_lcdc {
            self.interrupt_flag.set(Interrupt::LCD_STAT, true);
            self.ppu.interrupt_lcdc = false;
        }

        if self.timer.interrupt {
            self.interrupt_flag.set(Interrupt::TIMER, true);
            self.timer.interrupt = false;
        }
    }

    fn dma_transfer(&mut self, v: u8) {
        let src = (v as u16) << 8;
        for i in 0..OAM_SIZE as u16 {
            let b = self.read_byte(src | i);
            self.write_byte(0xfe00 | i, b);
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7fff => self.cartridge.read_byte(addr),
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
            0xff03 | 0xff08..=0xff0e | 0xff4c | 0xff4e..=0xff4f | 0xff51..=0xff7e => {
                unimplemented!("read: I/O register: {:x}", addr)
            }
            0xff4d => 0xff, // CGB register
            0xff10..=0xff3f => self.apu.read_byte(addr),
            0xff40..=0xff45 | 0xff47..=0xff4b => self.ppu.read_byte(addr),
            0xff46 => 0xff,
            0xff50 => panic!("BIOS DISABLE"),
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
            0xff0f => self.interrupt_flag = Interrupt::from_bits_truncate(v),
            0xff03 | 0xff08..=0xff0e | 0xff4c | 0xff4e..=0xff4f | 0xff51..=0xff7e => {
                unimplemented!("write: I/O register: {:04x} {:02x}", addr, v)
            }
            0xff4d => (), // CGB register
            0xff10..=0xff3f => self.apu.write_byte(addr, v),
            0xff40..=0xff45 | 0xff47..=0xff4b => self.ppu.write_byte(addr, v),
            0xff46 => self.dma_transfer(v),
            0xff50 => panic!("BIOS DISABLE"),
            0xff7f => (), // unused
            0xff80..=0xfffe => self.hram[(addr & (HIGH_RAM_SIZE as u16 - 1)) as usize] = v,
            0xffff..=0xffff => self.interrupt_enable = Interrupt::from_bits_truncate(v),
            _ => unreachable!("write: not support the address: {:04x} {:02x}", addr, v),
        }
    }

    pub fn read_word(&self, addr: u16) -> u16 {
        ((self.read_byte(addr + 1) as u16) << 8) | self.read_byte(addr) as u16
    }

    pub fn write_word(&mut self, addr: u16, v: u16) {
        self.write_byte(addr, v as u8);
        self.write_byte(addr.wrapping_add(1), (v >> 8) as u8);
    }
}

impl fmt::Debug for MMU {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MMU {{ ppu: {:?}, apu: {:?} }}", self.ppu, self.apu)
    }
}
