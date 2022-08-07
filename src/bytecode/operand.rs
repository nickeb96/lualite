
use std::fmt;
use super::instruction::Instruction;

/// Bit offset for the destination byte
pub const DESTINATION_OFFSET: u32 = 8;
/// Bit offset for the first source byte
pub const FIRST_SOURCE_OFFSET: u32 = 16;
/// Bit offset for the second source byte
pub const SECOND_SOURCE_OFFSET: u32 = 24;
/// Bit offset for an instruction pointer (2 bytes)
///
/// Used by the jump family of instructions.
pub const INSTRUCTION_POINTER_OFFSET: u32 = 16;

/// Ability for a type to be the destination byte in an instruction
pub trait AsDestination {
  fn as_destination(self) -> Instruction;
}

/// Ability for a type to be created from an instruction destination
pub trait FromDestination {
  fn from_destination(instruction: Instruction) -> Self;
}

/// Ability for a type to be either of the source bytes in an instruction
pub trait AsSource {
  fn as_first(self) -> Instruction;
  fn as_second(self) -> Instruction;
}

/// Ability for a type to be created from an instruction source
pub trait FromSource {
  fn from_first(instruction: Instruction) -> Self;
  fn from_second(instruction: Instruction) -> Self;
}

impl<L: AsSource, R: AsSource> AsSource for either::Either<L, R> {
  fn as_first(self) -> Instruction {
    either::for_both!(self, inner => inner.as_first())
  }
  fn as_second(self) -> Instruction {
    either::for_both!(self, inner => inner.as_second())
  }
}

impl<L: AsDestination, R: AsDestination> AsDestination for either::Either<L, R> {
  fn as_destination(self) -> Instruction {
    either::for_both!(self, inner => inner.as_destination())
  }
}

/// Types that can be used as a register
pub trait Register: AsSource + AsDestination + Clone + fmt::Display { }

/// Register with a known register number
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct RawRegister(pub u8);

impl fmt::Display for RawRegister {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "R{}", self.0)
  }
}

impl Register for RawRegister { }

impl AsDestination for RawRegister {
  fn as_destination(self) -> Instruction {
    Instruction((self.0 as u32) << DESTINATION_OFFSET)
  }
}

impl AsSource for RawRegister {
  fn as_first(self) -> Instruction {
    Instruction((self.0 as u32) << FIRST_SOURCE_OFFSET)
  }
  fn as_second(self) -> Instruction {
    Instruction((self.0 as u32) << SECOND_SOURCE_OFFSET)
  }
}

impl FromDestination for RawRegister {
  fn from_destination(instruction: Instruction) -> Self {
    Self((instruction.0 >> DESTINATION_OFFSET) as u8)
  }
}

impl FromSource for RawRegister {
  fn from_first(instruction: Instruction) -> Self {
    Self((instruction.0 >> FIRST_SOURCE_OFFSET) as u8)
  }
  fn from_second(instruction: Instruction) -> Self {
    Self((instruction.0 >> SECOND_SOURCE_OFFSET) as u8)
  }
}

/// TODO
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct Global(pub u8);

impl fmt::Display for Global {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "G{}", self.0)
  }
}

impl AsDestination for Global {
  fn as_destination(self) -> Instruction {
    Instruction((self.0 as u32) << DESTINATION_OFFSET)
  }
}

impl FromDestination for Global {
  fn from_destination(instruction: Instruction) -> Self {
    Global((instruction.0 >> DESTINATION_OFFSET) as u8)
  }
}

impl AsSource for Global {
  fn as_first(self) -> Instruction {
    Instruction((self.0 as u32) << FIRST_SOURCE_OFFSET)
  }
  fn as_second(self) -> Instruction {
    Instruction((self.0 as u32) << SECOND_SOURCE_OFFSET)
  }
}

impl FromSource for Global {
  fn from_first(instruction: Instruction) -> Self {
    Global((instruction.0 >> FIRST_SOURCE_OFFSET) as u8)
  }
  fn from_second(instruction: Instruction) -> Self {
    Global((instruction.0 >> SECOND_SOURCE_OFFSET) as u8)
  }
}

/// Literal integer operand small enough to fit in a single byte
///
/// Must be in the range `-128..=127` to be an immediate.  Integers outside this
/// range and other literal types like double-precision floats must use [`ConstantKey`].
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct Immediate(pub i8);

impl fmt::Display for Immediate {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "#{}", self.0)
  }
}

impl AsSource for Immediate {
  fn as_first(self) -> Instruction {
    Instruction((self.0 as u32) << FIRST_SOURCE_OFFSET)
  }
  fn as_second(self) -> Instruction {
    Instruction((self.0 as u32) << SECOND_SOURCE_OFFSET)
  }
}

impl FromSource for Immediate {
  fn from_first(instruction: Instruction) -> Self {
    Immediate((instruction.0 >> FIRST_SOURCE_OFFSET) as u8 as i8)
  }
  fn from_second(instruction: Instruction) -> Self {
    Immediate((instruction.0 >> SECOND_SOURCE_OFFSET) as u8 as i8)
  }
}
/// A key into a function's constant table
///
/// Used for integer literals outsize the range of `-127..=128` and other literal types.
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct ConstantKey(pub u8);

