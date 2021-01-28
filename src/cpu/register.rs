use bitflags::bitflags;
use std::fmt;

pub struct Register {
    pub a: u8,
    f: Flags,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16,
}

bitflags!(
    pub struct Flags: u8 {
        const Z = 0b_1000_0000;
        const N = 0b_0100_0000;
        const H = 0b_0010_0000;
        const C = 0b_0001_0000;
    }
);

pub enum Register16 {
    AF,
    BC,
    DE,
    HL,
}

impl Register {
    pub fn new() -> Register {
        Register {
            a: 0x00,
            f: Flags::empty(),
            b: 0x00,
            c: 0x00,
            d: 0x00,
            e: 0x00,
            h: 0x00,
            l: 0x00,
            sp: 0x0000,
            pc: 0x0000,
        }
    }

    pub fn read_word(&self, reg: Register16) -> u16 {
        match reg {
            Register16::AF => (self.a as u16) << 8 | self.f.bits() as u16,
            Register16::BC => (self.b as u16) << 8 | self.c as u16,
            Register16::DE => (self.d as u16) << 8 | self.e as u16,
            Register16::HL => (self.h as u16) << 8 | self.l as u16,
        }
    }

    pub fn write_word(&mut self, reg: Register16, v: u16) {
        match reg {
            Register16::AF => {
                self.a = (v >> 8) as u8;
                self.f = Flags::from_bits_truncate(v as u8);
            }
            Register16::BC => {
                self.b = (v >> 8) as u8;
                self.c = v as u8;
            }
            Register16::DE => {
                self.d = (v >> 8) as u8;
                self.e = v as u8;
            }
            Register16::HL => {
                self.h = (v >> 8) as u8;
                self.l = v as u8;
            }
        }
    }

    pub fn get_flag(&self, f: Flags) -> bool {
        self.f.contains(f)
    }

    pub fn set_flag(&mut self, f: Flags, v: bool) {
        self.f.set(f, v);
    }
}

impl fmt::Debug for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Register {{ \
a: 0x{:02x}, f: 0x{:02x}, b: 0x{:02x}, c: 0x{:02x}, d: 0x{:02x}, e: 0x{:02x}, h: 0x{:02x}, l: 0x{:02x}, \
sp: 0x{:04x}, pc: 0x{:04x}, \
Z: {:?} N: {:?}, H: {:?}, C: {:?} }}",
            self.a, self.f, self.b, self.c, self.d, self.e, self.h, self.l,
            self.sp, self.pc,
            self.get_flag(Flags::Z),
            self.get_flag(Flags::N),
            self.get_flag(Flags::H),
            self.get_flag(Flags::C),
        )
    }
}
