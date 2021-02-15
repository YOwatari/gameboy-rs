use bitflags::bitflags;
use std::cmp::Ordering;
use std::env::split_paths;
use std::fmt;

const VRAM_SIZE: usize = 8 * 1024;
pub const OAM_SIZE: usize = 160;
pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;
pub const SCREEN_PIXELS: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

pub struct PPU {
    vram: [u8; VRAM_SIZE],
    pub oam: [u8; OAM_SIZE],
    mode: Mode,
    bgp: u8,
    obp0: u8,
    obp1: u8,
    clocks: u32,
    ly: u8,
    lyc: u8,
    stat: Stat,
    scy: u8,
    scx: u8,
    control: Control,
    wy: u8,
    wx: u8,
    pub frame_buffer: [u32; SCREEN_PIXELS],
    pub interrupt_vblank: bool,
    pub interrupt_lcdc: bool,
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

struct Sprite {
    addr: usize,
    y: u8,
    x: u8,
    tile_number: u8,
    flags: SpriteFlags,
}

bitflags!(
    struct SpriteFlags: u8 {
        const PRIORITY = 0b_1000_0000;
        const FLIP_Y   = 0b_0100_0000;
        const FLIP_X   = 0b_0010_0000;
        const PALETTE  = 0b_0001_0000;
    }
);

#[derive(Debug, Eq, PartialEq)]
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
            oam: [0; OAM_SIZE],
            mode: Mode::HBlank,
            bgp: 0,
            obp0: 0,
            obp1: 0,
            clocks: 0,
            ly: 0,
            lyc: 0,
            stat: Stat::empty(),
            scy: 0,
            scx: 0,
            control: Control::empty(),
            wy: 0,
            wx: 0,
            frame_buffer: [0; SCREEN_PIXELS],
            interrupt_vblank: false,
            interrupt_lcdc: false,
        }
    }

    fn check_lyc(&mut self) {
        if self.ly == self.lyc {
            self.stat.set(Stat::LYC_FLAG, true);

            if self.stat.contains(Stat::LYC_INTERRUPT) {
                self.interrupt_lcdc = true;
            }
        } else {
            self.stat.set(Stat::LYC_FLAG, false);
        }
    }

    fn check_interrupt(&mut self) {
        match self.mode {
            Mode::HBlank => self.interrupt_lcdc = true,
            Mode::VBlank => self.interrupt_lcdc = true,
            Mode::AccessOAM => self.interrupt_lcdc = true,
            _ => (),
        }
    }

    pub fn run(&mut self, tick: u32) {
        if !self.control.contains(Control::LCD_ENABLE) {
            return;
        }
        self.clocks += tick;

        match self.mode {
            Mode::AccessOAM => {
                if self.clocks >= 80 {
                    self.clocks -= 80;
                    self.mode = Mode::AccessVRAM;
                    self.render_line();
                }
            }
            Mode::AccessVRAM => {
                if self.clocks >= 172 {
                    self.clocks -= 172;
                    self.mode = Mode::HBlank;
                    self.check_interrupt();
                }
            }
            Mode::HBlank => {
                if self.clocks >= 204 {
                    self.clocks -= 204;
                    self.ly = self.ly.wrapping_add(1);

                    if self.ly >= SCREEN_HEIGHT as u8 {
                        self.mode = Mode::VBlank;
                        self.interrupt_vblank = true;
                    } else {
                        self.mode = Mode::AccessOAM;
                    }

                    self.check_lyc();
                    self.check_interrupt();
                }
            }
            Mode::VBlank => {
                if self.clocks >= 456 {
                    self.clocks -= 456;
                    self.ly = self.ly.wrapping_add(1);

                    if self.ly >= SCREEN_HEIGHT as u8 + 10 {
                        self.mode = Mode::AccessOAM;
                        self.ly = 0;
                        self.check_interrupt();
                    }

                    self.check_lyc();
                }
            }
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x8000..=0x9fff => {
                if self.mode == Mode::AccessVRAM {
                    return 0xff;
                }
                self.vram[(addr & (VRAM_SIZE as u16 - 1)) as usize]
            }
            0xfe00..=0xfe9f => {
                if self.mode != Mode::HBlank && self.mode != Mode::VBlank {
                    return 0xff;
                }
                self.oam[(addr & (OAM_SIZE as u16 - 1)) as usize]
            }
            0xff40 => self.control.bits,
            0xff41 => self.stat.bits,
            0xff42 => self.scy,
            0xff43 => self.scx,
            0xff44 => self.ly,
            0xff45 => self.lyc,
            0xff47 => self.bgp,
            0xff48 => self.obp0,
            0xff49 => self.obp1,
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
            0xfe00..=0xfe9f => {
                if self.mode != Mode::HBlank && self.mode != Mode::VBlank {
                    return;
                }
                self.oam[(addr & 0x00ff) as usize] = v;
            }
            0xff40 => {
                let val = Control::from_bits_truncate(v);
                if self.control.contains(Control::LCD_ENABLE) != val.contains(Control::LCD_ENABLE) {
                    self.ly = 0;
                    self.clocks = 0;
                    self.mode = if val.contains(Control::LCD_ENABLE) {
                        Mode::AccessOAM
                    } else {
                        Mode::HBlank
                    };
                    self.check_interrupt();
                }
                self.control = val;
            }
            0xff41 => self.stat = Stat::from_bits_truncate(v & 0b_1111_1000),
            0xff42 => self.scy = v,
            0xff43 => self.scx = v,
            0xff44 => (), // read only
            0xff45 => {
                if self.lyc != v {
                    self.lyc = v;
                    self.check_lyc();
                }
            }
            0xff47 => self.bgp = v,
            0xff48 => self.obp0 = v,
            0xff49 => self.obp1 = v,
            0xff4a => self.wy = v,
            0xff4b => self.wx = v,
            _ => unreachable!("write: not support address: {:04x} {:02x}", addr, v),
        }
    }

    pub fn is_lcd_on(&self) -> bool {
        self.control.contains(Control::LCD_ENABLE)
    }

    fn palette_color(palette: u8, number: u8) -> u32 {
        match (palette >> (number << 1)) & 0b_0011 {
            0 => 0xffffff, // while
            1 => 0xaaaaaa, // light gray
            2 => 0x555555, // dark gray
            3 => 0x000000, // black
            _ => 0xffffff,
        }
    }

    fn tile_addr(control: Control, number: u8) -> usize {
        if control.contains(Control::BG_WINDOW_TILE) {
            // block0: 0x0000-0x07ff block1: 0x0800-0x0fff
            (((number as i8 as i16) << 4) as u16 & 0x0fff) as usize
        } else {
            // block2: 0x1000-0x17ff block1: 0x0800-0x0fff
            0x1000_u16.wrapping_add(((number as i8 as i16) << 4) as u16) as usize
        }
    }

    fn render_line(&mut self) {
        let start = SCREEN_WIDTH * (self.ly as usize);
        let end = start + SCREEN_WIDTH;
        let pixels = &mut self.frame_buffer[start..end];
        let mut priority = [false; SCREEN_WIDTH];

        if self.control.contains(Control::BG_ENABLE) {
            let tile_map_addr_base = if self.control.contains(Control::BG_TILE_MAP) {
                0x1c00 // 0x1c00-0x1fff
            } else {
                0x1800 // 0x1800-0x1bff
            };

            let y = self.ly.wrapping_add(self.scy);
            let row = (y / 8) as usize;

            for i in 0..SCREEN_WIDTH as u8 {
                let x = i.wrapping_add(self.scx);
                let col = (x / 8) as usize;

                let tile_number = self.vram[((row * 32 + col) | tile_map_addr_base) & 0x1fff];
                let tile_addr = PPU::tile_addr(self.control, tile_number);

                let line = ((y % 8) * 2) as usize;
                let data0 = self.vram[(tile_addr | line) & 0x1fff];
                let data1 = self.vram[(tile_addr | (line + 1)) & 0x1fff];

                let color_mask = (0b_0111 - (x & 0b_0111)) as usize;
                let color_num = (((data0 >> color_mask) & 1) << 1) | ((data1 >> color_mask) & 1);
                let color = PPU::palette_color(self.bgp, color_num);

                priority[i as usize] = color_num != 0;
                pixels[i as usize] = color;
            }
        }

        if self.control.contains(Control::WINDOW_ENABLE) && self.wy <= self.ly {
            let tile_map_addr_base = if self.control.contains(Control::WINDOW_TILE_MAP) {
                0x1c00 // 0x1c00-0x1fff
            } else {
                0x1800 // 0x1800-0x1bff
            };
            let wx = self.wx.wrapping_add(7);

            let y = self.ly - self.wy;
            let row = (y / 8) as usize;
            for i in wx..SCREEN_WIDTH as u8 {
                let mut x = i.wrapping_add(self.scx);
                if x >= wx {
                    x = i - wx;
                }
                let col = (x / 8) as usize;

                let tile_number = self.vram[((row * 32 + col) | tile_map_addr_base) & 0x1fff];
                let tile_addr = PPU::tile_addr(self.control, tile_number);

                let line = ((y % 8) * 2) as usize;
                let data0 = self.vram[(tile_addr | line) & 0x1fff];
                let data1 = self.vram[(tile_addr | (line + 1)) & 0x1fff];

                let color_mask = (0b_0111 - (x & 0b_0111)) as usize;
                let color_num = (((data0 >> color_mask) & 1) << 1) | ((data1 >> color_mask) & 1);
                let color = PPU::palette_color(self.bgp, color_num);

                priority[i as usize] = color_num != 0;
                pixels[i as usize] = color;
            }
        }

        if self.control.contains(Control::OBJ_ENABLE) {
            let size: u8 = if self.control.contains(Control::OBJ_SIZE) {
                16
            } else {
                8
            };

            // load sprites on line
            let mut sprite_num = 0;
            let mut sprites: Vec<Sprite> = Vec::with_capacity(10);
            for i in 0..(OAM_SIZE / 4) {
                let addr = i << 2;
                let mut sprite = Sprite {
                    addr,
                    y: self.oam[addr].wrapping_sub(16),
                    x: self.oam[addr + 1].wrapping_sub(8),
                    tile_number: 0,
                    flags: SpriteFlags::from_bits_truncate(self.oam[addr + 3]),
                };

                if self.ly.wrapping_sub(sprite.y) > size {
                    // not on line
                    continue;
                }

                sprite_num += 1;
                if sprite_num > 10 {
                    // 10 sprite on 1 line
                    break;
                }

                if sprite.x == 0 || SCREEN_WIDTH as u8 - 1 < sprite.x {
                    // out of screen
                    continue;
                }

                sprite.tile_number = if self.control.contains(Control::OBJ_SIZE) {
                    // 8x16
                    if (self.ly < sprite.y) ^ sprite.flags.contains(SpriteFlags::FLIP_Y) {
                        self.oam[addr + 2] & 0xfe
                    } else {
                        self.oam[addr + 2] | 0x01
                    }
                } else {
                    // 8x8
                    self.oam[addr + 2]
                };
                sprites.push(sprite);
            }

            // sprites priority
            sprites.sort_by(|a, b| {
                match a.x.cmp(&b.x) {
                    // low index
                    Ordering::Equal => a.addr.cmp(&b.addr).reverse(),
                    // low x
                    other => other.reverse(),
                }
            });

            for sprite in sprites.iter() {
                let mut line = if sprite.flags.contains(SpriteFlags::FLIP_Y) {
                    7 - (self.ly.wrapping_sub(sprite.y) & 0x07)
                } else {
                    self.ly.wrapping_sub(sprite.y) & 0x07
                } as u16;
                line *= 2;
                let tile_addr = PPU::tile_addr(self.control, sprite.tile_number) as u16;
                let data0 = self.vram[((tile_addr | line) & 0x1fff) as usize];
                let data1 = self.vram[((tile_addr | (line + 1)) & 0x1fff) as usize];

                let palette = if sprite.flags.contains(SpriteFlags::PALETTE) {
                    self.obp1
                } else {
                    self.obp0
                };
                for x in (0..7).rev() {
                    let color_mask = if sprite.flags.contains(SpriteFlags::FLIP_X) {
                        7 - (x & 0x07)
                    } else {
                        x & 0x07
                    } as usize;
                    let color_num =
                        (((data0 >> color_mask) & 1) << 1) | ((data1 >> color_mask) & 1);
                    let color = PPU::palette_color(palette, color_num);
                    let target = sprite.x.wrapping_add(7 - x) - 1;

                    if color_num == 0 {
                        continue;
                    }
                    if sprite.flags.contains(SpriteFlags::PRIORITY) && priority[target as usize] {
                        continue;
                    }
                    pixels[target as usize] = color;
                }
            }
        }
    }
}

impl fmt::Debug for PPU {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "PPU: {{ lcdc: {:02x}, scy: {:02x}, scx: {:02x}, bgp: {:02x}, wy: {:02x}, wx: {:02x} }}",
            self.control.bits, self.scy, self.scx, self.bgp, self.wy, self.wx,
        )
    }
}
