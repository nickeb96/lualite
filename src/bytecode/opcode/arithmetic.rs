//! Arithmetic subcodes
//!
//! ```text
//! 11 xxx x xx  dddddddd  ffffffff  ssssssss
//! |/ \ / | \|  +-------  +-------  +-------
//! |   |  |  |  |         |         |
//! |   |  |  |  |         |         |
//! |   |  |  |  |         |         +-- the second source operand (wildcard or register depending on bit 5)
//! |   |  |  |  |         +-- first source operand (wildcard or register depending on bit 5)
//! |   |  |  |  +-- the destination operand (always interpreted as a register)
//! |   |  |  +-- the type of the wildcard source operand
//! |   |  +-- which source operand is a wildcard
//! |   +-- arithmetic operation type subcode
//! +-- super code (always 0b11 for arithmetic)
//! ```

use std::fmt;
use super::super::instruction::Instruction;
use super::super::operand::{
  FromDestination, FromSource, Register,
  RawRegister, Global, Immediate, ConstantKey,
  WildSource,
};
use super::common;

// 3 bits
#[derive(Debug, Copy, Clone)]
#[repr(u32)]
pub enum Subcode {
  Add = 0b_000_00,
  Sub = 0b_001_00,
  Mul = 0b_010_00,
  Div = 0b_011_00,
  Rem = 0b_100_00,
  Pow = 0b_101_00,
  Rot = 0b_110_00,
  Log = 0b_111_00,
}

impl Subcode {
  pub const OFFSET: u32 = 2;
  pub const MASK: u32 = 0b_111;
  pub fn op_str(&self) -> &'static str {
    use Subcode::*;
    match self {
      Add => "+",
      Sub => "-",
      Mul => "*",
      Div => "/",
      Rem => "%",
      Pow => "^",
      Rot => unimplemented!(),
      Log => unimplemented!(),
    }
  }
}

impl From<Instruction> for Subcode {
  fn from(instruction: Instruction) -> Self {
    use Subcode::*;
    match instruction.0 >> Self::OFFSET & Self::MASK {
      0b_000 => Add,
      0b_001 => Sub,
      0b_010 => Mul,
      0b_011 => Div,
      0b_100 => Rem,
      0b_101 => Pow,
      0b_110 => Rot,
      0b_111 => Log,
      _ => unreachable!(),
    }
  }
}

impl fmt::Display for Subcode {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    use Subcode::*;
    let s = match self {
      Add => "add",
      Sub => "sub",
      Mul => "mul",
      Div => "div",
      Rem => "rem",
      Pow => "pow",
      Rot => "root",
      Log => "log",
    };
    s.fmt(f)
  }
}

impl From<Subcode> for Instruction {
  fn from(value: Subcode) -> Instruction {
    Instruction(value as u32)
  }
}

pub type WhichSourceIsWild = common::WhichSourceIsWild<5>;

pub type WildSourceType = common::WildSourceType<6>;

#[derive(Debug, Clone)]
pub enum Sources {
  FirstIsWild(WildSource<RawRegister>, RawRegister),
  SecondIsWild(RawRegister, WildSource<RawRegister>),
}

#[derive(Debug, Clone)]
pub struct DecodedArithmetic {
  pub subcode: Subcode,
  pub destination: RawRegister,
  pub sources: Sources,
}

pub fn decode(instruction: Instruction) -> DecodedArithmetic {
  let subcode = Subcode::from(instruction);
  let destination = RawRegister::from_destination(instruction);
  let wild_source_type = WildSourceType::from(instruction);
  let sources = match WhichSourceIsWild::from(instruction) {
    WhichSourceIsWild::First => {
      let second = RawRegister::from_second(instruction);
      let first = match wild_source_type {
        WildSourceType::Register => WildSource::from(RawRegister::from_first(instruction)),
        WildSourceType::Global => WildSource::from(Global::from_first(instruction)),
        WildSourceType::Immediate => WildSource::from(Immediate::from_first(instruction)),
        WildSourceType::Constant => WildSource::from(ConstantKey::from_first(instruction)),
      };
      Sources::FirstIsWild(first, second)
    }
    WhichSourceIsWild::Second => {
      let first = RawRegister::from_first(instruction);
      let second = match wild_source_type {
        WildSourceType::Register => WildSource::from(RawRegister::from_second(instruction)),
        WildSourceType::Global => WildSource::from(Global::from_second(instruction)),
        WildSourceType::Immediate => WildSource::from(Immediate::from_second(instruction)),
        WildSourceType::Constant => WildSource::from(ConstantKey::from_second(instruction)),
      };
      Sources::SecondIsWild(first, second)
    }
  };
  DecodedArithmetic { subcode, destination, sources }
}

