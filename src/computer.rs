use std::sync::mpsc;
use std::time;
use std::thread;

mod decode;
#[derive(Clone, Debug)]
pub struct Info {
    pub msg: String,
    pub qty: u64,
}




#[derive(Eq, Hash, PartialEq, Clone, Copy, Debug)]
pub enum AdressingMode {
    Immediate = 0,
    ZeroPage = 1,
    ZeroPageX = 2,
    Absolute = 3,
    AbsoluteX = 4,
    AbsoluteY = 5,
    IndirectX = 6,
    IndirectY = 7,
    Indirect = 8,
    ZeroPageY = 9,
    Accumulator = 10,
    ZeroPageIndirect = 11,
    None = 12,   
}

pub enum ControllerMessage {
    ButtonPressed(String),
    GetMemory,
    GetProc,
    Reset,
    SendChar(char)
}

pub enum ComputerMessage {
    Info(String),
    Output(u8),
    Memory(Vec<u8>),
    Processor(Processor)
}

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


#[derive(Clone, Debug)]
pub struct Processor {
    pub flags: u8,
    pub acc: u8,
    pub rx: u8,
    pub ry: u8,
    pub pc: u16,
    pub sp: u8,
    pub clock: u128,
    pub inst: u8,
}

#[derive(Debug)]
pub struct Computer {
    log_level: u8,
    processor: Processor,
    paused: bool,
    step: bool,
    lba: u32,
    disk_cnt: u16,
    command: DiskCommand,
    speed: u64,
    data: Vec<u8>,
    disk: Vec<u8>,
    tx: mpsc::Sender<ComputerMessage>,
    rx: mpsc::Receiver<ControllerMessage>,
    pub info: Vec<Info>,
}
const FLAG_C: u8 = 1;
const FLAG_Z: u8 = 2;
const FLAG_I: u8 = 4;
const FLAG_D: u8 = 8;
const FLAG_O: u8 = 0x40;
const FLAG_N: u8 = 0x80;

const CF_ADDRESS: u16 = 0xFFD0;

impl Computer {
    pub fn new(tx: mpsc::Sender<ComputerMessage>, rx:  mpsc::Receiver<ControllerMessage>, mut data: Vec<u8>, disk: Vec<u8>) -> Computer {
        let rom_size = data.len();
        let mut ram: Vec<u8> = vec![0; 0x10000-rom_size];
        ram.fill(0);
        ram.append(&mut data);


        Self {
            log_level: 0,
            data: ram,
            disk,
            lba: 0,
            disk_cnt: 0,
            command: DiskCommand::None,
            tx,
            rx,
            paused: false,
            step: false,
            speed: 0,
            info: vec![],
            processor: Processor {
                flags: 0b00110000,
                acc: 0,
                rx: 0,
                ry: 0,
                pc: 0x400,
                sp: 0,
                
                clock: 0,
                inst: 0xea,
            },
        }
    }

    pub fn step(&mut self) -> bool {
        while let Some(message) = self.rx.try_iter().next() {
            // Handle messages arriving from the controller.
            match message {
                ControllerMessage::GetMemory => {
                    let _ = self.tx.send(ComputerMessage::Memory(self.data.clone()));
                }
                ControllerMessage::GetProc => {
                    let _ = self.tx.send(ComputerMessage::Processor(self.processor.clone()));
                }
                ControllerMessage::Reset => {
                    self.reset();
                }
                ControllerMessage::SendChar(c) => {
                    self.data[0xFFE0] = c as u8;
                    self.data[0xFFE1] = 0x08;
                }
                _ => {},
            };
        }

        if self.paused && !self.step {
            thread::sleep(time::Duration::from_millis(100));
            return true;
        }

        if (self.paused && self.step) || !self.paused {
            self.step = false;
            let _ = self.run_instruction();
            if self.speed > 0 {
                thread::sleep(time::Duration::from_millis(self.speed));
            }
        }

        true
    }

    fn read(&mut self, addr: u16) -> u8 {
        // Ignore IO
        if self.disk.len() > 0 && (addr >= CF_ADDRESS)  && addr < (CF_ADDRESS + 0x10) {
            let reg = addr & 7;
            // let _ = self.tx.send(ComputerMessage::Info(format!("disk read reg {:?}", reg)));
            if reg == 0 {
                if self.command == DiskCommand::Read {
                    let v = self.disk[(self.lba * 512 + self.disk_cnt as u32) as usize];
                    //let _ = self.tx.send(ComputerMessage::Info(format!("read disk {:?} {:?} {:?}, {:#x}", self.lba, self.disk_cnt, (self.lba * 512 + self.disk_cnt as u32), v)));

                    self.disk_cnt += 1;
                    if self.disk_cnt > 512 {
                        self.command = DiskCommand::None;
                    }
                    return v;
                }
                return 0;
            } else if reg == 7 {
                if self.command != DiskCommand::None {
                    return 0x58;
                }
                return 0x50;
            }
        } if addr == 0xFFE0 {
            self.data[0xFFE1] = 0;
            let v = self.data[0xFFE0];
            self.data[0xFFE0] = 0;
            return v;
        }
        return self.data[addr as usize];
    }

    fn write(&mut self, addr: u16, value: u8) {
        if self.disk.len() > 0 && (addr >= CF_ADDRESS)  && addr < (CF_ADDRESS + 0x10) {
            
            let reg = addr & 7;
            //let _ = self.tx.send(ComputerMessage::Info(format!("disk write {:?} {:#x}", reg, value)));
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
                
                //let _ = self.tx.send(ComputerMessage::Info(format!("disk command {:?}", self.command)));

            }

        } else if addr == 0xFFE0 {
            // Serial out
            let _ = self.tx.send(ComputerMessage::Output(value));
        }