impl fmt::Display for ConstantKey {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    // TODO: find a better way to forward format args
    format!("&{}", self.0).fmt(f)
  }
}

impl AsSource for ConstantKey {
  fn as_first(self) -> Instruction {
    Instruction((self.0 as u32) << FIRST_SOURCE_OFFSET)
  }
  fn as_second(self) -> Instruction {
    Instruction((self.0 as u32) << SECOND_SOURCE_OFFSET)
  }
}

impl FromSource for ConstantKey {
  fn from_first(instruction: Instruction) -> Self {
    ConstantKey((instruction.0 >> FIRST_SOURCE_OFFSET) as u8)
  }
  fn from_second(instruction: Instruction) -> Self {
    ConstantKey((instruction.0 >> SECOND_SOURCE_OFFSET) as u8)
  }
}

/// Wildcard destination operand (register or global)
#[derive(Debug, Clone)]
pub enum WildDestination<R: Register> {
  Register(R),
  Global(Global),
}

impl<R: Register> From<R> for WildDestination<R> {
  fn from(register: R) -> Self {
    Self::Register(register)
  }
}

impl<R: Register> From<Global> for WildDestination<R> {
  fn from(global: Global) -> Self {
    Self::Global(global)
  }
}

impl<R: Register> fmt::Display for WildDestination<R> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    use WildDestination::*;
    match self {
      Register(register) => write!(f, "{register}"),
      Global(global) => write!(f, "{global}"),
    }
  }
}

impl<R: Register> AsDestination for WildDestination<R> {
  fn as_destination(self) -> Instruction {
    use WildDestination::*;
    match self {
      Register(register) => register.as_destination(),
      Global(global) => global.as_destination(),
    }
  }
}

/// Wildcard source operand (register, global, immediate, or constant)
#[derive(Debug, Clone)]
pub enum WildSource<R: Register> {
  Register(R),
  Global(Global),
  Immediate(Immediate),
  Constant(ConstantKey),
}

impl<R: Register> From<R> for WildSource<R> {
  fn from(register: R) -> Self {
    Self::Register(register)
  }
}

impl<R: Register> From<Global> for WildSource<R> {
  fn from(global: Global) -> Self {
    Self::Global(global)
  }
}

impl<R: Register> From<Immediate> for WildSource<R> {
  fn from(immediate: Immediate) -> Self {
    Self::Immediate(immediate)
  }
}

impl<R: Register> From<ConstantKey> for WildSource<R> {
  fn from(constant: ConstantKey) -> Self {
    Self::Constant(constant)
  }
}

impl<R: Register> fmt::Display for WildSource<R> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    use WildSource::*;
    match self {
      Register(register) => write!(f, "{register}"),
      Global(global) => write!(f, "{global}"),
      Immediate(immediate) => write!(f, "{immediate}"),
      Constant(constant) => write!(f, "{constant}"),
    }
  }
}

impl<R: Register> AsSource for WildSource<R> {
  fn as_first(self) -> Instruction {
    use WildSource::*;
    match self {
      Register(register) => register.as_first(),
      Global(global) => global.as_first(),
      Immediate(immediate) => immediate.as_first(),
      Constant(constant) => constant.as_first(),
    }
  }
  fn as_second(self) -> Instruction {
    use WildSource::*;
    match self {
      Register(register) => register.as_second(),
      Global(global) => global.as_second(),
      Immediate(immediate) => immediate.as_second(),
      Constant(constant) => constant.as_second(),
    }
  }
}

/// An index into the bytecode array
///
/// A 16 bit unsigned integer taking up both source operands.  It is used by the
/// jump family of instructions under [misc](../opcode/misc/index.html).
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct InstructionPointer(pub u16);

impl InstructionPointer {
  pub fn empty_place_holder() -> Self {
    InstructionPointer(0)
  }
  pub fn as_both_operands(self) -> Instruction {
    Instruction((self.0 as u32) << INSTRUCTION_POINTER_OFFSET)
  }
  pub fn from_both_operands(instruction: Instruction) -> Self {
    InstructionPointer((instruction.0 >> FIRST_SOURCE_OFFSET & 0x_ff_ff) as u16)
  }
}

impl fmt::Display for InstructionPointer {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "ip ")?;
    self.0.fmt(f)
  }
}

/// A key into a function table
///
/// Each compiled function has a list of other functions it refers to.  A `FunctionKey`
/// is an index into this list.  `FunctionKey`s are currently only used by the call
/// instruction under [misc](../opcode/misc/index.html).
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct FunctionKey(pub u8);

impl FunctionKey {
  pub fn as_first(self) -> Instruction {
    Instruction((self.0 as u32) << FIRST_SOURCE_OFFSET)
  }
  pub fn from_first(instruction: Instruction) -> Self {
    FunctionKey((instruction.0 >> FIRST_SOURCE_OFFSET & 0xff) as u8)
  }
}

impl fmt::Display for FunctionKey {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    format!("F{}", self.0).fmt(f)
  }
}

