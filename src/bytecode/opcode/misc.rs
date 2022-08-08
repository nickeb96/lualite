#![allow(unused)]
//! Miscellaneous instruction subcodes
//!
//! Misc has 4 different sub-categories of instructions:
//! - Jump
//! - Move
//! - Call
//! - Interrupt (currently unimplemented)

use std::fmt;
use super::super::instruction::Instruction;
use super::super::operand::{
  FromDestination, FromSource,
  Register, RawRegister, Global, Immediate, ConstantKey,
  WildDestination, WildSource,
};
use super::common;

/// Instruction type (bits 2..4)
#[derive(Debug, Copy, Clone)]
#[repr(u32)]
pub enum Subcode {
  Jump      = 0b_00_00,
  Move      = 0b_01_00,
  Call      = 0b_10_00,
  Interrupt = 0b_11_00,
}

impl Subcode {
  pub const OFFSET: u32 = 2;
  pub const MASK: u32 = 0b_11;
}

impl From<Instruction> for Subcode {
  fn from(instruction: Instruction) -> Self {
    use Subcode::*;
    match instruction.0 >> Self::OFFSET & Self::MASK {
      0b_00 => Jump,
      0b_01 => Move,
      0b_10 => Call,
      0b_11 => Interrupt,
      _ => unreachable!(),
    }
  }
}

impl From<Subcode> for Instruction {
  fn from(subcode: Subcode) -> Instruction {
    Instruction(subcode as u32)
  }
}

pub mod jump_subcode {
  use super::*;
  /// Reason for jumping (bits 4..6)
  ///
  /// For conditional jumps, bits 8..16 hold a conditional operand which will be
  /// either a register or a global key depending on [`ConditionType`].
  ///
  /// For conditional and unconditional jumps, bits 16..32 hold an
  /// [`InstructionPointer`](crate::bytecode::operand::InstructionPointer) to jump to.
  #[derive(Debug, Copy, Clone)]
  #[repr(u32)]
  pub enum Reason {
    /// See [`jump_subcode::Special`]
    Special = 0b_00_0000,
    /// Unconditional jump (no conditional operand)
    Always  = 0b_01_0000, // remaining 2 bits are unused
    /// Jump if conditional operand (bits 8..16) is false
    IfFalse = 0b_10_0000,
    /// Jump if conditional operand (bits 8..16) is true
    IfTrue  = 0b_11_0000,
  }

  impl Reason {
    pub const OFFSET: u32 = 4;
    pub const MASK: u32 = 0b_11;
  }

  impl From<Instruction> for Reason {
    fn from(instruction: Instruction) -> Self {
      use Reason::*;
      match instruction.0 >> Self::OFFSET & Self::MASK  {
        0b_00 => Special,
        0b_01 => Always,
        0b_10 => IfFalse,
        0b_11 => IfTrue,
        _ => unreachable!(),
      }
    }
  }

  impl fmt::Display for Reason {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      use Reason::*;
      match self {
        Special => write!(f, "special"),
        Always => write!(f, "always"),
        IfFalse => write!(f, "if false"),
        IfTrue => write!(f, "if true"),
      }
    }
  }

  impl From<Reason> for Instruction {
    fn from(reason: Reason) -> Instruction {
      Instruction(reason as u32)
    }
  }

  /// Special reasons for jumping (bits 6..8)
  #[derive(Debug, Copy, Clone)]
  #[repr(u32)]
  pub enum Special {
    /// Don't jump at all (no-op)
    NoOp    = 0b_00_000000,
    /// Function return
    Return  = 0b_01_000000,
    /// Currently unused
    Xa      = 0b_10_000000,
    /// Currently unused
    Xb      = 0b_11_000000,
  }

  impl Special {
    pub const OFFSET: u32 = 6;
    pub const MASK: u32 = 0b_11;
  }

  impl From<Instruction> for Special {
    fn from(instruction: Instruction) -> Self {
      use Special::*;
      match instruction.0 >> Self::OFFSET & Self::MASK {
        0b_00 => NoOp,
        0b_01 => Return,
        0b_10 => Xa,
        0b_11 => Xb,
        _ => unreachable!(),
      }
    }
  }

  impl From<Special> for Instruction {
    fn from(special: Special) -> Instruction {
      Instruction(special as u32)
    }
  }

  /// How to interpret the conditional operand (bit 6)
  ///
  /// Possible values are:
  /// - `ConditionType::Register` --- conditional operand will be interpreted as a register
  /// - `ConditionType::Global` --- conditional operand will be interpreted as a key into the global table
  ///
  /// The conditional operand is in bits 8..16 of the instruction
  pub type ConditionType = common::WildDestinationType<6>;
}

pub mod move_subcode {
  use super::*;
  pub type DestinationType = common::WildDestinationType<4>;
  pub type SourceType = common::WildSourceType<5>;

  pub struct DecodedMove {
    pub destination: WildDestination<RawRegister>,
    pub source: WildSource<RawRegister>,
  }

  pub fn decode(instruction: Instruction) -> DecodedMove {
    let destination_type = DestinationType::from(instruction);
    let source_type = SourceType::from(instruction);
    let destination: WildDestination<RawRegister> = match destination_type {
      DestinationType::Register => RawRegister::from_destination(instruction).into(),
      DestinationType::Global => Global::from_destination(instruction).into(),
    };
    let source: WildSource<RawRegister> = match source_type {
      SourceType::Register => RawRegister::from_first(instruction).into(),
      SourceType::Global => Global::from_first(instruction).into(),
      SourceType::Immediate => Immediate::from_first(instruction).into(),
      SourceType::Constant => ConstantKey::from_first(instruction).into(),
    };
    DecodedMove { destination, source }
  }
}

pub mod call_subcode {
  use super::*;
  // 4 bits

  #[derive(Debug, Copy, Clone)]
  #[repr(transparent)]
  pub struct ArgCount(pub u8);

  impl ArgCount {
    pub const OFFSET: u32 = 4;
    pub const MASK: u32 = 0b_1111;
  }

  impl From<Instruction> for ArgCount {
    fn from(instruction: Instruction) -> ArgCount {
      ArgCount((instruction.0 >> ArgCount::OFFSET & ArgCount::MASK) as u8)
    }
  }

  impl From<ArgCount> for Instruction {
    fn from(arg_count: ArgCount) -> Instruction {
      Instruction((arg_count.0 as u32) << ArgCount::OFFSET)
    }
  }
}

