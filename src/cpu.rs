/////////////////////////
/// 
/// cpu.rs
/// 
/// Sources:
/// https://gbdev.io/gb-opcodes/optables/ - Instructions
/// https://forums.nesdev.org/viewtopic.php?t=15944 - DAA instruction
/// 
use crate::bus::Bus;
use crate::log::Logger;
use crate::log::create_file;

#[derive(Clone, Copy)]
enum Reg8{ A, B, C, D, E, F, H, L}
#[derive(Clone, Copy)]
enum Reg16{ AF, BC, DE, HL, SP, PC }

#[derive(Clone, Copy)]
enum Cond{NZ, NC, Z, C, NONE}


#[derive(Clone, Copy)]
struct Registers{
    a: u8, f: u8,
    b: u8, c: u8,
    d: u8, e: u8,
    h: u8, l: u8,
    sp: u16,
    pc: u16,
}

impl Registers{
    pub fn new() -> Self {
        Self{
            a: 0x01, f: 0xB0,
            b: 0x00, c: 0x13,
            d: 0x00, e: 0xD8,
            h: 0x01, l: 0x4D,
            sp: 0xFFFE, pc: 0x0100,
        }
    }

    /**
     * Flag Modifier 
     * Z - bit 7
     * N - bit 6
     * H - bit 5
     * C - bit 4
     **/
    fn get_z(&self) -> bool { (self.f >> 7) != 0}
    fn get_n(&self) -> bool { ((self.f << 1) >> 7) != 0 }
    fn get_h(&self) -> bool { ((self.f << 2) >> 7) != 0}
    fn get_c(&self) -> bool { ((self.f << 3) >> 7) != 0}
    fn get_bc(&self) -> u16 { (self.b as u16) << 8 | self.c as u16 }
    fn get_de(&self) -> u16 { (self.d as u16) << 8 | self.e as u16 }
    fn get_hl(&self) -> u16 { (self.h as u16) << 8 | self.l as u16 }
    fn get_af(&self) -> u16 { (self.a as u16) << 8 | self.f as u16 }

    fn set_z(&mut self, val: bool){
         let mask: u8 = 1 << 7;
         let flag: u8 = if val {1} else {0};
         self.f = (self.f & !mask) | ( (flag << 7) & mask);
    }
    fn set_n(&mut self, val: bool){
        let mask: u8 = 1 << 6;
        let flag: u8 = if val {1} else {0};
        self.f = (self.f & !mask) | ( (flag << 6) & mask);
    }
    fn set_h(&mut self, val: bool){
        let mask: u8 = 1 << 5;
        let flag: u8 = if val {1} else {0};
        self.f = (self.f & !mask) | ( (flag << 5) & mask);
    }
    fn set_c(&mut self, val: bool){
        let mask: u8 = 1 << 4;
        let flag: u8 = if val {1} else {0};
        self.f = (self.f & !mask) | ( (flag << 4) & mask);
    }
    fn set_af(&mut self, val: u16){
        self.a = (val >> 8) as u8;
        self.f = ((val << 8) >> 8) as u8;   
    }
    fn set_bc(&mut self, val: u16){
        self.b = (val >> 8) as u8;
        self.c = ((val << 8) >> 8) as u8;   
    }
    fn set_de(&mut self, val: u16){
        self.d = (val >> 8) as u8;
        self.e = ((val << 8) >> 8) as u8;
    }
    fn set_hl(&mut self, val: u16){
        self.h = (val >> 8) as u8;
        self.l = ((val << 8) >> 8) as u8;     
    }

    fn set_reg8(&mut self, reg: Reg8, val: u8){
        match reg {
            Reg8::A => self.a = val,
            Reg8::B => self.b = val,
            Reg8::C => self.c = val,
            Reg8::D => self.d = val,
            Reg8::E => self.e = val,
            Reg8::F => self.f = val,
            Reg8::H => self.h = val,
            Reg8::L => self.l = val
        }
    }
    fn set_reg16(&mut self, reg: Reg16, val: u16){
        match reg{
            Reg16::BC => self.set_bc(val),
            Reg16::DE => self.set_de(val),
            Reg16::HL => self.set_hl(val),
            Reg16::AF => self.set_af(val),
            Reg16::SP => self.sp = val,
            Reg16::PC => self.pc = val,
            _ => panic!("Invalid Register")
        }
    }
    fn get_reg8(self, reg: Reg8)->u8{
        match reg {
            Reg8::A => self.a,
            Reg8::B => self.b,
            Reg8::C => self.c,
            Reg8::D => self.d,
            Reg8::E => self.e,
            Reg8::F => self.f,
            Reg8::H => self.h,
            Reg8::L => self.l,
        }
    }
    fn get_reg16(self, reg: Reg16)->u16{
        match reg{
            Reg16::BC => self.get_bc(),
            Reg16::DE => self.get_de(),
            Reg16::HL => self.get_hl(),
            Reg16::SP => self.sp,
            Reg16::PC => self.pc,
            Reg16::AF => self.get_af(),
        }
    }
}

/*
 * reg: registers
 * curr_instr: opcode of the current instruction being executed
 * IME: the IME flag which is used to disable all interrupts, overriding any enabled bits in the IE register.
 * halted: pauses emulatiom
 */
pub struct CPU{
    reg: Registers,
    ime: bool,
    halted: bool,
    bus: Bus,
    cycles: u64,
    log: Logger,
    
}
impl CPU {
    pub fn new(bus_in: Bus) -> Self{
        let file: std::fs::File = create_file().expect("Failed to create file");
        Self{
            reg: Registers::new(),
            ime: false,
            halted: false,
            bus: bus_in,
            cycles: 0,
            log: Logger {log: file},
        } 
    }

    pub fn step(&mut self){
        let opcode: u8 = self.fetch();

        self.log.write_instr(opcode);
        self.log_reg();

        self.execute(opcode);
    }

    pub fn log_reg(&mut self){
        self.log.write(
        format!("A: {:#01x} F: {:#01x} B: {:#01x} C: {:#01x} D: {:#01x} E: {:#01x} H: {:#01x} L: {:#01x} PC:{:01x} SP:{:01x}\n",
                    self.reg.a, self.reg.f, self.reg.b, self.reg.c, self.reg.d, self.reg.e, self.reg.h, self.reg.l, self.reg.pc, self.reg.sp))
    }

    ///
    /// Returns
    ///     opcode: u8 - opcode of instructionS
    /// 
    fn fetch(&mut self)->u8{
        let op: u8 = self.bus.read(self.reg.pc);
        self.reg.pc += 1;
        self.clock_tick();
        return op;
    }

