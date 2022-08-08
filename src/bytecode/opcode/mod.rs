//! Representation of the opcode byte
//!
//! The opcode byte is the first byte (in little-endian) of an [`Instruction`].  All
//! instructions are 32 bits.  The remaining 3 bytes are [operands] which are interpreted
//! differently depending on the opcode.
//!
//! [operands]: super::operand

mod common;
pub mod misc;
pub mod index;
pub mod comparison;
pub mod arithmetic;

use std::ops::BitOr;
use super::instruction::Instruction;

/// Determines instruction category (bits 0..2)
#[derive(Debug, Copy, Clone)]
#[repr(u32)]
pub enum SuperCode {
  /// Miscellaneous instructions
  ///
  /// - Conditional and unconditional jumps
  /// - Function calls
  /// - Function returns
  /// - Move instructions
  ///
  /// Subcode module: [mod@misc]
  Misc          = 0b_00,
  /// Indexing instructions
  ///
  /// - Move to slot in indexable container
  /// - Move out of a slot in an indexable container
  ///
  /// Subcode module: [mod@index]
  Index         = 0b_01,
  /// Comparison instructions (`==`, `<`, `>=`, etc.)
  ///
  /// Subcode module: [mod@comparison]
  Comparison    = 0b_10,
  /// Arithmetic instructions (`+`, `*`, `%`, etc.)
  ///
  /// Subcode module: [mod@arithmetic]
  Arithmetic    = 0b_11,
}

impl SuperCode {
  pub const OFFSET: u32 = 0;
  pub const MASK: u32 = 0b_11;
}

impl From<Instruction> for SuperCode {
  fn from(instruction: Instruction) -> Self {
    use SuperCode::*;
    match instruction.0 & Self::MASK {
      0b_00 => Misc,
      0b_01 => Index,
      0b_10 => Comparison,
      0b_11 => Arithmetic,
      _ => unreachable!(),
    }
  }
}

impl From<SuperCode> for Instruction {
  fn from(value: SuperCode) -> Instruction {
    Instruction(value as u32)
  }
}

impl<Rhs: Into<Instruction>> BitOr<Rhs> for SuperCode {
  type Output = Instruction;
  fn bitor(self, rhs: Rhs) -> Self::Output {
    Instruction::from(self) | rhs.into()
  }
}

