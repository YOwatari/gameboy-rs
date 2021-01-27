mod register;

use crate::cpu::register::Register;
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
        let b = self.mmu.read_byte(self.register.pc);
        self.register.pc = self.register.pc.wrapping_add(1);
        b
    }

    fn execute(&mut self, opcode: u8) -> u32 {
        match opcode {
            _ => unimplemented!("unknown opcode: {:x}\ncpu: {:?}", opcode, self),
        }
    }
}
