
use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Weak;
use either::{Either, Left, Right};
use crate::ast::{
  self, Identifier, IntegerLiteral, FloatLiteral, BooleanLiteral, StringLiteral,
  Statement, Expression, BinaryOperator,
};
use crate::bytecode;
use crate::bytecode::operand::{
  self, AsDestination, AsSource, Register,
  RawRegister, Immediate, ConstantKey,
  WildSource, WildDestination,
  InstructionPointer, FunctionKey,
};
use crate::bytecode::constant_value::ConstantValue;
use crate::bytecode::Instruction;
use crate::bytecode::Procedure;
use crate::compiler::temporary::{Temporary, TempManager};

type RawOrTemp = Either<RawRegister, Temporary>;
impl Register for RawOrTemp { }

#[derive(Debug)]
pub struct FunctionCompiler {
  bytecode: Vec<Instruction>,
  next_register: usize,
  param_count: usize,
  ident_map: HashMap<Identifier, RawRegister>,
  temps: TempManager,
  constants: Vec<ConstantValue>,
  function_keys: Vec<Identifier>,
}

impl FunctionCompiler {
  pub fn with_parameters(parameters: &[Identifier]) -> Self {
    Self {
      bytecode: vec![bytecode::nop()],
      next_register: 1 + parameters.len(), // +1 for return register
      param_count: parameters.len(),
      ident_map: HashMap::from_iter(parameters.iter().zip(1..)
                                    .map(|(ident, reg_num)| (ident.clone(), RawRegister(reg_num)))),
      temps: TempManager::new(1),
      constants: Vec::default(),
      function_keys: Vec::default(),
    }
  }

  pub fn finish(mut self) -> Procedure {
    self.temps.reconcile_deferred_temps(self.next_register as u8, &mut self.bytecode);
    Procedure {
      bytecode: self.bytecode,
      register_count: 1 + self.ident_map.len() + self.temps.count(), // +1 for return register
      max_args: self.param_count,
      constants: self.constants,
      functions: self.function_keys.into_iter().map(|name| name.0).collect(),
    }
  }
}

impl FunctionCompiler {
  pub fn next_instruction_pointer(&self) -> InstructionPointer {
    InstructionPointer((self.bytecode.len() - 1) as u16)
  }

  pub fn register_for_return(&self) -> RawRegister {
    RawRegister(0)
  }

  pub fn register_for(&mut self, ident: &Identifier) -> RawRegister {
    *self.ident_map.entry(ident.clone()).or_insert_with(|| {
      let reg = RawRegister(self.next_register.try_into().unwrap());
      self.next_register += 1;
      reg
    })
  }

  pub fn immediate_or_constant_for(&mut self, integer: &IntegerLiteral) -> WildSource<RawOrTemp> {
    match integer {
      IntegerLiteral(small_int @ -128..=127) => Immediate(*small_int as i8).into(),
      IntegerLiteral(large_int) => {
        let integer_as_constant = ConstantValue::Integer(*large_int);
        for (index, constant) in self.constants.iter().enumerate() {
          if &integer_as_constant == constant {
            return ConstantKey(index as u8).into();
          }
        }
        self.constants.push(integer_as_constant);
        ConstantKey((self.constants.len() - 1) as u8).into()
      }
    }
  }

  pub fn constant_for_string(&mut self, string: &StringLiteral) -> ConstantKey {
    for (index, constant) in self.constants.iter().enumerate() {
      match constant {
        ConstantValue::String(existing_string) if existing_string == &string.0 =>
          return ConstantKey(index as u8),
        _ => (),
      }
    }
    self.constants.push(ConstantValue::String(string.0.clone()));
    ConstantKey((self.constants.len() - 1) as u8)
  }

  pub fn constant_for_float(&mut self, float: &FloatLiteral) -> ConstantKey {
    for (index, constant) in self.constants.iter().enumerate() {
      match constant {
        ConstantValue::Float(existing_float) if existing_float == &float.0 =>
          return ConstantKey(index as u8),
        _ => (),
      }
    }
    self.constants.push(ConstantValue::Float(float.0));
    ConstantKey((self.constants.len() - 1) as u8)
  }

  pub fn constant_for_boolean(&mut self, boolean: &BooleanLiteral) -> ConstantKey {
    for (index, constant) in self.constants.iter().enumerate() {
      match constant {
        ConstantValue::Boolean(existing_boolean) if existing_boolean == &boolean.0 =>
          return ConstantKey(index as u8),
        _ => (),
      }
    }
    self.constants.push(ConstantValue::Boolean(boolean.0));
    ConstantKey((self.constants.len() - 1) as u8)
  }

  pub fn function_key_for(&mut self, function_name: Identifier) -> FunctionKey {
    for (index, existing_function_name) in self.function_keys.iter().enumerate() {
      if function_name == *existing_function_name {
        return FunctionKey(index as u8);
      }
    }
    self.function_keys.push(function_name);
    FunctionKey((self.function_keys.len() - 1) as u8)
  }

  pub fn push(&mut self, instruction: Instruction) {
    self.bytecode.push(instruction);
    self.temps.set_next_instruction_offset(self.bytecode.len());
  }
}