        self.data[addr as usize] = value;
        
    }


    pub fn reset(&mut self) {
        self.paused = true;
        self.lba = 0;
        self.processor.clock = 0;
        self.disk_cnt = 0;
        self.command = DiskCommand::None;
        self.processor.pc = self.get_word(0xfffc);
        self.paused = false;
    }

    fn run_instruction(&mut self) {
        let inst = self.read(self.processor.pc);
        self.processor.inst = inst;
        let opcode = decode::get_opcode_name(self.processor.inst);

        //self.add_info(format!("{:#x} - running instruction {} ({:#x})", self.processor.pc, opcode, inst));

        match opcode {
            "ADC" => self.adc(),
            "AND" => self.and(),
            "ASL" => self.asl(),
            "BCC" => self.bcc(),
            "BCS" => self.bcs(),
            "BEQ" => self.beq(),
            "BIT" => self.bit(),
            "BMI" => self.bmi(),
            "BNE" => self.bne(),
            "BPL" => self.bpl(),
            "BRA" => self.bra(),
            "BRK" => self.brk(),
            "BVC" => self.bvc(),
            "BVS" => self.bvs(),
            "CLC" => self.clc(),
            "CLD" => self.cld(),
            "CLI" => self.cli(),
            "CLV" => self.clv(),
            "CMP" => self.cmp(),
            "CPX" => self.cpx(),
            "CPY" => self.cpy(),
            "DEC" => self.dec(),
            "DEX" => self.dex(),
            "DEY" => self.dey(),
            "EOR" => self.eor(),
            "INC" => self.inc(),
            "INX" => self.inx(),
            "INY" => self.iny(),
            "JMP" => self.jmp(),
            "JSR" => self.jsr(),
            "LDA" => self.lda(),
            "LDX" => self.ldx(),
            "LDY" => self.ldy(),
            "LSR" => self.lsr(),
            "NOP" => self.nop(),
            "ORA" => self.ora(),
            "PHA" => self.pha(),
            "PHX" => self.phx(),
            "PHY" => self.phy(),
            "PHP" => self.php(),
            "PLA" => self.pla(),
            "PLX" => self.plx(),
            "PLY" => self.ply(),
            "PLP" => self.plp(),
            "ROL" => self.rol(),
            "ROR" => self.ror(),
            "RTI" => self.rti(),
            "RTS" => self.rts(),
            "SBC" => self.sbc(),
            "SEC" => self.sec(),
            "SED" => self.sed(),
            "SEI" => self.sei(),
            "STA" => self.sta(),
            "STX" => self.stx(),
            "STY" => self.sty(),
            "TAX" => self.tax(),
            "TAY" => self.tay(),
            "TSX" => self.tsx(),
            "TXA" => self.txa(),
            "TXS" => self.txs(),
            "TYA" => self.tya(),

            "BBS0" => self.bbs(0),
            "BBS1" => self.bbs(1),
            "BBS2" => self.bbs(2),
            "BBS3" => self.bbs(3),
            "BBS4" => self.bbs(4),
            "BBS5" => self.bbs(5),
            "BBS6" => self.bbs(6),
            "BBS7" => self.bbs(7),

            "BBR0" => self.bbr(0),
            "BBR1" => self.bbr(1),
            "BBR2" => self.bbr(2),
            "BBR3" => self.bbr(3),
            "BBR4" => self.bbr(4),
            "BBR5" => self.bbr(5),
            "BBR6" => self.bbr(6),
            "BBR7" => self.bbr(7),

            "STZ" => self.stz(),

            "NOP2" => {
                self.nop();
                self.nop()
            },
            "NOP3" => {
                self.nop();
                self.nop();
                self.nop();

            }
            
            _ => {
                //panic!("Running instruction nop : {:x?}", inst);
                self.nop();
            },
        };
    }

    fn add_info(&mut self, info: String) {
        let _ = self.tx.send(ComputerMessage::Info(info.clone()));
        let len = self.info.len();
        if len > 0 && self.info[len-1].msg == info {
            let last_element = self.info.pop().unwrap();
            self.info.push(Info {msg: info, qty: last_element.qty + 1});
            self.paused = true;
            let _ = self.tx.send(ComputerMessage::Info(String::from("Computer paused")));
        } else {
            self.info.push(Info {msg: info, qty: 1});
        }

    }

    fn cld(&mut self) {
        if self.log_level > 0 {
            self.add_info(format!("{:#x} - Running instruction cld: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize]));
        }
        self.processor.pc = self.processor.pc.wrapping_add(1);
        self.processor.flags = self.processor.flags & !FLAG_D;
        self.processor.clock = self.processor.clock.wrapping_add(2);
    }

    fn txs(&mut self) {
        if self.log_level > 0 {
            self.add_info(format!("{:#x} - Running instruction txs: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize]));
        }
        self.processor.pc = self.processor.pc.wrapping_add(1);
        self.processor.clock  = self.processor.clock.wrapping_add(2);
        self.processor.sp = self.processor.rx;
    }

    fn tsx(&mut self) {
        self.processor.flags = Self::set_flags( self.processor.flags, self.processor.sp);
        if self.log_level > 0 {
            self.add_info(format!("{:#x} - Running instruction tsx: {:#x} val: {:#x} flags:{:#x} ", self.processor.pc, self.data[(self.processor.pc) as usize], self.processor.sp, self.processor.flags));
        }
        self.processor.pc = self.processor.pc.wrapping_add(1);
        self.processor.clock  = self.processor.clock.wrapping_add(2);
        self.processor.rx = self.processor.sp;
    }

    fn tya(&mut self) {
        if self.log_level > 0 {
            self.add_info(format!("{:#x} - Running instruction tya: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize]));
        }
        self.processor.pc = self.processor.pc.wrapping_add(1);
        self.processor.clock  = self.processor.clock.wrapping_add(2);
        self.processor.acc = self.processor.ry;
        self.processor.flags = Self::set_flags(self.processor.flags, self.processor.acc);
    }

    fn tay(&mut self) {
        if self.log_level > 0 {
            self.add_info(format!("{:#x} - Running instruction tay: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize]));
        }
        self.processor.pc = self.processor.pc.wrapping_add(1);
        self.processor.clock  = self.processor.clock.wrapping_add(2);
        self.processor.ry = self.processor.acc;
        self.processor.flags = Self::set_flags(self.processor.flags, self.processor.ry);
    }

    fn tax(&mut self) {
        if self.log_level > 0 {
            self.add_info(format!("{:#x} - Running instruction tax: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize]));
        }
        self.processor.pc = self.processor.pc.wrapping_add(1);
        self.processor.clock  = self.processor.clock.wrapping_add(2);
        self.processor.rx = self.processor.acc;
        self.processor.flags = Self::set_flags(self.processor.flags, self.processor.rx);
    }

    fn txa(&mut self) {
        if self.log_level > 0 {
            self.add_info(format!("{:#x} - Running instruction txa: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize]));
        }
        self.processor.flags = Self::set_flags(self.processor.flags, self.processor.rx);
        self.processor.pc = self.processor.pc.wrapping_add(1);
        self.processor.clock  = self.processor.clock.wrapping_add(2);
        self.processor.acc = self.processor.rx;
    }

    /// Jump to subroutine
    fn jsr(&mut self) {
        // Place current address on stack
        let sp: u16 = (self.processor.sp as u16 + 0x100 as u16).into();
        let sp1: u16 = (self.processor.sp.wrapping_sub(1) as u16 + 0x100 as u16).into();
        
        let this_pc = self.processor.pc + 2;

        self.write(sp, ((this_pc>>8) & 0xff) as u8);
        self.write(sp1, (this_pc & 0xff) as u8);
        // Send to new address
        let addr = self.get_word(self.processor.pc + 1);
        if self.log_level > 0 {
            self.add_info(format!("{:#x} - Running instruction jsr to: {:#x}", self.processor.pc, addr));
        }
        self.processor.sp = self.processor.sp.wrapping_sub(2);
        self.processor.clock  = self.processor.clock.wrapping_add(6);
        self.processor.pc = addr;
    }

    fn brk(&mut self) {
        let sp: u16 = (self.processor.sp as u16 + 0x100 as u16).into();
        let sp1: u16 = (self.processor.sp.wrapping_sub(1) as u16 + 0x100 as u16).into();
        let sp2: u16 = (self.processor.sp.wrapping_sub(2) as u16 + 0x100 as u16).into();

        let this_pc = self.processor.pc + 2;

        self.write(sp, ((this_pc>>8) & 0xff) as u8);
        self.write(sp1, (this_pc & 0xff) as u8);
        self.write(sp2, (self.processor.flags) | 0x30);

        self.processor.flags |= FLAG_I;
        self.processor.sp = self.processor.sp.wrapping_sub(3);

        let new_addr: u16 = self.get_word(0xfffe);
        if self.log_level > 0 {
            self.add_info(format!("{:#x} - Running instruction brk ({:#x}) to: {:#x} flags: {:#b}", self.processor.pc, self.processor.inst, new_addr, self.processor.flags));
        }
        self.processor.pc = new_addr;

        self.processor.clock  = self.processor.clock.wrapping_add(7);
    }

    fn rti(&mut self) {
        // Place current address on stack
        let sp1: u16 = (self.processor.sp.wrapping_add(1) as u16 + 0x100 as u16).into();
        let sp2: u16 = (self.processor.sp.wrapping_add(2) as u16 + 0x100 as u16).into();
        let sp3: u16 = (self.processor.sp.wrapping_add(3) as u16 + 0x100 as u16).into();
        let high_byte = self.read(sp3);
        let low_byte = self.read(sp2);
        let flags = self.read(sp1);
        // Unset interrupt disabled flag
        self.processor.flags = flags;
        let addr: u16 = low_byte as u16 | ((high_byte as u16) << 8) as u16;
        // Send to new address
        if self.log_level > 0 {
            self.add_info(format!("{:#x} - Running instruction rti to: {:#x} flags: {:#x}", self.processor.pc, addr, self.processor.flags));
        }
        self.processor.sp = self.processor.sp.wrapping_add(3);
        self.processor.pc = addr;
        self.processor.clock  = self.processor.clock.wrapping_add(7);
    }

    fn rts(&mut self) {
        // Place current address on stack
        let sp1: u16 = (self.processor.sp.wrapping_add(1) as u16 + 0x100 as u16).into();
        let sp2: u16 = (self.processor.sp.wrapping_add(2) as u16 + 0x100 as u16).into();
        let low_byte = self.read(sp1);
        let high_byte = self.read(sp2);
        let addr: u16 = low_byte as u16 | ((high_byte as u16) << 8) as u16;
        // Send to new address
        if self.log_level > 0 {
            self.add_info(format!("{:#x} - Running instruction rts to: {:#x}", self.processor.pc, addr));
        }
        self.processor.sp = self.processor.sp.wrapping_add(2);
        self.processor.pc = addr.wrapping_add(1);
        self.processor.clock  = self.processor.clock.wrapping_add(6);
    }

    /// Clear carry flag
    fn clc(&mut self) {
        self.processor.flags &= !FLAG_C;
        if self.log_level > 0 {
            self.add_info(format!("{:#x} - Running instruction clc: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize]));
        }
        self.processor.pc = self.processor.pc.wrapping_add(1);
        self.processor.clock  = self.processor.clock.wrapping_add(2);
    }

    /// Set carry flag
    fn sec(&mut self) {
        self.processor.flags |= FLAG_C;
        if self.log_level > 0 {
            self.add_info(format!("{:#x} - Running instruction sec: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize]));
        }
        self.processor.pc = self.processor.pc.wrapping_add(1);
        self.processor.clock  = self.processor.clock.wrapping_add(2);
    }

    /// Set decimal flag
    fn sed(&mut self) {
        self.processor.flags |= FLAG_D;
        if self.log_level > 0 {
            self.add_info(format!("{:#x} - Running instruction sed: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize]));
        }
        self.processor.pc = self.processor.pc.wrapping_add(1);
        self.processor.clock  = self.processor.clock.wrapping_add(2);
    }

    /// Clear interrupt disabled flag
    fn cli(&mut self) {
        self.processor.flags &= !FLAG_I;
        if self.log_level > 0 {
            self.add_info(format!("{:#x} - Running instruction cli: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize]));
        }
        self.processor.pc = self.processor.pc.wrapping_add(1);
        self.processor.clock  = self.processor.clock.wrapping_add(2);
    }

    /// Set interrupt disabled flag
    fn sei(&mut self) {
        self.processor.flags |= FLAG_I;
        if self.log_level > 0 {
            self.add_info(format!("{:#x} - Running instruction sei: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize]));
        }
        self.processor.pc = self.processor.pc.wrapping_add(1);
        self.processor.clock  = self.processor.clock.wrapping_add(2);
    }

    /// clear overflow flag
    fn clv(&mut self) {
        self.processor.flags &= !FLAG_O;
        if self.log_level > 0 {
            self.add_info(format!("{:#x} - Running instruction clv: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize]));
        }
        self.processor.pc = self.processor.pc.wrapping_add(1);
        self.processor.clock  = self.processor.clock.wrapping_add(2);
    }

    /// Push accumulator to stack
    fn pha(&mut self) {
        let addr: u16 = (self.processor.sp as u16 + 0x100 as u16).into();
        
        self.write(addr, self.processor.acc);

        if self.log_level > 0 {
            self.add_info(format!("{:#x} - Running instruction pha at: {:#x} val: {:#x}", self.processor.pc, addr, self.processor.acc));
        }
        self.processor.sp = self.processor.sp.wrapping_sub(1);
        self.processor.pc = self.processor.pc.wrapping_add(1);
        self.processor.clock  = self.processor.clock.wrapping_add(3);
    }

    /// Push X to stack
    fn phx(&mut self) {
        let addr: u16 = (self.processor.sp as u16 + 0x100 as u16).into();
        
        self.write(addr, self.processor.rx);

        if self.log_level > 0 {
            self.add_info(format!("{:#x} - Running instruction phx at: {:#x} val: {:#x}", self.processor.pc, addr, self.processor.acc));
        }
        self.processor.sp = self.processor.sp.wrapping_sub(1);
        self.processor.pc = self.processor.pc.wrapping_add(1);
        self.processor.clock  = self.processor.clock.wrapping_add(3);
    }
    

    /// Push Y to stack
    fn phy(&mut self) {
        let addr: u16 = (self.processor.sp as u16 + 0x100 as u16).into();
        
        self.write(addr, self.processor.ry);

        if self.log_level > 0 {
            self.add_info(format!("{:#x} - Running instruction phx at: {:#x} val: {:#x}", self.processor.pc, addr, self.processor.acc));
        }
        self.processor.sp = self.processor.sp.wrapping_sub(1);
        self.processor.pc = self.processor.pc.wrapping_add(1);
        self.processor.clock  = self.processor.clock.wrapping_add(3);
    }

    /// Push flags to stack
    fn php(&mut self) {
        let addr: u16 = (self.processor.sp as u16 + 0x100 as u16).into();

        self.write(addr, self.processor.flags | 0x30);
        if self.log_level > 0 {
            self.add_info(format!("{:#x} - Running instruction php at: {:#x} flags: {:#x}", self.processor.pc, addr, self.processor.flags | 0x30));
        }
        self.processor.sp = self.processor.sp.wrapping_sub(1);
        self.processor.pc = self.processor.pc.wrapping_add(1);
        self.processor.clock  = self.processor.clock.wrapping_add(3);
    }

    /// Pull stack to accumulator
    fn pla(&mut self) {
        self.processor.sp = self.processor.sp.wrapping_add(1);
        let addr: u16 = (self.processor.sp as u16 + 0x100 as u16).into();
        
        self.processor.acc = self.read(addr);
        let flags = self.processor.flags;
        self.processor.flags = Self::set_flags(flags, self.processor.acc);
        if self.log_level > 0 {
            self.add_info(format!("{:#x} - Running instruction pla at: {:#x} val: {:#x}", self.processor.pc, addr, self.processor.acc));
        }
        self.processor.pc = self.processor.pc.wrapping_add(1);
        self.processor.clock  = self.processor.clock.wrapping_add(4);
    }

    /// Pull stack to X
    fn plx(&mut self) {
        self.processor.sp = self.processor.sp.wrapping_add(1);
        let addr: u16 = (self.processor.sp as u16 + 0x100 as u16).into();
        
        self.processor.rx = self.read(addr);
        let flags = self.processor.flags;
        self.processor.flags = Self::set_flags(flags, self.processor.rx);
        if self.log_level > 0 {
            self.add_info(format!("{:#x} - Running instruction plx at: {:#x} val: {:#x}", self.processor.pc, addr, self.processor.acc));
        }
        self.processor.pc = self.processor.pc.wrapping_add(1);
        self.processor.clock  = self.processor.clock.wrapping_add(4);
    }

    /// Pull stack to Y
    fn ply(&mut self) {
        self.processor.sp = self.processor.sp.wrapping_add(1);
        let addr: u16 = (self.processor.sp as u16 + 0x100 as u16).into();
        
        self.processor.ry = self.read(addr);
        let flags = self.processor.flags;
        self.processor.flags = Self::set_flags(flags, self.processor.ry);
        if self.log_level > 0 {
            self.add_info(format!("{:#x} - Running instruction ply at: {:#x} val: {:#x}", self.processor.pc, addr, self.processor.acc));
        }
        self.processor.pc = self.processor.pc.wrapping_add(1);
        self.processor.clock  = self.processor.clock.wrapping_add(4);
    }

    // 0X28 Pull value from the stack into the processor registers
    fn plp(&mut self) {
        self.processor.sp = self.processor.sp.wrapping_add(1);
        let addr: u16 = (self.processor.sp as u16 + 0x100 as u16).into();
        
        self.processor.flags = self.read(addr);
        if self.log_level > 0 {
            self.add_info(format!("{:#x} - Running instruction plp at: {:#x} flags: {:#x}", self.processor.pc, addr, self.processor.flags));
        }
        self.processor.pc = self.processor.pc.wrapping_add(1);
        self.processor.clock  = self.processor.clock.wrapping_add(4);
    }


    fn get_ld_adddr(&mut self, addressing_mode: AdressingMode) -> u16 {
        if self.log_level > 3 {
            self.add_info(format!("{:#x} - Getting address with mode {:?} for inst {:#x}", self.processor.pc, addressing_mode, self.processor.inst));
        }

        if addressing_mode == AdressingMode::Immediate {
            return self.processor.pc + 1;
        } else if addressing_mode == AdressingMode::Absolute {
            //Absolute adressing
            let start = self.processor.pc + 1;
            let addr = self.get_word(start);
            return addr;
        } else if addressing_mode == AdressingMode::AbsoluteX {
            //Absolute adressing
            let start = self.processor.pc + 1;
            let start_addr = self.get_word(start);
            let rx = self.processor.rx;
            let addr: u16 = start_addr.wrapping_add(rx.into());
            if self.log_level > 2 {
                self.add_info(format!("{:#x} - Getting absolute_x address from: {:#x} rx: {:#x} gives: {:#x}", self.processor.pc, start_addr, rx, addr));
            }
            return addr;
        } else if addressing_mode == AdressingMode::AbsoluteY {
            //Absolute adressing
            let start = self.processor.pc + 1;
            let start_addr = self.get_word(start);
            let ry = self.processor.ry;
            let addr: u16 = start_addr.wrapping_add(ry.into());
            if self.log_level > 2 {
                self.add_info(format!("{:#x} - Getting absolute_y address from: {:#x} ry: {:#x} gives: {:#x}", self.processor.pc, start_addr, ry, addr));
            }
            return addr;
        } else if addressing_mode == AdressingMode::ZeroPage {
            //Absolute adressing
            let start = self.processor.pc + 1;
            let addr: u16 = self.read(start).into();
            if self.log_level > 2 {
                self.add_info(format!("{:#x} - Getting ZERO_PAGE address from: {:#x} gives: {:#x}", self.processor.pc, start, addr));
            }
            return addr;
        } else if addressing_mode == AdressingMode::ZeroPageY {
            //Absolute adressing
            let start = self.processor.pc + 1;
            let start_addr = self.read(start).wrapping_add(self.processor.ry);
            let addr: u16 = start_addr.into();
            if self.log_level > 2 {
                self.add_info(format!("{:#x} - Getting ZERO_PAGE_Y address from: {:#x} with ry: {:#x} gives: {:#x}", self.processor.pc, start, self.processor.ry, addr));
            }
            return addr;
        } else if addressing_mode == AdressingMode::ZeroPageX {
            //Absolute adressing
            let start = self.processor.pc + 1;
            let start_addr = self.read(start).wrapping_add(self.processor.rx);
            let addr: u16 = start_addr.into();
            if self.log_level > 2 {
                self.add_info(format!("{:#x} - Getting ZERO_PAGE_X address from: {:#x} with rx: {:#x} gives: {:#x}", self.processor.pc, start, self.processor.rx, addr));
            }
            return addr;
        } else if addressing_mode == AdressingMode::IndirectY {
            //Absolute adressing
            let start = self.processor.pc + 1;
            let zp_addr = self.read(start);
            let base_addr = self.get_word(zp_addr.into());
            let addr: u16 = base_addr.wrapping_add(self.processor.ry as u16);
            if self.log_level > 2 {
                self.add_info(format!("{:#x} - Getting Indirect_Y address from: {:#x} with ry: {:#x} gives: {:#x}", self.processor.pc, start, self.processor.ry, addr));
            }
            return addr;
        } else if addressing_mode == AdressingMode::IndirectX {
            //Absolute adressing
            let start = self.processor.pc + 1;
            let zp_addr = self.read(start).wrapping_add(self.processor.rx);
            let addr: u16 = self.get_word(zp_addr.into());
            
            if self.log_level > 2 {
                self.add_info(format!("{:#x} - Getting Indirect_X address from: {:#x} with ry: {:#x} gives: {:#x}", self.processor.pc, start, self.processor.ry, addr));
            }
            return addr;
        } else if addressing_mode == AdressingMode::Accumulator {
            // Address ignored
            return 0;
        } else if addressing_mode == AdressingMode::ZeroPageIndirect {
            let start = self.processor.pc + 1;
            let zp_addr = self.read(start);
            let addr: u16 = self.get_word(zp_addr.into());
            if self.log_level > 2 {
                self.add_info(format!("{:#x} - Getting ZERO_PAGE_Indirect address from: {:#x} with zp addr: {:#x} gives: {:#x}", self.processor.pc, start, zp_addr, addr));
            }
            return addr;
        }
        self.add_info(format!("unknown addressing mode {:?} {:#x}", addressing_mode, self.processor.inst));

        self.paused = true;
        0
    }

    fn inc(&mut self) {
        let addressing_mode = decode::get_adressing_mode(self.processor.inst);
        let mut value: u8 = self.processor.acc;
        let mode = addressing_mode;
        self.processor.clock  = self.processor.clock.wrapping_add(2);
    

        let addr = self.get_ld_adddr(mode);
        if addressing_mode == AdressingMode::ZeroPage || addressing_mode == AdressingMode::ZeroPageX {
            value = self.read(addr);
            if self.log_level > 0 {
                self.add_info(format!("{:#x} - Running instruction inc ZP with effective addr: {:#x} and val: {:#x}", self.processor.pc, addr, value));
            }
            self.processor.pc = self.processor.pc.wrapping_add(2);
            self.processor.clock  = self.processor.clock.wrapping_add(3);
        } else if addressing_mode == AdressingMode::Absolute || addressing_mode == AdressingMode::AbsoluteX {
            value = self.read(addr);
            if self.log_level > 0 {
                self.add_info(format!("{:#x} - Running instruction inc ABS with effective addr: {:#x} and val: {:#x}", self.processor.pc, addr, value));
            }
            self.processor.pc = self.processor.pc.wrapping_add(3);
            self.processor.clock  = self.processor.clock.wrapping_add(4);
            if addressing_mode == AdressingMode::AbsoluteX {
                self.processor.clock  = self.processor.clock.wrapping_add(1);
            }
        }

        let result = value.wrapping_add(1);

        self.write(addr, result);

        self.processor.flags = Self::set_flags(self.processor.flags, result);
    }

    fn dec(&mut self) {
        let addressing_mode = decode::get_adressing_mode(self.processor.inst);
        let mut value: u8 = self.processor.acc;
        let mode = addressing_mode;

        self.processor.clock  = self.processor.clock.wrapping_add(2);

        let addr = self.get_ld_adddr(mode);
        if addressing_mode == AdressingMode::ZeroPage || addressing_mode == AdressingMode::ZeroPageX {
            value = self.read(addr);
            if self.log_level > 0 {
                self.add_info(format!("{:#x} - Running instruction dec ZP with effective addr: {:#x} and val: {:#x}", self.processor.pc, addr, value));
            }
            self.processor.pc = self.processor.pc.wrapping_add(2);
            self.processor.clock  = self.processor.clock.wrapping_add(3);
        } else if addressing_mode == AdressingMode::Absolute || addressing_mode == AdressingMode::AbsoluteX {
            value = self.read(addr);
            if self.log_level > 0 {
                self.add_info(format!("{:#x} - Running instruction dec ABS with effective addr: {:#x} and val: {:#x}", self.processor.pc, addr, value));
            }
            self.processor.pc = self.processor.pc.wrapping_add(3);
            self.processor.clock  = self.processor.clock.wrapping_add(4);
            if addressing_mode == AdressingMode::AbsoluteX {
                self.processor.clock  = self.processor.clock.wrapping_add(1);
            }
        }

        let result = value.wrapping_sub(1);

        self.write(addr, result);

        self.processor.flags = Self::set_flags(self.processor.flags, result);
    }

    fn ldx(&mut self) {
        let addressing_mode = decode::get_adressing_mode(self.processor.inst);
        let mut value: u8 = 0;
        let mode = addressing_mode;

        let addr = self.get_ld_adddr(mode);

        if addressing_mode == AdressingMode::Immediate {
            value = self.read(addr);
            if self.log_level > 0 {
                self.add_info(format!("{:#x} - Running instruction ldx val: {:#x}", self.processor.pc, value));
            }
            self.processor.pc = self.processor.pc.wrapping_add(2);
            self.processor.clock  = self.processor.clock.wrapping_add(2);
        } else if addressing_mode == AdressingMode::Absolute || addressing_mode == AdressingMode::AbsoluteX || addressing_mode == AdressingMode::AbsoluteY {
            value = self.read(addr);
            if self.log_level > 0 {
                self.add_info(format!("{:#x} - Running instruction ldx absolute with addr: {:#x} and val: {:#x}", self.processor.pc, addr, value));
            }
            self.processor.pc = self.processor.pc.wrapping_add(3);
            self.processor.clock  = self.processor.clock.wrapping_add(4);
        }else if addressing_mode == AdressingMode::ZeroPage || addressing_mode == AdressingMode::ZeroPageY {
            value = self.read(addr);
            if self.log_level > 0 {
                self.add_info(format!("{:#x} - Running instruction ldx ZP with effective addr: {:#x} and val: {:#x}", self.processor.pc, addr, value));
            }
            self.processor.pc = self.processor.pc.wrapping_add(2);
            self.processor.clock  = self.processor.clock.wrapping_add(3);
            if addressing_mode == AdressingMode::ZeroPageY {
                self.processor.clock  = self.processor.clock.wrapping_add(1);
            }
        }
        self.processor.rx = value;
        self.processor.flags = Self::set_flags(self.processor.flags, self.processor.rx);
    }

    fn ldy(&mut self) {
        let addressing_mode = decode::get_adressing_mode(self.processor.inst);
        let mut value: u8 = 0;
        let mode = addressing_mode;
        let addr = self.get_ld_adddr(mode);

        if addressing_mode == AdressingMode::Immediate {
            value = self.read(addr);
            if self.log_level > 0 {
                self.add_info(format!("{:#x} - Running instruction ldy val: {:#x}", self.processor.pc, value));
            }
            self.processor.pc = self.processor.pc.wrapping_add(2);
            self.processor.clock  = self.processor.clock.wrapping_add(2);
        } else if addressing_mode == AdressingMode::Absolute || addressing_mode == AdressingMode::AbsoluteX || addressing_mode == AdressingMode::AbsoluteY {
            value = self.read(addr);
            if self.log_level > 0 {
                self.add_info(format!("{:#x} - Running instruction ldy absolute with addr: {:#x} and val: {:#x}", self.processor.pc, addr, value));
            }
            self.processor.pc = self.processor.pc.wrapping_add(3);
            self.processor.clock  = self.processor.clock.wrapping_add(4);
        } else if addressing_mode == AdressingMode::ZeroPage || addressing_mode == AdressingMode::ZeroPageX {
            value = self.read(addr);
            if self.log_level > 0 {
                self.add_info(format!("{:#x} - Running instruction ldy ZP with effective addr: {:#x} and val: {:#x}", self.processor.pc, addr, value));
            }
            self.processor.pc = self.processor.pc.wrapping_add(2);
            self.processor.clock  = self.processor.clock.wrapping_add(3);
            if addressing_mode == AdressingMode::ZeroPageY {
                self.processor.clock  = self.processor.clock.wrapping_add(1);
            }
        }

        self.processor.ry = value;
        self.processor.flags = Self::set_flags(self.processor.flags, self.processor.ry);
        
    }

    fn lda(&mut self) {
        let addressing_mode = decode::get_adressing_mode(self.processor.inst);
        let value;
        let mode = addressing_mode;
        let addr = self.get_ld_adddr(mode);
        if addressing_mode == AdressingMode::Immediate {
            value = self.read(addr);
            
            self.processor.pc = self.processor.pc.wrapping_add(2);
            self.processor.clock  = self.processor.clock.wrapping_add(2);
        } else if addressing_mode == AdressingMode::Absolute || addressing_mode == AdressingMode::AbsoluteX|| addressing_mode == AdressingMode::AbsoluteY {
            value = self.read(addr);
            self.processor.pc = self.processor.pc.wrapping_add(3);
            self.processor.clock  = self.processor.clock.wrapping_add(4);
        } else if addressing_mode == AdressingMode::ZeroPage || addressing_mode == AdressingMode::ZeroPageX {
            value = self.read(addr);
            self.processor.pc = self.processor.pc.wrapping_add(2);
            self.processor.clock  = self.processor.clock.wrapping_add(3);
        } else if addressing_mode == AdressingMode::IndirectY || addressing_mode == AdressingMode::IndirectX {
            value = self.read(addr);
            self.processor.pc = self.processor.pc.wrapping_add(2);
            self.processor.clock  = self.processor.clock.wrapping_add(5);
        } else if addressing_mode == AdressingMode::ZeroPageIndirect {
            value = self.read(addr);
            self.processor.pc = self.processor.pc.wrapping_add(2);
            self.processor.clock  = self.processor.clock.wrapping_add(6);
        } else {
            panic!("This adressing mode is not implemented yet, sorry");
        }

        if self.log_level > 0 {
            self.add_info(format!("{:#x} - Running instruction lda {:?} addr: {:#x} val: {:#x}", self.processor.pc, addressing_mode, addr, value));
        }
        
        self.processor.acc = value;
        self.processor.flags = Self::set_flags(self.processor.flags, value);
    }

    fn asl(&mut self) {
        let addressing_mode = decode::get_adressing_mode(self.processor.inst);
        let mode = addressing_mode;

        let value;
        let addr = self.get_ld_adddr(mode);
        if self.log_level > 0 {
            self.add_info(format!("{:#x} - Running instruction asl {:?} with effective addr: {:#x}", self.processor.pc, mode, addr));
        }
        if mode == AdressingMode::Accumulator {
            value = self.processor.acc;
            self.processor.pc = self.processor.pc.wrapping_add(1);
            self.processor.clock  = self.processor.clock.wrapping_add(2);
        } else if mode == AdressingMode::Absolute || mode == AdressingMode::AbsoluteX {
            self.processor.pc = self.processor.pc.wrapping_add(3);
            self.processor.clock  = self.processor.clock.wrapping_add(6);
            value = self.read(addr);
        } else {
            self.processor.pc = self.processor.pc.wrapping_add(2);
            self.processor.clock  = self.processor.clock.wrapping_add(6);
            value = self.read(addr);
        }
        if value >> 7 & 1 == 1 {
            self.processor.flags |= FLAG_C;
        } else {
            self.processor.flags &= !FLAG_C;
        }
        if value == 0 {
            self.processor.flags |= FLAG_Z;
        } else {
            self.processor.flags &= !FLAG_Z;
        }

        let result = value << 1;
        if result >> 7 & 1 == 1 {
            self.processor.flags |= FLAG_N;
        } else {
            self.processor.flags &= !FLAG_N;
        }
        if mode == AdressingMode::Accumulator {
            self.processor.acc = result;
        } else {
            self.write(addr, result);
        }
    }

    fn lsr(&mut self) {
        let addressing_mode = decode::get_adressing_mode(self.processor.inst);
        let mode = addressing_mode;

        let value;
        let addr = self.get_ld_adddr(mode);
        if mode == AdressingMode::Accumulator {
            value = self.processor.acc;
        } else {
            value = self.read(addr);
        }
        let old_flags = self.processor.flags;
        if value & 1 == 1 {
            self.processor.flags |= FLAG_C;
        } else {
            self.processor.flags &= !FLAG_C;
        }
        if value == 0 {
            self.processor.flags |= FLAG_Z;
        } else {
            self.processor.flags &= !FLAG_Z;
        }

        let result = value >> 1;
        if result >> 7 & 1 == 1 {
            self.processor.flags |= FLAG_N;
        } else {
            self.processor.flags &= !FLAG_N;
        }
        if self.log_level > 0 {
            self.add_info(format!("{:#x} - Running instruction lsr val: {:#x} result: {:#x} flags: {:#x} old flags: {:#x}", self.processor.pc, value, result, self.processor.flags, old_flags));
        }
        if mode == AdressingMode::Accumulator {
            self.processor.pc = self.processor.pc.wrapping_add(1);
            self.processor.clock  = self.processor.clock.wrapping_add(2);
            self.processor.acc = result;
        } else if mode == AdressingMode::Absolute || mode == AdressingMode::AbsoluteX {
            self.processor.pc = self.processor.pc.wrapping_add(3);
            self.processor.clock  = self.processor.clock.wrapping_add(6);

            self.write(addr, result);
        } else {
            self.processor.pc = self.processor.pc.wrapping_add(2);
            self.processor.clock  = self.processor.clock.wrapping_add(5);
            self.write(addr, result);
        }

    }

    fn rol(&mut self) {
        let addressing_mode = decode::get_adressing_mode(self.processor.inst);
        let mode = addressing_mode;

        let value;
        let addr = self.get_ld_adddr(mode);
        if mode == AdressingMode::Accumulator {
            value = self.processor.acc;
            self.processor.pc = self.processor.pc.wrapping_add(1);
            self.processor.clock  = self.processor.clock.wrapping_add(2);
        } else if mode == AdressingMode::Absolute || mode == AdressingMode::AbsoluteX {
            value = self.processor.acc;
            self.processor.pc = self.processor.pc.wrapping_add(3);
            self.processor.clock  = self.processor.clock.wrapping_add(6);
        } else {
            self.processor.pc = self.processor.pc.wrapping_add(2);
            self.processor.clock  = self.processor.clock.wrapping_add(6);
            value = self.read(addr);
        }
        
        let old_flags = self.processor.flags;
        let result = (value << 1) | (self.processor.flags & FLAG_C);
        if value >> 7 & 1 == 1 {
            self.processor.flags |= FLAG_C;
        } else {
            self.processor.flags &= !FLAG_C;
        }
        if result == 0 {
            self.processor.flags |= FLAG_Z;
        } else {
            self.processor.flags &= !FLAG_Z;
        }
        if result >> 7 & 1 == 1 {
            self.processor.flags |= FLAG_N;
        } else {
            self.processor.flags &= !FLAG_N;
        }
        if self.log_level > 0 {
            self.add_info(format!("{:#x} - Running instruction rol val: {:#x} result: {:#x} flags: {:#x} old flags: {:#x}", self.processor.pc, value, result, self.processor.flags, old_flags));
        }
        if mode == AdressingMode::Accumulator {
            self.processor.acc = result;
        } else {
            self.write(addr, result);
        }
    }

    fn ror(&mut self) {
        let addressing_mode = decode::get_adressing_mode(self.processor.inst);
        let mode = addressing_mode;

        let value;
        let addr = self.get_ld_adddr(mode);
        if mode == AdressingMode::Accumulator {
            value = self.processor.acc;
            self.processor.pc = self.processor.pc.wrapping_add(1);
            self.processor.clock  = self.processor.clock.wrapping_add(2);
        } else if mode == AdressingMode::Absolute || mode == AdressingMode::AbsoluteX {
            value = self.processor.acc;
            self.processor.pc = self.processor.pc.wrapping_add(3);
            self.processor.clock  = self.processor.clock.wrapping_add(6);
        } else {
            self.processor.pc = self.processor.pc.wrapping_add(2);
            self.processor.clock  = self.processor.clock.wrapping_add(6);
            value = self.read(addr);
        }
        
        let old_flags = self.processor.flags;
        let result = (value >> 1) | ((self.processor.flags & FLAG_C) << 7);
        if value & 1 == 1 {
            self.processor.flags |= FLAG_C;
        } else {
            self.processor.flags &= !FLAG_C;
        }
        if result == 0 {
            self.processor.flags |= FLAG_Z;
        } else {
            self.processor.flags &= !FLAG_Z;
        }
        if result >> 7 & 1 == 1 {
            self.processor.flags |= FLAG_N;
        } else {
            self.processor.flags &= !FLAG_N;
        }
        if self.log_level > 0 {
            self.add_info(format!("{:#x} - Running instruction ror val: {:#x} result: {:#x} flags: {:#x} old flags: {:#x}", self.processor.pc, value, result, self.processor.flags, old_flags));
        }
        if mode == AdressingMode::Accumulator {
            self.processor.acc = result;
        } else {
            self.write(addr, result);
        }
    }

    fn bit(&mut self) {
        let addressing_mode = decode::get_adressing_mode(self.processor.inst);
        let mode = addressing_mode;

        let addr = self.get_ld_adddr(mode);
        let value = self.read(addr);

        let result = self.processor.acc & value;

        if self.log_level > 0 {
            self.add_info(format!("{:#x} - Running instruction bit val: {:#x} result: {:#x}", self.processor.pc, value, result));
        }
        if addressing_mode == AdressingMode::ZeroPage || addressing_mode == AdressingMode::Immediate || addressing_mode == AdressingMode::ZeroPageX {
            self.processor.pc = self.processor.pc.wrapping_add(2);
            self.processor.clock  = self.processor.clock.wrapping_add(3);
        } else if addressing_mode == AdressingMode::Absolute || addressing_mode == AdressingMode::AbsoluteX{
            self.processor.pc = self.processor.pc.wrapping_add(3);
            self.processor.clock  = self.processor.clock.wrapping_add(4);
        } else {
            panic!("Sorry, the adressing mode {:?} does not exist for instruction {:#x}", addressing_mode, self.processor.inst)
        }

        if result == 0 {
            self.processor.flags |= FLAG_Z;
        } else {
            self.processor.flags &= !FLAG_Z;
        }
        if value >> 7 & 1 == 1 {
            self.processor.flags |= FLAG_N;
        } else {
            self.processor.flags &= !FLAG_N;
        }
        if value >> 6 & 1 == 1 {
            self.processor.flags |= FLAG_O;
        } else {
            self.processor.flags &= !FLAG_O;
        }
    }

    fn inx(&mut self) {
        self.processor.rx = self.processor.rx.wrapping_add(1);
        self.processor.flags = Self::set_flags(self.processor.flags, self.processor.rx);
        if self.log_level > 0 {
            self.add_info(format!("{:#x} - Running instruction inx: new val: {:#x} flags: {:#x}", self.processor.pc, self.processor.rx, self.processor.flags));
        }
        self.processor.pc = self.processor.pc.wrapping_add(1);
        self.processor.clock  = self.processor.clock.wrapping_add(2);
    }

    fn iny(&mut self) {
        self.processor.ry = self.processor.ry.wrapping_add(1);
        self.processor.flags = Self::set_flags(self.processor.flags, self.processor.ry);
        if self.log_level > 0 {
            self.add_info(format!("{:#x} - Running instruction iny: new val: {:#x} flags: {:#x}", self.processor.pc, self.processor.ry, self.processor.flags));
        }
        self.processor.pc = self.processor.pc.wrapping_add(1);
        self.processor.clock  = self.processor.clock.wrapping_add(2);
    }

    fn dex(&mut self) {
        self.processor.rx = self.processor.rx.wrapping_sub(1);
        self.processor.flags = Self::set_flags(self.processor.flags, self.processor.rx);
        if self.log_level > 0 {
            self.add_info(format!("{:#x} - Running instruction dex: new val: {:#x} flags: {:#x}", self.processor.pc, self.processor.rx, self.processor.flags));
        }
        self.processor.pc = self.processor.pc.wrapping_add(1);
        self.processor.clock  = self.processor.clock.wrapping_add(2);
    }

    fn dey(&mut self) {
        self.processor.ry = self.processor.ry.wrapping_sub(1);
        self.processor.flags = Self::set_flags(self.processor.flags,  self.processor.ry);
        if self.log_level > 0 {
            self.add_info(format!("{:#x} - Running instruction dey: {:#x} new val: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize], self.processor.ry));
        }
        self.processor.pc = self.processor.pc.wrapping_add(1);
        self.processor.clock  = self.processor.clock.wrapping_add(2);
    }

    fn cmp(&mut self) {
        let addressing_mode = decode::get_adressing_mode(self.processor.inst);
        let acc = self.processor.acc;
        let mut pc = self.processor.pc + 2;
        let addr = self.get_ld_adddr(addressing_mode);
        let value = self.read(addr);
        if addressing_mode == AdressingMode::Absolute || addressing_mode == AdressingMode::AbsoluteY || addressing_mode == AdressingMode::AbsoluteX {
            pc += 1;
        }
        
        let mut flags = self.processor.flags;
        
        //If equal, all flags are zero
        // if a > cmp carry flag is set
        //if cmp > a neg flag is set
        
        if acc == value {
            flags |= FLAG_Z | FLAG_C;
            flags &= !FLAG_N;
        } else if acc > value {
            flags |= FLAG_C;
            flags &= !(FLAG_N | FLAG_Z);
        } else {
            flags |= FLAG_N;
            flags &= !(FLAG_C | FLAG_Z);
        }
        if self.log_level > 0 {
            self.add_info(format!("{:#x} - Running instruction cmp: {:#x} with acc: {:#x} val: {:#x} flags: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize], acc, value, flags));
        }

        self.processor.flags = flags;
        self.processor.pc = pc;
        // TODO fix clock counts
        self.processor.clock  = self.processor.clock.wrapping_add(4);
        
    }

    fn cpy(&mut self) {
        let addressing_mode = decode::get_adressing_mode(self.processor.inst);
        let ry = self.processor.ry;
        let value: u8;
        let mut pc = self.processor.pc.wrapping_add(2);
        let addr = self.get_ld_adddr(addressing_mode);
        if addressing_mode == AdressingMode::Immediate {
            value = self.read(addr);
        } else if addressing_mode == AdressingMode::Absolute {
            pc = pc.wrapping_add(1);
            value = self.read(addr);
        } else if addressing_mode == AdressingMode::ZeroPage {
            value = self.read(addr);
        } else {
            panic!("Unknown address type {:?} {:#b}, {:#x}", addressing_mode, self.processor.inst, self.processor.inst);
        }
        
        let mut flags = self.processor.flags;

        if ry == value {
            flags |= FLAG_Z | FLAG_C;
            flags &= !FLAG_N;
        } else if ry > value {
            flags |= FLAG_C;
            flags &= !(FLAG_N | FLAG_Z);
        } else {
            flags |= FLAG_N;
            flags &= !(FLAG_C | FLAG_Z);
        }
        if self.log_level > 0 {
            self.add_info(format!("{:#x} - Running instruction cpy ry: {:#x} with val: {:#x} flags: {:#x}", self.processor.pc, ry, value, flags));
        }

        self.processor.flags = flags;
        self.processor.pc = pc;
        // TODO fix clock counts
        self.processor.clock  = self.processor.clock.wrapping_add(4);
    }

    fn cpx(&mut self) {
        let addressing_mode = decode::get_adressing_mode(self.processor.inst);
        let rx = self.processor.rx;
        let value: u8;
        let mut pc = self.processor.pc.wrapping_add(2);
        let addr = self.get_ld_adddr(addressing_mode);
        if addressing_mode == AdressingMode::Immediate {
            value = self.read(addr);
        } else if addressing_mode == AdressingMode::Absolute {
            pc = pc.wrapping_add(1);
            value = self.read(addr);
        } else if addressing_mode == AdressingMode::ZeroPage {
            value = self.read(addr);
        } else {
            panic!("Unknown address type {:?} inst: {:#x}", addressing_mode, self.processor.inst);
        }
        
        let mut flags = self.processor.flags;

        if rx == value {
            flags |= FLAG_Z | FLAG_C;
            flags &= !FLAG_N;
        } else if rx > value {
            flags |= FLAG_C;
            flags &= !(FLAG_N | FLAG_Z);
        } else {
            flags |= FLAG_N;
            flags &= !(FLAG_C | FLAG_Z);
        }
        if self.log_level > 0 {
            self.add_info(format!("{:#x} - Running instruction cpx rx: {:#x} with val: {:#x} flags: {:#x}", self.processor.pc, rx, value, flags));
        }

        self.processor.flags = flags;
        self.processor.pc = pc;
        // TODO fix clock counts
        self.processor.clock  = self.processor.clock.wrapping_add(4);
    }

    fn sta(&mut self) {
        let addressing_mode = decode::get_adressing_mode(self.processor.inst);

        let mut pc = self.processor.pc;
        let addr = self.get_ld_adddr(addressing_mode);
    // // println!("sta addr 0x{:x?}", addr);
        if addressing_mode == AdressingMode::Absolute || addressing_mode == AdressingMode::AbsoluteX || addressing_mode == AdressingMode::AbsoluteY {
            if self.log_level > 0 {
                self.add_info(format!("{:#x} - Running instruction sta ABS at: {:#x} val: {:#x}", self.processor.pc, addr, self.processor.acc));
            }

            pc += 3;
        } else if addressing_mode == AdressingMode::ZeroPage || addressing_mode == AdressingMode::ZeroPageX || addressing_mode == AdressingMode::ZeroPageY || addressing_mode == AdressingMode::ZeroPageIndirect {
            if self.log_level > 0 {
                self.add_info(format!("{:#x} - Running instruction sta ZP at: {:#x} val: {:#x}", self.processor.pc, addr, self.processor.acc));
            }

            pc += 2;
        } else if addressing_mode == AdressingMode::IndirectY || addressing_mode == AdressingMode::IndirectX {
            if self.log_level > 0 {
                self.add_info(format!("{:#x} - Running instruction sta Indirect at: {:#x} val: {:#x}", self.processor.pc, addr, self.processor.acc));
            }

            pc += 2;
        } else {
            panic!("Adressing mode {:?} not implemented for STA", addressing_mode);
        }
        self.write(addr, self.processor.acc);

        self.processor.pc = pc;
        // TODO fix clock counts
        self.processor.clock  = self.processor.clock.wrapping_add(5);
    }

    fn stz(&mut self) {
        let addressing_mode = decode::get_adressing_mode(self.processor.inst);

        let mut pc = self.processor.pc;

        if addressing_mode == AdressingMode::ZeroPageX || addressing_mode == AdressingMode::ZeroPage {
            pc += 2;
        } else if addressing_mode == AdressingMode::Absolute || addressing_mode == AdressingMode::AbsoluteX {
            pc += 3;
        }

        let addr = self.get_ld_adddr(addressing_mode);

        self.write(addr, 0);

        self.processor.pc = pc;
        // TODO fix clock counts
        self.processor.clock  = self.processor.clock.wrapping_add(4);
    }

    fn stx(&mut self) {
        let addressing_mode = decode::get_adressing_mode(self.processor.inst);
        let mut pc = 2;
        let addr = self.get_ld_adddr(addressing_mode);
    // // println!("sta addr 0x{:x?}", addr);
        if addressing_mode == AdressingMode::Absolute {
            if self.log_level > 0 {
                self.add_info(format!("{:#x} - Running instruction stx ABS at: {:#x} val: {:#x}", self.processor.pc, addr, self.processor.rx));
            }
            pc = 3;
        } else if addressing_mode == AdressingMode::ZeroPage || addressing_mode == AdressingMode::ZeroPageY {
            if self.log_level > 0 {
                self.add_info(format!("{:#x} - Running instruction stx ZP at: {:#x} val: {:#x}", self.processor.pc, addr, self.processor.rx));
            }
        }
        if addr == 0x200 {
            //self.paused = true;
        }

        self.write(addr, self.processor.rx);

        self.processor.pc += pc;
        // TODO fix clock counts
        self.processor.clock  = self.processor.clock.wrapping_add(4);
    }

    fn sty(&mut self) {
        let addressing_mode = decode::get_adressing_mode(self.processor.inst);

        let mut pc = 2;
        let addr = self.get_ld_adddr(addressing_mode);
    // // println!("sta addr 0x{:x?}", addr);
        if addressing_mode == AdressingMode::Absolute {
            if self.log_level > 0 {
                self.add_info(format!("{:#x} - Running instruction sty ABS at: {:#x} val: {:#x}", self.processor.pc, addr, self.processor.rx));
            }
            pc = 3;
        } else if addressing_mode == AdressingMode::ZeroPage || addressing_mode == AdressingMode::ZeroPageX {
            if self.log_level > 0 {
                self.add_info(format!("{:#x} - Running instruction sty ZP at: {:#x} val: {:#x}", self.processor.pc, addr, self.processor.rx));
            }
        }
        if addr == 0x200 {
            //self.paused = true;
        }
        self.write(addr, self.processor.ry);

        self.processor.pc += pc;
        // TODO fix clock counts
        self.processor.clock  = self.processor.clock.wrapping_add(4);
    }

    fn jmp(&mut self) {
        let addressing_mode = decode::get_adressing_mode(self.processor.inst);
        let value: u16;
        if addressing_mode == AdressingMode::Absolute {
            value = self.get_word(self.processor.pc + 1);
            self.processor.clock  = self.processor.clock.wrapping_add(5);
        } else if addressing_mode == AdressingMode::Indirect {
            let start = self.processor.pc + 1;
    
            let addr = self.get_word(start);
            value = self.get_word(addr);

            self.processor.clock  = self.processor.clock.wrapping_add(3);
        } else if addressing_mode == AdressingMode::IndirectX {
            let start = self.processor.pc + 1;
            let addr = self.get_word(start).wrapping_add(self.processor.rx as u16);
            value = self.get_word(addr);
            self.processor.clock  = self.processor.clock.wrapping_add(6);
        } else {
            panic!("Adressing mode not implemented yet {:?} inst: {:#x}", addressing_mode, self.processor.inst);
        }
        self.processor.clock += 5;
        if self.log_level > 0 {
            self.add_info(format!("{:#x} - Running instruction jmp: {:#x} to: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize], value));
        }
        //// println!("Jumping to 0x{:x?}", addr);
        self.processor.pc = value;
    }

    fn bne(&mut self) {
        let offset = self.read(self.processor.pc + 1);

        let should_jump = (self.processor.flags >> 1) & 1 == 0;
        let mut new_addr :u16;
        new_addr = self.processor.pc.wrapping_add(2);
        
        if should_jump {
            let rel_address = offset as i8;
            // // println!("Jumping offset {:?}", rel_address);
            new_addr = ((new_addr as i32) + (rel_address as i32)) as u16;
            if self.log_level > 0 {
                self.add_info(format!("{:#x} - Running instruction bne {:#x} jumping to: {:#x} flags: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize], new_addr, self.processor.flags));
            }
        } else {
            if self.log_level > 0 {
                self.add_info(format!("{:#x} - Running instruction bne NOT jumping to: {:#x} flags: {:#x}", self.processor.pc, new_addr, self.processor.flags));
            }
        }

        self.processor.clock  = self.processor.clock.wrapping_add(3);
        self.processor.pc = new_addr;

        

    }

    /// Branch if not equal
    fn beq(&mut self) {
        let offset = self.read(self.processor.pc + 1);
        // // println!("Jumping RAW offset is {:?} or 0x{:x?}", offset, offset);
        let should_jump = self.processor.flags & FLAG_Z != 0;
        let mut new_addr :u16 = self.processor.pc.wrapping_add(2);
        

        if should_jump {
            let rel_address = offset as i8;
            // // println!("Jumping offset {:?}", rel_address);
            new_addr = ((new_addr as i32) + (rel_address as i32)) as u16;
            if self.log_level > 0 {
                self.add_info(format!("{:#x} - Running instruction beq {:#x} jumping to: {:#x} flags: {:#x} offset {}", self.processor.pc, self.data[(self.processor.pc) as usize], new_addr, self.processor.flags, offset as i8));
            }
        } else {
            if self.log_level > 0 {
                self.add_info(format!("{:#x} - Running instruction beq not jumping to: {:#x} flags: {:#x}", self.processor.pc, new_addr, self.processor.flags));
            }
        }
        self.processor.clock  = self.processor.clock.wrapping_add(3);
        self.processor.pc = new_addr;
        
    }

    /// Branch if carry clear
    fn bcc(&mut self) {
        let offset = self.read(self.processor.pc + 1);
        // // println!("Jumping RAW offset is {:?} or 0x{:x?}", offset, offset);
        let should_jump = self.processor.flags & FLAG_C == 0;
        let mut new_addr = self.processor.pc + 2;
        
        if should_jump {
            let rel_address = offset as i8;
            // // println!("Jumping offset {:?}", rel_address);
            new_addr = ((new_addr as i32) + (rel_address as i32)) as u16;
            if self.log_level > 0 {
                self.add_info(format!("{:#x} - Running instruction bcc jumping to: {:#x} flags: {:#x} offset: {}", self.processor.pc, new_addr, self.processor.flags, offset as i8));
            }
        } else {
            if self.log_level > 0 {
                self.add_info(format!("{:#x} - Running instruction bcc NOT jumping to: {:#x} flags: {:#x} offset: {}", self.processor.pc, new_addr, self.processor.flags, offset as i8));
            }
        }
        self.processor.clock  = self.processor.clock.wrapping_add(3);
        self.processor.pc = new_addr;
    }

    /// Branch if carry set
    fn bcs(&mut self) {
        let offset = self.read(self.processor.pc + 1);
        // // println!("Jumping RAW offset is {:?} or 0x{:x?}", offset, offset);
        let should_jump = (self.processor.flags) & FLAG_C == 1;
        let mut new_addr :u16 = self.processor.pc + 2;

        if should_jump {
            let rel_address = offset as i8;
            // // println!("Jumping offset {:?}", rel_address);
            new_addr = ((new_addr as i32) + (rel_address as i32)) as u16;
            if self.log_level > 0 {
                self.add_info(format!("{:#x} - Running instruction bcs {:#x} jumping to: {:#x} flags: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize], new_addr, self.processor.flags));
            } else {
                if self.log_level > 0 {
                    self.add_info(format!("{:#x} - Running instruction bcs not jumping to: {:#x} flags: {:#x}", self.processor.pc, new_addr, self.processor.flags));
                }
            }
        }
        self.processor.clock  = self.processor.clock.wrapping_add(3);
        self.processor.pc = new_addr;
        
    }

    /// Branch if overflow clear
    fn bvc(&mut self) {
        let offset = self.read(self.processor.pc + 1);
        // // println!("Jumping RAW offset is {:?} or 0x{:x?}", offset, offset);
        let should_jump = self.processor.flags & FLAG_O == 0;
        let mut new_addr = self.processor.pc.wrapping_add(2);
        
        if should_jump {
            let rel_address = offset as i8;
            // // println!("Jumping offset {:?}", rel_address);
            new_addr = ((new_addr as i32) + (rel_address as i32)) as u16;
            if self.log_level > 0 {
                self.add_info(format!("{:#x} - Running instruction bvc {:#x} jumping to: {:#x} flags: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize], new_addr, self.processor.flags));
            }
        } else {
            if self.log_level > 0 {
                self.add_info(format!("{:#x} - Running instruction bvc {:#x} NOT jumping to: {:#x} flags: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize], new_addr, self.processor.flags));
            }
        }
        
        self.processor.clock  = self.processor.clock.wrapping_add(3);
        self.processor.pc = new_addr;
    }

    /// Branch if overflow set
    fn bvs(&mut self) {
        let offset = self.read(self.processor.pc + 1);
        // // println!("Jumping RAW offset is {:?} or 0x{:x?}", offset, offset);
        let should_jump = self.processor.flags & FLAG_O != 0;
        let mut new_addr = self.processor.pc.wrapping_add(2);
           
        if should_jump {
            let rel_address = offset as i8;
            // // println!("Jumping offset {:?}", rel_address);
            new_addr = ((new_addr as i32) + (rel_address as i32)) as u16;
            if self.log_level > 0 {
                self.add_info(format!("{:#x} - Running instruction bvs {:#x} jumping to: {:#x} flags: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize], new_addr, self.processor.flags));
            }  
        } else {
            if self.log_level > 0 {
                self.add_info(format!("{:#x} - Running instruction bvs {:#x} NOT jumping to: {:#x} flags: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize], new_addr, self.processor.flags));
            }
        }
        self.processor.clock  = self.processor.clock.wrapping_add(3);
        self.processor.pc = new_addr;
    }

    fn bpl(&mut self) {
        let offset = self.read(self.processor.pc + 1);
        // println!("Jumping RAW offset is {:?} or 0x{:x?}", offset, offset);
        let should_jump = (self.processor.flags >> 7) & 1 == 0;
        let mut new_addr :u16;
        new_addr = self.processor.pc + 2;
        if should_jump {
            let rel_address = offset as i8;
            // println!("BPL Jumping offset {:?}", rel_address);
            new_addr = ((new_addr as i32) + (rel_address as i32)) as u16;
        }
        self.processor.pc = new_addr;
        self.processor.clock  = self.processor.clock.wrapping_add(3);
        
    }
    
    fn bra(&mut self) {
        let offset = self.read(self.processor.pc + 1);

        let mut new_addr :u16;
        new_addr = self.processor.pc + 2;
        let rel_address = offset as i8;
        // println!("BPL Jumping offset {:?}", rel_address);
        new_addr = ((new_addr as i32) + (rel_address as i32)) as u16;
        self.processor.pc = new_addr;
        self.processor.clock  = self.processor.clock.wrapping_add(3);
        
    }

    fn bbs(&mut self, num: u8) {
        let offset = self.read(self.processor.pc + 2);
        let zpa = self.read(self.processor.pc + 1);
        let should_jump = (self.read(zpa as u16) >> num) & 1 == 1;
        let mut new_addr = self.processor.pc + 3;
        if should_jump {
            let rel_address = offset as i8;
            // println!("BPL Jumping offset {:?}", rel_address);
            new_addr = ((new_addr as i32) + (rel_address as i32)) as u16;
        }
        self.processor.pc = new_addr;
        self.processor.clock  = self.processor.clock.wrapping_add(4);
    }

    fn bbr(&mut self, num: u8) {
        let offset = self.read(self.processor.pc + 2);
        let zpa = self.read(self.processor.pc + 1);
        let should_jump = (self.read(zpa as u16) >> num) & 1 == 0;
        let mut new_addr = self.processor.pc + 3;
        if should_jump {
            let rel_address = offset as i32;
            new_addr = ((new_addr as i32) + rel_address) as u16;
        }
        self.processor.pc = new_addr;
        self.processor.clock  = self.processor.clock.wrapping_add(4);
    }

    /// Branch if negative flag is set
    fn bmi(&mut self) {
        let offset = self.read(self.processor.pc + 1);
        // println!("Jumping RAW offset is {:?} or 0x{:x?}", offset, offset);
        let should_jump = (self.processor.flags >> 7) & 1 == 1;
        let mut new_addr :u16;
        new_addr = self.processor.pc + 2;
        if should_jump {
            let rel_address = offset as i8;
            // println!("BPL Jumping offset {:?}", rel_address);
            new_addr = ((new_addr as i32) + (rel_address as i32)) as u16;
        }
        self.processor.pc = new_addr;
        self.processor.clock  = self.processor.clock.wrapping_add(3);
        
    }

    fn get_logical_op_value(&mut self) -> u8 {
        let addressing_mode = decode::get_adressing_mode(self.processor.inst);
        let addr = self.get_ld_adddr(addressing_mode);
        return self.read(addr);
    }

    fn after_logical_op(&mut self) {
        let addressing_mode = decode::get_adressing_mode(self.processor.inst);
        if addressing_mode == AdressingMode::Immediate {
            self.processor.pc = self.processor.pc.wrapping_add(2);
            self.processor.clock  = self.processor.clock.wrapping_add(2);
        } else if addressing_mode == AdressingMode::ZeroPage || addressing_mode == AdressingMode::ZeroPageX {
            self.processor.pc = self.processor.pc.wrapping_add(2);
            self.processor.clock  = self.processor.clock.wrapping_add(3);
        } else if addressing_mode == AdressingMode::IndirectX || addressing_mode == AdressingMode::IndirectY {
            self.processor.pc = self.processor.pc.wrapping_add(2);
            self.processor.clock  = self.processor.clock.wrapping_add(6);
        } else if addressing_mode == AdressingMode::Absolute || addressing_mode == AdressingMode::AbsoluteX || addressing_mode == AdressingMode::AbsoluteY {
            self.processor.pc = self.processor.pc.wrapping_add(3);
            self.processor.clock  = self.processor.clock.wrapping_add(4);
        } else {
            self.add_info(format!("{:#x} - this addressing mode not implemented for instruction {:?}", self.processor.pc, addressing_mode));
        }
    }

    fn and(&mut self) {
        let value = self.get_logical_op_value();

        let result = self.processor.acc & value;
        self.processor.flags = Self::set_flags(self.processor.flags, result);
        if self.log_level > 0 {
            self.add_info(format!("{:#x} - Running instruction and with acc: {:#x} value: {:#x} result: {:#x} flags: {:#x}", self.processor.pc, self.processor.acc, value, result, self.processor.flags));
        }

        self.processor.acc = result;
        self.after_logical_op();
    }

    fn eor(&mut self) {
        let value = self.get_logical_op_value();

        let result = self.processor.acc ^ value;
        self.processor.flags = Self::set_flags(self.processor.flags, result);
        if self.log_level > 0 {
            self.add_info(format!("{:#x} - Running instruction eor {:#x} with acc: {:#x} value: {:#x} result: {:#x} flags: {:#x}", self.processor.pc, self.processor.inst, value, result, self.processor.acc, self.processor.flags));
        }

        self.processor.acc = result;
        self.after_logical_op();
    }

    fn ora(&mut self) {
        let value = self.get_logical_op_value();

        let result = self.processor.acc | value;
        self.processor.flags = Self::set_flags(self.processor.flags, result);
        if self.log_level > 0 {
            self.add_info(format!("{:#x} - Running instruction ora {:#x} with acc: {:#x} value: {:#x} result: {:#x} flags: {:#x}", self.processor.pc, self.processor.inst, value, result, self.processor.acc, self.processor.flags));
        }

        self.processor.acc = result;
        self.after_logical_op();
    }

    fn adc(&mut self) {
        
        let addressing_mode = decode::get_adressing_mode(self.processor.inst);
        let addr = self.get_ld_adddr(addressing_mode);
        let val = self.read(addr);
        let acc = self.processor.acc;
        let carry = self.processor.flags & FLAG_C != 0;
        let decimal = self.processor.flags & FLAG_D != 0;

        let sum;

        if decimal {
            let mut ln = (acc & 0xF) + (val &0xF) + (self.processor.flags & FLAG_C);
            if ln > 9 {
                ln = 0x10 | ((ln + 6) & 0xf);
            }
            let hn: u16 = (acc & 0xf0) as u16 + (val & 0xf0) as u16;
            let mut s = hn + ln as u16;

            if s >= 160 {
                self.processor.flags |= FLAG_C;
                if (self.processor.flags & FLAG_O) != 0 && s >= 0x180 { self.processor.flags &= !FLAG_O; }
                s += 0x60;
            } else {
                self.processor.flags &= !FLAG_C;
                if (self.processor.flags & FLAG_O) != 0 && s < 0x80 { self.processor.flags &= !FLAG_O; }
            }
            sum  = (s & 0xff) as u8;
            self.processor.flags = Self::set_flags(self.processor.flags, sum);
        } else {
            sum = self.do_add(val);
        }
        

        if self.log_level > 0 {
            self.add_info(format!("{:#x} - Running instruction adc with acc: {:#x} memval: {:#x} flags: {:#x} carry: {} result: {:#x}", self.processor.pc, self.processor.acc, val, self.processor.flags, carry, sum));
        }
        self.processor.acc = sum;
        self.after_logical_op();
    }

    fn sbc(&mut self) {
        let addressing_mode = decode::get_adressing_mode(self.processor.inst);
        let addr = self.get_ld_adddr(addressing_mode);
        let val = self.read(addr);
        let decimal = self.processor.flags & FLAG_D != 0;
        let acc= self.processor.acc;

        let sum;

        if decimal {
            let mut w: u16;
            let mut tmp = 0xf + (acc & 0xf) - (val & 0xf) + (self.processor.flags & FLAG_C);
            if tmp < 0x10 {
                w = 0;
              tmp -= 6;
            } else {
              w = 0x10;
              tmp -= 0x10;
            }
            w += 0xf0 + ((acc as u16) & 0xf0) - ((val as u16) & 0xf0);
            if w < 0x100 {
              self.processor.flags &= !FLAG_C;
              if (self.processor.flags & FLAG_O) != 0 && w < 0x80 { self.processor.flags &= !FLAG_O; }
              w -= 0x60;
            } else {
                self.processor.flags |= FLAG_C;
              if (self.processor.flags & FLAG_O) != 0  && w >= 0x180 { self.processor.flags &= !FLAG_O; }
            }
            w += tmp as u16;
            sum = w as u8
        } else {
            sum = self.do_add(!val);
        }

        if self.log_level > 0 {
            self.add_info(format!("{:#x} - Running instruction sbc with acc: {:#x} memval: {:#x} flags: {:#x}", self.processor.pc, self.processor.acc, val, self.processor.flags));
        }
        self.processor.acc = sum as u8;
        self.after_logical_op();
    }

    fn do_add(&mut self, val: u8) -> u8 {
        let acc = self.processor.acc;
        let mut carry = false;
        let s = acc as u16 + val as u16 + (self.processor.flags & FLAG_C) as u16;
        
        if s > 255 {
            carry = true;
        }
    
        let mut sum = acc.wrapping_add(val);
        //Add carry bit
        sum = sum.wrapping_add(self.processor.flags & FLAG_C);
        self.processor.flags = Self::set_flags(self.processor.flags, sum);

        if carry {
            self.processor.flags |= FLAG_C;
        } else {
            self.processor.flags &= !FLAG_C;
        }

        if (acc ^ sum) & (val ^ sum) & 0x80 != 0 {
            self.processor.flags |= FLAG_O;
        } else {
            self.processor.flags &= !FLAG_O;
        }


        

        return sum;
    }

    fn nop(&mut self) {
        if self.log_level > 0 {
            self.add_info(format!("{:#x} - Running instruction nop: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize]));
        }
        if self.processor.inst != 0xea && self.log_level > 1 {
            self.speed = 10;
        }
        
        self.processor.pc = self.processor.pc.wrapping_add(1);
        self.processor.clock  = self.processor.clock.wrapping_add(2);
        
    }

    pub fn set_flags(flags:u8, val:u8) -> u8 {
        let mut _flags = flags;
        if val == 0 {
            //Set zero flag
            _flags |= FLAG_Z & !FLAG_N;
        } else {
            _flags &= !FLAG_Z;
        }
        if val >> 7 == 1 {
            _flags |= FLAG_N;
        }else {
            _flags &= !FLAG_N;
        }

        _flags |= 0x30;

        // // println!("Setting flags to {:#b}", _flags);
        return _flags;
    }

    pub fn get_word(&mut self, address: u16) -> u16 {
        let low_byte: u16 = self.read(address).into();
        let high_byte: u16 = self.read(address.wrapping_add(1)).into();
        return low_byte + (high_byte << 8);
    }
}