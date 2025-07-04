use crate::computer::card::Card;


#[derive(Clone, Debug)]
pub struct Via {
    pub interrupt: bool,
    pub timer1cnt: u16,
    pub timer2cnt: u16,
    pub timer1latch: u16,
    pub timer2latch: u16,
    pub ifr: u8,
    pub ier: u8,
    pub acr: u8,
    pub pcr: u8,
}

impl Card for Via {
    fn get_interrupt(&mut self) -> bool {
        return self.interrupt;
    }
    fn tick(&mut self) {
        // TODO tick timers and trigger interrupt if necessary
        if self.timer1cnt > 0 {
            self.interrupt = false;
            self.timer1cnt -= 1;
            if self.timer1cnt == 0 && (self.ier & 0x40) != 0 {
                self.ifr |= 0xC0;
                self.interrupt = true;
                if (self.acr & 0x40) != 0 {
                    self.timer1cnt = self.timer1latch;
                }
            }
        }
    }

    fn read(&mut self, reg: u16) -> u8 {
        // TODO return read value
        if reg == 4  {
            self.interrupt = false;
            self.ifr = 0;
            return (self.timer1cnt & 0xFF) as u8;
        } else if reg == 5 {
            return (self.timer1cnt >> 8) as u8;
        } else if reg == 6 {
            return (self.timer1latch & 0xFF) as u8;
        } else if reg == 7 {
            return (self.timer1latch >> 8) as u8;
        } else if reg == 0xB {
            return self.acr;
        } else if reg == 0xC {
            return self.pcr;
        } else if reg == 0xD {
            log::info!("ifr is {:0x}", self.ifr);
            return self.ifr;
        } else if reg == 0xE {
            return self.ier;
        }

        return 0;
    }

    fn write(&mut self, reg: u16, val: u8) {
        // Set registers to correct values
        log::info!("write to VIA reg {:?} value {:?}", reg, val);
        if reg == 4 {
            self.timer1latch |= val as u16;
        } else if reg == 5 {
            self.timer1latch |= (val as u16) << 8;
            self.timer1cnt = self.timer1latch;
            self.interrupt = false;
            self.ifr = 0;
        } else if reg == 6 {
            self.timer1latch |= val as u16;
        } else if reg == 7 {
            self.interrupt = false;
            self.ifr = 0;
            self.timer1latch |= (val as u16) << 8;
        } else if reg == 0xB {
            self.acr = val;
        } else if reg == 0xC {
            self.pcr = val;
        } else if reg == 0xD {
            self.ifr = val;
        } else if reg == 0xE {
            self.ier = val;
        }
    }
}