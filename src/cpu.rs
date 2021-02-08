mod register;

use crate::cpu::register::Flags;
use crate::cpu::register::Register;
use crate::cpu::register::Register16::{AF, BC, DE, HL};
use crate::mmu::MMU;

#[derive(Debug)]
pub struct CPU {
    register: Register,
    pub mmu: MMU,
    ime: bool,
}

impl CPU {
    pub fn new(bios: Vec<u8>, rom: Vec<u8>) -> CPU {
        CPU {
            register: Register::new(),
            mmu: MMU::new(bios, rom),
            ime: false,
        }
    }

    pub fn run(&mut self) -> u32 {
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

    fn read_r(&self, idx: u8) -> u8 {
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

    fn write_r(&mut self, idx: u8, v: u8) {
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
            // loads
            0x01 | 0x11 | 0x21 | 0x31 => self.ld_n_nn(opcode),
            0x06 | 0x0e | 0x16 | 0x1e | 0x26 | 0x2e | 0x36 | 0x3e => self.ld_nn_n(opcode),
            0x78..=0x7f => self.ld_a_r(opcode),
            0x0a | 0x1a => self.ld_a_rp(opcode),
            0xfa => self.ld_a_nn(),
            0x47 | 0x4f | 0x57 | 0x5f | 0x67 | 0x6f | 0x77 /*| 0x7f*/ => self.ld_r_a(opcode),
            0x02 | 0x12 => self.ld_rp_a(opcode),
            0xea => self.ld_nn_a(),
            0xe2 => self.ld_c_a(),
            0xf2 => self.ld_a_c(),
            0xe0 => self.ldh_n_a(),
            0xf0 => self.ldh_a_n(),
            0x22 => self.ldi_hl_a(),
            0x2a => self.ldi_a_hl(),
            0x32 => self.ldd_hl_a(),
            0x3a => self.ldd_a_hl(),
            0xc1 | 0xd1 | 0xe1 | 0xf1 => self.pop_rp2(opcode),
            0xc5 | 0xd5 | 0xe5 | 0xf5 => self.push_rp2(opcode),

            // arithmetic
            0x80..=0x87 => self.add_a_r(opcode),
            0xc6 => self.add_a_n(),
            0xa0..=0xa7 => self.and_r(opcode),
            0xe6 => self.and_n(),
            0x90..=0x97 => self.sub_r(opcode),
            0xd6 => self.sub_n(),
            0xb0..=0xb7 => self.or_r(opcode),
            0xf6 => self.or_n(),
            0xa8..=0xaf => self.xor_r(opcode),
            0xee => self.xor_n(),
            0xb8..=0xbf => self.cp_r(opcode),
            0xfe => self.cp_n(),
            0x04 | 0x0c | 0x14 | 0x1c | 0x24 | 0x2c | 0x34 | 0x3c => self.inc_r(opcode),
            0x05 | 0x0d | 0x15 | 0x1d | 0x25 | 0x2d | 0x35 | 0x3d => self.dec_r(opcode),
            0x03 | 0x13 | 0x23 | 0x33 => self.inc_rp(opcode),
            0x0b | 0x1b | 0x2b | 0x3b => self.dec_rp(opcode),

            // rotates & shifts
            0x17 => self.rla(),

            // jumps
            0xc3 => self.jp_nn(),
            0x18 => self.jr_n(),
            0x20 | 0x28 | 0x30 | 0x38 => self.jr_cc_n(opcode),

            // calls
            0xcd => self.call_nn(),

            // returns
            0xc9 => self.ret(),

            // miscellaneous
            0x2f => self.cpl(),
            0x00 => 4, // nop
            0x37 => self.scf(),
            0xf3 => self.di(),
            0xfb => self.ei(),

            0xcb => self.prefix(),
            _ => unimplemented!("unknown opcode: 0x{:02x}\ncpu: {:?}", opcode, self),
        }
    }

    fn prefix(&mut self) -> u32 {
        let opcode = self.fetch_byte();
        match opcode {
            // rotates & shifts
            0x10..=0x17 => self.rl_n(opcode),
            // miscellaneous
            0x30..=0x37 => self.swap_r(opcode),
            // bit
            0x40..=0x7f => self.bit_b_r(opcode),
            _ => unimplemented!("unknown cb opcode: 0x{:02x}\ncput: {:?}", opcode, self),
        }
    }

    fn ld_n_nn(&mut self, opcode: u8) -> u32 {
        let p = (opcode & 0b_0011_0000) >> 4;
        let nn = self.fetch_word();
        self.write_rp(p, nn);
        12
    }

    fn ld_nn_n(&mut self, opcode: u8) -> u32 {
        let y = (opcode & 0b_0011_1000) >> 3;
        let n = self.fetch_byte();
        self.write_r(y, n);
        match y {
            6 => 12,
            _ => 8,
        }
    }

    fn ld_a_r(&mut self, opcode: u8) -> u32 {
        let z = opcode & 0b_0000_0111;
        let n = self.read_r(z);
        self.register.a = n;
        match z {
            6 => 8,
            _ => 4,
        }
    }

    fn ld_a_rp(&mut self, opcode: u8) -> u32 {
        let p = (opcode & 0b_0011_0000) >> 4;
        let nn = self.read_rp(p);
        self.register.a = self.mmu.read_byte(nn);
        8
    }

    fn ld_a_nn(&mut self) -> u32 {
        let nn = self.fetch_word();
        self.register.a = self.mmu.read_byte(nn);
        16
    }

    fn ld_r_a(&mut self, opcode: u8) -> u32 {
        let y = (opcode & 0b_0011_1000) >> 3;
        self.write_r(y, self.register.a);
        match y {
            6 => 8,
            _ => 4,
        }
    }

    fn ld_rp_a(&mut self, opcode: u8) -> u32 {
        let p = (opcode & 0b_0011_0000) >> 4;
        let nn = self.read_rp(p);
        self.mmu.write_byte(nn, self.register.a);
        8
    }

    fn ld_nn_a(&mut self) -> u32 {
        let nn = self.fetch_word();
        self.mmu.write_byte(nn, self.register.a);
        16
    }

    fn ld_c_a(&mut self) -> u32 {
        let addr = 0xff00 | (self.register.c as u16);
        self.mmu.write_byte(addr, self.register.a);
        8
    }

    fn ld_a_c(&mut self) -> u32 {
        let addr = 0xff00 | (self.register.c as u16);
        let n = self.mmu.read_byte(addr);
        self.register.a = n;
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

    fn ldi_hl_a(&mut self) -> u32 {
        let hl = self.register.read_word(HL);
        self.mmu.write_byte(hl, self.register.a);
        self.register.write_word(HL, hl.wrapping_add(1));
        8
    }

    fn ldi_a_hl(&mut self) -> u32 {
        let hl = self.register.read_word(HL);
        self.register.a = self.mmu.read_byte(hl);
        self.register.write_word(HL, hl.wrapping_add(1));
        8
    }

    fn ldd_hl_a(&mut self) -> u32 {
        let hl = self.register.read_word(HL);
        self.mmu.write_byte(hl, self.register.a);
        self.register.write_word(HL, hl.wrapping_sub(1));
        8
    }

    fn ldd_a_hl(&mut self) -> u32 {
        let hl = self.register.read_word(HL);
        self.register.a = self.mmu.read_byte(hl);
        self.register.write_word(HL, hl.wrapping_sub(1));
        8
    }

    fn push_rp2(&mut self, opcode: u8) -> u32 {
        let p = (opcode & 0b_0011_0000) >> 4;
        let nn = self.read_rp2(p);
        self.push_stack(nn);
        16
    }

    fn pop_rp2(&mut self, opcode: u8) -> u32 {
        let p = (opcode & 0b_0011_0000) >> 4;
        let nn = self.pop_stack();
        self.write_rp2(p, nn);
        12
    }

    fn _add(&mut self, n: u8) {
        let a = self.register.a;
        let h = (a & 0x0f) + (n & 0x0f) > 0x0f;
        let (result, c) = a.overflowing_add(n);
        self.register.a = result;
        self.register.set_flag(Flags::Z, result == 0);
        self.register.set_flag(Flags::N, false);
        self.register.set_flag(Flags::H, h);
        self.register.set_flag(Flags::C, c);
    }

    fn add_a_r(&mut self, opcode: u8) -> u32 {
        let z = opcode & 0b_0000_0111;
        let n = self.read_r(z);
        self._add(n);
        match z {
            6 => 8,
            _ => 4,
        }
    }

    fn add_a_n(&mut self) -> u32 {
        let n = self.fetch_byte();
        self._add(n);
        8
    }

    fn _and(&mut self, n: u8) {
        let result = self.register.a & n;
        self.register.a = result;
        self.register.set_flag(Flags::Z, result == 0);
        self.register.set_flag(Flags::N, false);
        self.register.set_flag(Flags::H, true);
        self.register.set_flag(Flags::C, false);
    }

    fn and_r(&mut self, opcode: u8) -> u32 {
        let z = opcode & 0b_0000_0111;
        let n = self.read_r(z);
        self._and(n);
        match z {
            6 => 8,
            _ => 4,
        }
    }

    fn and_n(&mut self) -> u32 {
        let n = self.fetch_byte();
        self._and(n);
        8
    }

    fn _sub(&mut self, n: u8) {
        let a = self.register.a;
        let (result, c) = a.overflowing_sub(n);
        self.register.a = result;
        self.register.set_flag(Flags::Z, result == 0);
        self.register.set_flag(Flags::N, true);
        self.register.set_flag(Flags::H, (a & 0x0f) < (n & 0x0f));
        self.register.set_flag(Flags::C, c);
    }

    fn sub_r(&mut self, opcode: u8) -> u32 {
        let z = opcode & 0b_0000_0111;
        let n = self.read_r(z);
        self._sub(n);
        match z {
            6 => 8,
            _ => 4,
        }
    }

    fn sub_n(&mut self) -> u32 {
        let n = self.fetch_byte();
        self._sub(n);
        8
    }

    fn _or(&mut self, n: u8) {
        let result = self.register.a | n;
        self.register.a = result;
        self.register.set_flag(Flags::Z, result == 0);
        self.register.set_flag(Flags::N, false);
        self.register.set_flag(Flags::H, false);
        self.register.set_flag(Flags::C, false);
    }

    fn or_r(&mut self, opcode: u8) -> u32 {
        let z = opcode & 0b_0000_0111;
        let n = self.read_r(z);
        self._or(n);
        match z {
            6 => 8,
            _ => 4,
        }
    }

    fn or_n(&mut self) -> u32 {
        let n = self.fetch_byte();
        self._or(n);
        8
    }

    fn _xor(&mut self, n: u8) {
        let result = self.register.a ^ n;
        self.register.a = result;
        self.register.set_flag(Flags::Z, result == 0);
        self.register.set_flag(Flags::N, false);
        self.register.set_flag(Flags::H, false);
        self.register.set_flag(Flags::C, false);
    }

    fn xor_r(&mut self, opcode: u8) -> u32 {
        let z = opcode & 0b_0000_0111;
        let n = self.read_r(z);
        self._xor(n);
        match z {
            6 => 8,
            _ => 4,
        }
    }

    fn xor_n(&mut self) -> u32 {
        let n = self.fetch_byte();
        self._xor(n);
        8
    }

    fn _cp(&mut self, n: u8) {
        let a = self.register.a;
        self.register.set_flag(Flags::Z, a == n);
        self.register.set_flag(Flags::N, true);
        self.register.set_flag(Flags::H, (a & 0x0f) < (n & 0x0f));
        self.register.set_flag(Flags::C, a < n);
    }

    fn cp_r(&mut self, opcode: u8) -> u32 {
        let z = opcode & 0b_0000_0111;
        let n = self.read_r(z);
        self._cp(n);
        match z {
            6 => 8,
            _ => 4,
        }
    }

    fn cp_n(&mut self) -> u32 {
        let n = self.fetch_byte();
        self._cp(n);
        8
    }

    fn inc_r(&mut self, opcode: u8) -> u32 {
        let y = (opcode & 0b_0011_1000) >> 3;
        let n = self.read_r(y);

        let result = n.wrapping_add(1);
        self.write_r(y, result);
        self.register.set_flag(Flags::Z, result == 0);
        self.register.set_flag(Flags::N, false);
        self.register.set_flag(Flags::H, n & 0x0f == 0x0f);

        match y {
            6 => 12,
            _ => 4,
        }
    }

    fn dec_r(&mut self, opcode: u8) -> u32 {
        let y = (opcode & 0b_0011_1000) >> 3;
        let n = self.read_r(y);

        let result = n.wrapping_sub(1);
        self.write_r(y, result);
        self.register.set_flag(Flags::Z, result == 0);
        self.register.set_flag(Flags::N, true);
        self.register.set_flag(Flags::H, n & 0x0f == 0x0f);

        match y {
            6 => 12,
            _ => 4,
        }
    }

    fn inc_rp(&mut self, opcode: u8) -> u32 {
        let p = (opcode & 0b_0011_0000) >> 4;
        let nn = self.read_rp(p);
        self.write_rp(p, nn.wrapping_add(1));
        8
    }

    fn dec_rp(&mut self, opcode: u8) -> u32 {
        let p = (opcode & 0b_0011_0000) >> 4;
        let nn = self.read_rp(p);
        self.write_rp(p, nn.wrapping_sub(1));
        8
    }

    fn rla(&mut self) -> u32 {
        let a = self.register.a;
        let c = if self.register.get_flag(Flags::C) {
            1
        } else {
            0
        };

        let result = (a << 1) | c;
        self.register.a = result;
        self.register.set_flag(Flags::Z, result == 0);
        self.register.set_flag(Flags::N, false);
        self.register.set_flag(Flags::H, false);
        self.register.set_flag(Flags::C, a & (1 << 7) == (1 << 7));
        4
    }

    fn rl_n(&mut self, opcode: u8) -> u32 {
        let z = opcode & 0b_0000_0111;
        let n = self.read_r(z);
        let c = if self.register.get_flag(Flags::C) {
            1
        } else {
            0
        };
        let result = (n << 1) | c;
        self.write_r(z, result);
        self.register.set_flag(Flags::Z, result == 0);
        self.register.set_flag(Flags::N, false);
        self.register.set_flag(Flags::H, false);
        self.register.set_flag(Flags::C, n & (1 << 7) == (1 << 7));

        match z {
            6 => 16,
            _ => 8,
        }
    }

    fn bit_b_r(&mut self, opcode: u8) -> u32 {
        let b = (opcode & 0b_0011_1000) >> 3;
        let r8_idx = opcode & 0b_0000_0111;
        let reg = self.read_r(r8_idx);
        let result = reg & (1 << b);
        self.register.set_flag(Flags::Z, result == 0);
        self.register.set_flag(Flags::N, false);
        self.register.set_flag(Flags::H, true);

        match r8_idx {
            6 => 12,
            _ => 8,
        }
    }

    fn jp_nn(&mut self) -> u32 {
        let nn = self.fetch_word();
        self.register.pc = nn;
        16
    }

    fn jr_n(&mut self) -> u32 {
        let n = self.fetch_byte() as i8;
        self.register.pc = self.register.pc.wrapping_add(n as u16);
        12
    }

    fn jr_cc_n(&mut self, opcode: u8) -> u32 {
        let y = (opcode & 0b_0011_1000) >> 3;
        let cc = self.read_cc(y - 4);
        let n = self.fetch_byte() as i8;

        if cc {
            self.register.pc = self.register.pc.wrapping_add(n as u16);
            12
        } else {
            8
        }
    }

    fn call_nn(&mut self) -> u32 {
        let nn = self.fetch_word();
        self.push_stack(self.register.pc);
        self.register.pc = nn;
        24
    }

    fn ret(&mut self) -> u32 {
        let nn = self.pop_stack();
        self.register.pc = nn;
        16
    }

    fn swap_r(&mut self, opcode: u8) -> u32 {
        let z = opcode & 0b_0000_00111;
        let n = self.read_r(z);
        let result = ((n & 0x0f) << 4) | ((n & 0xf0) >> 4);
        self.register.set_flag(Flags::Z, result == 0);
        self.register.set_flag(Flags::N, false);
        self.register.set_flag(Flags::H, false);
        self.register.set_flag(Flags::C, false);
        match z {
            6 => 16,
            _ => 8,
        }
    }

    fn cpl(&mut self) -> u32 {
        self.register.a != self.register.a;
        self.register.set_flag(Flags::N, true);
        self.register.set_flag(Flags::H, true);
        4
    }

    fn scf(&mut self) -> u32 {
        self.register.set_flag(Flags::N, false);
        self.register.set_flag(Flags::H, false);
        self.register.set_flag(Flags::C, true);
        4
    }

    fn di(&mut self) -> u32 {
        self.ime = false;
        4
    }

    fn ei(&mut self) -> u32 {
        self.ime = true;
        4
    }
}
