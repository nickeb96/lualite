//! Instruction bytecode
//!
//! Bytecode represents a state in the `parse -> compile -> interpret` pipeline.  It is
//! the output of the [`compiler`] and the input of a [`VirtualMachine`].
//!
//! [`compiler`]: crate::compiler
//! [`VirtualMachine`]: crate::runtime::VirtualMachine
//!
//! All instructions are 32 bits, with the first byte being the [opcode] and the
//! remaining 3 bytes being used for [operand]s.  The opcode byte determines how
//! the operand bytes are interpreted.
//!
//! For the opcode byte, the first 2 bits are the [`SuperCode`](opcode::SuperCode).
//! This bit field separates the opcode into 4 instruction categories:
//! [mod@arithmetic], [mod@comparison], [mod@index], and [mod@misc].  The
//! remaining 6 bits of the opcode are interpreted differently depending on the
//! category.

mod instruction;
mod procedure;
pub mod opcode;
pub mod operand;
pub mod disassemble;
pub mod constant_value;

use opcode::{SuperCode, misc, index, comparison, arithmetic};
use operand::{AsDestination, AsSource, Register, WildSource, WildDestination, InstructionPointer, FunctionKey};

pub use instruction::Instruction;
pub use procedure::Procedure;

// Misc

pub fn nop() -> Instruction {
  use misc::{Subcode, jump_subcode::{Reason, Special}};
  SuperCode::Misc | Subcode::Jump | Reason::Special | Special::NoOp
}

pub fn ret() -> Instruction {
  use misc::{Subcode, jump_subcode::{Reason, Special}};
  SuperCode::Misc | Subcode::Jump | Reason::Special | Special::Return
}

pub fn jmp(ip: InstructionPointer) -> Instruction {
  use misc::{Subcode, jump_subcode::Reason};
  SuperCode::Misc | Subcode::Jump | Reason::Always | ip.as_both_operands()
}

pub fn jmp_if_true<R: Register>(condition: WildDestination<R>, ip: InstructionPointer) -> Instruction {
  use misc::{Subcode, jump_subcode::{Reason, ConditionType}};
  let condition_type = ConditionType::from(condition.clone());
  SuperCode::Misc | Subcode::Jump | Reason::IfTrue | condition_type
    | condition.as_destination() | ip.as_both_operands()
}

pub fn jmp_if_false<R: Register>(condition: WildDestination<R>, ip: InstructionPointer) -> Instruction {
  use misc::{Subcode, jump_subcode::{Reason, ConditionType}};
  let condition_type = ConditionType::from(condition.clone());
  SuperCode::Misc | Subcode::Jump | Reason::IfFalse | condition_type
    | condition.as_destination() | ip.as_both_operands()
}

pub fn mov<RD: Register, RS: Register>(dest: WildDestination<RD>, source: WildSource<RS>) -> Instruction {
  use misc::{Subcode, move_subcode::{DestinationType, SourceType}};
  let dest_type = DestinationType::from(dest.clone());
  let source_type = SourceType::from(source.clone());
  SuperCode::Misc | Subcode::Move | dest_type | source_type
    | dest.as_destination() | source.as_first()
}

pub fn call<RD: Register, RA: Register>(
  arg_count: misc::call_subcode::ArgCount,
  dest: RD,
  fn_key: FunctionKey,
  arg_start: RA,
) -> Instruction {
  use misc::Subcode;
  SuperCode::Misc | Subcode::Call | arg_count
    | dest.as_destination() | fn_key.as_first() | arg_start.as_second()
}

// Index

pub fn index<RD: Register, RS: Register, RI: Register, D: Into<WildDestination<RD>>,
    S: Into<WildSource<RS>>, I: Into<WildSource<RI>>>(
  on: index::On,
  dest: D,
  source: S,
  index: I,
) -> Instruction {
  use index::{DestinationType, SourceType, IndexType};
  let dest = dest.into();
  let source = source.into();
  let index = index.into();
  let dest_type = DestinationType::from(dest.clone());
  let source_type = SourceType::from(source.clone());
  let index_type = IndexType::from(index.clone());
  SuperCode::Index | on | dest_type | source_type | index_type
    | dest.as_destination() | source.as_first() | index.as_second()
}

// Comparison

pub fn cmp_wr<RD: Register, RF: Register, RS: Register, W: Into<WildSource<RF>>>(
  subcode: comparison::Subcode,
  dest: RD,
  first: W,
  second: RS,
) -> Instruction {
  use comparison::{WhichSourceIsWild, WildSourceType};
  let first = first.into();
  let which_source_is_wild = WhichSourceIsWild::First;
  let wild_source_type = WildSourceType::from(first.clone());
  SuperCode::Comparison | subcode | which_source_is_wild | wild_source_type
    | dest.as_destination() | first.as_first() | second.as_second()
}

pub fn cmp_rw<RD: Register, RF: Register, RS: Register, W: Into<WildSource<RS>>>(
  subcode: comparison::Subcode,
  dest: RD,
  first: RF,
  second: W,
) -> Instruction {
  use comparison::{WhichSourceIsWild, WildSourceType};
  let second = second.into();
  let which_source_is_wild = WhichSourceIsWild::Second;
  let wild_source_type = WildSourceType::from(second.clone());
  SuperCode::Comparison | subcode | which_source_is_wild | wild_source_type
    | dest.as_destination() | first.as_first() | second.as_second()
}

// Arithmetic

pub fn math_wr<RD: Register, RF: Register, RS: Register, W: Into<WildSource<RF>>>(
  subcode: arithmetic::Subcode,
  dest: RD,
  first: W,
  second: RS,
) -> Instruction {
  use arithmetic::{WhichSourceIsWild, WildSourceType};
  let first = first.into();
  let which_source_is_wild = WhichSourceIsWild::First;
  let wild_source_type = WildSourceType::from(first.clone());
  SuperCode::Arithmetic | subcode | which_source_is_wild | wild_source_type
    | dest.as_destination() | first.as_first() | second.as_second()
}

pub fn math_rw<RD: Register, RF: Register, RS: Register, W: Into<WildSource<RS>>>(
  subcode: arithmetic::Subcode,
  dest: RD,
  first: RF,
  second: W,
) -> Instruction {
  use arithmetic::{WhichSourceIsWild, WildSourceType};
  let second = second.into();
  let which_source_is_wild = WhichSourceIsWild::Second;
  let wild_source_type = WildSourceType::from(second.clone());
  SuperCode::Arithmetic | subcode | which_source_is_wild | wild_source_type
    | dest.as_destination() | first.as_first() | second.as_second()
}