    /*
     * Executes current instruction
     */
    fn execute(&mut self, opcode: u8){
        match opcode {
            0x00 => self.clock_tick(),
            0x01 => self.LD_r16_n16(Reg16::BC),
            0x02 => self.LD_MR_R(Reg16::BC, Reg8::A),
            0x03 => self.INC_r16(Reg16::BC),
            0x04 => self.INC_r8(Reg8::B),
            0x05 => self.DEC_r8(Reg8::B),
            0x06 => self.LD_r8_n8(Reg8::B),
            0x07 => self.RLCA(),
            0x08 => self.LD_A16_SP(),
            0x09 => self.ADD_r16_r16(Reg16::HL, Reg16::BC),
            0x0A => self.LD_R_MR(Reg8::A, Reg16::BC),
            0x0B => self.DEC_r16(Reg16::BC),
            0x0C => self.INC_r8(Reg8::C),
            0x0D => self.DEC_r8(Reg8::C),
            0x0E => self.LD_r8_n8(Reg8::C),
            0x0F => self.RRCA(),
            0x10 => self.STOP(),
            0x11 => self.LD_r16_n16(Reg16::DE),
            0x12 => self.LD_MR_R(Reg16::DE, Reg8::A),
            0x13 => self.INC_r16(Reg16::DE),
            0x14 => self.INC_r8(Reg8::D),
            0x15 => self.DEC_r8(Reg8::D),
            0x16 => self.LD_r8_n8(Reg8::D),
            0x17 => self.RLA(),
            0x18 => self.JR_e8(),
            0x19 => self.ADD_r16_r16(Reg16::HL, Reg16::DE),
            0x1A => self.LD_R_MR(Reg8::A, Reg16::DE),
            0x1B => self.DEC_r16(Reg16::DE),
            0x1C => self.INC_r8(Reg8::E),
            0x1D => self.DEC_r8(Reg8::E),
            0x1E => self.LD_r8_n8(Reg8::E),
            0x1F => self.RRA(),
            0x20 => self.JR(Cond::NZ),
            0x21 => self.LD_r16_n16(Reg16::HL),
            0x22 => self.LD_HLI_R(Reg8::A),
            0x23 => self.INC_r16(Reg16::HL),
            0x24 => self.INC_r8(Reg8::H),
            0x25 => self.DEC_r8(Reg8::H),
            0x26 => self.LD_r8_n8(Reg8::H),
            0x27 => self.DAA(),
            0x28 => self.JR(Cond::Z),
            0x29 => self.ADD_r16_r16(Reg16::HL, Reg16::HL),
            0x2A => self.LD_R_HLI(Reg8::A),
            0x2B => self.DEC_r16(Reg16::HL),
            0x2C => self.INC_r8(Reg8::L),
            0x2D => self.DEC_r8(Reg8::L),
            0x2E => self.LD_r8_n8(Reg8::L),
            0x2F => self.CPL(),
            0x30 => self.JR(Cond::NC),
            0x31 => self.LD_r16_n16(Reg16::SP),
            0x32 => self.LD_HLD_R(Reg8::A),
            0x33 => self.INC_r16(Reg16::SP),
            0x34 => self.INC_MR(Reg16::HL),
            0x35 => self.DEC_MR(Reg16::HL),
            0x36 => self.LD_MR_n8(Reg16::HL),
            0x37 => self.SCF(),
            0x38 => self.JR(Cond::C),
            0x39 => self.ADD_r16_r16(Reg16::HL, Reg16::SP),
            0x3A => self.LD_R_HLD(Reg8::A),
            0x3B => self.DEC_r16(Reg16::SP),
            0x3C => self.INC_r8(Reg8::A),
            0x3D => self.DEC_r8(Reg8::A),
            0x3E => self.LD_r8_n8(Reg8::A),
            0x3F => self.CCF(),
            0x40 => self.LD_R_R(Reg8::B, Reg8::B),
            0x41 => self.LD_R_R(Reg8::B, Reg8::C),
            0x42 => self.LD_R_R(Reg8::B, Reg8::D),
            0x43 => self.LD_R_R(Reg8::B, Reg8::E),
            0x44 => self.LD_R_R(Reg8::B, Reg8::H),
            0x45 => self.LD_R_R(Reg8::B, Reg8::L),
            0x46 => self.LD_R_MR(Reg8::B, Reg16::HL),
            0x47 => self.LD_R_R(Reg8::B, Reg8::A),
            0x48 => self.LD_R_R(Reg8::C, Reg8::B),
            0x49 => self.LD_R_R(Reg8::C, Reg8::C),
            0x4A => self.LD_R_R(Reg8::C, Reg8::D),
            0x4B => self.LD_R_R(Reg8::C, Reg8::E),
            0x4C => self.LD_R_R(Reg8::C, Reg8::H),
            0x4D => self.LD_R_R(Reg8::C, Reg8::L),
            0x4E => self.LD_R_MR(Reg8::C, Reg16::HL),
            0x4F => self.LD_R_R(Reg8::C, Reg8::A),
            0x50 => self.LD_R_R(Reg8::D, Reg8::B),
            0x51 => self.LD_R_R(Reg8::D, Reg8::C),
            0x52 => self.LD_R_R(Reg8::D, Reg8::D),
            0x53 => self.LD_R_R(Reg8::D, Reg8::E),
            0x54 => self.LD_R_R(Reg8::D, Reg8::H),
            0x55 => self.LD_R_R(Reg8::D, Reg8::L),
            0x56 => self.LD_R_MR(Reg8::D, Reg16::HL),
            0x57 => self.LD_R_R(Reg8::D, Reg8::A),
            0x58 => self.LD_R_R(Reg8::E, Reg8::B),
            0x59 => self.LD_R_R(Reg8::E, Reg8::C),
            0x5A => self.LD_R_R(Reg8::E, Reg8::D),
            0x5B => self.LD_R_R(Reg8::E, Reg8::E),
            0x5C => self.LD_R_R(Reg8::E, Reg8::H),
            0x5D => self.LD_R_R(Reg8::E, Reg8::L),
            0x5E => self.LD_R_MR(Reg8::E, Reg16::HL),
            0x5F => self.LD_R_R(Reg8::E, Reg8::A),
            0x60 => self.LD_R_R(Reg8::H, Reg8::B),
            0x61 => self.LD_R_R(Reg8::H, Reg8::C),
            0x62 => self.LD_R_R(Reg8::H, Reg8::D),
            0x63 => self.LD_R_R(Reg8::H, Reg8::E),
            0x64 => self.LD_R_R(Reg8::H, Reg8::H),
            0x65 => self.LD_R_R(Reg8::H, Reg8::L),
            0x66 => self.LD_R_MR(Reg8::H, Reg16::HL),
            0x67 => self.LD_R_R(Reg8::H, Reg8::A),
            0x68 => self.LD_R_R(Reg8::L, Reg8::B),
            0x69 => self.LD_R_R(Reg8::L, Reg8::C),
            0x6A => self.LD_R_R(Reg8::L, Reg8::D),
            0x6B => self.LD_R_R(Reg8::L, Reg8::E),
            0x6C => self.LD_R_R(Reg8::L, Reg8::H),
            0x6D => self.LD_R_R(Reg8::L, Reg8::L),
            0x6E => self.LD_R_MR(Reg8::L, Reg16::HL),
            0x6F => self.LD_R_R(Reg8::L, Reg8::A),
            0x70 => self.LD_MR_R(Reg16::HL, Reg8::B),
            0x71 => self.LD_MR_R(Reg16::HL, Reg8::C),
            0x72 => self.LD_MR_R(Reg16::HL, Reg8::D),
            0x73 => self.LD_MR_R(Reg16::HL, Reg8::E),
            0x74 => self.LD_MR_R(Reg16::HL, Reg8::H),
            0x75 => self.LD_MR_R(Reg16::HL, Reg8::L),
            0x76 => self.HALT(),
            0x77 => self.LD_MR_R(Reg16::HL, Reg8::A),
            0x78 => self.LD_R_R(Reg8::A, Reg8::B),
            0x79 => self.LD_R_R(Reg8::A, Reg8::C),
            0x7A => self.LD_R_R(Reg8::A, Reg8::D),
            0x7B => self.LD_R_R(Reg8::A, Reg8::E),
            0x7C => self.LD_R_R(Reg8::A, Reg8::H),
            0x7D => self.LD_R_R(Reg8::A, Reg8::L),
            0x7E => self.LD_R_MR(Reg8::A, Reg16::HL),
            0x7F => self.LD_R_R(Reg8::A, Reg8::A),
            0x80 => self.ADD_r8_r8(Reg8::A, Reg8::B),
            0x81 => self.ADD_r8_r8(Reg8::A, Reg8::C),
            0x82 => self.ADD_r8_r8(Reg8::A, Reg8::D),
            0x83 => self.ADD_r8_r8(Reg8::A, Reg8::E),
            0x84 => self.ADD_r8_r8(Reg8::A, Reg8::H),
            0x85 => self.ADD_r8_r8(Reg8::A, Reg8::L),
            0x86 => self.ADD_r8_mr(Reg8::A, Reg16::HL),
            0x87 => self.ADD_r8_r8(Reg8::A, Reg8::A),
            0x88 => self.ADC_A_r8(Reg8::B),
            0x89 => self.ADC_A_r8(Reg8::C),
            0x8A => self.ADC_A_r8(Reg8::D),
            0x8B => self.ADC_A_r8(Reg8::E),
            0x8C => self.ADC_A_r8(Reg8::H),
            0x8D => self.ADC_A_r8(Reg8::L),
            0x8E => self.ADC_A_HL(),
            0x8F => self.LD_R_R(Reg8::A, Reg8::A),
            0x90 => self.SUB(Reg8::B),
            0x91 => self.SUB(Reg8::C),
            0x92 => self.SUB(Reg8::D),
            0x93 => self.SUB(Reg8::E),
            0x94 => self.SUB(Reg8::H),
            0x95 => self.SUB(Reg8::L),
            0x96 => self.SUB_HL(),
            0x97 => self.SUB(Reg8::A),
            0x98 => self.SBC(Reg8::B),
            0x99 => self.SBC(Reg8::C),
            0x9A => self.SBC(Reg8::D),
            0x9B => self.SBC(Reg8::E),
            0x9C => self.SBC(Reg8::H),
            0x9D => self.SBC(Reg8::L),
            0x9E => self.SBC_HL(),
            0x9F => self.SBC(Reg8::A),
            0xA0 => self.AND(Reg8::B),
            0xA1 => self.AND(Reg8::C),
            0xA2 => self.AND(Reg8::D),
            0xA3 => self.AND(Reg8::E),
            0xA4 => self.AND(Reg8::H),
            0xA5 => self.AND(Reg8::L),
            0xA6 => self.AND_HL(),
            0xA7 => self.AND(Reg8::A),
            0xA8 => self.XOR(Reg8::B),
            0xA9 => self.XOR(Reg8::C),
            0xAA => self.XOR(Reg8::D),
            0xAB => self.XOR(Reg8::E),
            0xAC => self.XOR(Reg8::H),
            0xAD => self.XOR(Reg8::L),
            0xAE => self.XOR_HL(),
            0xAF => self.XOR(Reg8::A),
            0xB0 => self.OR(Reg8::B),
            0xB1 => self.OR(Reg8::C),
            0xB2 => self.OR(Reg8::D),
            0xB3 => self.OR(Reg8::E),
            0xB4 => self.OR(Reg8::H),
            0xB5 => self.OR(Reg8::L),
            0xB6 => self.OR_HL(),
            0xB7 => self.OR(Reg8::A),
            0xB8 => self.CP(Reg8::B),
            0xB9 => self.CP(Reg8::C),
            0xBA => self.CP(Reg8::D),
            0xBB => self.CP(Reg8::E),
            0xBC => self.CP(Reg8::H),
            0xBD => self.CP(Reg8::L),
            0xBE => self.CP_HL(),
            0xBF => self.CP(Reg8::A),
            0xC0 => self.RET(Cond::NC),
            0xC1 => self.POP(Reg16::BC),
            0xC2 => self.JP_a16(Cond::NZ),
            0xC3 => self.JP_a16(Cond::NONE),
            0xC4 => self.CALL(Cond::NZ),
            0xC5 => self.PUSH(Reg16::BC),
            0xC6 => self.ADD_r8_n8(Reg8::A),
            0xC7 => self.RST(0x00),
            0xC8 => self.RET(Cond::Z),
            0xC9 => self.RET(Cond::NONE),
            0xCA => self.JP_a16(Cond::Z),
            0xCB => self.CB(),
            0xCC => self.CALL(Cond::Z),
            0xCD => self.CALL(Cond::NONE),
            0xCE => self.ADC_A_r8(Reg8::A),
            0xCF => self.RST(0x08),
            0xD0 => self.RET(Cond::NC),
            0xD1 => self.POP(Reg16::DE),
            0xD2 => self.JP_a16(Cond::NC),
            0xD4 => self.CALL(Cond::NC),
            0xD5 => self.PUSH(Reg16::DE),
            0xD6 => self.SUB_D8(),
            0xD7 => self.RST(0x10),
            0xD8 => self.RET(Cond::C),
            0xD9 => self.RETI(),
            0xDA => self.JP_a16(Cond::C),
            0xDC => self.CALL(Cond::C),
            0xDE => self.SBC_D8(),
            0xDF => self.RST(18),
            0xE0 => self.LDH_A8_A(),
            0xE1 => self.POP(Reg16::HL),
            0xE2 => self.LD_C_A(),
            0xE5 => self.PUSH(Reg16::HL),
            0xE6 => self.AND_D8(),
            0xE7 => self.RST(0x20),
            0xE8 => self.ADD_sp_e8(),
            0xE9 => self.JP_HL(),
            0xEA => self.LD_A16_A(),
            0xEE => self.XOR_D8(),
            0xEF => self.RST(0x28),
            0xF0 => self.LDH_A_A8(),
            0xF1 => self.POP(Reg16::AF),
            0xF2 => self.LD_A_C(),
            0xF3 => self.DI(),
            0xF5 => self.PUSH(Reg16::AF),
            0xF6 => self.OR_D8(),
            0xF7 => self.RST(0x30),
            0xF8 => self.LD_HL_SP_E8(),
            0xF9 => self.LD_SP_HL(),
            0xFA => self.LD_A_A16(),
            0xFB => self.EI(),
            0xFE => self.CP_d8(),
            0xFF => self.RST(0x38),
            _ => panic!("Invalid opcode used")
        }

    }

