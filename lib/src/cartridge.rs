use log::info;

pub struct Cartridge {
    bios: Vec<u8>,
    rom: Vec<u8>,
    rom_banks: u8,
    rom_bank_number_hi: u8,
    rom_bank_number_lo: u8,
    ram: Vec<u8>,
    ram_enable: bool,
    mode: bool,
}

impl Cartridge {
    pub fn new(rom: Vec<u8>) -> Cartridge {
        info!("MBC type: {:02x}", rom[0x147]);
        let ram_size: usize = match rom[0x0149] {
            0 => 0,
            1 => 2 * 1024,
            2 => 8 * 1024,
            3 => 32 * 1024,
            4 => 128 * 1024,
            _ => unreachable!("RAM size is invalid: {:02x}", rom[0x0149]),
        };
        let rom_banks: u8 = match rom[0x0148] {
            0 => 2,
            1 => 4,
            2 => 8,
            3 => 16,
            4 => 32,
            5 => 64,
            6 => 128,
            _ => unreachable!("ROM size is invalid: {:02}", rom[0x148]),
        };

        Cartridge {
            bios: Vec::<u8>::new(),
            rom,
            rom_banks,
            rom_bank_number_hi: 0,
            rom_bank_number_lo: 0,
            ram: vec![0; ram_size],
            ram_enable: false,
            mode: false,
        }
    }

    pub fn load_bios(&mut self, bios: Vec<u8>) {
        self.bios = bios;
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            // BIOS
            0x0000..=0x00ff => {
                if self.bios.len() > 0 {
                    return self.bios[addr as usize];
                }
                self.rom[addr as usize]
            }
            // ROM bank 00
            0x0100..=0x3fff => self.rom[addr as usize],
            // ROM bank 01-1f
            0x4000..=0x7fff => self.rom[(addr & 0x3fff) as usize + self.rom_offset()],
            // RAM bank 00-03
            0xa000..=0xbfff => {
                if !self.ram_enable {
                    return 0xff;
                }
                self.ram[(addr & 0x1fff) as usize + self.ram_offset()]
            }
            _ => 0xff,
        }
    }

    pub fn write_byte(&mut self, addr: u16, v: u8) {
        match addr {
            // RAM enable
            0x0000..=0x1fff => self.ram_enable = v & 0x0f == 0x0a,
            // ROM bank number(lower 5 bit)
            0x2000..=0x3fff => self.rom_bank_number_lo = v & 0x1f,
            // RAM bank number or ROM bank number(higher 2 bit)
            0x4000..=0x5fff => self.rom_bank_number_hi = v & 0x03,
            // ROM/RAM mode (0=ROM, 1=RAM)
            0x6000..=0x7fff => self.mode = v & 0x01 != 0,
            // RAM bank 00-03
            0xa000..=0xbfff => {
                if !self.ram_enable {
                    return;
                }
                let offset = self.ram_offset();
                self.ram[(addr & 0x1fff) as usize + offset] = v;
            }
            _ => (),
        }
    }

    fn rom_offset(&self) -> usize {
        let bank_number = if self.mode {
            self.rom_bank_number_lo
        } else {
            self.rom_bank_number_hi << 5 | self.rom_bank_number_lo
        };
        let bank_number = match bank_number {
            0x00 | 0x20 | 0x40 | 0x60 => bank_number + 1,
            _ => bank_number,
        };
        16 * 1024 * ((bank_number & (self.rom_banks - 1)) as usize)
    }

    fn ram_offset(&self) -> usize {
        if self.mode {
            8 * 1024 * (self.rom_bank_number_hi as usize)
        } else {
            0
        }
    }
}
