const VRAM_SIZE: usize = 8 * 1024;

pub struct PPU {
    vram: [u8; VRAM_SIZE],
    mode: Mode,
}

#[derive(Eq, PartialEq)]
enum Mode {
    HBlank,
    VBlank,
    AccessOam,
    AccessVram,
}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            vram: [0; VRAM_SIZE],
            mode: Mode::HBlank,
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x8000..=0x9fff => {
                if self.mode == Mode::AccessVram {
                    return 0xff;
                }
                self.vram[(addr & (VRAM_SIZE as u16 - 1)) as usize]
            }
            _ => 0xff,
        }
    }

    pub fn write_byte(&mut self, addr: u16, v: u8) {
        match addr {
            0x8000..=0x9fff => {
                if self.mode == Mode::AccessVram {
                    return;
                }
                self.vram[(addr & (VRAM_SIZE as u16 - 1)) as usize] = v;
            }
            _ => unreachable!("write: not support address: 0x{:04x}", addr),
        }
    }
}
