pub struct Cartridge {
    bios: Vec<u8>,
    rom: Vec<u8>,
}

impl Cartridge {
    pub fn new(bios: Vec<u8>, rom: Vec<u8>) -> Cartridge {
        Cartridge { bios, rom }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x00ff => self.bios[addr as usize],
            0x0100..=0x7fff => self.rom[addr as usize],
            _ => 0xff,
        }
    }
}
