
use std::path::PathBuf;
use std::{error, fmt, io};
use std::fs::File;
use std::io::{BufReader, BufRead, BufWriter, Write};
use std::num::ParseIntError;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

use regex::Regex;

#[derive(Debug)]
pub enum AssemblerError {
    FileOpen(PathBuf, io::Error),
    FileRead(PathBuf, io::Error),
    FileWrite(PathBuf, io::Error),
    Syntax(usize, String),
    InvalidInstruction(usize, String),
    MissingOperand(usize, String),
    TooManyOperands(usize),
    InvalidRegister(usize, String),
    InvalidIntegerLiteral(usize, String, ParseIntError),
    InvalidCondition(usize, String),
    DuplicateLabel(usize, String),
    UndefinedLabel(String),
    AddressSpaceExhausted(),
}

impl fmt::Display for AssemblerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            AssemblerError::FileOpen(path, _) => write!(f, "unable to open {}", path.display()),
            AssemblerError::FileRead(path, _) => write!(f, "failed to read {}", path.display()),
            AssemblerError::FileWrite(path, _) => write!(f, "failed to write {}", path.display()),
            AssemblerError::Syntax(line_number, line) => write!(f, "syntax error at line {}: {}", line_number, line),
            AssemblerError::InvalidInstruction(line_number, instr) => write!(f, "invalid instruction \"{}\" at line {}", instr, line_number),
            AssemblerError::MissingOperand(line_number, name) => write!(f, "missing operand \"{}\" at line {}", name, line_number),
            AssemblerError::TooManyOperands(line_number) => write!(f, "too many operands at line {}", line_number),
            AssemblerError::InvalidRegister(line_number, name) => write!(f, "invalid register \"{}\" at line {}", name, line_number),
            AssemblerError::InvalidIntegerLiteral(line_number, literal, _) => write!(f, "invalid integer literal \"{}\" at line {}", literal, line_number),
            AssemblerError::InvalidCondition(line_number, instruction) => write!(f, "invalid condition \"{}\" at line {}", instruction, line_number),
            AssemblerError::DuplicateLabel(line_number, label) => write!(f, "duplicate label \"{}\" at line {}", label, line_number),
            AssemblerError::UndefinedLabel(label) => write!(f, "usage of undefined label \"{}\"", label),
            AssemblerError::AddressSpaceExhausted() => write!(f, "address space exhausted"),
        }
    }
}

impl error::Error for AssemblerError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            AssemblerError::FileOpen(_, io_error) => Some(io_error),
            AssemblerError::FileRead(_, io_error) => Some(io_error),
            AssemblerError::FileWrite(_, io_error) => Some(io_error),
            AssemblerError::InvalidIntegerLiteral(_, _, parse_error) => Some(parse_error),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum OpCode {
    MOV = 0x00,
    LD = 0x01,
    LDI = 0x11,
    ST = 0x02,
    AND = 0x03,
    ANDI = 0x13,
    OR = 0x04,
    ORI = 0x14,
    XOR = 0x05,
    XORI = 0x15,
    NOT = 0x06,
    ADD = 0x07,
    ADDI = 0x17,
    SUB = 0x08,
    SL = 0x09,
    SLI = 0x19,
    SR = 0x0A,
    SRI = 0x1A,
    CMP = 0x0B,
    CMPI = 0x1B,
    JMP = 0x0C,
}

#[derive(Copy, Clone, Debug)]
enum Register {
    A = 0x0,
    B = 0x1,
    Addr = 0x2,
    SP = 0x3,
    SR = 0x4,
    PC = 0x5,
}

