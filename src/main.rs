mod bus;
mod cpu;
mod cart;
mod util;
mod log;
mod io;
mod ppu;
mod screen;


use cpu::CPU;
use cart::Cart;
use ppu::GPU;
use bus::Bus;
use io::IO;
use screen::Screen;
use std::env;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    //Load rom file in
    let args:Vec<String> = env::args().collect();
    let rom_path = format!("../roms/{}.gb", &args[1]);
    let mut rom: Cart = Cart::new();
    rom.load_rom(rom_path);
    let mut io: IO = IO::new();
    let mut gpu: GPU = GPU::new();
    let mut screen: Screen = Screen::new();

    //attach necessary items to the bus
    let bus: Bus = Bus::new(rom, io, gpu);

    //give cpu access to bus and run the rom
    let mut cpu: CPU = CPU::new(bus);
    let mut running: bool = true;
    while running {
        cpu.run();
    }
    
}

