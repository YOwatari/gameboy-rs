use bitflags::bitflags;
use std::fmt;

const VRAM_SIZE: usize = 8 * 1024;
pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;
pub const SCREEN_PIXELS: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

pub struct PPU {
    pub vram: [u8; VRAM_SIZE],
    mode: Mode,
    bgp: u8,
    clocks: u32,
    ly: u8,
    stat: Stat,
    scy: u8,
    scx: u8,
    control: Control,
    wy: u8,
    wx: u8,
    pub frame_buffer: [u32; SCREEN_PIXELS],
}

bitflags!(
    struct Control: u8 {
        const LCD_ENABLE      = 0b_1000_0000;
        const WINDOW_TILE_MAP = 0b_0100_0000;
        const WINDOW_ENABLE   = 0b_0010_0000;
        const BG_WINDOW_TILE  = 0b_0001_0000;
        const BG_TILE_MAP     = 0b_0000_1000;
        const OBJ_SIZE        = 0b_0000_0100;
        const OBJ_ENABLE      = 0b_0000_0010;
        const BG_ENABLE       = 0b_0000_0001;
    }
);

bitflags!(
    struct Stat: u8 {
        const LYC_INTERRUPT    = 0b_0100_0000;
        const OAM_INTERRUPT    = 0b_0010_0000;
        const VBLANK_INTERRUPT = 0b_0001_0000;
        const HBLANK_INTERRUPT = 0b_0000_1000;
        const LYC_FLAG         = 0b_0000_0100;
        const HBLANK_MODE      = 0b_0000_0000;
        const VBLANK_MODE      = 0b_0000_0001;
        const ACCESS_OAM_MODE  = 0b_0000_0010;
        const ACCESS_VRAM_MODE = 0b_0000_0011;
    }
);