// Statements
impl FunctionCompiler {
  pub fn compile_statement(&mut self, statement: &Statement) {
    use Statement::*;
    match statement {
      SingleStatement(expression) => self.compile_expression(Left(RawRegister(0)), expression), // TODO: Create a "void" destination
      AssignStatement(identifier, expression) => {
        let dest = self.register_for(identifier);
        self.compile_expression(Left(dest), expression);
      }
      IndexAssignStatement { table, index, value } => {
        let dest = self.compile_into_register(table);
        let index_wildcard = self.compile_into_wildcard(index);
        let value_wildcard = self.compile_into_wildcard(value);
        let index_on = bytecode::opcode::index::On::Destination;
        self.push(bytecode::index(index_on, dest, value_wildcard, index_wildcard));
      }
      ReturnStatement(maybe_expression) => {
        if let Some(expression) = maybe_expression {
          self.compile_expression(Left(self.register_for_return()), expression);
        }
        self.push(bytecode::ret());
      }
      WhileStatement { condition, body } =>
        self.compile_while_statement(condition, body),
      IfStatement { condition, body, else_body } =>
        self.compile_if_statement(condition, body, else_body.as_deref()),
    }
  }

  pub fn compile_while_statement(&mut self, condition: &Expression, body: &[Statement]) {
    let begin_ip = self.next_instruction_pointer();
    // while
    let conditional_register = match condition {
      Expression::Identifier(conditional_ident) => Left(self.register_for(conditional_ident)),
      other => {
        let temp_dest = Right(self.temps.take_temp());
        self.compile_expression(temp_dest.clone(), condition);
        temp_dest
      }
    };
    let jump_offset = self.bytecode.len();
    self.push(bytecode::jmp_if_false(conditional_register.into(), InstructionPointer::empty_place_holder()));
    // do
    for statement in body.iter() {
      self.compile_statement(statement);
    }
    self.push(bytecode::jmp(begin_ip));
    // end
    let end_ip = self.next_instruction_pointer();
    self.bytecode[jump_offset] |= end_ip.as_both_operands();
  }

  pub fn compile_if_statement(&mut self, condition: &Expression, body: &[Statement], else_body: Option<&[Statement]>) {
    // if
    let conditional_register = match condition {
      Expression::Identifier(conditional_ident) => Left(self.register_for(conditional_ident)),
      other => {
        let temp_dest = Right(self.temps.take_temp());
        self.compile_expression(temp_dest.clone(), condition);
        temp_dest
      }
    };
    let if_false_jump_offset = self.bytecode.len();
    self.push(bytecode::jmp_if_false(WildDestination::Register(conditional_register), InstructionPointer::empty_place_holder()));
    // then
    for statement in body.iter() {
      self.compile_statement(statement);
    }
    if let Some(else_body) = else_body {
      // else
      let jump_over_else_offset = self.bytecode.len();
      self.push(bytecode::jmp(InstructionPointer::empty_place_holder())); // jump to end_ip
      let else_body_ip = self.next_instruction_pointer();
      for statement in else_body.iter() {
        self.compile_statement(statement);
      }
      let end_ip = self.next_instruction_pointer();
      self.bytecode[if_false_jump_offset] |= else_body_ip.as_both_operands();
      self.bytecode[jump_over_else_offset] |= end_ip.as_both_operands();
      // end
    } else {
      let end_ip = self.next_instruction_pointer();
      self.bytecode[if_false_jump_offset] |= end_ip.as_both_operands();
      // end
    }
  }
}

