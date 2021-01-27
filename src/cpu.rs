use crate::mmu::MMU;

pub struct CPU {
    pub mmu: MMU,
}

impl CPU {
    pub(crate) fn new(bios: Vec<u8>, rom: Vec<u8>) -> CPU {
        CPU {
            mmu: MMU::new(bios, rom),
        }
    }
}
