mod register;

use crate::cpu::register::Flags;
use crate::cpu::register::Register;
use crate::cpu::register::Register16::{AF, BC, DE, HL};
use crate::mmu::{Interrupt, MMU};

#[derive(Debug)]
pub struct CPU {
    register: Register,
    pub mmu: MMU,
    ime: bool,
    ei: u8,
    di: u8,
    halted: bool,
}

impl CPU {
    pub fn new(rom: Vec<u8>) -> CPU {
        CPU {
            register: Register::new(),
            mmu: MMU::new(rom),
            ime: false,
            ei: 0,
            di: 0,
            halted: false,
        }
    }

    pub fn init(&mut self) {
        self.register.write_word(AF, 0x01b0);
        self.register.write_word(BC, 0x0013);
        self.register.write_word(DE, 0x00d8);
        self.register.write_word(HL, 0x014d);
        self.register.sp = 0xfffe;
        self.register.pc = 0x0100;
        self.mmu.init();
    }

    pub fn run(&mut self) -> u32 {
        let ticks = self.run_with_interrupt();
        self.mmu.run(ticks);
        ticks
    }

    pub fn run_with_interrupt(&mut self) -> u32 {
        self.update_ime();
        match self.handle_interrupt() {
            0 => (),
            n => return n,
        };

        if self.halted {
            self.execute(0x00) // nop
        } else {
            let opcode = self.fetch_byte();
            self.execute(opcode)
        }
    }

    fn update_ime(&mut self) {
        self.di = match self.di {
            2 => 1,
            1 => {
                self.ime = false;
                0
            }
            _ => 0,
        };
        self.ei = match self.ei {
            2 => 1,
            1 => {
                self.ime = true;
                0
            }
            _ => 0,
        };
    }

