use crate::computer::card::Card;


#[derive(Debug, PartialEq, Eq)]
pub enum DiskCommand {
    Read = 0x20,
    Write = 0x30,
    None = 0,
}



impl TryFrom<u8> for DiskCommand {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            x if x == DiskCommand::Read as u8 => Ok(DiskCommand::Read),
            x if x == DiskCommand::Write as u8 => Ok(DiskCommand::Write),
            x if x == DiskCommand::None as u8 => Ok(DiskCommand::None),
            _ => Err(()),
        }
    }
}


#[derive(Debug)]
pub struct Cf {
    pub disk_cnt: u16,
    pub command: DiskCommand,
    pub disk: Vec<u8>,
    pub lba: u32,
}

impl Card for Cf {

    fn tick(&mut self) {
    }

    fn get_interrupt(&mut self) -> bool {
        false
    }

    fn read(&mut self, reg: u16) -> u8 {
        if self.disk.len() <= 0 {
            return 0;
        }
    
        if reg == 0 && self.command == DiskCommand::Read {
            let v = self.disk[(self.lba * 512 + self.disk_cnt as u32) as usize];
            //let _ = self.tx.send(ComputerMessage::Info(format!("read disk {:?} {:?} {:?}, {:#x}", self.lba, self.disk_cnt, (self.lba * 512 + self.disk_cnt as u32), v)));

            self.disk_cnt += 1;
            if self.disk_cnt > 512 {
                self.command = DiskCommand::None;
            }
            return v;
        } else if reg == 7 {
            if self.command != DiskCommand::None {
                return 0x58;
            }
            return 0x50;
        }
        

        return 0;
    }

    fn write(&mut self, addr: u16, value: u8) {

        let reg = addr & 7;

        if reg == 0 {
            if self.command == DiskCommand::Write {
                self.disk[(self.lba * 512 + self.disk_cnt as u32) as usize] = value;
                self.disk_cnt += 1;
                if self.disk_cnt > 512 {
                    self.command = DiskCommand::None;
                }
            }
        } else if reg == 2 {
            // TODO set number of sectors to read
        } else if reg == 3 {
            self.lba &= 0xFFFFFF00;
            self.lba |= value as u32;
        } else if reg == 4 {
            self.lba &= 0xFFFF00FF;
            self.lba |= (value as u32) << 8;
        } else if reg == 5 {
            self.lba &= 0xFF00FFFF;
            self.lba |= (value as u32) << 16;
        } else if reg == 6 {
            self.lba &= 0x00FFFFFF;
            self.lba |= ((value as u32) << 24) & 0xF;
        } else if reg == 7 {
            self.command = match value.try_into() {
                Ok(c) => c,
                Err(_) => DiskCommand::None,
            };
            if self.command != DiskCommand::None {
                // set count of bytes in sector to zero
                self.disk_cnt = 0;
            }
        }
    }
}