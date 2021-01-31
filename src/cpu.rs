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
        //info!("address: 0x{:04x}", self.register.pc);
        let opcode = self.fetch_byte();
        let tick = self.execute(opcode);
        self.mmu.run(tick);
        tick
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

    fn read_rp(&self, idx: u8) -> u16 {
        match idx {
            0 => self.register.read_word(BC),
            1 => self.register.read_word(DE),
            2 => self.register.read_word(HL),
            3 => self.register.sp,
            _ => unreachable!("invalid operand index: {}", idx),
        }
    }

    fn write_rp(&mut self, idx: u8, v: u16) {
        match idx {
            0 => self.register.write_word(BC, v),
            1 => self.register.write_word(DE, v),
            2 => self.register.write_word(HL, v),
            3 => self.register.sp = v,
            _ => unreachable!("invalid operand index: {}", idx),
        }
    }

    fn read_rp2(&self, idx: u8) -> u16 {
        match idx {
            0 => self.register.read_word(BC),
            1 => self.register.read_word(DE),
            2 => self.register.read_word(HL),
            3 => self.register.read_word(AF),
            _ => unreachable!("invalid operand index: {}", idx),
        }
    }

    fn write_rp2(&mut self, idx: u8, v: u16) {
        match idx {
            0 => self.register.write_word(BC, v),
            1 => self.register.write_word(DE, v),
            2 => self.register.write_word(HL, v),
            3 => self.register.write_word(AF, v),
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

    fn push_stack(&mut self, v: u16) {
        self.register.sp = self.register.sp.wrapping_sub(2);
        self.mmu.write_word(self.register.sp, v);
    }

    fn pop_stack(&mut self) -> u16 {
        let nn = self.mmu.read_word(self.register.sp);
        self.register.sp = self.register.sp.wrapping_add(2);
        nn
    }

    fn execute(&mut self, opcode: u8) -> u32 {
        match opcode {
            0x00 => 4,
            0x01 | 0x11 | 0x21 | 0x31 => self.ld_n_nn(opcode),
            0xaf | 0xa8 | 0xa9 | 0xaa | 0xab | 0xac | 0xad | 0xae | 0xee => self.xor_n(opcode),
            0x32 => self.ldd_hl_a(),
            0xcb => self.prefix(),
            0x20 | 0x28 | 0x30 | 0x38 => self.jr_cc_n(opcode),
            0x06 | 0x0e | 0x16 | 0x1e | 0x26 | 0x2e => self.ld_nn_n(opcode),
            0x78..=0x7f | 0x0a | 0x1a | 0xfa | 0x3e => self.ld_a_n(opcode),
            0xe2 => self.ld_c_a(),
            0x04 | 0x0c | 0x14 | 0x1c | 0x24 | 0x2c | 0x34 | 0x3c => self.inc_n(opcode),
            0x47 | 0x4f | 0x57 | 0x5f | 0x67 | 0x6f | 0x77 | 0x02 | 0x12 | 0xea => {
                self.ld_n_a(opcode)
            }
            0xe0 => self.ldh_n_a(),
            0xcd => self.call_nn(),
            0xf5 | 0xc5 | 0xd5 | 0xe5 => self.push_nn(opcode),
            0x17 => self.rla(),
            0xc1 | 0xd1 | 0xe1 | 0xf1 => self.pop_nn(opcode),
            0x05 | 0x0d | 0x15 | 0x1d | 0x25 | 0x2d | 0x35 | 0x3d => self.dec_n(opcode),
            0x22 => self.ldi_hl_a(),
            0x03 | 0x13 | 0x23 | 0x33 => self.inc_nn(opcode),
            0xc9 => self.ret(),
            0xb8..=0xbf | 0xfe => self.cp_n(opcode),
            0x18 => self.jr_n(),
            0xf0 => self.ldh_a_n(),
            0x90..=0x97 | 0xd6 => self.sub_n(opcode),
            0x80..=0x87 | 0xc6 => self.add_a_n(opcode),
            _ => unimplemented!("unknown opcode: 0x{:02x}\ncpu: {:?}", opcode, self),
        }
    }

    fn prefix(&mut self) -> u32 {
        let opcode = self.fetch_byte();
        match opcode {
            0x10..=0x17 => self.rl(opcode),
            0x40..=0x7f => self.bit_b_r(opcode),
            _ => unimplemented!("unknown cb opcode: 0x{:02x}\ncput: {:?}", opcode, self),
        }
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
                let r8_idx = opcode & 0b_0000_0111;
                let n = self.read_r8(r8_idx);
                self._xor_n(n);

                match r8_idx {
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

    fn ldi_hl_a(&mut self) -> u32 {
        let hl = self.register.read_word(HL);
        self.mmu.write_byte(hl, self.register.a);
        self.register.write_word(HL, hl.wrapping_add(1));
        8
    }

    fn bit_b_r(&mut self, opcode: u8) -> u32 {
        let b = (opcode & 0b_0011_1000) >> 3;
        let r8_idx = opcode & 0b_0000_0111;
        let reg = self.read_r8(r8_idx);
        let result = reg & (1 << b);
        self.register.set_flag(Flags::Z, result == 0);
        self.register.set_flag(Flags::N, false);
        self.register.set_flag(Flags::H, true);

        match r8_idx {
            6 => 16,
            _ => 8,
        }
    }

    fn jr_cc_n(&mut self, opcode: u8) -> u32 {
        let cc_idx = (opcode & 0b_0011_1000) >> 3;
        let cc = self.read_cc(cc_idx - 4);
        let n = self.fetch_byte() as i8;

        if cc {
            self.register.pc = self.register.pc.wrapping_add(n as u16);
            12
        } else {
            8
        }
    }

    fn ld_n_nn(&mut self, opcode: u8) -> u32 {
        let rp_idx = (opcode & 0b_0011_0000) >> 4;
        let nn = self.fetch_word();
        self.write_rp(rp_idx, nn);
        12
    }

    fn ld_nn_n(&mut self, opcode: u8) -> u32 {
        let r8_idx = (opcode & 0b_0011_1000) >> 3;
        let n = self.fetch_byte();
        self.write_r8(r8_idx, n);
        8
    }

    fn ld_a_n(&mut self, opcode: u8) -> u32 {
        match opcode {
            0x78..=0x7f => {
                let r8_idx = opcode & 0b_0000_0111;
                let n = self.read_r8(r8_idx);
                self.register.a = n;

                match r8_idx {
                    6 => 8,
                    _ => 4,
                }
            }
            0x0a | 0x1a => {
                let rp2_idx = (opcode & 0b_0011_0000) >> 4;
                let nn = self.read_rp2(rp2_idx);
                self.register.a = self.mmu.read_byte(nn);
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

    fn ld_n_a(&mut self, opcode: u8) -> u32 {
        match opcode {
            0x47 | 0x4f | 0x57 | 0x5f | 0x67 | 0x6f | 0x77 | 0x7f => {
                let r8_idx = (opcode & 0b_0011_1000) >> 3;
                let n = self.read_r8(r8_idx);
                self.register.a = n;

                match r8_idx {
                    6 => 8,
                    _ => 4,
                }
            }
            0x02 | 0x12 => {
                let rp2 = (opcode & 0b_0011_0000) >> 4;
                let nn = self.read_rp2(rp2);
                let n = self.mmu.read_byte(nn);
                self.register.a = n;
                8
            }
            0xea => {
                let nn = self.fetch_word();
                self.register.a = self.mmu.read_byte(nn);
                16
            }
            _ => unreachable!("not LD n,A: 0x{:02x}", opcode),
        }
    }

    fn ld_c_a(&mut self) -> u32 {
        let addr = 0xff00 | (self.register.c as u16);
        self.mmu.write_byte(addr, self.register.a);
        8
    }

    fn ldh_n_a(&mut self) -> u32 {
        let n = self.fetch_byte();
        self.mmu.write_byte(0xff00 | (n as u16), self.register.a);
        12
    }

    fn ldh_a_n(&mut self) -> u32 {
        let n = self.fetch_byte();
        self.register.a = self.mmu.read_byte(0xff00 | (n as u16));
        12
    }

    fn inc_n(&mut self, opcode: u8) -> u32 {
        let r8_idx = (opcode & 0b_0011_1000) >> 3;
        let n = self.read_r8(r8_idx);
        let result = n.wrapping_add(1);
        self.write_r8(r8_idx, result);
        self.register.set_flag(Flags::Z, result == 0);
        self.register.set_flag(Flags::N, false);
        self.register.set_flag(Flags::H, n & 0x0f == 0x0f);

        match r8_idx {
            6 => 12,
            _ => 4,
        }
    }

    fn inc_nn(&mut self, opcode: u8) -> u32 {
        let rp_idx = (opcode & 0b_0011_0000) >> 4;
        let nn = self.read_rp(rp_idx);
        self.write_rp(rp_idx, nn.wrapping_add(1));
        8
    }

    fn call_nn(&mut self) -> u32 {
        let nn = self.fetch_word();
        self.push_stack(self.register.pc);
        self.register.pc = nn;
        12
    }

    fn push_nn(&mut self, opcode: u8) -> u32 {
        let rp2_idx = (opcode & 0b_0011_0000) >> 4;
        let nn = self.read_rp2(rp2_idx);
        self.push_stack(nn);
        16
    }

    fn pop_nn(&mut self, opcode: u8) -> u32 {
        let rp2_idx = (opcode & 0b_0011_0000) >> 4;
        let nn = self.pop_stack();
        self.write_rp2(rp2_idx, nn);
        12
    }

    fn rl(&mut self, opcode: u8) -> u32 {
        let r8_idx = opcode & 0b_0000_0111;
        let n = self.read_r8(r8_idx);
        let result = n << 1
            | (if self.register.get_flag(Flags::C) {
                1
            } else {
                0
            });
        self.write_r8(r8_idx, result);
        self.register.set_flag(Flags::Z, result == 0);
        self.register.set_flag(Flags::N, false);
        self.register.set_flag(Flags::H, false);
        self.register.set_flag(Flags::C, n & (1 << 7) != 0);

        match r8_idx {
            6 => 16,
            _ => 8,
        }
    }

    fn rla(&mut self) -> u32 {
        let r8_idx: u8 = 7;
        let n = self.read_r8(r8_idx);
        let result = n << 1
            | (if self.register.get_flag(Flags::C) {
                1
            } else {
                0
            });
        self.write_r8(7, result);
        self.register.set_flag(Flags::Z, result == 0);
        self.register.set_flag(Flags::N, false);
        self.register.set_flag(Flags::H, false);
        self.register.set_flag(Flags::C, n & (1 << 7) != 0);
        4
    }

    fn dec_n(&mut self, opcode: u8) -> u32 {
        let r8_idx = (opcode & 0b_0011_1000) >> 3;
        let n = self.read_r8(r8_idx);
        let result = n.wrapping_sub(1);
        self.write_r8(r8_idx, result);
        self.register.set_flag(Flags::Z, result == 0);
        self.register.set_flag(Flags::N, true);
        self.register.set_flag(Flags::H, n & 0x0f == 0);

        match r8_idx {
            6 => 12,
            _ => 4,
        }
    }

    fn ret(&mut self) -> u32 {
        let nn = self.pop_stack();
        self.register.pc = nn;
        8
    }

    fn _cp_n(&mut self, n: u8) {
        let a = self.register.a;
        let h = (a & 0x0f) < (n & 0x0f);
        self.register.set_flag(Flags::Z, a == n);
        self.register.set_flag(Flags::N, true);
        self.register.set_flag(Flags::H, h);
        self.register.set_flag(Flags::C, a < n);
    }

    fn cp_n(&mut self, opcode: u8) -> u32 {
        match opcode {
            0xb8..=0xbf => {
                let r8_idx = opcode & 0b_0000_0111;
                let n = self.read_r8(r8_idx);
                self._cp_n(n);

                match r8_idx {
                    6 => 8,
                    _ => 4,
                }
            }
            0xfe => {
                let n = self.fetch_byte();
                self._cp_n(n);
                8
            }
            _ => unreachable!("not CP n: 0x{:02x}", opcode),
        }
    }

    fn jr_n(&mut self) -> u32 {
        let n = self.fetch_byte() as i8;
        self.register.pc = self.register.pc.wrapping_add(n as u16);
        8
    }

    fn _sub_n(&mut self, n: u8) {
        let a = self.register.a;
        let (result, c) = a.overflowing_sub(n);
        let h = (a & 0x0f) < (n & 0x0f);
        self.register.a = result;
        self.register.set_flag(Flags::Z, result == 0);
        self.register.set_flag(Flags::N, true);
        self.register.set_flag(Flags::H, h);
        self.register.set_flag(Flags::C, c);
    }

    fn sub_n(&mut self, opcode: u8) -> u32 {
        match opcode {
            0x90..=0x97 => {
                let r8_idx = opcode & 0b_0000_0111;
                let n = self.read_r8(r8_idx);
                self._sub_n(n);

                match r8_idx {
                    6 => 8,
                    _ => 4,
                }
            }
            0xd6 => {
                let n = self.fetch_byte();
                self._sub_n(n);
                8
            }
            _ => unreachable!("not SUB n: 0x{:02x}", opcode),
        }
    }

    fn _add_a_n(&mut self, n: u8) {
        let a = self.register.a;
        let (result, c) = a.overflowing_add(n);
        let h = (a & 0x0f) < (n & 0x0f);
        self.register.a = result;
        self.register.set_flag(Flags::Z, result == 0);
        self.register.set_flag(Flags::N, false);
        self.register.set_flag(Flags::H, h);
        self.register.set_flag(Flags::C, c);
    }

    fn add_a_n(&mut self, opcode: u8) -> u32 {
        match opcode {
            0x80..=0x87 => {
                let r8_idx = opcode & 0b_0000_0111;
                let n = self.read_r8(r8_idx);
                self._add_a_n(n);
                match r8_idx {
                    6 => 8,
                    _ => 4,
                }
            }
            0xc6 => {
                let n = self.fetch_byte();
                self._add_a_n(n);
                8
            }
            _ => unreachable!("not ADD A,n: 0x{:02x}", opcode),
        }
    }
}
