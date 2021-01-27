mod register;

use crate::cpu::register::Register;
use crate::mmu::MMU;

#[derive(Debug)]
pub struct CPU {
    register: Register,
    pub mmu: MMU,
}

impl CPU {
    pub(crate) fn new(bios: Vec<u8>, rom: Vec<u8>) -> CPU {
        CPU {
            register: Register::new(),
            mmu: MMU::new(bios, rom),
        }
    }
}
