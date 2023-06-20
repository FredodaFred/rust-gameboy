////////////////
/// 
/// cart.rs
/// 
/// Sources:
/// https://gbdev.io/pandocs/The_Cartridge_Header.html - Locating data in the header
/// 
/// 

use std::fs::File;
use std::io::Read;

use std::str::from_utf8;

pub struct Cart {
  rom: Vec<u8>,
  cartloaded: bool,
  title: String,
  rom_size: u8,
  ram_size: u8,
  dest_code: u8,
  lic_code: u8,
  version: u8,
  checksum: u16,
}

impl Cart{
  pub fn new() -> Cart{
    Cart {
      rom: Vec::new(),
      cartloaded: false,
      title: String::new(),
      rom_size: 0,
      ram_size: 0,
      dest_code: 0,
      lic_code: 0,
      version: 0,
      checksum: 0,
    }
  }
  pub fn load_rom(&mut self, name: String){
    self.read_rom(name);
    self.load_header();
    self.cartloaded = true;
  }
  pub fn read(&mut self, addr: u16)->u8{
    if !self.cartloaded {
      panic!("Read from unloaded Cart");
    }
    return self.rom[addr as usize];
  }
  pub fn write(&mut self, addr: u16, data: u8){
    if !self.cartloaded {
      panic!("Write to unloaded Cart");
    }
    self.rom[addr as usize] = data;
  }

  fn read_rom(&mut self, path: String){
    let mut file: File = File::open(path).expect("Can't open file!");
    file.read_to_end(&mut self.rom).expect("Couldn't read file");
  }
  fn load_header(&mut self){

    //header data
    let header: Vec<u8> = self.rom[0x0104..0x0150].to_vec();
    // Title
    let title_str: &str = std::str::from_utf8(&self.rom[0x0134..0x0143]).unwrap();
    let mut title: String = String::new();
    title_str.clone_into(&mut title);
    self.title = title.clone();

    //Ram and Rom Size
    self.ram_size = self.rom[0x0148];
    self.rom_size = self.rom[0x0149];
    self.dest_code = self.rom[0x014A];
    self.lic_code = self.rom[0x014B];
    self.version = self.rom[0x014C];

    //Checksum
    let mut checksum: u8 = 0;
    for byte in 0x0134..0x014D{
        let val: u8 = self.rom[byte];
        checksum = checksum.wrapping_sub(val).wrapping_sub(1);
    }
    self.checksum = self.checksum;

    println!("Cartridge Loaded");
    println!("\tTitle: {}", self.title);
    println!("\tRom Size: {}", self.rom_size);
    println!("\tRam Size: {}", self.ram_size);
    println!("\tVersion: {}", self.version);
    println!("Checksum: {}", if checksum == self.rom[0x014D] {"Passed"} else {"Failed"})
    //for (uint16_t i=0x0134; i<=0x014C; i++) { x = x - rom_data[i] - 1; } 

  }
}