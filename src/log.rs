use std::fs::File;
use std::io::prelude::*;
use std::io::Result;

pub fn create_file() -> Result<File> {
    let mut file = File::create("log.txt")?;
    file.write_all(b"CPU STARTED\n")?;
    file.flush()?;
    Ok(file)
}
pub struct Logger{
    pub log: File
}
impl Logger{
    
    pub fn write_instr(&mut self, opcode: u8){
        let readable = match opcode {
            0x00 => "NOP\n",
            0x01 => "LD  BC  n16\n" ,
            0x02 => "LD  BC  A\n" ,
            0x03 => "INC  BC\n" ,
            0x04 => "INC  B\n" ,
            0x05 => "DEC  B\n" ,
            0x06 => "LD  B  n8\n" ,
            0x07 => "RLCA\n" ,
            0x08 => "LD  a16  SP\n" ,
            0x09 => "ADD  HL  BC\n" ,
            0x0A => "LD  A  BC\n" ,
            0x0B => "DEC  BC\n" ,
            0x0C => "INC  C\n" ,
            0x0D => "DEC  C\n" ,
            0x0E => "LD  C  n8\n" ,
            0x0F => "RRCA\n" ,
            0x10 => "STOP  n8\n" ,
            0x11 => "LD  DE  n16\n" ,
            0x12 => "LD  DE  A\n" ,
            0x13 => "INC  DE\n" ,
            0x14 => "INC  D\n" ,
            0x15 => "DEC  D\n" ,
            0x16 => "LD  D  n8\n" ,
            0x17 => "RLA\n" ,
            0x18 => "JR  e8\n" ,
            0x19 => "ADD  HL  DE\n" ,
            0x1A => "LD  A  DE\n" ,
            0x1B => "DEC  DE\n" ,
            0x1C => "INC  E\n" ,
            0x1D => "DEC  E\n" ,
            0x1E => "LD  E  n8\n" ,
            0x1F => "RRA\n" ,
            0x20 => "JR  NZ  e8\n" ,
            0x21 => "LD  HL  n16\n" ,
            0x22 => "LD  HL  A\n" ,
            0x23 => "INC  HL\n" ,
            0x24 => "INC  H\n" ,
            0x25 => "DEC  H\n" ,
            0x26 => "LD  H  n8\n" ,
            0x27 => "DAA\n" ,
            0x28 => "JR  Z  e8\n" ,
            0x29 => "ADD  HL  HL\n" ,
            0x2A => "LD  A  HL\n" ,
            0x2B => "DEC  HL\n" ,
            0x2C => "INC  L\n" ,
            0x2D => "DEC  L\n" ,
            0x2E => "LD  L  n8\n" ,
            0x2F => "CPL\n" ,
            0x30 => "JR  NC  e8\n" ,
            0x31 => "LD  SP  n16\n" ,
            0x32 => "LD  HL  A\n" ,
            0x33 => "INC  SP\n" ,
            0x34 => "INC  HL\n" ,
            0x35 => "DEC  HL\n" ,
            0x36 => "LD  HL  n8\n" ,
            0x37 => "SCF\n" ,
            0x38 => "JR  C  e8\n" ,
            0x39 => "ADD  HL  SP\n" ,
            0x3A => "LD  A  HL\n" ,
            0x3B => "DEC  SP\n" ,
            0x3C => "INC  A\n" ,
            0x3D => "DEC  A\n" ,
            0x3E => "LD  A  n8\n" ,
            0x3F => "CCF\n" ,
            0x40 => "LD  B  B\n" ,
            0x41 => "LD  B  C\n" ,
            0x42 => "LD  B  D\n" ,
            0x43 => "LD  B  E\n" ,
            0x44 => "LD  B  H\n" ,
            0x45 => "LD  B  L\n" ,
            0x46 => "LD  B  HL\n" ,
            0x47 => "LD  B  A\n" ,
            0x48 => "LD  C  B\n" ,
            0x49 => "LD  C  C\n" ,
            0x4A => "LD  C  D\n" ,
            0x4B => "LD  C  E\n" ,
            0x4C => "LD  C  H\n" ,
            0x4D => "LD  C  L\n" ,
            0x4E => "LD  C  HL\n" ,
            0x4F => "LD  C  A\n" ,
            0x50 => "LD  D  B\n" ,
            0x51 => "LD  D  C\n" ,
            0x52 => "LD  D  D\n" ,
            0x53 => "LD  D  E\n" ,
            0x54 => "LD  D  H\n" ,
            0x55 => "LD  D  L\n" ,
            0x56 => "LD  D  HL\n" ,
            0x57 => "LD  D  A\n" ,
            0x58 => "LD  E  B\n" ,
            0x59 => "LD  E  C\n" ,
            0x5A => "LD  E  D\n" ,
            0x5B => "LD  E  E\n" ,
            0x5C => "LD  E  H\n" ,
            0x5D => "LD  E  L\n" ,
            0x5E => "LD  E  HL\n" ,
            0x5F => "LD  E  A\n" ,
            0x60 => "LD  H  B\n" ,
            0x61 => "LD  H  C\n" ,
            0x62 => "LD  H  D\n" ,
            0x63 => "LD  H  E\n" ,
            0x64 => "LD  H  H\n" ,
            0x65 => "LD  H  L\n" ,
            0x66 => "LD  H  HL\n" ,
            0x67 => "LD  H  A\n" ,
            0x68 => "LD  L  B\n" ,
            0x69 => "LD  L  C\n" ,
            0x6A => "LD  L  D\n" ,
            0x6B => "LD  L  E\n" ,
            0x6C => "LD  L  H\n" ,
            0x6D => "LD  L  L\n" ,
            0x6E => "LD  L  HL\n" ,
            0x6F => "LD  L  A\n" ,
            0x70 => "LD  HL  B\n" ,
            0x71 => "LD  HL  C\n" ,
            0x72 => "LD  HL  D\n" ,
            0x73 => "LD  HL  E\n" ,
            0x74 => "LD  HL  H\n" ,
            0x75 => "LD  HL  L\n" ,
            0x76 => "HALT\n" ,
            0x77 => "LD  HL  A\n" ,
            0x78 => "LD  A  B\n" ,
            0x79 => "LD  A  C\n" ,
            0x7A => "LD  A  D\n" ,
            0x7B => "LD  A  E\n" ,
            0x7C => "LD  A  H\n" ,
            0x7D => "LD  A  L\n" ,
            0x7E => "LD  A  HL\n" ,
            0x7F => "LD  A  A\n" ,
            0x80 => "ADD  A  B\n" ,
            0x81 => "ADD  A  C\n" ,
            0x82 => "ADD  A  D\n" ,
            0x83 => "ADD  A  E\n" ,
            0x84 => "ADD  A  H\n" ,
            0x85 => "ADD  A  L\n" ,
            0x86 => "ADD  A  HL\n" ,
            0x87 => "ADD  A  A\n" ,
            0x88 => "ADC  A  B\n" ,
            0x89 => "ADC  A  C\n" ,
            0x8A => "ADC  A  D\n" ,
            0x8B => "ADC  A  E\n" ,
            0x8C => "ADC  A  H\n" ,
            0x8D => "ADC  A  L\n" ,
            0x8E => "ADC  A  HL\n" ,
            0x8F => "ADC  A  A\n" ,
            0x90 => "SUB  B\n" ,
            0x91 => "SUB  C\n" ,
            0x92 => "SUB  D\n" ,
            0x93 => "SUB  E\n" ,
            0x94 => "SUB  H\n" ,
            0x95 => "SUB  L\n" ,
            0x96 => "SUB  HL\n" ,
            0x97 => "SUB  A\n" ,
            0x98 => "SBC  A  B\n" ,
            0x99 => "SBC  A  C\n" ,
            0x9A => "SBC  A  D\n" ,
            0x9B => "SBC  A  E\n" ,
            0x9C => "SBC  A  H\n" ,
            0x9D => "SBC  A  L\n" ,
            0x9E => "SBC  A  HL\n" ,
            0x9F => "SBC  A  A\n" ,
            0xA0 => "AND  B\n" ,
            0xA1 => "AND  C\n" ,
            0xA2 => "AND  D\n" ,
            0xA3 => "AND  E\n" ,
            0xA4 => "AND  H\n" ,
            0xA5 => "AND  L\n" ,
            0xA6 => "AND  HL\n" ,
            0xA7 => "AND  A\n" ,
            0xA8 => "XOR  B\n" ,
            0xA9 => "XOR  C\n" ,
            0xAA => "XOR  D\n" ,
            0xAB => "XOR  E\n" ,
            0xAC => "XOR  H\n" ,
            0xAD => "XOR  L\n" ,
            0xAE => "XOR  HL\n" ,
            0xAF => "XOR  A\n" ,
            0xB0 => "OR  B\n" ,
            0xB1 => "OR  C\n" ,
            0xB2 => "OR  D\n" ,
            0xB3 => "OR  E\n" ,
            0xB4 => "OR  H\n" ,
            0xB5 => "OR  L\n" ,
            0xB6 => "OR  HL\n" ,
            0xB7 => "OR  A\n" ,
            0xB8 => "CP  B\n" ,
            0xB9 => "CP  C\n" ,
            0xBA => "CP  D\n" ,
            0xBB => "CP  E\n" ,
            0xBC => "CP  H\n" ,
            0xBD => "CP  L\n" ,
            0xBE => "CP  HL\n" ,
            0xBF => "CP  A\n" ,
            0xC0 => "RET  NZ\n" ,
            0xC1 => "POP  BC\n" ,
            0xC2 => "JP  NZ  a16\n" ,
            0xC3 => "JP  a16\n" ,
            0xC4 => "CALL  NZ  a16\n" ,
            0xC5 => "PUSH  BC\n" ,
            0xC6 => "ADD  A  n8\n" ,
            0xC7 => "RST  00H\n" ,
            0xC8 => "RET  Z\n" ,
            0xC9 => "RET\n" ,
            0xCA => "JP  Z  a16\n" ,
            0xCB => "PREFIX\n" ,
            0xCC => "CALL  Z  a16\n" ,
            0xCD => "CALL  a16\n" ,
            0xCE => "ADC  A  n8\n" ,
            0xCF => "RST  08H\n" ,
            0xD0 => "RET  NC\n" ,
            0xD1 => "POP  DE\n" ,
            0xD2 => "JP  NC  a16\n" ,
            0xD3 => "ILLEGAL_D3\n" ,
            0xD4 => "CALL  NC  a16\n" ,
            0xD5 => "PUSH  DE\n" ,
            0xD6 => "SUB  n8\n" ,
            0xD7 => "RST  10H\n" ,
            0xD8 => "RET  C\n" ,
            0xD9 => "RETI\n" ,
            0xDA => "JP  C  a16\n" ,
            0xDB => "ILLEGAL_DB\n" ,
            0xDC => "CALL  C  a16\n" ,
            0xDD => "ILLEGAL_DD\n" ,
            0xDE => "SBC  A  n8\n" ,
            0xDF => "RST  18H\n" ,
            0xE0 => "LDH  a8  A\n" ,
            0xE1 => "POP  HL\n" ,
            0xE2 => "LD  C  A\n" ,
            0xE3 => "ILLEGAL_E3\n" ,
            0xE4 => "ILLEGAL_E4\n" ,
            0xE5 => "PUSH  HL\n" ,
            0xE6 => "AND  n8\n" ,
            0xE7 => "RST  20H\n" ,
            0xE8 => "ADD  SP  e8\n" ,
            0xE9 => "JP  HL\n" ,
            0xEA => "LD  a16  A\n" ,
            0xEB => "ILLEGAL_EB\n" ,
            0xEC => "ILLEGAL_EC\n" ,
            0xED => "ILLEGAL_ED\n" ,
            0xEE => "XOR  n8\n" ,
            0xEF => "RST  28H\n" ,
            0xF0 => "LDH  A  a8\n" ,
            0xF1 => "POP  AF\n" ,
            0xF2 => "LD  A  C\n" ,
            0xF3 => "DI\n" ,
            0xF4 => "ILLEGAL_F4\n" ,
            0xF5 => "PUSH  AF\n" ,
            0xF6 => "OR  n8\n" ,
            0xF7 => "RST  30H\n" ,
            0xF8 => "LD  HL  SP  e8\n" ,
            0xF9 => "LD  SP  HL\n" ,
            0xFA => "LD  A  a16\n" ,
            0xFB => "EI\n" ,
            0xFC => "ILLEGAL_FC\n" ,
            0xFD => "ILLEGAL_FD\n" ,
            0xFE => "CP  n8\n" ,
            0xFF => "RST  38H\n" ,
        };
        self.log.write(readable.to_string().as_bytes()).expect("Invalid Write");
        self.log.flush();
    }

    pub fn write(&mut self, text: String){
        self.log.write(text.as_bytes()).expect("Invalid Write");
        self.log.flush();
    }

}


