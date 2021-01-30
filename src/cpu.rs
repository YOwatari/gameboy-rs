mod register;

use crate::cpu::register::Flags;
use crate::cpu::register::Register;
use crate::cpu::register::Register16::{AF, BC, DE, HL};
use crate::mmu::MMU;

use log::info;

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
        info!("address: {:04x}", self.register.pc);
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

    fn read_r8(&self, idx: u8) -> u8 {
        match idx {
            0 => self.register.b,
            1 => self.register.c,
            2 => self.register.d,
            3 => self.register.e,
            4 => self.register.h,
            5 => self.register.l,
            6 => {
                let hl = self.register.read_word(HL);
                self.mmu.read_byte(hl)
            }
            7 => self.register.a,
            _ => unreachable!("invalid operand index: {}", idx),
        }
    }

    fn write_r8(&mut self, idx: u8, v: u8) {
        match idx {
            0 => self.register.b = v,
            1 => self.register.c = v,
            2 => self.register.d = v,
            3 => self.register.e = v,
            4 => self.register.h = v,
            5 => self.register.l = v,
            6 => {
                let hl = self.register.read_word(HL);
                self.mmu.write_byte(hl, v);
            }
            7 => self.register.a = v,
            _ => unreachable!("invalid operand index: {}", idx),
        }
    }

    fn read_cc(&self, idx: u8) -> bool {
        match idx {
            0 => !self.register.get_flag(Flags::Z),
            1 => self.register.get_flag(Flags::Z),
            2 => !self.register.get_flag(Flags::C),
            3 => self.register.get_flag(Flags::C),
            _ => unreachable!("invalid operand index: {}", idx),
        }
    }

    fn execute(&mut self, opcode: u8) -> u32 {
        match opcode {
            0x01 | 0x11 | 0x21 | 0x31 => self.ld_n_nn(opcode),
            0xaf | 0xa8 | 0xa9 | 0xaa | 0xab | 0xac | 0xad | 0xae | 0xee => self.xor_n(opcode),
            0x32 => self.ldd_hl_a(),
            0xcb => self.prefix(),
            0x20 | 0x28 | 0x30 | 0x38 => self.jr_cc_n(opcode),
            0x06 | 0x0e | 0x16 | 0x1e | 0x26 | 0x2e => self.ld_nn_n(opcode),
            0x78..=0x7f | 0x0a | 0x1a | 0xfa | 0x3e => self.ld_a_n(opcode),
            0xe2 => self.ld_c_a(),
            //0xe0 => self.ldd_hl_a(),
            _ => unimplemented!("unknown opcode: 0x{:02x}\ncpu: {:?}", opcode, self),
        }
    }

    fn prefix(&mut self) -> u32 {
        let opcode = self.fetch_byte();
        match opcode {
            0x40..=0x7f => self.bit_b_r(opcode),
            _ => unimplemented!("unknown cb opcode: 0x{:02x}\ncput: {:?}", opcode, self),
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
            0xa8..=0xaf => {
                let r8 = opcode & 0b_0000_0111;
                let n = self.read_r8(r8);
                self._xor_n(n);

                match r8 {
                    6 => 8,
                    _ => 4,
                }
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

    fn bit_b_r(&mut self, opcode: u8) -> u32 {
        let b = (opcode & 0b_0011_1000) >> 3;
        let r8 = opcode & 0b_0000_0111;
        let reg = self.read_r8(r8);
        let result = reg & (1 << b);
        self.register.set_flag(Flags::Z, result == 0);
        self.register.set_flag(Flags::N, false);
        self.register.set_flag(Flags::H, true);

        match r8 {
            6 => 16,
            _ => 8,
        }
    }

    fn jr_cc_n(&mut self, opcode: u8) -> u32 {
        let b = (opcode & 0b_0011_1000) >> 3;
        let cc = self.read_cc(b - 4);
        let n = self.fetch_byte() as i8;

        if cc {
            self.register.pc = self.register.pc.wrapping_add(n as u16);
            12
        } else {
            8
        }
    }

    fn ld_nn_n(&mut self, opcode: u8) -> u32 {
        let b = (opcode & 0b_0011_1000) >> 3;
        let n = self.fetch_byte();
        self.write_r8(b, n);
        8
    }

    fn ld_a_n(&mut self, opcode: u8) -> u32 {
        match opcode {
            0x78..=0x7f => {
                let b = (opcode & 0b_0011_1000) >> 3;
                let r8 = opcode & 0b_0000_0111;
                let reg = self.read_r8(r8);
                self.write_r8(b, reg);

                match b {
                    6 => 8,
                    _ => 4,
                }
            }
            0x0a => {
                let bc = self.register.read_word(BC);
                self.register.a = self.mmu.read_byte(bc);
                8
            }
            0x1a => {
                let de = self.register.read_word(DE);
                self.register.a = self.mmu.read_byte(de);
                8
            }
            0xfa => {
                let nn = self.fetch_word();
                self.register.a = self.mmu.read_byte(nn);
                16
            }
            0x3e => self.ld_nn_n(opcode),
            _ => unreachable!("not LD A,n: 0x{:02x}", opcode),
        }
    }

    fn ld_c_a(&mut self) -> u32 {
        let addr = 0xff00 | (self.register.c as u16);
        self.mmu.write_byte(addr, self.register.a);
        8
    }

    /*
    fn ldh_a_n(&mut self) -> u32 {
        let n = self.fetch_byte() as u16;
        self.mmu.write_byte(0xff00 | n, self.register.a);
        12
    }
     */
}
