#![allow(unused)]
//! Indexing subcodes
//!
//! Instructions from the index category are parsed like so:
//! ```text
//! 01 x x xx xx  dddddddd  ssssssss  iiiiiiii
//! |/ | | \| \|  +-------  +-------  +-------
//! |  | |  |  |  |         |         |
//! |  | |  |  |  |         |         |
//! |  | |  |  |  |         |         +-- the index operand (register, global, immediate, or constant)
//! |  | |  |  |  |         +-- source operand (register, global, immediate, or constant)
//! |  | |  |  |  +-- the destination operand (register or global)
//! |  | |  |  +-- the type of the index operand
//! |  | |  +-- the type of the source operand
//! |  | +-- the type of the destination operand
//! |  +-- index on source or destination
//! +-- super code (always 0b_01 for index)
//! ```

use super::common;
use super::super::operand::{
  FromDestination, FromSource,
  Register, RawRegister, Global, Immediate, ConstantKey,
  WildSource, WildDestination,
};
use super::super::instruction::Instruction;

/// Determines whether the index is on the source or the destination (bit 2)
#[derive(Debug, Copy, Clone)]
#[repr(u32)]
pub enum On {
  Destination = 0b_0_00,
  Source      = 0b_1_00,
}

impl On {
  pub const OFFSET: u32 = 2;
  pub const MASK: u32 = 0b_1;
}

impl From<Instruction> for On {
  fn from(instruction: Instruction) -> Self {
    match instruction.0 >> Self::OFFSET & Self::MASK {
      0b_0 => On::Destination,
      0b_1 => On::Source,
      _ => unreachable!(),
    }
  }
}

impl From<On> for Instruction {
  fn from(value: On) -> Instruction {
    Instruction(value as u32)
  }
}

/// How to interpret the destination operand (bit 3)
pub type DestinationType = common::WildDestinationType<3>;

/// How to interpret the source operand (bits 4..6)
pub type SourceType = common::WildSourceType<4>;

/// How to interpret the index operand (bits 6..8)
pub type IndexType = common::WildSourceType<6>;

pub struct DecodedIndex {
  pub index_on: On,
  pub destination: WildDestination<RawRegister>,
  pub source: WildSource<RawRegister>,
  pub index: WildSource<RawRegister>,
}

pub fn decode(instruction: Instruction) -> DecodedIndex {
  let index_on = On::from(instruction);
  let destination_type = DestinationType::from(instruction);
  let destination = match destination_type {
    DestinationType::Register => WildDestination::Register(RawRegister::from_destination(instruction)),
    DestinationType::Global => WildDestination::Global(Global::from_destination(instruction)),
  };
  let source_type = SourceType::from(instruction);
  let source = match source_type {
    SourceType::Register => WildSource::Register(RawRegister::from_first(instruction)),
    SourceType::Global => WildSource::Global(Global::from_first(instruction)),
    SourceType::Immediate => WildSource::Immediate(Immediate::from_first(instruction)),
    SourceType::Constant => WildSource::Constant(ConstantKey::from_first(instruction)),
  };
  let index_type = IndexType::from(instruction);
  let index = match index_type {
    IndexType::Register => WildSource::Register(RawRegister::from_second(instruction)),
    IndexType::Global => WildSource::Global(Global::from_second(instruction)),
    IndexType::Immediate => WildSource::Immediate(Immediate::from_second(instruction)),
    IndexType::Constant => WildSource::Constant(ConstantKey::from_second(instruction)),
  };
  DecodedIndex { index_on, destination, source, index }
}

