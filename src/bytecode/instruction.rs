
use std::fmt;
use std::ops::{BitOr, BitOrAssign};

/// Transparent wrapper around an unsigned 32 bit integer
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct Instruction(pub u32);

impl Instruction {
  pub fn empty() -> Self {
    Instruction(0)
  }
  fn raw_bitor(self, other: Self) -> Self {
    Instruction(self.0 | other.0)
  }
  fn raw_bitor_assign(&mut self, other: Self) {
    self.0 |= other.0
  }
}

impl<Rhs> BitOr<Rhs> for Instruction where Rhs: Into<Instruction> {
  type Output = Instruction;
  fn bitor(self, rhs: Rhs) -> Instruction {
    self.raw_bitor(rhs.into())
  }
}

impl<Rhs> BitOrAssign<Rhs> for Instruction where Rhs: Into<Instruction> {
  fn bitor_assign(&mut self, rhs: Rhs) {
    self.raw_bitor_assign(rhs.into());
  }
}

impl fmt::Binary for Instruction {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let [a, b, c, d] = self.0.to_le_bytes().map(|byte| byte.reverse_bits());
    write!(f, "{a:08b} {b:08b} {c:08b} {d:08b}")
  }
}

impl fmt::LowerHex for Instruction {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:08x}", self.0)
  }
}

impl fmt::UpperHex for Instruction {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:08X}", self.0)
  }
}

impl fmt::Display for Instruction {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    super::disassemble::disassemble_instruction(f, *self)
  }
}
