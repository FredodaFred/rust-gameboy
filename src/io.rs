pub struct IO{
    serialData: [char; 2],

}
impl IO{
    pub fn new()-> Self{
        Self{
            serialData: [' ', ' ']
        }
    }
    pub fn write(&mut self, addr: u16, val: u8){
        //println!("addr: {:#04x}, val: {:#02x}", addr, val);
        if addr == 0xFF01{
            println!(" sd0 {}", val);
            self.serialData[0] = val as char;
        }
        else if addr == 0xFF02{
            self.serialData[1] = val as char;
            println!(" sd1 {}", val);
        }
        else if addr <= 0xFF07 && addr >= 0xFF07{
            //timer
        }
        else if addr == 0xFF0F{
            //return cpu flags
        }

    }

    pub fn read(&mut self, addr: u16) -> u8{
        if addr == 0xFF01{
            return self.serialData[0] as u8
        }
        else if addr == 0xFF02{
            return self.serialData[1] as u8
        }
        else if addr <= 0xFF07 && addr >= 0xFF07{
            //timer
        } 
        
        //println!("{}, {}",self.serialData[0], self.serialData[1]);
        return 0;
    }
}