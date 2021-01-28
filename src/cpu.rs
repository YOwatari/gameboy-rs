mod register;

use crate::cpu::register::Flags;
use crate::cpu::register::Register;
use crate::cpu::register::Register16::{AF, BC, DE, HL};
use crate::mmu::MMU;

#[derive(Debug)]
pub struct CPU {
    register: Register,
    mmu: MMU,
}

impl CPU {
    pub fn new(bios: Vec<u8>, rom: Vec<u8>) -> CPU {
        CPU {
            register: Register::new(),
            mmu: MMU::new(bios, rom),
        }
    }

    pub fn run(&mut self) -> u32 {
        let opcode = self.fetch_byte();
        self.execute(opcode)
    }

    fn fetch_byte(&mut self) -> u8 {
        let n = self.mmu.read_byte(self.register.pc);
        self.register.pc = self.register.pc.wrapping_add(1);
        n
    }

    fn fetch_word(&mut self) -> u16 {
        let nn = self.mmu.read_word(self.register.pc);
        self.register.pc = self.register.pc.wrapping_add(2);
        nn
    }

    fn execute(&mut self, opcode: u8) -> u32 {
        match opcode {
            0x01 | 0x11 | 0x21 | 0x31 => self.ld_n_nn(opcode),
            0xaf | 0xa8 | 0xa9 | 0xaa | 0xab | 0xac | 0xad | 0xae | 0xee => self.xor_n(opcode),
            0x32 => self.ldd_hl_a(),
            _ => unimplemented!("unknown opcode: 0x{:02x}\ncpu: {:?}", opcode, self),
        }
    }

    fn ld_n_nn(&mut self, opcode: u8) -> u32 {
        let nn = self.fetch_word();
        match opcode {
            0x01 => self.register.write_word(BC, nn),
            0x11 => self.register.write_word(DE, nn),
            0x21 => self.register.write_word(HL, nn),
            0x31 => self.register.sp = nn,
            _ => unreachable!("not LD n,nn: 0x{:02x}", opcode),
        }
        12
    }

    fn _xor_n(&mut self, n: u8) {
        self.register.a ^= n;
        self.register.set_flag(Flags::Z, self.register.a == 0);
        self.register.set_flag(Flags::N, false);
        self.register.set_flag(Flags::H, false);
        self.register.set_flag(Flags::C, false);
    }

    fn xor_n(&mut self, opcode: u8) -> u32 {
        match opcode {
            0xaf => {
                let n = self.register.a;
                self._xor_n(n);
                4
            }
            0xa8 => {
                let n = self.register.b;
                self._xor_n(n);
                4
            }
            0xa9 => {
                let n = self.register.c;
                self._xor_n(n);
                4
            }
            0xaa => {
                let n = self.register.d;
                self._xor_n(n);
                4
            }
            0xab => {
                let n = self.register.e;
                self._xor_n(n);
                4
            }
            0xac => {
                let n = self.register.h;
                self._xor_n(n);
                4
            }
            0xad => {
                let n = self.register.l;
                self._xor_n(n);
                4
            }
            0xae => {
                let n = self.mmu.read_byte(self.register.read_word(HL));
                self._xor_n(n);
                8
            }
            0xee => {
                let n = self.fetch_byte();
                self._xor_n(n);
                8
            }
            _ => unreachable!("not XOR n: 0x{:02x}", opcode),
        }
    }

    fn ldd_hl_a(&mut self) -> u32 {
        let hl = self.register.read_word(HL);
        self.mmu.write_byte(hl, self.register.a);
        self.register.write_word(HL, hl.wrapping_sub(1));
        8
    }
}
