
use std::ops::BitOr;
use super::super::operand::{WildDestination, WildSource, Register};
use super::super::instruction::Instruction;

// 1 bits
#[derive(Debug)]
#[repr(u32)]
pub enum WhichSourceIsWild<const OFFSET: u32> { // Other is always a register
  First   = 0b_0,
  Second  = 0b_1,
}

impl<const OFFSET: u32> WhichSourceIsWild<OFFSET> {
  pub const OFFSET: u32 = OFFSET;
  pub const MASK: u32 = 0b_1;
}

impl<const OFFSET: u32> From<Instruction> for WhichSourceIsWild<OFFSET> {
  fn from(instruction: Instruction) -> Self {
    match instruction.0 >> Self::OFFSET & Self::MASK {
      0b_0 => WhichSourceIsWild::First,
      0b_1 => WhichSourceIsWild::Second,
      _ => unreachable!(),
    }
  }
}

impl<const OFFSET: u32> From<WhichSourceIsWild<OFFSET>> for Instruction {
  fn from(value: WhichSourceIsWild<OFFSET>) -> Instruction {
    Instruction((value as u32) << OFFSET)
  }
}

impl<Rhs: Into<Instruction>, const OFFSET: u32> BitOr<Rhs> for WhichSourceIsWild<OFFSET> {
  type Output = Instruction;
  fn bitor(self, rhs: Rhs) -> Self::Output {
    Instruction::from(self) | rhs.into()
  }
}

// 2 bits
#[derive(Debug)]
#[repr(u32)]
pub enum WildSourceType<const OFFSET: u32> {
  Register  = 0b_00,
  Global    = 0b_01,
  Immediate = 0b_10,
  Constant  = 0b_11,
}

impl<const OFFSET: u32> WildSourceType<OFFSET> {
  pub const OFFSET: u32 = OFFSET;
  pub const MASK: u32 = 0b_11;
}

impl<const OFFSET: u32> From<Instruction> for WildSourceType<OFFSET> {
  fn from(instruction: Instruction) -> Self {
    match instruction.0 >> Self::OFFSET & Self::MASK {
      0b_00 => WildSourceType::Register,
      0b_01 => WildSourceType::Global,
      0b_10 => WildSourceType::Immediate,
      0b_11 => WildSourceType::Constant,
      _ => unreachable!(),
    }
  }
}

impl<R: Register, const OFFSET: u32> From<WildSource<R>> for WildSourceType<OFFSET> {
  fn from(source: WildSource<R>) -> Self {
    match source {
      WildSource::Register(_) => WildSourceType::Register,
      WildSource::Global(_) => WildSourceType::Global,
      WildSource::Immediate(_) => WildSourceType::Immediate,
      WildSource::Constant(_) => WildSourceType::Constant,
    }
  }
}

impl<const OFFSET: u32> From<WildSourceType<OFFSET>> for Instruction {
  fn from(value: WildSourceType<OFFSET>) -> Instruction {
    Instruction((value as u32) << OFFSET)
  }
}

impl<Rhs: Into<Instruction>, const OFFSET: u32> BitOr<Rhs> for WildSourceType<OFFSET> {
  type Output = Instruction;
  fn bitor(self, rhs: Rhs) -> Self::Output {
    Instruction::from(self) | rhs.into()
  }
}

// 1 bit
#[derive(Debug, Copy, Clone)]
#[repr(u32)]
pub enum WildDestinationType<const OFFSET: u32> {
  Register  = 0b_0,
  Global    = 0b_1,
}

impl<const OFFSET: u32> WildDestinationType<OFFSET> {
  pub const OFFSET: u32 = OFFSET;
  pub const MASK: u32 = 0b_1;
}

impl<const OFFSET: u32> From<Instruction> for WildDestinationType<OFFSET> {
  fn from(instruction: Instruction) -> Self {
    match instruction.0 >> Self::OFFSET & Self::MASK {
      0b_0 => WildDestinationType::Register,
      0b_1 => WildDestinationType::Global,
      _ => unreachable!(),
    }
  }
}

impl<R: Register, const OFFSET: u32> From<WildDestination<R>> for WildDestinationType<OFFSET> {
  fn from(destination: WildDestination<R>) -> Self {
    match destination {
      WildDestination::Register(_) => WildDestinationType::Register,
      WildDestination::Global(_) => WildDestinationType::Global,
    }
  }
}

impl<const OFFSET: u32> From<WildDestinationType<OFFSET>> for Instruction {
  fn from(value: WildDestinationType<OFFSET>) -> Instruction {
    Instruction((value as u32) << OFFSET)
  }
}

impl<Rhs: Into<Instruction>, const OFFSET: u32> BitOr<Rhs> for WildDestinationType<OFFSET> {
  type Output = Instruction;
  fn bitor(self, rhs: Rhs) -> Self::Output {
    Instruction::from(self) | rhs.into()
  }
}

