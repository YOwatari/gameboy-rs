extern crate clap;
extern crate env_logger;
extern crate log;
extern crate minifb;

use log::info;
use minifb::{Key, KeyRepeat, Scale, Window, WindowOptions};
use std::fs::File;
use std::io::Read;
use std::path;
use std::{thread, time};

mod apu;
mod cartridge;
mod cpu;
mod joypad;
mod mmu;
mod ppu;
mod serial;
mod timer;

use crate::cpu::CPU;
use crate::joypad::KeyInput;
use crate::ppu::{SCREEN_HEIGHT, SCREEN_WIDTH};

const CPU_CYCLES_PER_FRAME: u32 = 70224;

fn key(key: Key) -> Option<KeyInput> {
    match key {
        Key::Down => Some(KeyInput::DOWN),
        Key::Up => Some(KeyInput::UP),
        Key::Left => Some(KeyInput::LEFT),
        Key::Right => Some(KeyInput::RIGHT),
        Key::Space => Some(KeyInput::START),
        Key::Enter => Some(KeyInput::SELECT),
        Key::Z => Some(KeyInput::B),
        Key::X => Some(KeyInput::A),
        _ => None,
    }
}

fn open_rom_file(filepath: &str) -> Vec<u8> {
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

    let matches = clap::App::new("rust-gameboy")
        .arg(
            clap::Arg::with_name("rom")
                .takes_value(true)
                .required(true)
                .long("rom"),
        )
        .arg(
            clap::Arg::with_name("bios")
                .takes_value(true)
                .required(false)
                .long("bios"),
        )
        .arg(
            clap::Arg::with_name("headless")
                .takes_value(false)
                .required(false)
                .long("headless"),
        )
        .get_matches();

    let opt_headless = matches.is_present("headless");
    let opt_bios = matches.is_present("bios");
    let rom_file = matches.value_of("rom").unwrap();

    let rom = open_rom_file(rom_file);
    let mut cpu = CPU::new(rom);

    if opt_bios {
        let bios_file = matches.value_of("bios").unwrap();
        let bios = open_rom_file(bios_file);
        cpu.mmu.cartridge.load_bios(bios);
    } else {
        cpu.init();
    }

    if opt_headless {
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

            window.get_keys_pressed(KeyRepeat::Yes).map(|keys| {
                for k in keys {
                    key(k).map(|input| cpu.mmu.joypad.key_down(input));
                }
            });

            window.get_keys_released().map(|keys| {
                for k in keys {
                    key(k).map(|input| cpu.mmu.joypad.key_up(input));
                }
            });

            sleep(now);
        }
    }
}