    pub fn run(&mut self){
        
        if !self.halted {
            self.step();
        }
        else{
            self.clock_tick();
            if self.reg.ie != 0 {
                self.halted = false;
            }
        }

        if self.ime {
            self.handle_interrupt();
        }
    }
    
    fn handle_interrupt(&mut self){
        let pc_hi: u8 = ((self.reg.pc & 0xFF00) >> 8) as u8;
        let pc_lo: u8 = (self.reg.pc & 0x00FF) as u8;
        self.stkpush(pc_hi);
        self.stkpush(pc_lo);

        
        let it_vblank: u8 = 1;
        let it_lcd_stat: u8 = 2;
        let it_timer: u8 = 4;
        let it_serial: u8 = 8;
        let it_joypad: u8 = 16;

        if((self.reg.ie & it_vblank) != 0 ) && ((self.reg.ifr & it_vblank) != 0){
            self.reg.pc = 0x40;
        }
        else if( (self.reg.ie & it_lcd_stat) != 0) && ((self.reg.ifr & it_lcd_stat) != 0) {
            self.reg.pc = 0x48;
        }
        else if( (self.reg.ie & it_timer) != 0) && ((self.reg.ifr & it_timer) != 0) {
            self.reg.pc = 0x50;
        }
        else if( (self.reg.ie & it_serial) != 0) && ((self.reg.ifr & it_serial) != 0) {
            self.reg.pc = 0x58;
        }
        else if( (self.reg.ie & it_joypad) != 0) && ((self.reg.ifr & it_joypad) != 0) {
            self.reg.pc = 0x60;
        }

        self.reg.ifr &= !self.reg.ifr;
        self.halted = false;
        self.ime = false;
        
    }
    fn clock_tick(&mut self){ self.cycles += 1; }

