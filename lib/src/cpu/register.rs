use bitflags::bitflags;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Registers {
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

pub enum Registers16 {
    AF,
    BC,
    DE,
    HL,
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
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

    pub fn read_word(&self, reg: Registers16) -> u16 {
        match reg {
            Registers16::AF => (self.a as u16) << 8 | self.f.bits() as u16,
            Registers16::BC => (self.b as u16) << 8 | self.c as u16,
            Registers16::DE => (self.d as u16) << 8 | self.e as u16,
            Registers16::HL => (self.h as u16) << 8 | self.l as u16,
        }
    }

    pub fn write_word(&mut self, reg: Registers16, v: u16) {
        match reg {
            Registers16::AF => {
                self.a = (v >> 8) as u8;
                self.f = Flags::from_bits_truncate(v as u8);
            }
            Registers16::BC => {
                self.b = (v >> 8) as u8;
                self.c = v as u8;
            }
            Registers16::DE => {
                self.d = (v >> 8) as u8;
                self.e = v as u8;
            }
            Registers16::HL => {
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
