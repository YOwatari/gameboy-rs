extern crate env_logger;
extern crate log;
extern crate minifb;

use log::info;
use minifb::{Key, Scale, Window, WindowOptions};
use std::fs::File;
use std::io::Read;
use std::path;
use std::{env, thread, time};

mod apu;
mod cartridge;
mod cpu;
mod joypad;
mod mmu;
mod ppu;
mod serial;
mod timer;

use crate::cpu::CPU;
use crate::ppu::{SCREEN_HEIGHT, SCREEN_WIDTH};

const CPU_CYCLES_PER_FRAME: u32 = 70224;

fn open_rom_file(filepath: &String) -> Vec<u8> {
    let mut data = Vec::<u8>::new();
    let p = path::PathBuf::from(filepath);
    let mut f = File::open(p).unwrap();
    let size = f.read_to_end(&mut data).unwrap();
    info!("file: {} size: {}", filepath, size);
    data
}

fn sleep(now: time::Instant) {
    // elapsed time per frame at 60fps
    let wait = time::Duration::from_micros(1 / 60 * 1000 * 1000);
    let elapsed = now.elapsed();
    if wait > elapsed {
        thread::sleep(wait - elapsed);
    }
}

fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    let bios = open_rom_file(&args[1]);
    let rom = open_rom_file(&args[2]);

    let mut cpu = CPU::new(bios, rom);

    if &args[3] == "--headless" {
        // for debug
        loop {
            let now = time::Instant::now();
            let mut elapsed_tick: u32 = 0;

            while elapsed_tick < CPU_CYCLES_PER_FRAME {
                elapsed_tick += cpu.run() as u32;
            }

            sleep(now);
        }
    } else {
        let mut window = Window::new(
            "rust-gameboy",
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            WindowOptions {
                scale: Scale::X2,
                ..WindowOptions::default()
            },
        )
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });
        window.set_position(200, 200);

        while window.is_open() && !window.is_key_down(Key::Escape) {
            let now = time::Instant::now();
            let mut elapsed_tick: u32 = 0;

            while elapsed_tick < CPU_CYCLES_PER_FRAME {
                elapsed_tick += cpu.run() as u32;
            }

            if cpu.mmu.ppu.is_lcd_on() {
                let buffer = Vec::from(cpu.mmu.ppu.frame_buffer);
                window
                    .update_with_buffer(&buffer, SCREEN_WIDTH, SCREEN_HEIGHT)
                    .unwrap();
            }

            sleep(now);
        }
    }
}