    /* Instructions */ 
    
    /* Load instructions */

    /// Load immediate little-endian 16-bit data to 16 bit reg 
    fn LD_r16_n16(&mut self, r: Reg16){
        let lsb: u8 = self.bus.read(self.reg.pc);
        self.reg.pc += 1;
        self.clock_tick();
        
        let msb: u8 = self.bus.read(self.reg.pc);
        self.reg.pc += 1;
        self.clock_tick();

        let n16: u16 = (msb as u16) << 8 | lsb as u16;
        self.reg.set_reg16(r, n16);
        self.clock_tick();
    }
    fn LD_r8_n8(&mut self, r: Reg8){
        let data: u8 = self.bus.read(self.reg.pc);
        self.reg.pc += 1;
        self.clock_tick();
        self.reg.set_reg8(r, data);
        self.clock_tick();
    }
    /// Load value at reg8 r into memory address stored in reg16 mr
    fn LD_MR_R(&mut self, mr: Reg16, r: Reg8){
        self.bus.write(self.reg.get_reg16(mr), self.reg.get_reg8(r));
        self.clock_tick();
        self.clock_tick();
    }

    fn LD_R_MR(&mut self, r: Reg8, mr: Reg16){
        let val: u8 = self.bus.read(self.reg.get_reg16(mr));
        self.clock_tick();
        self.reg.set_reg8(r, val);
        self.clock_tick();
    }

    fn LD_R_R(&mut self, r1: Reg8, r2: Reg8){
        self.reg.set_reg8(r1, self.reg.get_reg8(r2));
        self.clock_tick();
    }
    fn LD_SP_HL(&mut self){
        self.reg.sp = self.reg.get_hl();
        self.clock_tick();
        self.clock_tick();
    }
    ///reads memory from hl, increments hl, stores it in r
    fn LD_R_HLI(&mut self, r: Reg8){
        let hl: u16 = self.reg.get_hl();
        let val: u8 = self.bus.read(hl);
        self.clock_tick();
        self.reg.set_hl(hl.wrapping_add(1));
        self.reg.set_reg8(r, val);
        self.clock_tick();
    }

    fn LD_HLI_R(&mut self, r: Reg8){
        let data: u8 = self.reg.get_reg8(r);
        self.bus.write(self.reg.get_hl(), data);
        self.clock_tick();
        self.reg.set_hl(self.reg.get_hl().wrapping_add(1));
        self.clock_tick();
    }
    fn LD_HLD_R(&mut self, r: Reg8){
        let data: u8 = self.reg.get_reg8(r);
        self.bus.write(self.reg.get_hl(), data);
        self.clock_tick();
        self.reg.set_hl(self.reg.get_hl().wrapping_sub(1));
        self.clock_tick();
    }
    fn LD_R_HLD(&mut self, r: Reg8){
        let hl: u16 = self.reg.get_hl();
        let val: u8 = self.bus.read(hl);
        self.clock_tick();
        self.reg.set_hl(hl.wrapping_sub(1));
        self.reg.set_reg8(r, val);
        self.clock_tick();  
    }

    fn LD_A16_A(&mut self){
        let lo: u8 = self.bus.read(self.reg.pc);
        self.reg.pc += 1;
        let hi: u8 = self.bus.read(self.reg.pc);
        self.reg.pc += 1;
        let addr: u16 = ((hi as u16) << 8) | lo as u16 ;
        self.bus.write(addr, self.reg.a);
        self.clock_tick();
        self.clock_tick();
        self.clock_tick();
        self.clock_tick();
    }

    fn LD_A_A16(&mut self){
        let lo: u8 = self.bus.read(self.reg.pc);
        self.reg.pc += 1;
        let hi: u8 = self.bus.read(self.reg.pc);
        self.reg.pc += 1;
        let addr: u16 = ((hi as u16) << 8) | lo as u16 ;
        let data: u8 = self.bus.read(addr);
        self.reg.a = data;
        self.clock_tick();
        self.clock_tick();
        self.clock_tick();
        self.clock_tick();
    }
    ///Load value of stack pointer into memory address stored at addres of value a16
    /// a16 :=
    fn LD_A16_SP(&mut self){
        let lsb: u8 = self.bus.read(self.reg.pc);
        self.reg.pc += 1;
        self.clock_tick();
        let msb: u8 = self.bus.read(self.reg.pc);
        self.reg.pc += 1;
        self.clock_tick();
        let a16: u16 = (msb as u16)  << 8 | lsb as u16;
        self.clock_tick();
        //store lower byte of sp first, then upper byte
        let lsb_sp: u8 = (self.reg.sp & 0xff) as u8;
        let msb_sp: u8 = (self.reg.sp >> 8) as u8;
        self.bus.write(a16, lsb_sp);
        self.clock_tick();
        self.bus.write(a16, msb_sp);
        self.clock_tick();
    }

    fn LD_MR_n8(&mut self, mr: Reg16){
        let data: u8 = self.bus.read(self.reg.pc);
        self.reg.pc += 1;
        self.clock_tick();
        self.bus.write(self.reg.get_reg16(mr), data);
        self.clock_tick();
        self.clock_tick();
    }

    fn LDH_A8_A(&mut self){
        //LDH (a8),A has alternative mnemonic LD ($FF00+a8),A
         let a8: u8 = self.bus.read(self.reg.pc);
         self.reg.pc += 1;
         let mut addr: u16 = 0xFF00;
         addr = addr.wrapping_add(a8 as u16);
         self.bus.write(addr, self.reg.a);
         self.clock_tick();
         self.clock_tick();
         self.clock_tick();
    }

    fn LDH_A_A8(&mut self){
         // LDH A,(a8) has alternative mnemonic LD A,($FF00+a8)
         let a8: u8 = self.bus.read(self.reg.pc);
         self.reg.pc += 1;
         let mut addr: u16 = 0xFF00;
         addr = addr.wrapping_add(a8 as u16);
         let data: u8 = self.bus.read(addr);
         self.reg.a = data;
         self.clock_tick();
         self.clock_tick();
         self.clock_tick();
    }
    fn LD_A_C(&mut self){
        let addr: u16 = 0xFF00 + (self.reg.c as u16);
        let data: u8 = self.bus.read(addr);
        self.reg.a = data;
        self.clock_tick();
        self.clock_tick();
    }
    fn LD_C_A(&mut self){
        let addr: u16 = 0xFF00 + (self.reg.c as u16);
        self.bus.write(addr, self.reg.a);
        self.clock_tick();
        self.clock_tick();
    }
    
