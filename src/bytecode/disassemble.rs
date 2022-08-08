
use std::fmt;
use crate::bytecode::instruction::Instruction;
use crate::bytecode::opcode::{
  SuperCode,
  misc::{self, move_subcode::DecodedMove},
  index::{self, DecodedIndex},
  comparison::{self, DecodedComparison},
  arithmetic::{self, DecodedArithmetic},
};
use crate::bytecode::operand::{
  FromDestination, FromSource,
  RawRegister, Global,
  WildDestination,
  InstructionPointer, FunctionKey
};

pub fn disassemble_instruction(f: &mut fmt::Formatter, instruction: Instruction) -> fmt::Result {
  match SuperCode::from(instruction) {
    SuperCode::Misc => {
      let subcode = misc::Subcode::from(instruction);
      match subcode {
        misc::Subcode::Jump => {
          use misc::jump_subcode::Reason;
          match Reason::from(instruction) {
            Reason::Special => {
              use misc::jump_subcode::Special;
              match Special::from(instruction) {
                Special::NoOp => write!(f, "nop"),
                Special::Return => write!(f, "ret"),
                _ => unimplemented!(),
              }
            }
            Reason::Always => {
              let instruction_pointer = InstructionPointer::from_both_operands(instruction);
              write!(f, "jmp   {instruction_pointer:<8}")
            }
            Reason::IfFalse => {
              use misc::jump_subcode::ConditionType;
              let instruction_pointer = InstructionPointer::from_both_operands(instruction);
              let condition_type = ConditionType::from(instruction);
              let condition: WildDestination<RawRegister> = match condition_type {
                ConditionType::Register => RawRegister::from_destination(instruction).into(),
                ConditionType::Global => Global::from_destination(instruction).into(),
              };
              write!(f, "jmp   {instruction_pointer:<8}  if !{condition}")
            }
            Reason::IfTrue => {
              use misc::jump_subcode::ConditionType;
              let instruction_pointer = InstructionPointer::from_both_operands(instruction);
              let condition_type = ConditionType::from(instruction);
              let condition: WildDestination<RawRegister> = match condition_type {
                ConditionType::Register => RawRegister::from_destination(instruction).into(),
                ConditionType::Global => Global::from_destination(instruction).into(),
              };
              write!(f, "jmp   {instruction_pointer:<8}  if {condition}")
            }
          }
        }
        misc::Subcode::Move => {
          let DecodedMove { destination, source } = misc::move_subcode::decode(instruction);
          write!(f, "mov   {destination} = {source}")
        }
        misc::Subcode::Call => {
          use misc::call_subcode::ArgCount;
          let arg_count = ArgCount::from(instruction);
          let arg_start = RawRegister::from_second(instruction);
          let dest = RawRegister::from_destination(instruction);
          let function = FunctionKey::from_first(instruction);
          match arg_count {
            ArgCount(0) => write!(f, "call  {dest} = {function}()"),
            ArgCount(1) => write!(f, "call  {dest} = {function}({arg_start})"),
            ArgCount(count) => {
              let last = RawRegister(arg_start.0 + count - 1);
              write!(f, "call  {dest} = {function}({arg_start}...{last})")
            }
          }
        }
        misc::Subcode::Interrupt => unimplemented!(),
      }
    }
    SuperCode::Index => {
      let DecodedIndex { index_on, destination, source, index } = index::decode(instruction);
      match index_on {
        index::On::Destination => write!(f, "idx   {destination}[{index}] = {source}"),
        index::On::Source => write!(f, "idx   {destination} = {source}[{index}]"),
      }
    }
    SuperCode::Comparison => {
      use comparison::Sources;
      let DecodedComparison { subcode, destination, sources } = comparison::decode(instruction);
      let subcode_op_str = subcode.op_str();
      match sources {
        Sources::FirstIsWild(first, second) =>
          write!(f, "{subcode:<4}  {destination} = {first} {subcode_op_str} {second}"),
        Sources::SecondIsWild(first, second) =>
          write!(f, "{subcode:<4}  {destination} = {first} {subcode_op_str} {second}"),
      }
    }
    SuperCode::Arithmetic => {
      use arithmetic::Sources;
      let DecodedArithmetic { subcode, destination, sources } = arithmetic::decode(instruction);
      let subcode_op_str = subcode.op_str();
      match sources {
        Sources::FirstIsWild(first, second) =>
          write!(f, "{subcode:<4}  {destination} = {first} {subcode_op_str} {second}"),
        Sources::SecondIsWild(first, second) =>
          write!(f, "{subcode:<4}  {destination} = {first} {subcode_op_str} {second}"),
      }
    }
  }
}
