use crate::computer::ADRESSING_MODE;
use crate::computer::decode;

pub fn get_adressing_mode(opcode: u8) -> ADRESSING_MODE {
    let bbb = (opcode >> 2) & 7;
    let cc = opcode & 3;
    
    if opcode == 0x6C {
        return ADRESSING_MODE::INDIRECT;
    }
    if opcode == 0x4C {
        return ADRESSING_MODE::ABSOLUTE;
    }
    
    if opcode == 0x7C {
        return ADRESSING_MODE::INDIRECT_X;
    }
    
    if opcode == 0x89 {
        return ADRESSING_MODE::IMMEDIATE;
    }
    
    if opcode == 0x64 {
        return ADRESSING_MODE::ZERO_PAGE;
    }
    
    if opcode == 0x9C {
        return ADRESSING_MODE::ABSOLUTE;
    }
    
    if opcode == 0x74 {
        return ADRESSING_MODE::ZERO_PAGE_X;
    }
    
    if opcode == 0x9E {
        return ADRESSING_MODE::ABSOLUTE_X;
    }

    match cc {
        0 => {
            match bbb {
                0b000	=> return ADRESSING_MODE::IMMEDIATE,
                0b001	=> return ADRESSING_MODE::ZERO_PAGE,
                0b011	=> return ADRESSING_MODE::ABSOLUTE,
                0b101	=> return ADRESSING_MODE::ZERO_PAGE_X,
                0b111	=> return ADRESSING_MODE::ABSOLUTE_X,
                _ => {}
            };
        },
        1 => {
            match bbb {
                0b000	=> return ADRESSING_MODE::INDIRECT_X,
                0b001	=> return ADRESSING_MODE::ZERO_PAGE,
                0b010	=> return ADRESSING_MODE::IMMEDIATE,
                0b011	=> return ADRESSING_MODE::ABSOLUTE,
                0b100	=> return ADRESSING_MODE::INDIRECT_Y,
                0b101	=> return ADRESSING_MODE::ZERO_PAGE_X,
                0b110	=> return ADRESSING_MODE::ABSOLUTE_Y,
                0b111	=> return ADRESSING_MODE::ABSOLUTE_X,
                _ => {}
            };
        },
        2 => {
            match bbb {
                0b000	=> return ADRESSING_MODE::IMMEDIATE,
                0b001	=> return ADRESSING_MODE::ZERO_PAGE,
                0b010	=> return ADRESSING_MODE::ACCUMULATOR,
                0b011	=> return ADRESSING_MODE::ABSOLUTE,
                0b101	=> if decode::get_opcode_name(opcode) == "STX" || decode::get_opcode_name(opcode) == "LDX" { return ADRESSING_MODE::ZERO_PAGE_Y } else { return ADRESSING_MODE::ZERO_PAGE_X },
                0b111	=> if decode::get_opcode_name(opcode) == "LDX" { return ADRESSING_MODE::ABSOLUTE_Y } else { return ADRESSING_MODE::ABSOLUTE_X },
                _ => {}
            }
        },
        _ => {}
    }

    

    ADRESSING_MODE::NONE
}

pub fn get_opcode_name<'a>(opcode: u8) -> &'a str {
    let cc = opcode & 3;
    let aaa = (opcode >> 5) & 7;

    match opcode {
        0x02 => return "NOP2",
        0x22 => return "NOP2",
        0x42 => return "NOP2",
        0x62 => return "NOP2",
        0x82 => return "NOP2",
        0xC2 => return "NOP2",
        0xE2 => return "NOP2",
        0x44 => return "NOP2",
        0x54 => return "NOP2",
        0xD4 => return "NOP2",
        0xF4 => return "NOP2",
        0x5C => return "NOP3",
        0xDC => return "NOP3",
        0xFC => return "NOP3",
        0x10 => return "BPL",
        0x30 => return "BMI",
        0x50 => return "BVC",
        0x70 => return "BVS",
        0x90 => return "BCC",
        0xB0 => return "BCS",
        0xD0 => return "BNE",
        0xF0 => return "BEQ",
        // 65C02 instruction
        0x7C => return "JMP",
        0x5A => return "PHY",
        0x7A => return "PLY",
        0xDA => return "PHX",
        0xFA => return "PLX",
        0x80 => return "BRA",

        0 => return "BRK",
        0x20 => return "JSR",
        0x40 => return "RTI",
        0x60 => return "RTS",

        0x08 => return "PHP",
        0x28 => return "PLP",
        0x48 => return "PHA",
        0x68 => return "PLA",
        0x88 => return "DEY",
        0xa8 => return "TAY",
        0xc8 => return "INY",
        0xe8 => return "INX",

        0x18 => return "CLC",
        0x38 => return "SEC",
        0x58 => return "CLI",
        0x78 => return "SEI",
        0x98 => return "TYA",
        0xB8 => return "CLV",
        0xD8 => return "CLD",
        0xF8 => return "SED",

        0x8a => return "TXA",
        0x9a => return "TXS",
        0xaa => return "TAX",
        0xba => return "TSX",
        0xca => return "DEX",
        0xea => return "NOP",

        0x0F => return "BBR0",
        0x1F => return "BBR1",
        0x2F => return "BBR2",
        0x3F => return "BBR3",
        0x4F => return "BBR4",
        0x5F => return "BBR5",
        0x6F => return "BBR6",
        0x7F => return "BBR7",

        0x8F => return "BBS0",
        0x9F => return "BBS1",
        0xAF => return "BBS2",
        0xBF => return "BBS3",
        0xCF => return "BBS4",
        0xDF => return "BBS5",
        0xEF => return "BBS6",
        0xFF => return "BBS7",

        0x64 => return "STZ",
        0x9C => return "STZ",
        0x74 => return "STZ",
        0x9E => return "STZ",

        0x89 => return "BIT",

        _ => {}
    }

    match cc {
        0 => {
            match aaa {
                0b001	=> return "BIT",
                0b010	=> return "JMP",
                0b011	=> return "JMP",
                0b100	=> return "STY",
                0b101	=> return "LDY",
                0b110	=> return "CPY",
                0b111	=> return "CPX",
                _ => {}
            };
            
        },
        1 => {
            match aaa {
                0b000	=> return "ORA",
                0b001	=> return "AND",
                0b010	=> return "EOR",
                0b011	=> return "ADC",
                0b100	=> return "STA",
                0b101	=> return "LDA",
                0b110	=> return "CMP",
                0b111	=> return "SBC",
                _ => {}
            };
        },
        2 => {
            match aaa {
                0b000	=> return "ASL",
                0b001	=> return "ROL",
                0b010	=> return "LSR",
                0b011	=> return "ROR",
                0b100	=> return "STX",
                0b101	=> return "LDX",
                0b110	=> return "DEC",
                0b111	=> return "INC",
                _ => {}
            };
        },
        _ => {}
    }

    

    ""
}