    fn LD_HL_SP_E8(&mut self){
        let e8: i8 = self.bus.read(self.reg.pc) as i8;
        self.reg.pc += 1;
        self.clock_tick();
        self.reg.set_hl(self.reg.sp.wrapping_add_signed(e8 as i16));
        self.reg.set_z(false);
        self.reg.set_n(false);
        if (self.reg.get_hl() & 0xF0) > (self.reg.sp & 0xF0){
            self.reg.set_h(true);
        }
        else{
            self.reg.set_h(false);
        }
        if self.reg.get_hl() < self.reg.sp {
            self.reg.set_c(true);
        }
        else{
            self.reg.set_c(false);
        }
    }
    //Arithmetic
    fn INC_r8(&mut self, r: Reg8){
        self.reg.set_reg8(r, self.reg.get_reg8(r).wrapping_add(1));
        //flags Z 0 H -
        self.reg.set_z(self.reg.get_reg8(r) == 0);
        self.reg.set_n(false);
        self.reg.set_h(self.reg.get_reg8(r) & 0x0F != 0);
        self.clock_tick();
    }
    fn INC_r16(&mut self, r: Reg16){
        self.reg.set_reg16(r, self.reg.get_reg16(r).wrapping_add(1));
        self.clock_tick();
        self.clock_tick();
    }
    fn INC_MR(&mut self, mr: Reg16){
        let mut data: u8 = self.bus.read(self.reg.get_reg16(mr));
        self.clock_tick();
        data = data.wrapping_add(1);
        self.clock_tick();
        self.bus.write(self.reg.get_reg16(mr), data);
        self.clock_tick();
    }
    fn DEC_r8(&mut self, r: Reg8){
        self.reg.set_reg8(r, self.reg.get_reg8(r).wrapping_sub(1));
        //flags Z 0 H -
        self.reg.set_z(self.reg.get_reg8(r) == 0);
        self.reg.set_n(true);
        self.reg.set_h(self.reg.get_reg8(r) & 0x0F != 0);
        self.clock_tick();
    }
    fn DEC_r16(&mut self, r: Reg16){
        self.reg.set_reg16(r, self.reg.get_reg16(r).wrapping_sub(1));
        self.clock_tick();
        self.clock_tick(); 
    }
    fn DEC_MR(&mut self, mr: Reg16){
        let mut data: u8 = self.bus.read(self.reg.get_reg16(mr));
        self.clock_tick();
        data = data.wrapping_sub(1);
        self.clock_tick();
        self.bus.write(self.reg.get_reg16(mr), data);
        self.clock_tick();
    }
    fn ADD_r8_r8(&mut self, r1: Reg8, r2: Reg8){

        let val1: u8 = self.reg.get_reg8(r1);
        let val2: u8 = self.reg.get_reg8(r2);
        let sum: u8 = val1.wrapping_add(val2);
        self.reg.set_reg8(r1, sum);

        //flags - Z 0 H C
        self.reg.set_z(sum == 0);
        self.reg.set_n(false);
        if (sum & 0xF0) > (val1 & 0xF0){
            self.reg.set_h(true);
        }
        else{
            self.reg.set_h(false);
        }
        if sum < val1{
            self.reg.set_c(true);
        }
        else{
            self.reg.set_c(false);
        }
        self.clock_tick();
    }
    fn ADD_r8_mr(&mut self, r: Reg8, mr: Reg16){
        //Z 0 H C
        let mr_val: u8 = self.bus.read( self.reg.get_reg16(mr) );
        let r_val: u8 = self.reg.get_reg8(r);;
        let sum: u8 = r_val.wrapping_add(mr_val);
        self.clock_tick();

        self.reg.set_reg8(r, sum);
        self.reg.set_z(sum == 0);
        self.reg.set_n(false);
        if (sum & 0xF0) > (r_val & 0xF0){
            self.reg.set_h(true);
        }
        else{
            self.reg.set_h(false);
        }
        if sum < r_val {
            self.reg.set_c(true);
        }
        else{
            self.reg.set_c(false);
        }
        self.clock_tick();
    }
    fn ADD_r16_r16(&mut self, r1: Reg16, r2: Reg16){
        let val1: u16 = self.reg.get_reg16(r1);
        let val2: u16 = self.reg.get_reg16(r2);

        let sum: u16 = val1.wrapping_add(val2);
        self.reg.set_reg16(r1, sum);


        self.reg.set_n(false);
        if (sum & 0xF00) > (val1 & 0xF00){
            self.reg.set_h(true);
        }
        else{
            self.reg.set_h(false);
        }
        if sum < val1 {
            self.reg.set_c(true);
        }
        else{
            self.reg.set_c(false);
        }
        self.clock_tick();
        self.clock_tick();
    }   
    fn ADD_r8_n8(&mut self, r: Reg8){
        //Z 0 H C
        let n8: u8 = self.bus.read(self.reg.pc);
        self.reg.pc += 1;
        let val: u8 = self.reg.get_reg8(r);
        let sum: u8 = val.wrapping_add(n8);
        self.reg.set_reg8(r, sum);

        self.reg.set_z(sum == 0);
        self.reg.set_n(false);
        if (sum & 0xF0) > (val & 0xF0){
            self.reg.set_h(true);
        }
        else{
            self.reg.set_h(false);
        }
        if sum < val {
            self.reg.set_c(true);
        }
        else{
            self.reg.set_c(false);
        }
        self.clock_tick();  
    }
    fn ADD_sp_e8(&mut self){
        let e8: i8 = self.bus.read(self.reg.pc) as i8;
        self.reg.pc += 1;

        let sum: u16 = self.reg.sp.wrapping_add_signed(e8 as i16);
        self.reg.set_z(false);
        self.reg.set_n(false);
        if (sum & 0xF0) > (self.reg.sp & 0xF0){
            self.reg.set_h(true);
        }
        else{
            self.reg.set_h(false);
        }
        if sum < self.reg.sp {
            self.reg.set_c(true);
        }
        else{
            self.reg.set_c(false);
        }
        self.reg.sp = sum;
        self.clock_tick();
        self.clock_tick();
        self.clock_tick();
        self.clock_tick();
    }

    fn ADC_A_r8(&mut self, r: Reg8){
        let cf: u8 = if self.reg.get_c() {1} else {0};
        let val: u8 = self.reg.a.wrapping_add(cf).wrapping_add(self.reg.get_reg8(r));

        self.reg.set_z(val == 0);
        self.reg.set_n(false);
        if (val & 0xF0) > (self.reg.a & 0xF0){
            self.reg.set_h(true);
        }
        else{
            self.reg.set_h(false);
        }
        if val < self.reg.a {
            self.reg.set_c(true);
        }
        else{
            self.reg.set_c(false);
        }
        self.reg.a = val;
        self.clock_tick();
    }

    fn ADC_A_HL(&mut self){
        let data: u8 = self.bus.read(self.reg.get_hl());
        self.clock_tick();

        let cf: u8 = if self.reg.get_c() {1} else {0};
        let val: u8 = self.reg.a.wrapping_add(cf).wrapping_add(data);
        self.reg.set_z(val == 0);
        self.reg.set_n(false);
        if (val & 0xF0) > (self.reg.a & 0xF0){
            self.reg.set_h(true);
        }
        else{
            self.reg.set_h(false);
        }
        if val < self.reg.a {
            self.reg.set_c(true);
        }
        else{
            self.reg.set_c(false);
        }
        self.reg.a = val;
        self.clock_tick();
    }

