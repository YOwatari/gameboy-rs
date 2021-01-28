extern crate env_logger;
extern crate log;

use log::info;
use std::fs::File;
use std::io::Read;
use std::path;
use std::{env, thread, time};

mod cartridge;
mod cpu;
mod mmu;
mod ppu;

use crate::cpu::CPU;

const CPU_CYCLES_PER_FRAME: u32 = 70224;

fn open_rom_file(filepath: &String) -> Vec<u8> {
    let mut data = Vec::<u8>::new();
    let p = path::PathBuf::from(filepath);
    let mut f = File::open(p).unwrap();
    let size = f.read_to_end(&mut data).unwrap();
    info!("file: {} size: {}", filepath, size);
    data
}

fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    let bios = open_rom_file(&args[1]);
    let rom = open_rom_file(&args[2]);

    let mut cpu = CPU::new(bios, rom);

    loop {
        let now = time::Instant::now();
        let mut elapsed_tick: u32 = 0;

        while elapsed_tick < CPU_CYCLES_PER_FRAME {
            elapsed_tick += cpu.run() as u32;
        }

        // elapsed time per frame at 60fps
        let wait = time::Duration::from_micros(1 / 60 * 1000 * 1000);
        let elapsed = now.elapsed();
        if wait > elapsed {
            thread::sleep(wait - elapsed);
        }
    }
}