impl Register {
    fn from_str(reg_str: &str) -> Option<Register> {
        let reg_str = reg_str.to_lowercase();
        match reg_str.as_str() {
            "a" => Some(Register::A),
            "b" => Some(Register::B),
            "addr" => Some(Register::Addr),
            "sp" => Some(Register::SP),
            "sr" => Some(Register::SR),
            "pc" => Some(Register::PC),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum Condition {
    None = 0x0,
    Zero = 0x1,
    Equal = 0x2,
    NotEqual = 0x3,
    LessThan = 0x4,
    LessThanOrEqual = 0x5,
    GreaterThan = 0x6,
    GreaterThanOrEqual = 0x7,
}

#[derive(Clone, Debug)]
enum InstructionData {
    None,
    Immediate1(u16),
    Register2(Register, Register),
    Jump(Condition, Register),

    Immediate1Reference(String),
}

impl InstructionData {
    fn encode(&self) -> u16 {
        match self {
            InstructionData::None => 0x0000,
            InstructionData::Immediate1(value) => *value,
            InstructionData::Register2(reg1, reg2) => (*reg2 as u16) << 3 | (*reg1 as u16),
            InstructionData::Jump(cond, reg) => (*reg as u16) << 3 | (*cond as u16),
            InstructionData::Immediate1Reference(_) => unreachable!(),
        }
    }
}

#[derive(Clone, Debug)]
struct Instruction {
    opcode: OpCode,
    data: InstructionData,
}

impl Instruction {
    fn new(opcode: OpCode, data: InstructionData) -> Instruction {
        Instruction {
            opcode,
            data,
        }
    }

    fn size(&self) -> u16 {
        1
    }

    fn encode(&self) -> u16 {
        (self.opcode as u16) << 11 | (self.data.encode() & 0x7ff)
    }
}

fn parse_u16(text: &str) -> Result<u16, ParseIntError> {
    if text.starts_with("0x") {
        u16::from_str_radix(&text["0x".len()..], 16)
    } else if text.starts_with("0o") {
        u16::from_str_radix(&text["0o".len()..], 8)
    } else if text.starts_with("0b") {
        u16::from_str_radix(&text["0b".len()..], 2)
    } else {
        u16::from_str_radix(text, 10)
    }
}

pub fn run(source_path: PathBuf, output_path: PathBuf) -> Result<(), AssemblerError> {
    let source_file = File::open(&source_path)
        .map_err(|err| AssemblerError::FileOpen(source_path.clone(), err))?;

    let source_reader = BufReader::new(source_file);

    let mut instructions = Vec::new();

    let mut labels = HashMap::new();

    let mut current_address = 0u16;

    let re = Regex::new(r"(?x)
^\s*  # Start of line
(?:([a-zA-Z_]\w*)\s*:\s*)?  # Optional label
# Instruction
(?:
  ([a-zA-Z.]+)  # Opcode
  # Operands
  (?:
    \s+
    # Operand 1
    (\w+)
    # Operand 2
    (?:\s*,\s*(\w+))?
    # Operand 3
    (?:\s*,\s*(\w+))?
  )?
)?
\s*
(?:\#.*)?$  # Comment and end of line
").unwrap();

    for (line_number, line) in source_reader.lines().enumerate() {
        let line = line
            .map_err(|err| AssemblerError::FileRead(source_path.clone(), err))?;

        let m = re.captures(&line);

        if let Some(captures) = m {
            let label = captures.get(1).map(|m| m.as_str());
            let instruction = captures.get(2).map(|m| m.as_str());
            let operand1 = captures.get(3).map(|m| m.as_str());
            let operand2 = captures.get(4).map(|m| m.as_str());
            let operand3 = captures.get(5).map(|m| m.as_str());

            if let Some(label) = label {
                match labels.entry(label.to_string()) {
                    Entry::Occupied(_) => {
                        return Err(AssemblerError::DuplicateLabel(line_number, label.to_string()));
                    },
                    Entry::Vacant(v) => {
                        v.insert(current_address);
                    },
                }
            }

            if let Some(instruction) = instruction {
                let instruction = instruction.to_lowercase();
                let instr = match instruction.as_str() {
                    "mov" => {
                        let target_str = operand1
                            .ok_or_else(|| AssemblerError::MissingOperand(line_number, "target register".to_string()))?;
                        let source_str = operand2
                            .ok_or_else(|| AssemblerError::MissingOperand(line_number, "target register".to_string()))?;
                        if operand3.is_some() {
                            return Err(AssemblerError::TooManyOperands(line_number));
                        }

                        let target = Register::from_str(target_str)
                            .ok_or_else(|| AssemblerError::InvalidRegister(line_number, target_str.to_string()))?;
                        let source = Register::from_str(source_str)
                            .ok_or_else(|| AssemblerError::InvalidRegister(line_number, source_str.to_string()))?;

                        Instruction::new(OpCode::MOV, InstructionData::Register2(target, source))
                    },
                    "ld" => {
                        Instruction::new(OpCode::LD, InstructionData::None)
                    },
                    "ldi" => {
                        let value_str = operand1
                            .ok_or_else(|| AssemblerError::MissingOperand(line_number, "value".to_string()))?;
                        if operand2.is_some() {
                            return Err(AssemblerError::TooManyOperands(line_number));
                        }

                        let first_char = value_str.chars().next().unwrap();

                        let data = if first_char.is_alphabetic() {
                            // Label
                            InstructionData::Immediate1Reference(value_str.to_string())
                        } else {
                            // Constant
                            let value = parse_u16(value_str)
                                .map_err(|err| AssemblerError::InvalidIntegerLiteral(line_number, value_str.to_string(), err))?;
                            InstructionData::Immediate1(value)
                        };

                        Instruction::new(OpCode::LDI, data)
                    },
                    "st" => {
                        Instruction::new(OpCode::ST, InstructionData::None)
                    },
                    "and" => {
                        Instruction::new(OpCode::AND, InstructionData::None)
                    },
                    "andi" => {
                        let value_str = operand1
                            .ok_or_else(|| AssemblerError::MissingOperand(line_number, "value".to_string()))?;
                        if operand2.is_some() {
                            return Err(AssemblerError::TooManyOperands(line_number));
                        }

                        let value = parse_u16(value_str)
                            .map_err(|err| AssemblerError::InvalidIntegerLiteral(line_number, value_str.to_string(), err))?;

                        Instruction::new(OpCode::ANDI, InstructionData::Immediate1(value))
                    },
                    "or" => {
                        Instruction::new(OpCode::OR, InstructionData::None)
                    },
                    "ori" => {
                        let value_str = operand1
                            .ok_or_else(|| AssemblerError::MissingOperand(line_number, "value".to_string()))?;
                        if operand2.is_some() {
                            return Err(AssemblerError::TooManyOperands(line_number));
                        }

                        let value = parse_u16(value_str)
                            .map_err(|err| AssemblerError::InvalidIntegerLiteral(line_number, value_str.to_string(), err))?;

                        Instruction::new(OpCode::ORI, InstructionData::Immediate1(value))
                    },
                    "xor" => {
                        Instruction::new(OpCode::XOR, InstructionData::None)
                    },
                    "xori" => {
                        let value_str = operand1
                            .ok_or_else(|| AssemblerError::MissingOperand(line_number, "value".to_string()))?;
                        if operand2.is_some() {
                            return Err(AssemblerError::TooManyOperands(line_number));
                        }

                        let value = parse_u16(value_str)
                            .map_err(|err| AssemblerError::InvalidIntegerLiteral(line_number, value_str.to_string(), err))?;

                        Instruction::new(OpCode::XORI, InstructionData::Immediate1(value))
                    },
                    "not" => {
                        Instruction::new(OpCode::NOT, InstructionData::None)
                    },
                    "add" => {
                        Instruction::new(OpCode::ADD, InstructionData::None)
                    },
                    "addi" => {
                        let value_str = operand1
                            .ok_or_else(|| AssemblerError::MissingOperand(line_number, "value".to_string()))?;
                        if operand2.is_some() {
                            return Err(AssemblerError::TooManyOperands(line_number));
                        }

                        let value = parse_u16(value_str)
                            .map_err(|err| AssemblerError::InvalidIntegerLiteral(line_number, value_str.to_string(), err))?;

                        Instruction::new(OpCode::ADDI, InstructionData::Immediate1(value))
                    },
                    "sub" => {
                        Instruction::new(OpCode::SUB, InstructionData::None)
                    },
                    "sl" => {
                        Instruction::new(OpCode::SL, InstructionData::None)
                    },
                    "sli" => {
                        let value_str = operand1
                            .ok_or_else(|| AssemblerError::MissingOperand(line_number, "value".to_string()))?;
                        if operand2.is_some() {
                            return Err(AssemblerError::TooManyOperands(line_number));
                        }

                        let value = parse_u16(value_str)
                            .map_err(|err| AssemblerError::InvalidIntegerLiteral(line_number, value_str.to_string(), err))?;

                        Instruction::new(OpCode::SLI, InstructionData::Immediate1(value))
                    },
                    "sr" => {
                        Instruction::new(OpCode::SR, InstructionData::None)
                    },
                    "sri" => {
                        let value_str = operand1
                            .ok_or_else(|| AssemblerError::MissingOperand(line_number, "value".to_string()))?;
                        if operand2.is_some() {
                            return Err(AssemblerError::TooManyOperands(line_number));
                        }

                        let value = parse_u16(value_str)
                            .map_err(|err| AssemblerError::InvalidIntegerLiteral(line_number, value_str.to_string(), err))?;

                        Instruction::new(OpCode::SRI, InstructionData::Immediate1(value))
                    },
                    "cmp" => {
                        Instruction::new(OpCode::CMP, InstructionData::None)
                    },
                    "cmpi" => {
                        let value_str = operand1
                            .ok_or_else(|| AssemblerError::MissingOperand(line_number, "value".to_string()))?;
                        if operand2.is_some() {
                            return Err(AssemblerError::TooManyOperands(line_number));
                        }

                        let value = parse_u16(value_str)
                            .map_err(|err| AssemblerError::InvalidIntegerLiteral(line_number, value_str.to_string(), err))?;

                        Instruction::new(OpCode::CMPI, InstructionData::Immediate1(value))
                    },
                    "jmp" |
                    "jmp.z" |
                    "jmp.eq" |
                    "jmp.ne" |
                    "jmp.lt" |
                    "jmp.le" |
                    "jmp.gt" |
                    "jmp.ge" => {
                        let cond = if instruction.starts_with("jmp.") {
                            let cond_str = &instruction["jmp.".len()..];

                            match cond_str {
                                "z" => Condition::Zero,
                                "eq" => Condition::Equal,
                                "ne" => Condition::NotEqual,
                                "lt" => Condition::LessThan,
                                "le" => Condition::LessThanOrEqual,
                                "gt" => Condition::GreaterThan,
                                "ge" => Condition::GreaterThanOrEqual,
                                _ => return Err(AssemblerError::InvalidCondition(line_number, instruction.clone())),
                            }
                        } else {
                            Condition::None
                        };

                        let source_str = operand1
                            .ok_or_else(|| AssemblerError::MissingOperand(line_number, "source register".to_string()))?;
                        if operand2.is_some() {
                            return Err(AssemblerError::TooManyOperands(line_number));
                        }

                        let source = Register::from_str(source_str)
                            .ok_or_else(|| AssemblerError::InvalidRegister(line_number, source_str.to_string()))?;

                        Instruction::new(OpCode::JMP, InstructionData::Jump(cond, source))
                    },
                    _ => return Err(AssemblerError::InvalidInstruction(line_number, instruction.clone())),
                };

                current_address = current_address.checked_add(instr.size())
                    .ok_or_else(|| AssemblerError::AddressSpaceExhausted())?;

                instructions.push(instr);
            }
        } else {
            return Err(AssemblerError::Syntax(line_number, line));
        }
    }

    for instruction in &mut instructions {
        match &instruction.data {
            InstructionData::Immediate1Reference(label) => {
                let address = labels.get(label)
                    .ok_or_else(|| AssemblerError::UndefinedLabel(label.to_string()))?;

                instruction.data = InstructionData::Immediate1(*address);
            },
            _ => {}
        }
    }

    let output_file = File::create(&output_path)
        .map_err(|err| AssemblerError::FileOpen(output_path.clone(), err))?;

    let mut output_writer = BufWriter::new(output_file);

    for instruction in &instructions {
        let encoded = instruction.encode();
        let encoded_bytes = encoded.to_le_bytes();
        output_writer.write_all(&encoded_bytes)
            .map_err(|err| AssemblerError::FileWrite(output_path.clone(), err))?;
    }

    Ok(())
}
