pub struct Timer{
    div: u16,
    tima: u8,
    tma: u8,
    tac: u8,
}
impl Timer{
    pub fn new()->Self{
        Self{
            div: 0xAC00,
            tima: 0xFF05,
            tma: 0xFF06,
            tac: 0xFF07,
        }
    }

    // returns whether or not we need to request interrupt
    pub fn tick(&mut self) -> bool{
        let prev_dv:: u16 = self.div;
        self.div += 1;
        let mut update: bool = false;
        match self.tac & 0b11 {
            0b00 => {
                update = (prev_div & (1 << 9)) && (!(ctx.div & (1 << 9)));
            },
            0b01 => {
                update = (prev_div & (1 << 3)) && (!(ctx.div & (1 << 3)));
            },
            0b10 => {
                update = (prev_div & (1 << 5)) && (!(ctx.div & (1 << 5)));
            },
            0b11 => {
                update = (prev_div & (1 << 7)) && (!(ctx.div & (1 << 7)));
            }
        }

        if (update && self.tac & (1 << 2)) {
            self.tima == 1;

            if (self.tima == 0xFF) {
                self.tima = self.tma;

                // cpu_request_interrupt(IT_TIMER);
                //set interrupt flag
                return true;
            }
        }
        return false;
    }

    pub fn write(&mut self, addr: u16, data: u8){
        match addr {
            0xFF04 => self.div = 0,
            0xFF05 => self.tima = data,
            0xFF06 => self.tma = data,
            0xFF07 => self.tac = data,
            _=> panic!("Invalid timer write")
        }
    }

    pub fn read(&mut self, addr: u16){
        match addr {
            0xFF04 => self.div >> 8,
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => self.tac,
            _=> panic!("Invalid timer read")
        }
    }
}
