use crate::cartridge::Cartridge;

const WORKING_RAM_SIZE: usize = 8 * 1024;
const HIGH_RAM_SIZE: usize = 128;

pub struct MMU {
    cartridge: Cartridge,
    wram: [u8; WORKING_RAM_SIZE],
    hram: [u8; HIGH_RAM_SIZE],
}

impl MMU {
    pub fn new(bios: Vec<u8>, rom: Vec<u8>) -> MMU {
        MMU {
            cartridge: Cartridge::new(bios, rom),
            wram: [0; WORKING_RAM_SIZE],
            hram: [0; HIGH_RAM_SIZE],
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7fff => self.cartridge.read_byte(addr),
            0x8000..=0x9fff => panic!("read: Video RAM is not yet implemented: {:x}", addr),
            0xa000..=0xbeff => panic!("read: Cartridge RAM is not yet implemented: {:x}", addr),
            0xc000..=0xdfff => self.wram[(addr & (WORKING_RAM_SIZE as u16 - 1)) as usize],
            0xe000..=0xfdff => {
                self.wram[((addr - 0x2000) & (WORKING_RAM_SIZE as u16 - 1)) as usize]
            }
            0xfe00..=0xfe9f => panic!("read: OAM is not yet implemented: {:x}", addr),
            0xfea0..=0xfeff => 0xff, // unused
            0xff00..=0xff7f => panic!("read: I/O register is not yet implemented: {:x}", addr),
            0xff80..=0xfffe => self.hram[(addr & (HIGH_RAM_SIZE as u16 - 1)) as usize],
            0xffff => panic!(
                "read: Interrupt Enable Register is not yet implemented: {:x}",
                addr
            ),
            _ => 0xff,
        }
    }

    pub fn write_byte(&mut self, addr: u16, v: u8) {
        match addr {
            0x0000..=0x7fff => (),
            0x8000..=0x9fff => panic!("write: Video RAM is not yet implemented: {:x}", addr),
            0xa000..=0xbeff => panic!("write: Cartridge RAM is not yet implemented: {:x}", addr),
            0xc000..=0xdfff => self.wram[(addr & (WORKING_RAM_SIZE as u16 - 1)) as usize] = v,
            0xe000..=0xfdff => {
                self.wram[((addr - 0x2000) & (WORKING_RAM_SIZE as u16 - 1)) as usize] = v
            }
            0xfe00..=0xfe9f => panic!("write: OAM is not yet implemented: {:x}", addr),
            0xfea0..=0xfeff => (),
            0xff00..=0xff7f => panic!("read: I/O register is not yet implemented: {:x}", addr),
            0xff80..=0xfffe => self.hram[(addr & (HIGH_RAM_SIZE as u16 - 1)) as usize] = v,
            0xffff => {
                panic!(
                    "read: Interrupt Enable Register is not yet implemented: {:x}",
                    addr
                )
            }
            _ => unimplemented!("not support to write the address: {:x}", addr),
        }
    }
}
