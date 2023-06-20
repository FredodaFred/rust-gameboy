////////////////////////////////////
/// 
/// 
/// bus.rs
/// 
/// Sources:
/// 
/// Memory Map
/// 0x0000 - 0x3FFF : ROM Bank 0
/// 0x4000 - 0x7FFF : ROM Bank 1 - Switchable
/// 0x8000 - 0x97FF : CHR RAM
/// 0x9800 - 0x9BFF : BG Map 1
/// 0x9C00 - 0x9FFF : BG Map 2
/// 0xA000 - 0xBFFF : Cartridge RAM
/// 0xC000 - 0xCFFF : RAM Bank 0
/// 0xD000 - 0xDFFF : RAM Bank 1-7 - switchable - Color only
/// 0xE000 - 0xFDFF : Reserved - Echo RAM
/// 0xFE00 - 0xFE9F : Object Attribute Memory
/// 0xFEA0 - 0xFEFF : Reserved - Unusable
/// 0xFF00 - 0xFF7F : I/O Registers
/// 0xFF80 - 0xFFFE : Zero Page
/// 
use crate::{cart::Cart, io::IO, ppu::GPU};

const WRAMSIZE: usize = 0x2000;
const HRAMSIZE: usize = 0x80;
const VRAM_BEGIN: usize = 0x8000;
pub struct Bus {
    wram: [u8; WRAMSIZE],
    hram: [u8; HRAMSIZE],
    cart: Cart,
    io: IO,
    // ie_mirror: u8, //maybe use interrupts in bus
    gpu: GPU
}

impl Bus {
    pub fn new(p_cart: Cart, p_io: IO, p_gpu: GPU)-> Self{
        Self {
            wram: [0; WRAMSIZE],
            hram: [0; HRAMSIZE],
            cart: p_cart,
            io: p_io,
            // ie_mirror: 0x0,
            gpu: p_gpu,
        }
    }
    pub fn write(&mut self, addr: u16, data: u8){
        match addr {
            // 0x0000..0x8000 => todo!("Write to Cart"),
            // 0x8000..0xA000 => todo!("Char Map Data"),
            // 0xA000..0xC000 => todo!("Catridge RAM"),
            // 0xC000..0xE000 => todo!("WRAM write"),
            // 0xE000..0xFE00 => todo!("echo ram"),
            // 0xFE00..0xFEA0 => todo!("OAM"),
            // 0xFEA0..0xFF00 => println!("Map to unusable memory"),
            // 0xFF00..0xFF80 => todo!("io write"),
            // 0xFFFF => todo!("set ie reg"),
            // _ => println!("Out of bounds memory read.")
            0x0000..=0x7FFF => self.cart.write(addr, data),
            0x8000..=0x9FFF => self.gpu.write(addr - (VRAM_BEGIN as u16), data),
            0xA000..=0xBFFF => self.cart.write(addr, data),
            0xC000..=0xDFFF => self.wram_write(addr, data),
            0xE000..=0xFDFF => return,
            0xFE00..=0xFE9F => println!("OAM-TODO"),
            0xFEA0..=0xFEFF => println!("Map to unusable memory"),
            0xFF00..=0xFF7F => self.io_write(addr, data),
            0xFFFF => self.ie_mirror = data,
            _ => self.hram_write(addr, data)
        }
    }
    /**
     * Read from the bus
     */
    pub fn read(&mut self, addr: u16) -> u8{
        match addr {
            0x0000..=0x7FFF => self.cart.read(addr),
            0x8000..=0x9FFF => self.gpu.read_vram(addr - (VRAM_BEGIN as u16)),
            0xA000..=0xBFFF => self.cart.read(addr),
            0xC000..=0xDFFF => self.wram_read(addr),
            0xE000..=0xFDFF => return 0,
            0xFE00..=0xFE9F => {println!("Todo-OAM"); 0},
            0xFEA0..=0xFEFF => panic!("Map to unusable memory"),
            0xFF00..=0xFF7F => self.io_read(addr),
            0xFFFF => self.ie_mirror,
            _ => self.hram_read(addr)
        }
    }

    fn wram_write(&mut self, addr: u16, data: u8){
        let new_addr: u16 = addr - 0xC000;
        if new_addr >= 0x2000{
            panic!("Invalid wram address {:#02X}", new_addr);
        }
        self.wram[new_addr as usize] = data;
    }

    fn wram_read(&mut self, addr: u16) -> u8{
        let new_addr: u16 = addr - 0xC000;
        if new_addr >= 0x2000{
            panic!("Invalid wram address {:#02X}", new_addr);
        }
        return self.wram[new_addr as usize];
    }
    
    fn hram_write(&mut self, addr: u16, data: u8){
        let new_addr = addr- 0xFF80;
        self.hram[new_addr as usize] = data;
    }
    
    fn hram_read(&mut self, addr: u16) -> u8 {
        let new_addr = addr- 0xFF80;
        return self.hram[new_addr as usize];
    }

    fn io_write(&mut self, addr: u16, val: u8){
        self.io.write(addr, val);
    }

    fn io_read(&mut self, addr: u16)->u8{
        return self.io.read(addr);
    }
    pub fn get_ie_set(&mut self) -> u8{
        let ret: u8 =  self.ie_mirror;
        self.ie_mirror = 0;
        return ret;
    }
}