#![allow(unused)]
//! Comparison subcodes
//!
//! Instructions from the comparison category are parsed like so:
//! ```text
//! 10 xxx x xx  dddddddd  ffffffff  ssssssss
//! |/ \ / | \|  +-------  +-------  +-------
//! |   |  |  |  |         |         |
//! |   |  |  |  |         |         |
//! |   |  |  |  |         |         +-- the second source operand (wildcard or register depending on bit 5)
//! |   |  |  |  |         +-- first source operand (wildcard or register depending on bit 5)
//! |   |  |  |  +-- the destination operand (always interpreted as a register)
//! |   |  |  +-- the type of the wildcard source operand
//! |   |  +-- which source operand is a wildcard
//! |   +-- comparison operation type subcode
//! +-- super code (always 0b_10 for comparison)
//! ```

use std::fmt;
use super::super::instruction::Instruction;
use super::super::operand::{
  FromDestination, FromSource, Register,
  RawRegister, Global, Immediate, ConstantKey,
  WildSource,
};
use super::common;

/// Comparison sub-opcode type (bits 2..5)
#[derive(Debug, Copy, Clone)]
#[repr(u32)]
pub enum Subcode {
  /// `==`
  Eq = 0b_000_00,
  /// `!=`
  Ne = 0b_001_00,
  /// `<`
  Lt = 0b_010_00,
  /// `>`
  Gt = 0b_011_00,
  /// `<=`
  Le = 0b_100_00,
  /// `>=`
  Ge = 0b_101_00,
  /// Unused
  Xa = 0b_110_00,
  /// Unused
  Xb = 0b_111_00,
}

impl Subcode {
  pub const OFFSET: u32 = 2;
  pub const MASK: u32 = 0b_111;
  pub fn op_str(&self) -> &'static str {
    use Subcode::*;
    match self {
      Eq => "==",
      Ne => "!=",
      Lt => "<",
      Gt => ">",
      Le => "<=",
      Ge => ">=",
      Xa => "Xa",
      Xb => "Xb",
    }
  }
}

impl From<Instruction> for Subcode {
  fn from(instruction: Instruction) -> Self {
    use Subcode::*;
    match instruction.0 >> Self::OFFSET & Self::MASK {
      0b_000 => Eq,
      0b_001 => Ne,
      0b_010 => Lt,
      0b_011 => Gt,
      0b_100 => Le,
      0b_101 => Ge,
      0b_110 => Xa,
      0b_111 => Xb,
      _ => unreachable!(),
    }
  }
}

impl fmt::Display for Subcode {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    use Subcode::*;
    let s = match self {
      Eq => "eq",
      Ne => "ne",
      Lt => "lt",
      Gt => "gt",
      Le => "le",
      Ge => "ge",
      Xa => "xa",
      Xb => "xb",
    };
    s.fmt(f)
  }
}

impl From<Subcode> for Instruction {
  fn from(subcode: Subcode) -> Instruction {
    Instruction(subcode as u32)
  }
}

/// Whether the first source or the second source is a wildcard (bit 5)
pub type WhichSourceIsWild = common::WhichSourceIsWild<5>;

/// How to interpret the wildcard operand (bits 6..8)
pub type WildSourceType = common::WildSourceType<6>;

#[derive(Debug, Clone)]
pub enum Sources {
  FirstIsWild(WildSource<RawRegister>, RawRegister),
  SecondIsWild(RawRegister, WildSource<RawRegister>),
}

#[derive(Debug, Clone)]
pub struct DecodedComparison {
  pub subcode: Subcode,
  pub destination: RawRegister,
  pub sources: Sources,
}

pub fn decode(instruction: Instruction) -> DecodedComparison {
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
  DecodedComparison { subcode, destination, sources }
}