    fn handle_interrupt(&mut self) -> u32 {
        if !self.ime && !self.halted {
            return 0;
        }

        let request = self.mmu.interrupt_enable.bits() & self.mmu.interrupt_flag.bits();
        if request == 0 {
            return 0;
        }

        self.halted = false;
        if !self.ime {
            return 0;
        }
        self.ime = false;

        let interrupt = Interrupt::from_bits_truncate(request);
        self.mmu.interrupt_flag.set(interrupt, false);
        match interrupt {
            Interrupt::VBLANK => self._call(0x0040),
            Interrupt::LCD_STAT => self._call(0x0048),
            Interrupt::TIMER => self._call(0x0050),
            Interrupt::SERIAL => self._call(0x0080),
            Interrupt::JOYPAD => self._call(0x0070),
            _ => unreachable!("Invalid interrupt request: {:?}", interrupt),
        };
        16
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
            0x01 | 0x11 | 0x21 | 0x31 => self.ld_r16_d16(opcode),
            0x06 | 0x0e | 0x16 | 0x1e | 0x26 | 0x2e | 0x36 | 0x3e => self.ld_r8_d8(opcode),

            0x40..=0x46 | 0x48..=0x4e | 0x50..=0x56 | 0x58..=0x5e => self.ld_r8_r8(opcode),
            0x60..=0x66 | 0x68..=0x6e | 0x70..=0x75 => self.ld_r8_r8(opcode),
            0x78..=0x7f => self.ld_a_r8(opcode),
            0x47 | 0x4f | 0x57 | 0x5f | 0x67 | 0x6f | 0x77 => self.ld_r8_a(opcode),

            0x02 | 0x12 => self.ld_r16_a(opcode),
            0x0a | 0x1a => self.ld_a_r16(opcode),

            0xfa => self.ld_a_d16(),
            0xea => self.ld_d16_a(),

            0xe0 => self.ldh_d8_a(),
            0xf0 => self.ldh_a_d8(),
            0xe2 => self.ldh_c_a(),
            0xf2 => self.ldh_a_c(),

            0x22 => self.ldi_hl_a(),
            0x2a => self.ldi_a_hl(),
            0x32 => self.ldd_hl_a(),
            0x3a => self.ldd_a_hl(),

            0x08 => self.ld_d16_sp(),
            0xf8 => self.ld_hl_sp_d8(),
            0xf9 => self.ld_sp_hl(),
            0xc1 | 0xd1 | 0xe1 | 0xf1 => self.pop(opcode),
            0xc5 | 0xd5 | 0xe5 | 0xf5 => self.push(opcode),

            // arithmetic
            0x09 | 0x19 | 0x29 | 0x39 => self.add_hl(opcode),
            0xe8 => self.add_sp(),

            0x80..=0x87 => self.add(opcode),
            0x88..=0x8f => self.adc(opcode),
            0x90..=0x97 => self.sub(opcode),
            0x98..=0x9f => self.sbc(opcode),
            0xa0..=0xa7 => self.and(opcode),
            0xa8..=0xaf => self.xor(opcode),
            0xb0..=0xb7 => self.or(opcode),
            0xb8..=0xbf => self.cp(opcode),

            0xc6 => self.add_d8(),
            0xce => self.adc_d8(),
            0xd6 => self.sub_d8(),
            0xde => self.sbc_d8(),
            0xe6 => self.and_d8(),
            0xee => self.xor_d8(),
            0xf6 => self.or_d8(),
            0xfe => self.cp_d8(),

            0x04 | 0x0c | 0x14 | 0x1c | 0x24 | 0x2c | 0x34 | 0x3c => self.inc8(opcode),
            0x05 | 0x0d | 0x15 | 0x1d | 0x25 | 0x2d | 0x35 | 0x3d => self.dec8(opcode),
            0x03 | 0x13 | 0x23 | 0x33 => self.inc16(opcode),
            0x0b | 0x1b | 0x2b | 0x3b => self.dec16(opcode),

            // rotates & shifts
            0x07 => self.rlca(),
            0x0f => self.rrca(),
            0x17 => self.rla(),
            0x1f => self.rra(),

            // jumps
            0xc3 => self.jp_nn(),
            0xc2 | 0xca | 0xd2 | 0xda => self.jp_cc_nn(opcode),
            0xe9 => self.jp_hl(),
            0x18 => self.jr_n(),
            0x20 | 0x28 | 0x30 | 0x38 => self.jr_cc_n(opcode),

            // calls
            0xc4 | 0xcc | 0xd4 | 0xdc => self.call_cc(opcode),
            0xcd => self.call(),

            // restarts
            0xc7 | 0xcf | 0xd7 | 0xdf | 0xe7 | 0xef | 0xf7 | 0xff => self.rst(opcode),

            // returns
            0xc0 | 0xc8 | 0xd0 | 0xd8 => self.ret_cc(opcode),
            0xc9 => self.ret(),
            0xd9 => self.reti(),

            // miscellaneous
            0x00 => 4, // nop
            0x10 => 4, // stop
            0x27 => self.dda(),
            0x2f => self.cpl(),
            0x37 => self.scf(),
            0x3f => self.ccf(),
            0x76 => self.halt(),
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
            0x00..=0x07 => self.rlc(opcode),
            0x08..=0x0f => self.rrc(opcode),
            0x10..=0x17 => self.rl(opcode),
            0x18..=0x1f => self.rr(opcode),
            0x20..=0x27 => self.sla(opcode),
            0x28..=0x2f => self.sra(opcode),
            0x30..=0x37 => self.swap(opcode),
            0x38..=0x3f => self.srl(opcode),
            // bit
            0x40..=0x7f => self.bit(opcode),
            0x80..=0xbf => self.res(opcode),
            0xc0..=0xff => self.set(opcode),
            _ => unimplemented!("unknown cb opcode: 0x{:02x}\ncput: {:?}", opcode, self),
        }
    }

    fn ld_r16_d16(&mut self, opcode: u8) -> u32 {
        let p = (opcode & 0b_0011_0000) >> 4;
        let nn = self.fetch_word();
        self.write_rp(p, nn);
        12
    }

    fn ld_r8_d8(&mut self, opcode: u8) -> u32 {
        let y = (opcode & 0b_0011_1000) >> 3;
        let n = self.fetch_byte();
        self.write_r(y, n);
        match y {
            6 => 12,
            _ => 8,
        }
    }

    fn ld_r8_r8(&mut self, opcode: u8) -> u32 {
        let z = opcode & 0b_0000_0111;
        let y = (opcode & 0b_0011_1000) >> 3;
        let v = self.read_r(z);
        self.write_r(y, v);

        match (y, z) {
            (6, _) => 8,
            (_, 6) => 8,
            _ => 4,
        }
    }

    fn ld_a_r8(&mut self, opcode: u8) -> u32 {
        let z = opcode & 0b_0000_0111;
        let n = self.read_r(z);
        self.register.a = n;
        match z {
            6 => 8,
            _ => 4,
        }
    }

    fn ld_a_r16(&mut self, opcode: u8) -> u32 {
        let p = (opcode & 0b_0011_0000) >> 4;
        let nn = self.read_rp(p);
        self.register.a = self.mmu.read_byte(nn);
        8
    }

    fn ld_a_d16(&mut self) -> u32 {
        let nn = self.fetch_word();
        self.register.a = self.mmu.read_byte(nn);
        16
    }

    fn ld_r8_a(&mut self, opcode: u8) -> u32 {
        let y = (opcode & 0b_0011_1000) >> 3;
        self.write_r(y, self.register.a);
        match y {
            6 => 8,
            _ => 4,
        }
    }

    fn ld_r16_a(&mut self, opcode: u8) -> u32 {
        let p = (opcode & 0b_0011_0000) >> 4;
        let nn = self.read_rp(p);
        self.mmu.write_byte(nn, self.register.a);
        8
    }

    fn ld_d16_a(&mut self) -> u32 {
        let nn = self.fetch_word();
        self.mmu.write_byte(nn, self.register.a);
        16
    }

    fn ldh_c_a(&mut self) -> u32 {
        let addr = 0xff00 | (self.register.c as u16);
        self.mmu.write_byte(addr, self.register.a);
        8
    }

    fn ldh_a_c(&mut self) -> u32 {
        let addr = 0xff00 | (self.register.c as u16);
        let n = self.mmu.read_byte(addr);
        self.register.a = n;
        8
    }

    fn ldh_d8_a(&mut self) -> u32 {
        let n = self.fetch_byte();
        self.mmu.write_byte(0xff00 | (n as u16), self.register.a);
        12
    }

    fn ldh_a_d8(&mut self) -> u32 {
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

    fn ld_sp_hl(&mut self) -> u32 {
        let hl = self.register.read_word(HL);
        self.register.sp = hl;
        8
    }

    fn ld_hl_sp_d8(&mut self) -> u32 {
        let n = self.fetch_byte() as i8;
        let v = n as u16;
        let h = (self.register.sp & 0x0f) + (v & 0x0f) > 0x0f;
        let c = (self.register.sp & 0xff) + (v & 0xff) > 0xff;
        self.register
            .write_word(HL, self.register.sp.wrapping_add(v));
        self.register.set_flag(Flags::Z, false);
        self.register.set_flag(Flags::N, false);
        self.register.set_flag(Flags::H, h);
        self.register.set_flag(Flags::C, c);
        12
    }

    fn ld_d16_sp(&mut self) -> u32 {
        let nn = self.fetch_word();
        self.mmu.write_word(nn, self.register.sp);
        20
    }

    fn push(&mut self, opcode: u8) -> u32 {
        let p = (opcode & 0b_0011_0000) >> 4;
        let nn = self.read_rp2(p);
        self.push_stack(nn);
        16
    }

    fn pop(&mut self, opcode: u8) -> u32 {
        let p = (opcode & 0b_0011_0000) >> 4;
        let nn = self.pop_stack();
        self.write_rp2(p, nn);
        12
    }

    fn add_hl(&mut self, opcode: u8) -> u32 {
        let p = (opcode & 0b_0011_0000) >> 4;
        let nn = self.read_rp(p);
        let hl = self.register.read_word(HL);

        let h = (hl & 0x0fff) + (nn & 0x0fff) > 0x0fff;
        let (result, c) = hl.overflowing_add(nn);

        self.register.write_word(HL, result);
        self.register.set_flag(Flags::N, false);
        self.register.set_flag(Flags::H, h);
        self.register.set_flag(Flags::C, c);
        8
    }

    fn add_sp(&mut self) -> u32 {
        let n = self.fetch_byte() as i8;
        let v = n as u16;
        let h = (self.register.sp & 0x0f) + (v & 0x0f) > 0x0f;
        let c = (self.register.sp & 0xff) + (v & 0xff) > 0xff;
        self.register.sp = self.register.sp.wrapping_add(v);
        self.register.set_flag(Flags::Z, false);
        self.register.set_flag(Flags::N, false);
        self.register.set_flag(Flags::H, h);
        self.register.set_flag(Flags::C, c);
        16
    }

    fn _adc(&mut self, n: u8) {
        let a = self.register.a;
        let carry = if self.register.get_flag(Flags::C) {
            1
        } else {
            0
        };
        let result = a.wrapping_add(n).wrapping_add(carry);
        let h = (a & 0x0f) + (n & 0x0f) + carry > 0x0f;
        let c = (a as u16) + (n as u16) + (carry as u16) > 0xff;
        self.register.a = result;
        self.register.set_flag(Flags::Z, result == 0);
        self.register.set_flag(Flags::N, false);
        self.register.set_flag(Flags::H, h);
        self.register.set_flag(Flags::C, c);
    }

    fn adc_d8(&mut self) -> u32 {
        let n = self.fetch_byte();
        self._adc(n);
        8
    }

    fn adc(&mut self, opcode: u8) -> u32 {
        let z = opcode & 0b_0000_0111;
        let n = self.read_r(z);
        self._adc(n);
        match z {
            6 => 8,
            _ => 4,
        }
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

    fn add(&mut self, opcode: u8) -> u32 {
        let z = opcode & 0b_0000_0111;
        let n = self.read_r(z);
        self._add(n);
        match z {
            6 => 8,
            _ => 4,
        }
    }

    fn add_d8(&mut self) -> u32 {
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

    fn and(&mut self, opcode: u8) -> u32 {
        let z = opcode & 0b_0000_0111;
        let n = self.read_r(z);
        self._and(n);
        match z {
            6 => 8,
            _ => 4,
        }
    }

    fn and_d8(&mut self) -> u32 {
        let n = self.fetch_byte();
        self._and(n);
        8
    }

    fn _sub(&mut self, n: u8) {
        let a = self.register.a;
        let h = (a & 0x0f) < (n & 0x0f);
        let (result, c) = a.overflowing_sub(n);
        self.register.a = result;
        self.register.set_flag(Flags::Z, result == 0);
        self.register.set_flag(Flags::N, true);
        self.register.set_flag(Flags::H, h);
        self.register.set_flag(Flags::C, c);
    }

    fn sub(&mut self, opcode: u8) -> u32 {
        let z = opcode & 0b_0000_0111;
        let n = self.read_r(z);
        self._sub(n);
        match z {
            6 => 8,
            _ => 4,
        }
    }

    fn sub_d8(&mut self) -> u32 {
        let n = self.fetch_byte();
        self._sub(n);
        8
    }

    fn _sbc(&mut self, n: u8) {
        let a = self.register.a;
        let carry = if self.register.get_flag(Flags::C) {
            1
        } else {
            0
        };
        let result = a.wrapping_sub(n).wrapping_sub(carry);
        let h = (a & 0x0f) < (n & 0x0f) + carry;
        let c = (a as u16) < (n as u16) + (carry as u16);
        self.register.a = result;
        self.register.set_flag(Flags::Z, result == 0);
        self.register.set_flag(Flags::N, true);
        self.register.set_flag(Flags::H, h);
        self.register.set_flag(Flags::C, c);
    }

    fn sbc(&mut self, opcode: u8) -> u32 {
        let z = opcode & 0b_0000_0111;
        let n = self.read_r(z);
        self._sbc(n);
        match z {
            6 => 8,
            _ => 4,
        }
    }

    fn sbc_d8(&mut self) -> u32 {
        let n = self.fetch_byte();
        self._sbc(n);
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

    fn or(&mut self, opcode: u8) -> u32 {
        let z = opcode & 0b_0000_0111;
        let n = self.read_r(z);
        self._or(n);
        match z {
            6 => 8,
            _ => 4,
        }
    }

    fn or_d8(&mut self) -> u32 {
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

    fn xor(&mut self, opcode: u8) -> u32 {
        let z = opcode & 0b_0000_0111;
        let n = self.read_r(z);
        self._xor(n);
        match z {
            6 => 8,
            _ => 4,
        }
    }

    fn xor_d8(&mut self) -> u32 {
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

    fn cp(&mut self, opcode: u8) -> u32 {
        let z = opcode & 0b_0000_0111;
        let n = self.read_r(z);
        self._cp(n);
        match z {
            6 => 8,
            _ => 4,
        }
    }

    fn cp_d8(&mut self) -> u32 {
        let n = self.fetch_byte();
        self._cp(n);
        8
    }

    fn inc8(&mut self, opcode: u8) -> u32 {
        let y = (opcode & 0b_0011_1000) >> 3;
        let n = self.read_r(y);
        let result = n.wrapping_add(1);
        self.write_r(y, result);
        self.register.set_flag(Flags::Z, result == 0);
        self.register.set_flag(Flags::N, false);
        self.register.set_flag(Flags::H, (n & 0x0f) + 1 > 0x0f);
        match y {
            6 => 12,
            _ => 4,
        }
    }

    fn dec8(&mut self, opcode: u8) -> u32 {
        let y = (opcode & 0b_0011_1000) >> 3;
        let n = self.read_r(y);
        let result = n.wrapping_sub(1);
        self.write_r(y, result);
        self.register.set_flag(Flags::Z, result == 0);
        self.register.set_flag(Flags::N, true);
        self.register.set_flag(Flags::H, (n & 0x0f) == 0x00);
        match y {
            6 => 12,
            _ => 4,
        }
    }

    fn inc16(&mut self, opcode: u8) -> u32 {
        let p = (opcode & 0b_0011_0000) >> 4;
        let nn = self.read_rp(p);
        self.write_rp(p, nn.wrapping_add(1));
        8
    }

    fn dec16(&mut self, opcode: u8) -> u32 {
        let p = (opcode & 0b_0011_0000) >> 4;
        let nn = self.read_rp(p);
        self.write_rp(p, nn.wrapping_sub(1));
        8
    }

    fn _rlc(&mut self, r: u8) {
        let n = self.read_r(r);
        let result = n.rotate_left(1);
        self.write_r(r, result);
        self.register.set_flag(Flags::Z, result == 0);
        self.register.set_flag(Flags::N, false);
        self.register.set_flag(Flags::H, false);
        self.register.set_flag(Flags::C, n & 0x80 != 0);
    }

    fn rlca(&mut self) -> u32 {
        self._rlc(7);
        self.register.set_flag(Flags::Z, false);
        4
    }

    fn rlc(&mut self, opcode: u8) -> u32 {
        let z = opcode & 0b_0000_0111;
        self._rlc(z);
        match z {
            6 => 16,
            _ => 8,
        }
    }

    fn _rl(&mut self, r: u8) {
        let n = self.read_r(r);
        let c: u8 = if self.register.get_flag(Flags::C) {
            1
        } else {
            0
        };
        let result = (n << 1) | c;
        self.write_r(r, result);
        self.register.set_flag(Flags::Z, result == 0);
        self.register.set_flag(Flags::N, false);
        self.register.set_flag(Flags::H, false);
        self.register.set_flag(Flags::C, n & 0x80 != 0);
    }

    fn rla(&mut self) -> u32 {
        self._rl(7);
        self.register.set_flag(Flags::Z, false);
        4
    }

    fn rl(&mut self, opcode: u8) -> u32 {
        let z = opcode & 0b_0000_0111;
        self._rl(z);
        match z {
            6 => 16,
            _ => 8,
        }
    }

    fn _rrc(&mut self, r: u8) {
        let n = self.read_r(r);
        let result = n.rotate_right(1);
        self.write_r(r, result);
        self.register.set_flag(Flags::Z, result == 0);
        self.register.set_flag(Flags::N, false);
        self.register.set_flag(Flags::H, false);
        self.register.set_flag(Flags::C, n & 1 != 0);
    }

    fn rrca(&mut self) -> u32 {
        self._rrc(7);
        self.register.set_flag(Flags::Z, false);
        4
    }

    fn rrc(&mut self, opcode: u8) -> u32 {
        let z = opcode & 0b_0000_0111;
        self._rrc(z);
        match z {
            6 => 16,
            _ => 8,
        }
    }

    fn _rr(&mut self, r: u8) {
        let n = self.read_r(r);
        let c: u8 = if self.register.get_flag(Flags::C) {
            1
        } else {
            0
        };
        let result = (n >> 1) | (c << 7);
        self.write_r(r, result);
        self.register.set_flag(Flags::Z, result == 0);
        self.register.set_flag(Flags::N, false);
        self.register.set_flag(Flags::H, false);
        self.register.set_flag(Flags::C, n & 1 != 0);
    }

    fn rra(&mut self) -> u32 {
        self._rr(7);
        self.register.set_flag(Flags::Z, false);
        4
    }

    fn rr(&mut self, opcode: u8) -> u32 {
        let z = opcode & 0b_0000_0111;
        self._rr(z);
        match z {
            6 => 16,
            _ => 8,
        }
    }

    fn sla(&mut self, opcode: u8) -> u32 {
        let z = opcode & 0b_0000_0111;
        let n = self.read_r(z);
        let result = n << 1;
        self.write_r(z, result);
        self.register.set_flag(Flags::Z, result == 0);
        self.register.set_flag(Flags::N, false);
        self.register.set_flag(Flags::H, false);
        self.register.set_flag(Flags::C, n & 0x80 != 0);
        match z {
            6 => 16,
            _ => 8,
        }
    }

    fn sra(&mut self, opcode: u8) -> u32 {
        let z = opcode & 0b_0000_0111;
        let n = self.read_r(z);
        let result = (n >> 1) | (n & 0x80);
        self.write_r(z, result);
        self.register.set_flag(Flags::Z, result == 0);
        self.register.set_flag(Flags::N, false);
        self.register.set_flag(Flags::H, false);
        self.register.set_flag(Flags::C, n & 1 != 0);
        match z {
            6 => 16,
            _ => 8,
        }
    }

    fn swap(&mut self, opcode: u8) -> u32 {
        let z = opcode & 0b_0000_00111;
        let n = self.read_r(z);
        let result = ((n & 0x0f) << 4) | ((n & 0xf0) >> 4);
        self.write_r(z, result);
        self.register.set_flag(Flags::Z, result == 0);
        self.register.set_flag(Flags::N, false);
        self.register.set_flag(Flags::H, false);
        self.register.set_flag(Flags::C, false);
        match z {
            6 => 16,
            _ => 8,
        }
    }

    fn srl(&mut self, opcode: u8) -> u32 {
        let z = opcode & 0b_0000_0111;
        let n = self.read_r(z);
        let result = n >> 1;
        self.write_r(z, result);
        self.register.set_flag(Flags::Z, result == 0);
        self.register.set_flag(Flags::N, false);
        self.register.set_flag(Flags::H, false);
        self.register.set_flag(Flags::C, n & 1 != 0);
        match z {
            6 => 16,
            _ => 8,
        }
    }

    fn bit(&mut self, opcode: u8) -> u32 {
        let y = (opcode & 0b_0011_1000) >> 3;
        let z = opcode & 0b_0000_0111;
        let v = self.read_r(z);
        let result = (v >> y) & 1;
        self.register.set_flag(Flags::Z, result == 0);
        self.register.set_flag(Flags::N, false);
        self.register.set_flag(Flags::H, true);

        match z {
            6 => 12,
            _ => 8,
        }
    }

    fn res(&mut self, opcode: u8) -> u32 {
        let y = (opcode & 0b_0011_1000) >> 3;
        let z = opcode & 0b_0000_0111;
        let v = self.read_r(z);
        let result = v & !(1 << y);
        self.write_r(z, result);
        match z {
            6 => 16,
            _ => 8,
        }
    }

    fn set(&mut self, opcode: u8) -> u32 {
        let y = (opcode & 0b_0011_1000) >> 3;
        let z = opcode & 0b_0000_0111;
        let v = self.read_r(z);
        let result = v | (1 << y);
        self.write_r(z, result);
        match z {
            6 => 16,
            _ => 8,
        }
    }

    fn jp_nn(&mut self) -> u32 {
        let nn = self.fetch_word();
        self.register.pc = nn;
        16
    }

    fn jp_cc_nn(&mut self, opcode: u8) -> u32 {
        let y = (opcode & 0b_0011_1000) >> 3;
        let cc = self.read_cc(y);
        let nn = self.fetch_word();
        if cc {
            self.register.pc = nn;
            16
        } else {
            12
        }
    }

    fn jp_hl(&mut self) -> u32 {
        let hl = self.register.read_word(HL);
        self.register.pc = hl;
        4
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

    fn _call(&mut self, addr: u16) {
        self.push_stack(self.register.pc);
        self.register.pc = addr;
    }

    fn call_cc(&mut self, opcode: u8) -> u32 {
        let y = (opcode & 0b_0011_1000) >> 3;
        let cc = self.read_cc(y);
        let addr = self.fetch_word();
        if cc {
            self._call(addr);
            24
        } else {
            12
        }
    }

    fn call(&mut self) -> u32 {
        let addr = self.fetch_word();
        self._call(addr);
        24
    }

    fn rst(&mut self, opcode: u8) -> u32 {
        let y = (opcode & 0b_0011_1000) >> 3;
        let addr = y * 8;
        self._call(addr as u16);
        16
    }

    fn ret(&mut self) -> u32 {
        let nn = self.pop_stack();
        self.register.pc = nn;
        16
    }

    fn ret_cc(&mut self, opcode: u8) -> u32 {
        let y = (opcode & 0b_0011_1000) >> 3;
        let cc = self.read_cc(y);
        if cc {
            let nn = self.pop_stack();
            self.register.pc = nn;
            20
        } else {
            8
        }
    }

    fn dda(&mut self) -> u32 {
        let mut a = self.register.a;
        let mut adjust = 0u8;
        if self.register.get_flag(Flags::C) {
            adjust |= 0x60;
        }
        if self.register.get_flag(Flags::H) {
            adjust |= 0x06;
        };
        if !self.register.get_flag(Flags::N) {
            if a & 0x0f > 0x09 {
                adjust |= 0x06;
            };
            if a > 0x99 {
                adjust |= 0x60;
            };
            a = a.wrapping_add(adjust);
        } else {
            a = a.wrapping_sub(adjust);
        }

        self.register.a = a;
        self.register.set_flag(Flags::Z, a == 0);
        self.register.set_flag(Flags::H, false);
        self.register.set_flag(Flags::C, adjust >= 0x60);
        4
    }

    fn ccf(&mut self) -> u32 {
        let c = self.register.get_flag(Flags::C);
        self.register.set_flag(Flags::N, false);
        self.register.set_flag(Flags::H, false);
        self.register.set_flag(Flags::C, !c);
        4
    }

    fn cpl(&mut self) -> u32 {
        self.register.a = !self.register.a;
        self.register.set_flag(Flags::N, true);
        self.register.set_flag(Flags::H, true);
        4
    }

    fn halt(&mut self) -> u32 {
        self.halted = true;
        4
    }

    fn scf(&mut self) -> u32 {
        self.register.set_flag(Flags::N, false);
        self.register.set_flag(Flags::H, false);
        self.register.set_flag(Flags::C, true);
        4
    }

    fn di(&mut self) -> u32 {
        self.di = 2;
        4
    }

    fn ei(&mut self) -> u32 {
        self.ei = 2;
        4
    }

    fn reti(&mut self) -> u32 {
        let nn = self.pop_stack();
        self.register.pc = nn;
        self.ei = 1;
        16
    }
}
