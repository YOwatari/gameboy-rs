extern crate env_logger;
extern crate log;

use log::info;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path;

mod cartridge;
mod cpu;
mod mmu;

use crate::cpu::CPU;

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

    let cpu = CPU::new(bios, rom);

    println!("0x0000: {:x}", cpu.mmu.read_byte(0x0000));
    println!("0x0104: {:x}", cpu.mmu.read_byte(0x0104));
}