    fn SUB(&mut self, r: Reg8){
        let res: u8 = self.reg.a.wrapping_sub(self.reg.get_reg8(r));

        self.reg.set_z(res == 0);
        self.reg.set_n(true);
        if (res & 0xF0) < (self.reg.a & 0xF0){
            self.reg.set_h(true);
        }
        else{
            self.reg.set_h(false);
        }
        if (res & 0xF0) > (self.reg.a & 0xF0){
            self.reg.set_c(true);
        }
        else{
            self.reg.set_c(false);
        }
        self.reg.a = res;
        self.clock_tick();
    }
    fn SUB_D8(&mut self){
        let d8: u8 = self.bus.read(self.reg.pc);
        self.reg.pc += 1;
        self.clock_tick();
        let res: u8 = self.reg.a.wrapping_sub(d8);
        self.reg.set_z(res == 0);
        self.reg.set_n(true);
        if (res & 0xF0) < (self.reg.a & 0xF0){
            self.reg.set_h(true);
        }
        else{
            self.reg.set_h(false);
        }
        if (res & 0xF0) > (self.reg.a & 0xF0){
            self.reg.set_c(true);
        }
        else{
            self.reg.set_c(false);
        }
        self.reg.a = res;
        self.clock_tick();
    }

    fn SUB_HL(&mut self){
        let data: u8 = self.bus.read(self.reg.get_hl());
        self.clock_tick();

        let res: u8 = self.reg.a.wrapping_sub(data);
        self.reg.set_z(res == 0);
        self.reg.set_n(true);
        if (res & 0xF0) < (self.reg.a & 0xF0){
            self.reg.set_h(true);
        }
        else{
            self.reg.set_h(false);
        }
        if (res & 0xF0) > (self.reg.a & 0xF0){
            self.reg.set_c(true);
        }
        else{
            self.reg.set_c(false);
        }
        self.reg.a = res;
        self.clock_tick()
    }

    fn SBC(&mut self, r: Reg8){
        let cf: u8 = if self.reg.get_c() {1} else {0};
        let res: u8 = self.reg.a.wrapping_sub(self.reg.get_reg8(r))
                                    .wrapping_sub(cf);

        self.reg.set_z(res == 0);
        self.reg.set_n(true);
        if (res & 0xF0) < (self.reg.a & 0xF0){
            self.reg.set_h(true);
        }
        else{
            self.reg.set_h(false);
        }
        if (res & 0xF0) > (self.reg.a & 0xF0){
            self.reg.set_c(true);
        }
        else{
            self.reg.set_c(false);
        }
        self.reg.a = res;
        self.clock_tick();
    }
    fn SBC_D8(&mut self){
        let d8: u8 = self.bus.read(self.reg.pc);
        self.reg.pc += 1;
        let cf: u8 = if self.reg.get_c() {1} else {0};
        let res: u8 = self.reg.a.wrapping_sub(d8).wrapping_sub(cf);

        self.reg.set_z(res == 0);
        self.reg.set_n(true);
        if (res & 0xF0) < (self.reg.a & 0xF0){
            self.reg.set_h(true);
        }
        else{
            self.reg.set_h(false);
        }
        if (res & 0xF0) > (self.reg.a & 0xF0){
            self.reg.set_c(true);
        }
        else{
            self.reg.set_c(false);
        }
        self.reg.a = res;
        self.clock_tick();
    }

    fn SBC_HL(&mut self){
        let cf: u8 = if self.reg.get_c() {1} else {0};
        let data: u8 = self.bus.read(self.reg.get_hl());
        self.clock_tick();
        let res: u8 = self.reg.a.wrapping_sub(data).wrapping_sub(cf);

        self.reg.set_z(res == 0);
        self.reg.set_n(true);
        if (res & 0xF0) < (self.reg.a & 0xF0){
            self.reg.set_h(true);
        }
        else{
            self.reg.set_h(false);
        }
        if (res & 0xF0) > (self.reg.a & 0xF0){
            self.reg.set_c(true);
        }
        else{
            self.reg.set_c(false);
        }
        self.reg.a = res;
        self.clock_tick()
    }

    fn AND(&mut self, r: Reg8){
        self.reg.a = self.reg.a & self.reg.get_reg8(r);
        self.reg.set_z(self.reg.a == 0);
        self.reg.set_n(false);
        self.reg.set_h(true);
        self.reg.set_c(false);
        self.clock_tick();
    }
    fn AND_HL(&mut self){
        let data: u8 = self.bus.read(self.reg.get_hl());
        self.clock_tick();
        self.reg.a = self.reg.a & data;
        self.reg.set_z(self.reg.a == 0);
        self.reg.set_n(false);
        self.reg.set_h(true);
        self.reg.set_c(false);
        self.clock_tick();
    }
    fn AND_D8(&mut self){
        let data: u8 = self.bus.read(self.reg.pc);
        self.reg.pc = self.reg.pc + 1;
        self.clock_tick();
        self.reg.a = self.reg.a & data;
        self.reg.set_z(self.reg.a == 0);
        self.reg.set_n(false);
        self.reg.set_h(true);
        self.reg.set_c(false);
        self.clock_tick();
    }

    fn XOR(&mut self, r: Reg8){
        self.reg.a = self.reg.a ^ self.reg.get_reg8(r);
        self.reg.set_z(self.reg.a == 0);
        self.reg.set_n(false);
        self.reg.set_h(true);
        self.reg.set_c(false);
        self.clock_tick();
    }
    fn XOR_HL(&mut self){
        let data: u8 = self.bus.read(self.reg.get_hl());
        self.clock_tick();
        self.reg.a = self.reg.a ^  data;
        self.reg.set_z(self.reg.a == 0);
        self.reg.set_n(false);
        self.reg.set_h(true);
        self.reg.set_c(false);
        self.clock_tick();
    }
    fn XOR_D8(&mut self){
        let data: u8 = self.bus.read(self.reg.pc);
        self.reg.pc += 1;
        self.clock_tick();
        self.reg.a = self.reg.a ^  data;
        self.reg.set_z(self.reg.a == 0);
        self.reg.set_n(false);
        self.reg.set_h(true);
        self.reg.set_c(false);
        self.clock_tick();
    }

    fn OR(&mut self, r: Reg8){
        self.reg.a = self.reg.a | self.reg.get_reg8(r);
        self.reg.set_z(self.reg.a == 0);
        self.reg.set_n(false);
        self.reg.set_h(true);
        self.reg.set_c(false);
        self.clock_tick();
    }

    fn OR_HL(&mut self){
        let data: u8 = self.bus.read(self.reg.get_hl());
        self.clock_tick();
        self.reg.a = self.reg.a |  data;
        self.reg.set_z(self.reg.a == 0);
        self.reg.set_n(false);
        self.reg.set_h(true);
        self.reg.set_c(false);
        self.clock_tick();
    }

    fn OR_D8(&mut self){
        let data: u8 = self.bus.read(self.reg.get_hl());
        self.clock_tick();
        self.reg.a = self.reg.a |  data;
        self.reg.set_z(self.reg.a == 0);
        self.reg.set_n(false);
        self.reg.set_h(true);
        self.reg.set_c(false);
        self.clock_tick();
    }
    fn RLCA(&mut self){
        let mut a: u8 = self.reg.a;
        let c: bool = ((a >> 7) & 1) != 0;
        let c_val: u8 = if c {1} else {0};
        a = (a << 1) | c_val;
        self.reg.a = a;
        self.reg.set_z(false);
        self.reg.set_n(false);
        self.reg.set_h(false);
        self.reg.set_c(c);
        self.clock_tick();
    }