// Expressions
impl FunctionCompiler {
  pub fn compile_expression<D: Into<RawOrTemp>>(&mut self, dest: D, expression: &Expression) {
    let dest: RawOrTemp = dest.into();
    use Expression::*;
    match expression {
      Identifier(ident) => {
        let dest = WildDestination::Register(dest);
        let src = WildSource::Register(self.register_for(ident));
        self.push(bytecode::mov(dest, src));
      }
      Integer(int) => {
        let dest = WildDestination::Register(dest);
        let src = self.immediate_or_constant_for(int);
        self.push(bytecode::mov(dest, src));
      }
      Float(flt) => {
        let dest = WildDestination::Register(dest);
        let src: WildSource<RawRegister> = self.constant_for_float(flt).into();
        self.push(bytecode::mov(dest, src));
      }
      String(string) => {
        let dest = WildDestination::Register(dest);
        let src: WildSource<RawRegister> = self.constant_for_string(string).into();
        self.push(bytecode::mov(dest, src));
      }
      Boolean(b) => {
        let dest = WildDestination::Register(dest);
        let src: WildSource<RawRegister> = self.constant_for_boolean(b).into();
        self.push(bytecode::mov(dest, src));
      }
      Unary { op, right } => todo!(),
      Binary { left, op, right } => { // TODO: rewrite this arm
        use bytecode::opcode::{arithmetic, comparison};
        let arithmetic_subcode = match op {
          BinaryOperator::Add => Some(arithmetic::Subcode::Add),
          BinaryOperator::Sub => Some(arithmetic::Subcode::Sub),
          BinaryOperator::Mul => Some(arithmetic::Subcode::Mul),
          BinaryOperator::Div => Some(arithmetic::Subcode::Div),
          BinaryOperator::Rem => Some(arithmetic::Subcode::Rem),
          BinaryOperator::Pow => Some(arithmetic::Subcode::Pow),
          _ => None,
        };
        let comparison_subcode = match op {
          BinaryOperator::Eq => Some(comparison::Subcode::Eq),
          BinaryOperator::Ne => Some(comparison::Subcode::Ne),
          BinaryOperator::Lt => Some(comparison::Subcode::Lt),
          BinaryOperator::Gt => Some(comparison::Subcode::Gt),
          BinaryOperator::Le => Some(comparison::Subcode::Le),
          BinaryOperator::Ge => Some(comparison::Subcode::Ge),
          _ => None,
        };
        if self.needs_wildcard(left) {
          let first = self.compile_into_wildcard(left);
          let second = self.compile_into_register(right);
          match (arithmetic_subcode, comparison_subcode) {
            (Some(subcode), None) => self.push(bytecode::math_wr(subcode, dest, first, second)),
            (None, Some(subcode)) => self.push(bytecode::cmp_wr(subcode, dest, first, second)),
            _ => unreachable!(),
          }
        } else {
          let first = self.compile_into_register(left);
          let second = self.compile_into_wildcard(right);
          match (arithmetic_subcode, comparison_subcode) {
            (Some(subcode), None) => self.push(bytecode::math_rw(subcode, dest, first, second)),
            (None, Some(subcode)) => self.push(bytecode::cmp_rw(subcode, dest, first, second)),
            _ => unreachable!(),
          }
        }
      }
      FunctionCall { left, args } if matches!(**left, Expression::Identifier(_)) => {
        use crate::bytecode::opcode::misc::call_subcode::ArgCount;
        let arg_temps = self.temps.take_temp_range(args.len());
        for (arg_temp, arg_expression) in arg_temps.iter().zip(args.iter()) {
          self.compile_expression(Right(arg_temp.clone()), arg_expression);
        }
        let arg_start = match arg_temps.iter().next() {
          Some(arg) => Right(arg.clone()),
          None => Left(RawRegister(0)),
        };
        let function_name = match **left {
          Expression::Identifier(ref ident) => ident.clone(),
          _ => unreachable!(),
        };
        let fn_key = self.function_key_for(function_name);
        self.push(bytecode::call(ArgCount(args.len() as u8), dest, fn_key, arg_start));
      }
      FunctionCall { left, args } => todo!(),
      Index { left, index } => {
        let source_register = self.compile_into_wildcard(left);
        let index_wildcard = self.compile_into_wildcard(index);
        let index_on = bytecode::opcode::index::On::Source;
        self.push(bytecode::index(index_on, dest, source_register, index_wildcard));
      },
    }
  }
}

impl FunctionCompiler {
  pub fn needs_wildcard(&self, expression: &Expression) -> bool {
    use Expression::*;
    match expression {
      Identifier(_) => false,
      Integer(_) => true,
      Float(_) => true,
      String(_) => true,
      Boolean(_) => true,
      other => false,
    }
  }
  pub fn compile_into_register(&mut self, expression: &Expression) -> RawOrTemp {
    use Expression::*;
    match expression {
      Identifier(ident) => Left(self.register_for(ident)),
      Integer(int) => {
        let temp = Right(self.temps.take_temp());
        let source = self.immediate_or_constant_for(int);
        self.push(bytecode::mov(temp.clone().into(), source));
        temp
      }
      Float(flt) => {
        let temp = Right(self.temps.take_temp());
        let source: WildSource<RawRegister> = self.constant_for_float(flt).into();
        self.push(bytecode::mov(temp.clone().into(), source));
        temp
      }
      String(s) => {
        let temp = Right(self.temps.take_temp());
        let source: WildSource<RawRegister> = self.constant_for_string(s).into();
        self.push(bytecode::mov(temp.clone().into(), source));
        temp
      }
      Boolean(b) => {
        let temp = Right(self.temps.take_temp());
        let source: WildSource<RawRegister> = self.constant_for_boolean(b).into();
        self.push(bytecode::mov(temp.clone().into(), source));
        temp
      }
      other => {
        let temp = Right(self.temps.take_temp());
        self.compile_expression(temp.clone(), expression);
        temp
      }
    }
  }
  pub fn compile_into_wildcard(&mut self, expression: &Expression) -> WildSource<RawOrTemp> {
    use Expression::*;
    match expression {
      Identifier(ident) => WildSource::Register(Left(self.register_for(ident))),
      Integer(int) => self.immediate_or_constant_for(int),
      Float(flt) => self.constant_for_float(flt).into(),
      String(s) => self.constant_for_string(s).into(),
      Boolean(b) => self.constant_for_boolean(b).into(),
      other => {
        let temp = Right(self.temps.take_temp());
        self.compile_expression(temp.clone(), expression);
        WildSource::Register(temp)
      }
    }
  }
}
