use super::CPU;
use super::Bus;
use super::instr_utils::*;
use super::cpu_utils::*;

use std::io::Write;

impl CPU {
    pub fn decode(&mut self, bus: &mut Bus,op: u8) {
        match op {
            0x88..=0x8B => {
                self.instr.opcode = Opcode::MOV;
                self.instr.direction = Direction::new(op);
                self.instr.data_length = Length::new(op, 0);
                
                let operand = self.fetch(bus);
                decode_mod_reg_rm(self, bus, operand);

                self.instr.cycles += match (self.instr.operand1, self.instr.operand2) {
                    (OperandType::Register(_), OperandType::Register(_)) => 2,
                    (OperandType::Memory(_), OperandType::Register(_)) => 13 + self.instr.ea_cycles,
                    (OperandType::Register(_), OperandType::Memory(_)) => 12 + self.instr.ea_cycles,
                    _ => unreachable!(),
                }
            },
            0xC6 | 0xC7 => {
                self.instr.opcode = Opcode::MOV;
                self.instr.data_length = Length::new(op, 0);
                self.instr.operand2 = OperandType::Immediate;

                let operand = self.fetch(bus);
                decode_mod_n_rm(self, bus, operand);
                read_imm(self, bus);

                self.instr.cycles += match (self.instr.operand1, self.instr.operand2) {
                    (OperandType::Memory(_), OperandType::Immediate) => 14 + self.instr.ea_cycles,
                    (OperandType::Register(_), OperandType::Immediate) => 4,
                    _ => unreachable!(),
                }
            },
            0xB0..=0xBF => {
                self.instr.opcode = Opcode::MOV;
                self.instr.data_length = Length::new(op, 3);
                self.instr.operand1 = decode_reg(op, 0, self.instr.data_length);
                self.instr.operand2 = OperandType::Immediate;
                read_imm(self, bus);
                self.instr.cycles += 4;
            },
            0xA0..=0xA3 => {
                self.instr.opcode = Opcode::MOV;
                self.instr.direction = if 0x02 & op == 0 {Direction::ToReg} else {Direction::FromReg};
                self.instr.data_length = Length::new(op, 0);
                if self.instr.direction == Direction::ToReg {
                    if self.instr.data_length == Length::Byte {
                        self.instr.operand1 = OperandType::Register(Operand::AL);
                    } else {
                        self.instr.operand1 = OperandType::Register(Operand::AX);
                    }
                    self.instr.operand2 = OperandType::Memory(Operand::Disp);
                } else {
                    self.instr.operand1 = OperandType::Memory(Operand::Disp);
                    if self.instr.data_length == Length::Byte {
                        self.instr.operand2 = OperandType::Register(Operand::AL);
                    } else {
                        self.instr.operand2 = OperandType::Register(Operand::AX);
                    }
                }
                read_imm_addres(self, bus);
                self.instr.cycles += 14;
            },
            0x8E | 0x8C => {
                self.instr.opcode = Opcode::MOV;
                self.instr.direction = Direction::new(op);
                self.instr.data_length = Length::Word;

                let operand = self.fetch(bus);
                self.instr.addr_mode = decode_mod(operand);

                match self.instr.direction {
                    Direction::ToReg => {
                        self.instr.operand1 = decode_segment(operand, 3);
                        self.instr.operand2 = decode_rm(self, bus, operand, 0);
                    },
                    Direction::FromReg => {
                        self.instr.operand1 = decode_rm(self, bus, operand, 0);
                        self.instr.operand2 = decode_segment(operand, 3);
                    },
                    _ => unreachable!(),
                }

                self.instr.cycles += match (self.instr.operand1, self.instr.operand2) {
                    (OperandType::SegmentRegister(_), OperandType::Register(_)) => 2,
                    (OperandType::Register(_), OperandType::Memory(_)) => 12 + self.instr.ea_cycles,
                    (OperandType::Register(_), OperandType::SegmentRegister(_)) => 2,
                    (OperandType::Memory(_), OperandType::Register(_)) => 13 + self.instr.ea_cycles,
                    (OperandType::Memory(_), OperandType::SegmentRegister(_)) => 13 + self.instr.ea_cycles,
                    (OperandType::SegmentRegister(_), OperandType::Memory(_)) => 12 + self.instr.ea_cycles,
                    _ => unreachable!(),
                }
            },
            // Prefijo _MISC r/m16
            0xFF => {
                let operand = self.fetch(bus);

                match operand & 0b00111000 {
                    0x0 => {
                        self.instr.opcode = Opcode::INC;
                        self.instr.data_length = Length::Word;
                        decode_mod_n_rm(self, bus, operand);

                        self.instr.cycles += match self.instr.operand1 {
                            OperandType::Register(_) => 3,
                            OperandType::Memory(_) => 23 + self.instr.ea_cycles,
                            _ => unreachable!(),
                        }
                    },
                    0x8 => {
                        self.instr.opcode = Opcode::DEC;
                        self.instr.data_length = Length::Word;
                        decode_mod_n_rm(self, bus, operand);

                        self.instr.cycles += match self.instr.operand1 {
                            OperandType::Register(_) => 3,
                            OperandType::Memory(_) => 23 + self.instr.ea_cycles,
                            _ => unreachable!(),
                        }
                    },
                    0x10 => {
                        self.instr.opcode = Opcode::CALL;
                        self.instr.data_length = Length::Word;
                        decode_mod_n_rm(self, bus, operand);

                        self.instr.jump_type = JumpType::IndWithinSegment;

                        self.instr.cycles += match self.instr.operand1 {
                            OperandType::Register(_) => 24,
                            OperandType::Memory(_) => 29 + self.instr.ea_cycles,
                            _ => unreachable!(),
                        }
                    },
                    0x18 => {
                        self.instr.opcode = Opcode::CALL;
                        self.instr.data_length = Length::Word;
                        decode_mod_n_rm(self, bus, operand);

                        self.instr.jump_type = JumpType::IndIntersegment;

                        self.instr.cycles += match self.instr.operand1 {
                            OperandType::Memory(_) => 53 + self.instr.ea_cycles,
                            _ => unreachable!(),
                        }
                    },
                    0x20 => {
                        self.instr.opcode = Opcode::JMP;
                        self.instr.data_length = Length::Word;
                        decode_mod_n_rm(self, bus, operand);

                        self.instr.jump_type = JumpType::IndWithinSegment;

                        self.instr.cycles += match self.instr.operand1 {
                            OperandType::Register(_) => 11,
                            OperandType::Memory(_) => 18 + self.instr.ea_cycles,
                            _ => unreachable!(),
                        }
                    },
                    0x28 => {
                        self.instr.opcode = Opcode::JMP;
                        self.instr.data_length = Length::Word;
                        decode_mod_n_rm(self, bus, operand);

                        self.instr.jump_type = JumpType::IndIntersegment;

                        self.instr.cycles += match self.instr.operand1 {
                            OperandType::Memory(_) => 24 + self.instr.ea_cycles,
                            _ => unreachable!(),
                        }
                    },
                    0x30 => {
                        self.instr.opcode = Opcode::PUSH;
                        self.instr.data_length = Length::Word;
                        decode_mod_n_rm(self, bus, operand);
                        
                        self.instr.cycles += match self.instr.operand1 {
                            OperandType::Register(_) => 15,
                            OperandType::Memory(_) => 24 + self.instr.ea_cycles,
                            _ => unreachable!(),
                        }
                    },
                    _ => unreachable!(),    
                }
            },
            0x50..=0x57 => {
                self.instr.opcode = Opcode::PUSH;
                self.instr.data_length = Length::Word;
                self.instr.operand1 = decode_reg(op, 0, self.instr.data_length);
                self.instr.cycles += 15;
            },
            0x06 | 0x16 | 0x0E | 0x1E => {
                self.instr.opcode = Opcode::PUSH;
                self.instr.data_length = Length::Word;
                self.instr.operand1 = decode_segment(op, 3);
                self.instr.cycles += 14;
            },
            0x8F => {
                self.instr.opcode = Opcode::POP;
                let operand = self.fetch(bus);
                self.instr.addr_mode = decode_mod(operand);
                self.instr.data_length = Length::Word;
                self.instr.operand1 = decode_rm(self, bus, operand, 0);

                self.instr.cycles += match self.instr.operand1 {
                    OperandType::Register(_) => 12,
                    OperandType::Memory(_) => 25 + self.instr.ea_cycles,
                    _ => unreachable!(),
                }
            },
            0x58..=0x5F => {
                self.instr.opcode = Opcode::POP;
                self.instr.data_length = Length::Word;
                self.instr.operand1 = decode_reg(op, 0, self.instr.data_length);
                self.instr.cycles += 12;
            },
            0x07 | 0x17 | 0x0F | 0x1F => {
                self.instr.opcode = Opcode::POP;
                self.instr.data_length = Length::Word;
                self.instr.operand1 = decode_segment(op, 3);
                self.instr.cycles += 12;
            },
            0x86 | 0x87 => {
                self.instr.opcode = Opcode::XCHG;
                self.instr.data_length = Length::new(op, 0);
                self.instr.direction = Direction::ToReg;
                let operand = self.fetch(bus);
                decode_mod_reg_rm(self, bus, operand);

                self.instr.cycles += match (self.instr.operand1, self.instr.operand2) {
                    (OperandType::Register(_), OperandType::Register(_)) => 4,
                    (OperandType::Register(_), OperandType::Memory(_)) => 25 + self.instr.ea_cycles,
                    (OperandType::Memory(_), OperandType::Register(_)) => 25 + self.instr.ea_cycles, // Creo que no hace falta, pero por si acaso
                    _ => unreachable!(),
                }
            },
            0x90..=0x97 => {
                self.instr.opcode = Opcode::XCHG;
                self.instr.data_length = Length::Word;
                self.instr.direction = Direction::ToReg;
                self.instr.operand1 = OperandType::Register(Operand::AX);
                self.instr.operand2 = decode_reg(op, 0, self.instr.data_length);
                self.instr.cycles += 3;
            },
            // TODO IN
            0xE4 | 0xE5 => {
                self.instr.opcode = Opcode::IN;
                self.instr.cycles += 1;
            },
            0xEC | 0xED => {
                self.instr.opcode = Opcode::IN;
                self.instr.cycles += 1;
            },
            // TODO OUT
            0xE6 | 0xE7 => {
                self.instr.opcode = Opcode::OUT;
                self.instr.cycles += 1;
            },
            0xEE | 0xEF => {
                self.instr.opcode = Opcode::OUT;
                self.instr.cycles += 1;
            },
            0xD7 => {
                self.instr.opcode = Opcode::XLAT;
                self.instr.operand1 = OperandType::Register(Operand::AL);

                self.instr.cycles += 11;
            },
            0x8D => {
                self.instr.opcode = Opcode::LEA;
                self.instr.direction = Direction::ToReg;
                self.instr.data_length = Length::Word;
                let operand = self.fetch(bus);
                decode_mod_reg_rm(self, bus, operand);
                self.instr.cycles += 2 + self.instr.ea_cycles;
            },
            0xC5 => {
                self.instr.opcode = Opcode::LDS;
                self.instr.direction = Direction::ToReg;
                self.instr.data_length = Length::Word;
                let operand = self.fetch(bus);
                decode_mod_reg_rm(self, bus, operand);
                self.instr.cycles += 24 + self.instr.ea_cycles;
            },
            0xC4 => {
                self.instr.opcode = Opcode::LES;
                self.instr.direction = Direction::ToReg;
                self.instr.data_length = Length::Word;
                let operand = self.fetch(bus);
                decode_mod_reg_rm(self, bus, operand);
                self.instr.cycles += 24 + self.instr.ea_cycles;
            },
            0x9F => {
                self.instr.opcode = Opcode::LAHF;
                self.instr.cycles += 4;
            },
            0x9E => {
                self.instr.opcode = Opcode::SAHF;
                self.instr.cycles += 4;
            },
            0x9C => {
                self.instr.opcode = Opcode::PUSHF;
                self.instr.cycles += 14;
            },
            0x9D => {
                self.instr.opcode = Opcode::POPF;
                self.instr.cycles += 12;
            },

            0x00..=0x03 => {
                self.instr.opcode = Opcode::ADD;
                self.instr.direction = Direction::new(op);
                self.instr.data_length = Length::new(op, 0);
                let operand = self.fetch(bus);
                decode_mod_reg_rm(self, bus, operand);

                self.instr.cycles += match (self.instr.operand1, self.instr.operand2) {
                    (OperandType::Register(_), OperandType::Register(_)) => 3,
                    (OperandType::Memory(_), OperandType::Register(_)) => 24 + self.instr.ea_cycles,
                    (OperandType::Register(_), OperandType::Memory(_)) => 13 + self.instr.ea_cycles,
                    _ => unreachable!(),
                };
            },
            // Prefijos _ALU1
            0x80..=0x83 => {
                let operand = self.fetch(bus);
                self.instr.data_length = Length::new(op, 0);
                self.instr.addr_mode = decode_mod(operand);
                self.instr.operand1 = decode_rm(self, bus, operand, 0);
                self.instr.operand2 = OperandType::Immediate;

                match op & 0b00000011 {
                    0b00..=0b10 => read_imm(self, bus),
                    0b11 => self.instr.imm = sign_extend(self.fetch(bus)),
                    _ => unreachable!(),
                }

                self.instr.cycles += match self.instr.operand1 {
                    OperandType::Register(_) => 4,
                    OperandType::Memory(_) => 23 + self.instr.ea_cycles,
                    _ => unreachable!(),
                };

                match operand & 0b00111000 {
                    0x00 => self.instr.opcode = Opcode::ADD,
                    0x08 => self.instr.opcode = Opcode::OR,
                    0x10 => self.instr.opcode = Opcode::ADC,
                    0x18 => self.instr.opcode = Opcode::SBB,
                    0x20 => self.instr.opcode = Opcode::AND,
                    0x28 => self.instr.opcode = Opcode::SUB,
                    0x30 => self.instr.opcode = Opcode::XOR,
                    0x38 => self.instr.opcode = Opcode::CMP,
                    _ => unreachable!(),
                }
            },
            0x04 | 0x05 => {
                self.instr.opcode = Opcode::ADD;
                self.instr.data_length = Length::new(op, 0);
                self.instr.operand1 = match self.instr.data_length {
                    Length::Byte => OperandType::Register(Operand::AL),
                    Length::Word => OperandType::Register(Operand::AX),
                    _ => unreachable!(),
                };
                self.instr.operand2 = OperandType::Immediate;
                read_imm(self, bus);

                self.instr.cycles += 4;
            },
            0x10..=0x13 => {
                self.instr.opcode = Opcode::ADC;
                self.instr.direction = Direction::new(op);
                self.instr.data_length = Length::new(op, 0);
                let operand = self.fetch(bus);
                decode_mod_reg_rm(self, bus, operand);

                self.instr.cycles += match (self.instr.operand1, self.instr.operand2) {
                    (OperandType::Register(_), OperandType::Register(_)) => 3,
                    (OperandType::Memory(_), OperandType::Register(_)) => 24 + self.instr.ea_cycles,
                    (OperandType::Register(_), OperandType::Memory(_)) => 13 + self.instr.ea_cycles,
                    _ => unreachable!(),
                };
            },
            0x14 | 0x15 => {
                self.instr.opcode = Opcode::ADC;
                self.instr.data_length = Length::new(op, 0);
                self.instr.operand1 = match self.instr.data_length {
                    Length::Byte => OperandType::Register(Operand::AL),
                    Length::Word => OperandType::Register(Operand::AX),
                    _ => unreachable!(),
                };
                self.instr.operand2 = OperandType::Immediate;
                read_imm(self, bus);

                self.instr.cycles += 4;
            },
            // Prefijo _MISC r/m8
            0xFE => {
                let operand = self.fetch(bus);

                self.instr.data_length = Length::Byte;
                decode_mod_n_rm(self, bus, operand);

                self.instr.cycles += match self.instr.operand1 {
                    OperandType::Register(_) => 3,
                    OperandType::Memory(_) => 23 + self.instr.ea_cycles,
                    _ => unreachable!(),
                };

                match operand & 0b00111000 {
                    0x00 => self.instr.opcode = Opcode::INC,
                    0x08 => self.instr.opcode = Opcode::DEC,
                    _ => unreachable!(),
                }
            },
            0x40..=0x47 => {
                self.instr.opcode = Opcode::INC;
                self.instr.data_length = Length::Word;
                self.instr.operand1 = decode_reg(op, 0, self.instr.data_length);
                self.instr.cycles += 3;
            },
            0x37 => {
                self.instr.opcode = Opcode::AAA;
                self.instr.cycles += 8;
            },
            0x27 => {
                self.instr.opcode = Opcode::DAA;
                self.instr.cycles += 4;
            },
            0x28..=0x2B => {
                self.instr.opcode = Opcode::SUB;
                self.instr.direction = Direction::new(op);
                self.instr.data_length = Length::new(op, 0);
                let operand = self.fetch(bus);
                decode_mod_reg_rm(self, bus, operand);

                self.instr.cycles += match (self.instr.operand1, self.instr.operand2) {
                    (OperandType::Register(_), OperandType::Register(_)) => 3,
                    (OperandType::Memory(_), OperandType::Register(_)) => 24 + self.instr.ea_cycles,
                    (OperandType::Register(_), OperandType::Memory(_)) => 13 + self.instr.ea_cycles,
                    _ => unreachable!(),
                };
            },
            0x2C | 0x2D => {
                self.instr.opcode = Opcode::SUB;
                self.instr.data_length = Length::new(op, 0);
                self.instr.operand1 = match self.instr.data_length {
                    Length::Byte => OperandType::Register(Operand::AL),
                    Length::Word => OperandType::Register(Operand::AX),
                    _ => unreachable!(),
                };
                self.instr.operand2 = OperandType::Immediate;
                read_imm(self, bus);

                self.instr.cycles += 4;
            },
            0x18..=0x1B => {
                self.instr.opcode = Opcode::SBB;
                self.instr.direction = Direction::new(op);
                self.instr.data_length = Length::new(op, 0);
                let operand = self.fetch(bus);
                decode_mod_reg_rm(self, bus, operand);

                self.instr.cycles += match (self.instr.operand1, self.instr.operand2) {
                    (OperandType::Register(_), OperandType::Register(_)) => 3,
                    (OperandType::Memory(_), OperandType::Register(_)) => 24 + self.instr.ea_cycles,
                    (OperandType::Register(_), OperandType::Memory(_)) => 13 + self.instr.ea_cycles,
                    _ => unreachable!(),
                };
            },
            0x1C | 0x1D => {
                self.instr.opcode = Opcode::SBB;
                self.instr.data_length = Length::new(op, 0);
                self.instr.operand1 = match self.instr.data_length {
                    Length::Byte => OperandType::Register(Operand::AL),
                    Length::Word => OperandType::Register(Operand::AX),
                    _ => unreachable!(),
                };
                self.instr.operand2 = OperandType::Immediate;
                read_imm(self, bus);

                self.instr.cycles += 4;
            },
            0x48..=0x4F => {
                self.instr.opcode = Opcode::DEC;
                self.instr.data_length = Length::Word;
                self.instr.operand1 = decode_reg(op, 0, self.instr.data_length);
                self.instr.cycles += 3;
            },
            // _ALU 2
            0xF6 | 0xF7 => {
                let operand = self.fetch(bus);
                self.instr.data_length = Length::new(op, 0);
                
                match operand & 0b00111000 {
                    0x00 | 0x08 => {
                        self.instr.opcode = Opcode::TEST;
                        decode_mod_n_rm(self, bus, operand);
                        self.instr.operand2 = OperandType::Immediate;
                        read_imm(self, bus);
        
                        self.instr.cycles += match self.instr.operand1 {
                            OperandType::Register(_) => 5,
                            OperandType::Memory(_) => 11 + self.instr.ea_cycles,
                            _ => unreachable!(),
                        };
                    }
                    0x10 => {
                        self.instr.opcode = Opcode::NOT;
                        decode_mod_n_rm(self, bus, operand);

                        self.instr.cycles += match self.instr.operand1 {
                            OperandType::Register(_) => 3,
                            OperandType::Memory(_) => 24 + self.instr.ea_cycles,
                            _ => unreachable!(),
                        };
                    },
                    0x18 => {
                        self.instr.opcode = Opcode::NEG;
                        decode_mod_n_rm(self, bus, operand);

                        self.instr.cycles += match self.instr.operand1 {
                            OperandType::Register(_) => 3,
                            OperandType::Memory(_) => 24 + self.instr.ea_cycles,
                            _ => unreachable!(),
                        };
                    },
                    0x20 => {
                        self.instr.opcode = Opcode::MUL;
                        decode_mod_n_rm(self, bus, operand);

                        // TODO no se si esto esta bien del todo
                        self.instr.cycles += match self.instr.operand1 {
                            OperandType::Register(_) => if self.instr.data_length == Length::Byte {77} else {133},
                            OperandType::Memory(_) => if self.instr.data_length == Length::Byte {83 + self.instr.ea_cycles} else {139 + self.instr.ea_cycles},
                            _ => unreachable!(),
                        };
                    },
                    0x28 => {
                        self.instr.opcode = Opcode::IMUL;
                        decode_mod_n_rm(self, bus, operand);

                        // TODO no se si esto esta bien del todo
                        self.instr.cycles += match self.instr.operand1 {
                            OperandType::Register(_) => if self.instr.data_length == Length::Byte {98} else {154},
                            OperandType::Memory(_) => if self.instr.data_length == Length::Byte {104 + self.instr.ea_cycles} else {160 + self.instr.ea_cycles},
                            _ => unreachable!(),
                        };
                    },
                    0x30 => {
                        self.instr.opcode = Opcode::DIV;
                        decode_mod_n_rm(self, bus, operand);

                        // TODO no se si esto esta bien del todo
                        self.instr.cycles += match self.instr.operand1 {
                            OperandType::Register(_) => if self.instr.data_length == Length::Byte {90} else {162},
                            OperandType::Memory(_) => if self.instr.data_length == Length::Byte {96 + self.instr.ea_cycles} else {168 + self.instr.ea_cycles},
                            _ => unreachable!(),
                        };
                    }
                    0x38 => {
                        self.instr.opcode = Opcode::IDIV;
                        decode_mod_n_rm(self, bus, operand);

                        // TODO no se si esto esta bien del todo
                        self.instr.cycles += match self.instr.operand1 {
                            OperandType::Register(_) => if self.instr.data_length == Length::Byte {112} else {184},
                            OperandType::Memory(_) => if self.instr.data_length == Length::Byte {118 + self.instr.ea_cycles} else {190 + self.instr.ea_cycles},
                            _ => unreachable!(),
                        };
                    }
                    _ => unreachable!(),
                }
            },
            0x38..=0x3B => {
                self.instr.opcode = Opcode::CMP;
                self.instr.direction = Direction::new(op);
                self.instr.data_length = Length::new(op, 0);
                let operand = self.fetch(bus);
                decode_mod_reg_rm(self, bus, operand);

                self.instr.cycles += match (self.instr.operand1, self.instr.operand2) {
                    (OperandType::Register(_), OperandType::Register(_)) => 3,
                    (OperandType::Memory(_), OperandType::Register(_)) => 13 + self.instr.ea_cycles,
                    (OperandType::Register(_), OperandType::Memory(_)) => 13 + self.instr.ea_cycles,
                    _ => unreachable!(),
                };
            },
            0x3C | 0x3D => {
                self.instr.opcode = Opcode::CMP;
                self.instr.data_length = Length::new(op, 0);
                self.instr.operand1 = match self.instr.data_length {
                    Length::Byte => OperandType::Register(Operand::AL),
                    Length::Word => OperandType::Register(Operand::AX),
                    _ => unreachable!(),
                };
                self.instr.operand2 = OperandType::Immediate;
                read_imm(self, bus);

                self.instr.cycles += 4;
            },
            0x3F => {
                self.instr.opcode = Opcode::AAS;
                self.instr.cycles += 8;
            },
            0x2F => {
                self.instr.opcode = Opcode::DAS;
                self.instr.cycles += 4;
            },
            0xD4 => {
                self.instr.opcode = Opcode::AAM;
                self.instr.operand1 = OperandType::Immediate;
                self.instr.data_length = Length::Byte;
                read_imm(self, bus);
                self.instr.cycles += 83;
            },
            0xD5 => {
                self.instr.opcode = Opcode::AAD;
                self.instr.operand1 = OperandType::Immediate;
                self.instr.data_length = Length::Byte;
                read_imm(self, bus);
                self.instr.cycles += 60;
            },
            0x98 => {
                self.instr.opcode = Opcode::CBW;
                self.instr.cycles += 2;
            },
            0x99 => {
                self.instr.opcode = Opcode::CWD;
                self.instr.cycles += 5;
            },

            // _ROT 1
            0xD0..=0xD3 => {
                let operand = self.fetch(bus);

                self.instr.data_length = Length::new(op, 0);
                decode_mod_n_rm(self, bus, operand);
                self.instr.operand2 = if operand & 0x02 == 1 {
                    OperandType::Register(Operand::CL)
                } else {
                    self.instr.imm = 1;
                    OperandType::Immediate
                };

                match operand & 0b00111000 {
                    0x00 => self.instr.opcode = Opcode::ROL,
                    0x08 => self.instr.opcode = Opcode::ROR,
                    0x10 => self.instr.opcode = Opcode::RCL,
                    0x18 => self.instr.opcode = Opcode::RCR,
                    0x20 => self.instr.opcode = Opcode::SALSHL,
                    0x28 => self.instr.opcode = Opcode::SHR,
                    0x30 => self.instr.opcode = Opcode::SALSHL,
                    0x38 => self.instr.opcode = Opcode::SAR,
                    _ => unreachable!(),
                }
                
                self.instr.cycles += match (self.instr.operand1, self.instr.operand2) {
                    (OperandType::Register(_), OperandType::Immediate) => 2,
                    (OperandType::Memory(_), OperandType::Immediate) => 23 + self.instr.ea_cycles,
                    (OperandType::Register(_), OperandType::Register(_)) => 8,
                    (OperandType::Memory(_), OperandType::Register(_)) => 28 + self.instr.ea_cycles + 4 * (self.cx.low as u64),
                    _ => unreachable!(),
                };

            },
            0x20..=0x23 => {
                self.instr.opcode = Opcode::AND;
                self.instr.direction = Direction::new(op);
                self.instr.data_length = Length::new(op, 0);
                let operand = self.fetch(bus);
                decode_mod_reg_rm(self, bus, operand);

                self.instr.cycles += match (self.instr.operand1, self.instr.operand2) {
                    (OperandType::Register(_), OperandType::Register(_)) => 3,
                    (OperandType::Memory(_), OperandType::Register(_)) => 24 + self.instr.ea_cycles,
                    (OperandType::Register(_), OperandType::Memory(_)) => 13 + self.instr.ea_cycles,
                    _ => unreachable!(),
                };
            },
            0x24 | 0x25 => {
                self.instr.opcode = Opcode::AND;
                self.instr.data_length = Length::new(op, 0);
                self.instr.operand1 = match self.instr.data_length {
                    Length::Byte => OperandType::Register(Operand::AL),
                    Length::Word => OperandType::Register(Operand::AX),
                    _ => unreachable!(),
                };
                self.instr.operand2 = OperandType::Immediate;
                read_imm(self, bus);

                self.instr.cycles += 4;
            },
            0x08..=0x0B => {
                self.instr.opcode = Opcode::OR;
                self.instr.direction = Direction::new(op);
                self.instr.data_length = Length::new(op, 0);
                let operand = self.fetch(bus);
                decode_mod_reg_rm(self, bus, operand);

                self.instr.cycles += match (self.instr.operand1, self.instr.operand2) {
                    (OperandType::Register(_), OperandType::Register(_)) => 3,
                    (OperandType::Memory(_), OperandType::Register(_)) => 24 + self.instr.ea_cycles,
                    (OperandType::Register(_), OperandType::Memory(_)) => 13 + self.instr.ea_cycles,
                    _ => unreachable!(),
                };
            },
            0x0C | 0x0D => {
                self.instr.opcode = Opcode::OR;
                self.instr.data_length = Length::new(op, 0);
                self.instr.operand1 = match self.instr.data_length {
                    Length::Byte => OperandType::Register(Operand::AL),
                    Length::Word => OperandType::Register(Operand::AX),
                    _ => unreachable!(),
                };
                self.instr.operand2 = OperandType::Immediate;
                read_imm(self, bus);

                self.instr.cycles += 4;
            },
            0x30..=0x33 => {
                self.instr.opcode = Opcode::XOR;
                self.instr.direction = Direction::new(op);
                self.instr.data_length = Length::new(op, 0);
                let operand = self.fetch(bus);
                decode_mod_reg_rm(self, bus, operand);

                self.instr.cycles += match (self.instr.operand1, self.instr.operand2) {
                    (OperandType::Register(_), OperandType::Register(_)) => 3,
                    (OperandType::Memory(_), OperandType::Register(_)) => 24 + self.instr.ea_cycles,
                    (OperandType::Register(_), OperandType::Memory(_)) => 13 + self.instr.ea_cycles,
                    _ => unreachable!(),
                };
            },
            0x34 | 0x35 => {
                self.instr.opcode = Opcode::XOR;
                self.instr.data_length = Length::new(op, 0);
                self.instr.operand1 = match self.instr.data_length {
                    Length::Byte => OperandType::Register(Operand::AL),
                    Length::Word => OperandType::Register(Operand::AX),
                    _ => unreachable!(),
                };
                self.instr.operand2 = OperandType::Immediate;
                read_imm(self, bus);

                self.instr.cycles += 4;
            },
            0x84 | 0x85 => {
                self.instr.opcode = Opcode::TEST;
                self.instr.data_length = Length::new(op, 0);
                self.instr.direction = Direction::FromReg;
                let operand = self.fetch(bus);
                decode_mod_reg_rm(self, bus, operand);

                self.instr.cycles += match (self.instr.operand1, self.instr.operand2) {
                    (OperandType::Register(_), OperandType::Register(_)) => 3,
                    (OperandType::Memory(_), OperandType::Register(_)) => 13 + self.instr.ea_cycles,
                    (OperandType::Register(_), OperandType::Memory(_)) => 13 + self.instr.ea_cycles,
                    _ => unreachable!(),
                };
            },
            0xA8 | 0xA9 => {
                self.instr.opcode = Opcode::AND;
                self.instr.data_length = Length::new(op, 0);
                self.instr.operand1 = match self.instr.data_length {
                    Length::Byte => OperandType::Register(Operand::AL),
                    Length::Word => OperandType::Register(Operand::AX),
                    _ => unreachable!(),
                };
                self.instr.operand2 = OperandType::Immediate;
                read_imm(self, bus);

                self.instr.cycles += 4;
            },

            0x26 | 0x2E | 0x36 | 0x3E => {
                self.instr.segment = if let OperandType::SegmentRegister(operand) = decode_segment(op, 4) {
                    operand
                } else {
                    panic!()
                };
                
                self.instr.cycles += 2;

                let new_op = self.fetch(bus);
                self.decode(bus, new_op);
            },

            0xF2 | 0xF3 => {
                self.instr.repetition_prefix = if op & 0x01 == 1 {
                    RepetitionPrefix::REPEZ
                } else {
                    RepetitionPrefix::REPNEZ
                };

                let new_op = self.fetch(bus);
                self.decode(bus, new_op);
            },

            0xEA => {
                self.instr.opcode = Opcode::JMP;
                let offset_low = self.fetch(bus);
                let offset_high = self.fetch(bus);
                let seg_low = self.fetch(bus);
                let seg_high = self.fetch(bus);
                let offset = to_u16(offset_low, offset_high);
                let seg = to_u16(seg_low, seg_high);
                self.instr.jump_type = JumpType::DirIntersegment(offset, seg);

                self.instr.cycles += 15;
            },
            0xA4 => {
                self.instr.opcode = Opcode::MOVSB;
                self.instr.data_length = Length::Byte;

                self.instr.cycles += if self.instr.repetition_prefix != RepetitionPrefix::None {18} else {9}
            },
            0xA5 => {
                self.instr.opcode = Opcode::MOVSW;
                self.instr.data_length = Length::Word;

                self.instr.cycles += if self.instr.repetition_prefix != RepetitionPrefix::None {26} else {9}
            },
            0xA6 => {
                self.instr.opcode = Opcode::CMPSB;
                self.instr.data_length = Length::Byte;

                self.instr.cycles += if self.instr.repetition_prefix != RepetitionPrefix::None {22} else {9}
            },
            0xA7 => {
                self.instr.opcode = Opcode::CMPSW;
                self.instr.data_length = Length::Word;

                self.instr.cycles += if self.instr.repetition_prefix != RepetitionPrefix::None {30} else {9}
            },
            0xAE => {
                self.instr.opcode = Opcode::SCASB;
                self.instr.data_length = Length::Byte;

                self.instr.cycles += if self.instr.repetition_prefix != RepetitionPrefix::None {15} else {9}
            },
            0xAF => {
                self.instr.opcode = Opcode::SCASW;
                self.instr.data_length = Length::Word;

                self.instr.cycles += if self.instr.repetition_prefix != RepetitionPrefix::None {19} else {9}
            },
            0xAC => {
                self.instr.opcode = Opcode::LODSB;
                self.instr.data_length = Length::Byte;

                self.instr.cycles += if self.instr.repetition_prefix != RepetitionPrefix::None {12} else {9}
            },
            0xAD => {
                self.instr.opcode = Opcode::LODSW;
                self.instr.data_length = Length::Word;

                self.instr.cycles += if self.instr.repetition_prefix != RepetitionPrefix::None {16} else {9}
            },
            0xAA => {
                self.instr.opcode = Opcode::STOSB;
                self.instr.data_length = Length::Byte;

                self.instr.cycles += if self.instr.repetition_prefix != RepetitionPrefix::None {11} else {9}
            },
            0xAB => {
                self.instr.opcode = Opcode::STOSW;
                self.instr.data_length = Length::Word;

                self.instr.cycles += if self.instr.repetition_prefix != RepetitionPrefix::None {15} else {9}
            },

            0xE8 => {
                self.instr.opcode = Opcode::CALL;
                let val_low = self.fetch(bus);
                let val_high = self.fetch(bus);
                let val = to_u16(val_low, val_high);

                self.instr.jump_type = JumpType::DirWithinSegment(val);

                self.instr.cycles += 23;
            },
            0x9A => {
                self.instr.opcode = Opcode::CALL;

                let offset_low = self.fetch(bus);
                let offset_high = self.fetch(bus);
                let seg_low = self.fetch(bus);
                let seg_high = self.fetch(bus);
                let offset = to_u16(offset_low, offset_high);
                let seg = to_u16(seg_low, seg_high);
                self.instr.jump_type = JumpType::DirIntersegment(offset, seg);

                self.instr.cycles += 36;
            },
            0xE9 => {
                self.instr.opcode = Opcode::JMP;
                let val_low = self.fetch(bus);
                let val_high = self.fetch(bus);
                let val = to_u16(val_low, val_high);

                self.instr.jump_type = JumpType::DirWithinSegment(val);

                self.instr.cycles += 15;
            },
            0xEB => {
                self.instr.opcode = Opcode::JMP;
                let val = self.fetch(bus);

                self.instr.jump_type = JumpType::DirWithinSegmentShort(val);

                self.instr.cycles += 15;
            },
            0xC2 => {
                self.instr.opcode = Opcode::RET;
                let val = to_u16(self.fetch(bus), self.fetch(bus));
                self.instr.ret_type = RetType::NearAdd(val);
                self.instr.cycles += 24;
            },
            0xC3 => {
                self.instr.opcode = Opcode::RET;
                self.instr.ret_type = RetType::Near;
                self.instr.cycles += 20;
            },
            0xCA => {
                self.instr.opcode = Opcode::RET;
                let val = to_u16(self.fetch(bus), self.fetch(bus));
                self.instr.ret_type = RetType::FarAdd(val);
                self.instr.cycles += 34;
            },
            0xCB => {
                self.instr.opcode = Opcode::RET;
                self.instr.ret_type = RetType::Far;
                self.instr.cycles += 33;
            },
            0x74 => {
                self.instr.opcode = Opcode::JEJZ;
                let val = self.fetch(bus);
                self.instr.jump_type = JumpType::DirWithinSegmentShort(val);
            },
            0x7C => {
                self.instr.opcode = Opcode::JLJNGE;
                let val = self.fetch(bus);
                self.instr.jump_type = JumpType::DirWithinSegmentShort(val);
            },
            0x7E => {
                self.instr.opcode = Opcode::JLEJNG;
                let val = self.fetch(bus);
                self.instr.jump_type = JumpType::DirWithinSegmentShort(val);
            },
            0x72 => {
                self.instr.opcode = Opcode::JBJNAE;
                let val = self.fetch(bus);
                self.instr.jump_type = JumpType::DirWithinSegmentShort(val);
            },
            0x76 => {
                self.instr.opcode = Opcode::JBEJNA;
                let val = self.fetch(bus);
                self.instr.jump_type = JumpType::DirWithinSegmentShort(val);
            },
            0x7A => {
                self.instr.opcode = Opcode::JPJPE;
                let val = self.fetch(bus);
                self.instr.jump_type = JumpType::DirWithinSegmentShort(val);
            },
            0x70 => {
                self.instr.opcode = Opcode::JO;
                let val = self.fetch(bus);
                self.instr.jump_type = JumpType::DirWithinSegmentShort(val);
            },
            0x78 => {
                self.instr.opcode = Opcode::JS;
                let val = self.fetch(bus);
                self.instr.jump_type = JumpType::DirWithinSegmentShort(val);
            },
            0x75 => {
                self.instr.opcode = Opcode::JNEJNZ;
                let val = self.fetch(bus);
                self.instr.jump_type = JumpType::DirWithinSegmentShort(val);
            },
            0x7D => {
                self.instr.opcode = Opcode::JNLJGE;
                let val = self.fetch(bus);
                self.instr.jump_type = JumpType::DirWithinSegmentShort(val);
            }
            0x7F => {
                self.instr.opcode = Opcode::JNLEJG;
                let val = self.fetch(bus);
                self.instr.jump_type = JumpType::DirWithinSegmentShort(val);
            },
            0x73 => {
                self.instr.opcode = Opcode::JNBJAE;
                let val = self.fetch(bus);
                self.instr.jump_type = JumpType::DirWithinSegmentShort(val);
            },
            0x77 => {
                self.instr.opcode = Opcode::JNBEJA;
                let val = self.fetch(bus);
                self.instr.jump_type = JumpType::DirWithinSegmentShort(val);
            },
            0x7B => {
                self.instr.opcode = Opcode::JNPJPO;
                let val = self.fetch(bus);
                self.instr.jump_type = JumpType::DirWithinSegmentShort(val);
            },
            0x71 => {
                self.instr.opcode = Opcode::JNO;
                let val = self.fetch(bus);
                self.instr.jump_type = JumpType::DirWithinSegmentShort(val);
            },
            0x79 => {
                self.instr.opcode = Opcode::JNS;
                let val = self.fetch(bus);
                self.instr.jump_type = JumpType::DirWithinSegmentShort(val);
            },
            0xE2 => {
                self.instr.opcode = Opcode::LOOP;
                let val = self.fetch(bus);

                self.instr.jump_type = JumpType::DirWithinSegmentShort(val);
            },
            0xE1 => {
                self.instr.opcode = Opcode::LOOPZE;
                let val = self.fetch(bus);

                self.instr.jump_type = JumpType::DirWithinSegmentShort(val);
            },
            0xE0 => {
                self.instr.opcode = Opcode::LOOPNZNE;
                let val = self.fetch(bus);

                self.instr.jump_type = JumpType::DirWithinSegmentShort(val);
            },
            0xE3 => {
                self.instr.opcode = Opcode::JCXZ;
                let val = self.fetch(bus);

                self.instr.jump_type = JumpType::DirWithinSegmentShort(val);
            },

            0xCC => {
                self.instr.opcode = Opcode::INT;

                self.sw_int_type = 3;

                self.instr.cycles += 72;
            },
            0xCD => {
                self.instr.opcode = Opcode::INT;

                self.sw_int_type = self.fetch(bus);

                self.instr.cycles += 71;
            },
            0xCE => {
                self.instr.opcode = Opcode::INTO;

                self.sw_int_type = 4;
            },
            0xCF =>{
                self.instr.opcode = Opcode::IRET;

                self.instr.cycles += 44;
            }

            0xF8 => {
                self.instr.opcode = Opcode::CLC;
                self.instr.cycles += 2;
            },
            0xF5 => {
                self.instr.opcode = Opcode::CMC;
                self.instr.cycles += 2;
            },
            0xF9 => {
                self.instr.opcode = Opcode::STC;
                self.instr.cycles += 2;
            },
            0xFC => {
                self.instr.opcode = Opcode::CLD;
                self.instr.cycles += 2;
            },
            0xFD => {
                self.instr.opcode = Opcode::STD;
                self.instr.cycles += 2;
            },
            0xFA => {
                self.instr.opcode = Opcode::CLI;
                self.instr.cycles += 2;
            },
            0xFB => {
                self.instr.opcode = Opcode::STI;
                self.instr.cycles += 2;
            },

            _ => {
                writeln!(&mut self.file, "Instrucción sin hacer: {:02X}", op).unwrap();

                self.instr.cycles += 3
            }
            // _ => unreachable!(),
        }
    }
}