    fn CP(&mut self, r: Reg8){
        let val: u8 = self.reg.get_reg8(r);
        let res: u8 = self.reg.a - val;
        self.reg.set_z(res == 0);
        self.reg.set_n(true);
        //watch this line while testing (854)
        self.reg.set_h(((self.reg.a & 0x0F) - (val & 0x0F)) < (0 as u8));
        self.reg.set_c(res < (0 as u8));
        self.clock_tick();
    }
    fn CP_HL(&mut self){
        let data: u8 = self.bus.read(self.reg.get_hl());
        self.clock_tick();
        let res: u8 = self.reg.a - data;
        self.reg.set_z(res == 0);
        self.reg.set_n(true);
        //watch this line while testing (854)
        self.reg.set_h(((self.reg.a & 0x0F) - (data & 0x0F)) < (0 as u8));
        self.reg.set_c(res < (0 as u8));
        self.clock_tick();
    }

    fn CP_d8(&mut self){
        let data: u8 = self.bus.read(self.reg.pc);
        self.reg.pc += 1;
        self.clock_tick();
        let res: u8 = self.reg.a.wrapping_sub(data);
        self.reg.set_z(res == 0);
        self.reg.set_n(true);
        //watch this line while testing (854)
        self.reg.set_h(((self.reg.a & 0x0F).wrapping_sub((data & 0x0F)) < (0 as u8)));
        self.reg.set_c(res < (0 as u8));
        self.clock_tick();
    }

    fn RET(&mut self, cond: Cond){
        self.clock_tick();
        self.clock_tick();
        match cond {
            Cond::Z => {if !self.reg.get_z() {return;} },
            Cond::C => {if !self.reg.get_c() {return;}},
            Cond::NC => {if self.reg.get_c(){return;}},
            Cond::NZ => {if self.reg.get_z(){return;}}
            Cond::NONE=> {}
        }

        let lo: u8 = self.stkpop();
        let hi: u8 = self.stkpop();
        self.reg.pc = ((hi as u16) << 8) | lo as u16 ;
        self.clock_tick();
        self.clock_tick();
        self.clock_tick();
    }
    
    fn RETI(&mut self){
        self.ime = true;
        self.RET(Cond::NONE);
    }

    fn POP(&mut self, r: Reg16){
        let lo: u8 = self.stkpop();
        self.clock_tick();
        let hi: u8 = self.stkpop();
        self.clock_tick();
        let data: u16 = ((hi as u16) << 8) | lo as u16 ;
        self.reg.set_reg16(r, data);
        self.clock_tick();
    }

    fn PUSH(&mut self, r: Reg16){
        let val: u16 = self.reg.get_reg16(r);
        let hi: u8 = ((val & 0xFF00) >> 8) as u8;
        let lo: u8 = (val & 0x00FF) as u8;
        self.stkpush(hi);
        self.stkpush(lo);
        self.clock_tick();
        self.clock_tick();
        self.clock_tick();
        self.clock_tick();
    }
    fn RRCA(&mut self){
        let mut a: u8 = self.reg.a;
        let c: bool = (a & 1) != 0;
        let c_val: u8 = if c {1} else {0};
        a = a >> 1;
        a |= c_val << 7;     
        self.reg.a = a;
        self.reg.set_z(false);
        self.reg.set_n(false);
        self.reg.set_h(false);
        self.reg.set_c(c); 
        self.clock_tick();
    }
    
    fn STOP(&mut self){
        self.reg.pc += 1;
        self.clock_tick();
        self.clock_tick();
        std::process::exit(2);
    }

    fn RLA(&mut self){
        let mut a: u8 = self.reg.a;
        let MSB: bool = if (a >> 7) != 0 {true} else {false};
        let c: u8 = if self.reg.get_c() {1} else {0};
        a *= 2;
        a = a + c;
        self.reg.a = a;
        self.reg.set_z(false);
        self.reg.set_n(false);
        self.reg.set_h(false);
        self.reg.set_c(MSB);
        self.clock_tick();
    }

    fn RRA(&mut self){
        let mut a: u8 = self.reg.a;
        let LSB: bool = if ( (a >> 7) & 1) != 0 {true} else {false};
        let c: u8 = if self.reg.get_c() {1} else {0};
        a = a >> 1;
        a = a | (c << 7);
        self.reg.a = a;
        self.reg.set_z(false);
        self.reg.set_n(false);
        self.reg.set_h(false);
        self.reg.set_c(LSB);
        self.clock_tick();
    }

    fn CPL(&mut self){
        self.reg.a = !self.reg.a;
        self.reg.set_n(true);
        self.reg.set_h(true);
        self.clock_tick();
    }
    fn SCF(&mut self){
        self.reg.set_n(false);
        self.reg.set_h(false);
        self.reg.set_c(true);
        self.clock_tick();
    }

    fn CCF(&mut self){
        self.reg.set_n(false);
        self.reg.set_h(false);
        let c: u8 = if self.reg.get_c() {1} else {0};
        self.reg.set_c((c^1) != 0);
        self.clock_tick();
    }
    
    fn HALT(&mut self){
        self.halted = true;
        self.clock_tick();
    }

    fn JR_e8(&mut self){
        let e8: i8 = self.bus.read(self.reg.pc) as i8;
        self.reg.pc += 1;
        self.clock_tick();
        self.reg.pc = self.reg.pc.wrapping_add_signed(e8 as i16);
        self.clock_tick();
        self.clock_tick();
    }

    fn JR(&mut self, cond: Cond){
        let e8: i8 = self.bus.read(self.reg.pc) as i8;
        self.reg.pc += 1;
        self.clock_tick();
        self.clock_tick();
        match cond {
            Cond::Z => {if !self.reg.get_z() {return;} },
            Cond::C => {if !self.reg.get_c() {return;}},
            Cond::NC => {if self.reg.get_c(){return;}},
            Cond::NZ => {if self.reg.get_z(){return;}},
            Cond::NONE=> {},
        }
        self.reg.pc = self.reg.pc.wrapping_add_signed(e8 as i16);
       
        self.clock_tick();  
    }

    fn JP_a16(&mut self, cond: Cond){
        self.clock_tick();
        self.clock_tick();
        self.clock_tick();
        match cond {
            Cond::Z => {if !self.reg.get_z() {return;} },
            Cond::C => {if !self.reg.get_c() {return;}},
            Cond::NC => {if self.reg.get_c(){return;}},
            Cond::NZ => {if self.reg.get_z(){return;}},
            Cond::NONE=> {},
        }
        let lo: u8 = self.bus.read(self.reg.pc);
        self.reg.pc += 1;
        let hi: u8 = self.bus.read(self.reg.pc);
        self.reg.pc += 1;
        let addr:u16 = ((hi as u16) << 8) | lo as u16 ;
        self.reg.pc = addr;
        self.clock_tick(); 
    }

    fn JP_HL(&mut self){
        self.reg.pc = self.reg.get_hl();
        self.clock_tick();
    }

    fn CALL(&mut self, cond: Cond){
        self.clock_tick();
        self.clock_tick();
        self.clock_tick();
        match cond {
            Cond::Z => {if !self.reg.get_z() {return;} },
            Cond::C => {if !self.reg.get_c() {return;}},
            Cond::NC => {if self.reg.get_c(){return;}},
            Cond::NZ => {if self.reg.get_z(){return;}},
            Cond::NONE=> {},
        }
        let pc_hi: u8 = ((self.reg.pc & 0xFF00) >> 8) as u8;
        let pc_lo: u8 = (self.reg.pc & 0x00FF) as u8;
        self.stkpush(pc_hi);
        self.stkpush(pc_lo);
        let lo: u8 = self.bus.read(self.reg.pc);
        self.reg.pc += 1;
        let hi: u8 = self.bus.read(self.reg.pc);
        self.reg.pc += 1;
        let addr: u16 = ((hi as u16) << 8) | lo as u16 ;
        self.reg.pc = addr;
        self.clock_tick();
        self.clock_tick();
        self.clock_tick();
    }

