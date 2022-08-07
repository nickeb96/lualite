
use std::fmt;
use std::rc::Weak;
use std::cell::Cell;
use super::constant_value::ConstantValue;
use super::instruction::Instruction;
use super::operand::{ConstantKey, FunctionKey};

#[derive(Debug)]
pub struct Procedure {
  pub bytecode: Vec<Instruction>,
  pub register_count: usize,
  pub max_args: usize,
  pub constants: Vec<ConstantValue>,
  pub functions: Vec<String>,
}

impl fmt::Display for Procedure {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    writeln!(f, "registers: {}", self.register_count)?;
    writeln!(f, "arg count: {}", self.max_args)?;
    if self.constants.is_empty() {
      writeln!(f, "constant table: (empty)")?;
    } else {
      writeln!(f, "constant table:")?;
      for (constant_key, constant_value) in self.constants.iter().enumerate() {
        let constant_key = ConstantKey(constant_key as u8);
        writeln!(f, "{constant_key:>4}: {constant_value:<}")?;
      }
    }
    if self.functions.is_empty() {
      writeln!(f, "function table: (empty)")?;
    } else {
      writeln!(f, "function table:")?;
      for (function_key, function_name) in self.functions.iter().enumerate() {
        let function_key = FunctionKey(function_key as u8);
        writeln!(f, "{function_key:>4}: {function_name:?}")?;
      }
    }
    writeln!(f, "bytecode:")?;
    for (ip, instruction) in self.bytecode.iter().enumerate() {
      writeln!(f, "  {ip:>4}  {instruction}")?;
    }
    Ok(())
  }
}