#[derive(Eq, PartialEq)]
enum Mode {
    HBlank,
    VBlank,
    AccessOAM,
    AccessVRAM,
}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            vram: [0; VRAM_SIZE],
            mode: Mode::HBlank,
            bgp: 0,
            clocks: 0,
            ly: 0,
            stat: Stat::empty(),
            scy: 0,
            scx: 0,
            control: Control::empty(),
            wy: 0,
            wx: 0,
            frame_buffer: [0; SCREEN_PIXELS],
        }
    }

    pub fn run(&mut self, tick: u32) {
        self.clocks += tick;

        match self.mode {
            Mode::AccessOAM => {
                if self.clocks >= 80 {
                    self.clocks -= 80;
                    self.mode = Mode::AccessVRAM;
                }
            }
            Mode::AccessVRAM => {
                if self.clocks >= 172 {
                    self.clocks -= 172;
                    self.render_line();
                    self.mode = Mode::HBlank;
                    // interrupt
                }
            }
            Mode::HBlank => {
                if self.clocks >= 204 {
                    self.clocks -= 204;
                    self.ly = self.ly.wrapping_add(1);

                    if self.ly >= SCREEN_HEIGHT as u8 {
                        self.mode = Mode::VBlank;
                    // interrupt
                    } else {
                        self.mode = Mode::AccessOAM;
                    }
                    // interrupt
                }
            }
            Mode::VBlank => {
                if self.clocks >= 456 {
                    self.clocks -= 456;
                    self.ly = self.ly.wrapping_add(1);

                    if self.ly >= SCREEN_HEIGHT as u8 + 10 {
                        self.mode = Mode::AccessOAM;
                        self.ly = 0;
                        // interrupt
                    }
                    // interrupt
                }
            }
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x8000..=0x9fff => {
                /*if self.mode == Mode::AccessVRAM {
                    return 0xff;
                }*/
                self.vram[(addr & (VRAM_SIZE as u16 - 1)) as usize]
            }
            0xff40 => self.control.bits,
            0xff42 => self.scy,
            0xff43 => self.scx,
            0xff44 => self.ly,
            0xff47 => self.bgp,
            0xff4a => self.wy,
            0xff4b => self.wx,
            _ => 0xff,
        }
    }

    pub fn write_byte(&mut self, addr: u16, v: u8) {
        match addr {
            0x8000..=0x9fff => {
                /*if self.mode == Mode::AccessVRAM {
                    return;
                }*/
                self.vram[(addr & (VRAM_SIZE as u16 - 1)) as usize] = v;
            }
            0xff40 => {
                let val = Control::from_bits_truncate(v);
                if self.control.contains(Control::LCD_ENABLE) != val.contains(Control::LCD_ENABLE) {
                    self.ly = 0;
                    self.clocks = 0;
                    let mode = if val.contains(Control::LCD_ENABLE) {
                        Stat::ACCESS_OAM_MODE
                    } else {
                        Stat::HBLANK_MODE
                    };
                    self.stat.insert(mode);
                    // interrupt
                }
                self.control = val;
            }
            0xff42 => self.scy = v,
            0xff43 => self.scx = v,
            0xff44 => (), // read only
            0xff47 => self.bgp = v,
            0xff4a => self.wy = v,
            0xff4b => self.wx = v,
            _ => unreachable!("write: not support address: 0x{:04x}", addr),
        }
    }

    pub fn is_lcd_on(&self) -> bool {
        self.control.contains(Control::LCD_ENABLE)
    }

    fn render_line(&mut self) {
        let start = SCREEN_WIDTH * (self.ly as usize);
        let end = start + SCREEN_WIDTH;
        let pixels = &mut self.frame_buffer[start..end];

        if self.control.contains(Control::BG_ENABLE) {
            let tile_map_addr_base = if self.control.contains(Control::BG_TILE_MAP) {
                0x1c00 // 0x1c00-0x1fff 32x32 bytes
            } else {
                0x1800 // 0x1800-0x1bff 32x32 bytes
            };

            let y = self.ly.wrapping_add(self.scy);
            let row = (y / 8) as usize;

            for i in 0..SCREEN_WIDTH as u8 {
                let x = i.wrapping_add(self.scx);
                let col = (x / 8) as usize;

                let tile_number = self.vram[((row * 32 + col) | tile_map_addr_base) & 0x1fff];
                let tile_addr = if self.control.contains(Control::BG_WINDOW_TILE) {
                    // block0: 0x0000-0x07ff block1: 0x0800-0x0fff
                    (((tile_number as i8 as i16) << 4) as u16 & 0x0fff) as usize
                } else {
                    // block2: 0x1000-0x17ff block1: 0x0800-0x0fff
                    0x1000_u16.wrapping_add(((tile_number as i8 as i16) << 4) as u16) as usize
                };

                let line = ((y % 8) * 2) as usize;
                let data0 = self.vram[(tile_addr | line) & 0x1fff];
                let data1 = self.vram[(tile_addr | (line + 1)) & 0x1fff];

                let color_mask = (0b_0111 - (x & 0b_0111)) as usize;
                let color_num = (((data0 >> color_mask) & 1) << 1) | ((data1 >> color_mask) & 1);
                let color = match (self.bgp >> (color_num << 1)) & 0b_0011 {
                    0 => 0xffffff, // while
                    1 => 0xaaaaaa, // light gray
                    2 => 0x555555, // dark gray
                    3 => 0x000000, // black
                    _ => 0xffffff,
                };

                pixels[i as usize] = color;
            }
        }

        if self.control.contains(Control::OBJ_ENABLE) {
            // TODO
        }
    }
}

impl fmt::Debug for PPU {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "PPU: {{ \
lcdc: {:02x}, scy: {:02x}, scx: {:02x}, \
bgp: {:02x}, \
wy: {:02x}, wx: {:02x} }}",
            self.control.bits, self.scy, self.scx, self.bgp, self.wy, self.wx,
        )
    }
}