    fn RST(&mut self, lo: u8){
        let pc_hi: u8 = ((self.reg.pc & 0xFF00) >> 8) as u8;
        let pc_lo: u8 = (self.reg.pc & 0x00FF) as u8;
        self.stkpush(pc_hi);
        self.stkpush(pc_lo);
        self.reg.pc = lo as u16;
        self.clock_tick();
        self.clock_tick();
        self.clock_tick();
    }

    ///only to be used by cb
    fn set_reg_cb(&mut self, op: u8, val: u8){
        match op{
            0 => self.reg.a = val,
            1 => self.reg.c = val,
            2 => self.reg.d = val,
            3 => self.reg.e = val,
            4 => self.reg.h = val,
            5 => self.reg.l = val,
            6 => self.bus.write(self.reg.get_hl(), val),
            7 => self.reg.a = val,
            _=>panic!("Invalid cb reg")
        };
    }

    fn CB(&mut self){

        //B C D E H L HL A

        //byte to decode
        let cb_op: u8 = self.bus.read(self.reg.pc);
        //value being used (certain operations)
        let bit_val: u8 = (cb_op >> 3) & 0b111;
        //opcode of the instruction RLC RRC RL RR SLA SRA SWAP SRL BIT RES SET
        let bit_op: u8 = (cb_op >> 6) & 0b11;
        self.reg.pc += 1;
        self.clock_tick();
        //gets value at desired cb reg
        let val: u8 = match cb_op & 0b111 {
            0 => self.reg.get_reg8(Reg8::A),
            1 => self.reg.get_reg8(Reg8::C),
            2 => self.reg.get_reg8(Reg8::D),
            3 => self.reg.get_reg8(Reg8::E),
            4 => self.reg.get_reg8(Reg8::H),
            5 => self.reg.get_reg8(Reg8::L),
            6 => self.bus.read(self.reg.get_hl()),
            7 => self.reg.get_reg8(Reg8::A),
            _=>panic!("Invalid cb reg")
        };
    
        if (cb_op & 0b111) == 6{
            //HL case
            self.clock_tick();
            self.clock_tick();
        }
        let cf: u8 =  if self.reg.get_c() {1} else {0};
        match bit_op {
            1 => {
                //BIT
                self.reg.set_z( val & (1 << bit_val) == 0);
                self.reg.set_n(false);
                self.reg.set_h(true);
                self.clock_tick();
                return;
            },
            2 => {
                //RES
                let new_val: u8 = val & !(1 << bit_val);
                self.set_reg_cb(cb_op & 0b111 , new_val);
                self.clock_tick();
                if (cb_op & 0b111) == 6{
                    //HL case
                    self.clock_tick();
                }
                return;
            },
            3 => {
                //SET
                let new_val: u8 = val | (1 << bit_val);
                self.set_reg_cb(cb_op & 0b111 , new_val);
                self.clock_tick();
                return;
            },
            _=> match bit_val {
                0 => {
                    //RLC
                    let mut setC: bool = false;
                    let mut res: u8 = (val << 1) & 0xFF;
                    if val & (1 << 7) != 0 {
                        res |= 1;
                        setC = true;
                    }
                    self.set_reg_cb(cb_op & 0b111, res);
                    self.reg.set_z(res == 0);
                    self.reg.set_n(false);
                    self.reg.set_h(false);
                    self.reg.set_c(setC);
                    self.clock_tick();
                    return;
                },
                1 => {
                    //RRC
                    let old: u8 = val;
                    let mut new: u8 = val >> 1;
                    new |= old << 7;
        
                    self.set_reg_cb(cb_op & 0b11, new);
                    self.reg.set_z(new == 0);
                    self.reg.set_n(false);
                    self.reg.set_h(false);
                    self.reg.set_c((old & 1) != 0);
                    self.clock_tick();
                    return;
                },
                2 => {
                    //RL
                    let old: u8 = val;
                    let mut new: u8 =  old << 1;
                    new |= cf;
                    self.set_reg_cb(cb_op & 0b111, new);
                    self.reg.set_z(new == 0);
                    self.reg.set_n(false);
                    self.reg.set_h(false);
                    self.reg.set_c(!!(old & 0x80) != 0);
                    self.clock_tick();
                    return;
                },
                3 => {
                    //RR
                    let old: u8 = val;
                    let mut new: u8 = old >> 1;
                    new |= cf << 7;
                    self.set_reg_cb(cb_op & 0b111, new);
                    self.reg.set_z(new == 0);
                    self.reg.set_n(false);
                    self.reg.set_h(false);
                    self.reg.set_c(old & 1 != 0);
                    self.clock_tick();
                    return;
                },
                4 => {
                    //SLA
                    let old: u8 = val;
                    let mut new: u8 =  old << 1;

                    self.set_reg_cb(cb_op & 0b111, new);
                    self.reg.set_z(new == 0);
                    self.reg.set_n(false);
                    self.reg.set_h(false);
                    self.reg.set_c(!!(old & 0x80) != 0);
                    self.clock_tick();
                    return;
                },
                5 => {
                    //SRA
                    let u: u8 = ((val as i8) >> 1) as u8;
                    self.set_reg_cb(cb_op & 0b111, u);
        
                    self.reg.set_z(u != 0);
                    self.reg.set_n(false);
                    self.reg.set_h(false);
                    self.reg.set_c(val & 1 != 0);  
                    self.clock_tick();
                    return;
                },
                6 => {
                    //SWAP
                    let new: u8 = ((val & 0xF0) >> 4) | ((val & 0xF) << 4);
                    self.set_reg_cb(cb_op & 0b111, new);
                    self.reg.set_z(new == 0);
                    self.reg.set_n(false);
                    self.reg.set_h(false);
                    self.reg.set_c(false);
                    self.clock_tick();
                    return;
                },
                7 => {
                    //SRL
                    let u: u8 = val >> 1;
                    self.set_reg_cb(cb_op & 0b111, u);
        
                    self.reg.set_z(u != 0);
                    self.reg.set_n(false);
                    self.reg.set_h(false);
                    self.reg.set_c(val & 1 != 0);   
                    self.clock_tick();      
                }
                _=> panic!("Invalid CB")
            }
        }
    }

    fn DAA(&mut self){
        if !self.reg.get_n() {
            if self.reg.get_c() || self.reg.a > 0x99{
                self.reg.a = self.reg.a + 0x60;
                self.reg.set_c(true);
            }
            if self.reg.get_h() || ((self.reg.a & 0x0f) > 0x09){
                self.reg.a = self.reg.a + 0x6;
            }
        }
        else{
            if self.reg.get_c() {
                self.reg.a = self.reg.a - 0x60;
            }
            if self.reg.get_h() {
                self.reg.a = self.reg.a - 0x6;
            }
        }
        self.reg.set_z( self.reg.a == 0);
        self.reg.set_h(false);

        self.clock_tick();
    }

    fn DI(&mut self){
        self.ime = false;
        self.clock_tick();
    }
    
    fn EI(&mut self){
        self.ime = true;
        self.clock_tick();
    }

    fn stkpush(&mut self, data: u8){
        self.reg.sp = self.reg.sp.wrapping_sub(1) ;
        self.bus.write(self.reg.sp, data);
    }

    fn stkpop(&mut self)->u8{

        let val: u8 = self.bus.read(self.reg.sp);
        self.reg.sp = self.reg.sp.wrapping_add(1);
        return val;
    }
}
