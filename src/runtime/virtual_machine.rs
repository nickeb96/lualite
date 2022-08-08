
use std::collections::HashMap;
use std::rc::Rc;
use crate::bytecode::Instruction;
use crate::bytecode::opcode::{SuperCode, misc, index, comparison, arithmetic};
use crate::bytecode::operand::{
  FromDestination, FromSource,
  RawRegister, Immediate, ConstantKey,
  InstructionPointer, FunctionKey,
  WildDestination, WildSource,
};
use crate::bytecode::constant_value::ConstantValue;
use crate::bytecode::Procedure;
use super::error::RuntimeError;
use super::{Value, InstructionCount, ExecutionStatus};

#[derive(Debug)]
struct StackFrame {
  procedure: Rc<Procedure>,
  pc: usize,
  register_start: usize,
  return_index: usize,
}

/// Bytecode evaluation engine
///
/// Attach compiled functions to a `VirtualMachine` with [`insert_function`] or initialize it
/// with a list of functions using [`with_functions`].
///
/// Begin execution with [`run`].
///
/// [`insert_function`]: Self::insert_function
/// [`with_functions`]: Self::with_functions
/// [`run`]: Self::run
#[derive(Debug)]
pub struct VirtualMachine {
  call_stack: Vec<StackFrame>,
  functions: HashMap<String, Rc<Procedure>>,
  register_stack: Vec<Value>,
}

impl Default for VirtualMachine {
  fn default() -> Self {
    Self {
      call_stack: Vec::default(),
      functions: HashMap::default(),
      register_stack: vec![Value::Nil], // bottom of register stack is VM result
    }
  }
}

impl VirtualMachine {
  pub fn new() -> Self {
    Self::default()
  }

  /// Construct a `VirtualMachine` with an iterator of functions.
  pub fn with_functions<I, S, P>(functions: I) -> Self
  where
    I: IntoIterator<Item=(S, P)>,
    S: Into<String>,
    P: Into<Procedure>,
  {
    let functions = functions.into_iter()
      .map(|(name, proc)| (name.into(), Rc::new(proc.into())))
      .collect();
    Self { functions, ..Self::default() }
  }

  pub fn insert_function<S: Into<String>, P: Into<Procedure>>(&mut self, name: S, procedure: P) {
    self.functions.insert(name.into(), Rc::new(procedure.into()));
  }

  pub fn remove_function<S: AsRef<str>>(&mut self, name: S) -> Option<Rc<Procedure>> {
    self.functions.remove(name.as_ref())
  }

  pub fn get_function<S: AsRef<str>>(&self, name: S) -> Option<Rc<Procedure>> {
    self.functions.get(name.as_ref()).cloned()
  }
}

// Interacting with the current stack frame
impl VirtualMachine {
  fn register(&self, register: RawRegister) -> Result<&Value, RuntimeError> {
    let top = self.call_stack.last().ok_or_else(|| RuntimeError::EmptyCallStack)?;
    self.register_stack.get(top.register_start + register.0 as usize)
      .ok_or_else(|| RuntimeError::InvalidRegister)
  }

  fn register_mut(&mut self, register: RawRegister) -> Result<&mut Value, RuntimeError> {
    let top = self.call_stack.last().ok_or_else(|| RuntimeError::EmptyCallStack)?;
    self.register_stack.get_mut(top.register_start + register.0 as usize)
      .ok_or_else(|| RuntimeError::InvalidRegister)
  }

  fn constant(&self, constant: ConstantKey) -> Result<&ConstantValue, RuntimeError> {
    let top = self.call_stack.last().ok_or_else(|| RuntimeError::EmptyCallStack)?;
    top.procedure.constants.get(constant.0 as usize).ok_or_else(|| RuntimeError::MissingConstant)
  }

  fn set_pc(&mut self, pc: InstructionPointer) -> Result<(), RuntimeError> {
    let top = self.call_stack.last_mut().ok_or_else(|| RuntimeError::EmptyCallStack)?;
    top.pc = pc.0 as usize;
    Ok(())
  }
}

impl VirtualMachine {
  /// Run the virtual machine to completion, starting with the function referred to
  /// by `entry_name`.  If it does not exist in the virtual machine, a [`RuntimeError`]
  /// is returned.  Otherwise, is used to lookup the entry function by name.
  ///
  /// The entry function is called with the values in `args` and its return value is
  /// returned by this function as a [`Value`].
  ///
  /// # Infinite loops:
  ///
  /// Using `run` will continue to execute bytecode until `entry_name` finishes and
  /// returns.  This could potentially cause an unrecoverable infinite loop.  Use
  /// [`initialize_with_values`](Self::initialize_with_values) and
  /// [`execution_loop`](Self::execution_loop) for more fine-grained control over
  /// the number of instructions to allow the VM to execute.
  ///
  /// # Example:
  /// ```rust
  /// # use lualite::{parser, compiler, runtime::{VirtualMachine, Value}};
  /// let source_code = r"
  /// function trisum(a, b, c)
  ///   return a + b + c
  /// end
  /// ";
  /// let (_, declarations) = parser::parse_file(source_code).unwrap();
  /// let functions = compiler::compile_declarations(declarations.iter());
  /// let mut vm = VirtualMachine::with_functions(functions);
  ///
  /// let result = vm.run("trisum", [1.into(), 2.into(), 3.into()]);
  /// assert!(matches!(result, Ok(Value::Integer(6))));
  /// ```
  pub fn run(&mut self, entry_name: &str, args: impl IntoIterator<Item=Value>) -> Result<Value, RuntimeError> {
    let entry_procedure = self.functions.get(entry_name)
      .ok_or_else(|| RuntimeError::MissingFunction)?.clone();
    self.initialize_with_values(entry_procedure, args)?;
    self.execution_loop_infinite()?;
    Ok(self.get_result())
  }

  /// Gets the return value of the entry function.
  ///
  /// This will always be [`Value::Nil`] if the entry function has not finished.
  pub fn get_result(&self) -> Value {
    self.register_stack.get(0).cloned().unwrap_or_else(|| Value::Nil)
  }

  /// Sets the entry procedure of the virtual machine and sets its argument registers
  /// to the [`Value`]s in `args`.
  ///
  /// Does **not** begin execution, unlike [`run`](Self::run).
  pub fn initialize_with_values(&mut self,
    entry_procedure: Rc<Procedure>,
    args: impl IntoIterator<Item=Value>,
  ) -> Result<(), RuntimeError> {
    let frame_base = self.register_stack.len();
    self.register_stack.resize(frame_base + entry_procedure.register_count, Value::Nil);
    for (arg_index, arg_value) in (1..).into_iter().zip(args.into_iter()) {
      let register_slot = self.register_stack.get_mut(frame_base + arg_index)
        .ok_or_else(|| RuntimeError::InvalidRegister)?;
      *register_slot = arg_value;
    }
    self.call_stack.push(StackFrame {
      procedure: entry_procedure,
      register_start: frame_base,
      pc: 0,
      return_index: 0, // bottom of the register stack (index 0) is VM result
    });
    Ok(())
  }

  /// Execute instructions until either `limit` is reached or the entry procedure finishes.
  ///
  /// # Example:
  /// ```rust
  /// # use std::rc::Rc;
  /// # use lualite::{parser, compiler, runtime::{VirtualMachine, Value, InstructionCount, ExecutionStatus}};
  /// let source_code = r"
  /// function forever(n)  # causes an infinite loop when n >= 0
  ///   x = 0
  ///   while x >= 0 do
  ///     x = x + n
  ///   end
  /// end
  /// ";
  /// let (_, fn_decl) = parser::declaration::function_decl(source_code).expect("parse failed");
  /// let procedure = compiler::compile_function(&fn_decl);
  /// 
  /// let mut vm = VirtualMachine::new();
  /// // Initialize a call to: forever(10)
  /// vm.initialize_with_values(Rc::new(procedure), [Value::Integer(10)]);
  ///
  /// // Run for 200 instructions
  /// let status = vm.execution_loop(InstructionCount::Limited(200));
  /// assert!(matches!(status, Ok(ExecutionStatus::Unfinished))); // Did not finish
  ///
  /// // Continue to run for 5000 more instructions
  /// let status = vm.execution_loop(InstructionCount::Limited(5000));
  /// assert!(matches!(status, Ok(ExecutionStatus::Unfinished))); // Will always return Unfinished
  /// ```
  pub fn execution_loop(&mut self, limit: InstructionCount) -> Result<ExecutionStatus, RuntimeError> {
    match limit {
      InstructionCount::Unlimited => self.execution_loop_infinite(),
      InstructionCount::Limited(count) => self.execution_loop_finite(count),
    }
  }

  fn execution_loop_finite(&mut self, count: usize) -> Result<ExecutionStatus, RuntimeError> {
    for _ in 0..count {
      if let Some(top) = self.call_stack.last_mut() {
        top.pc += 1;
        let instruction = *top.procedure.bytecode.get(top.pc).ok_or_else(|| RuntimeError::InvalidPc)?;
        self.execute(instruction)?;
      } else {
        return Ok(ExecutionStatus::Finished);
      }
    }
    Ok(ExecutionStatus::Unfinished)
  }

  fn execution_loop_infinite(&mut self) -> Result<ExecutionStatus, RuntimeError> {
    while let Some(top) = self.call_stack.last_mut() {
      top.pc += 1;
      let instruction = *top.procedure.bytecode.get(top.pc).ok_or_else(|| RuntimeError::InvalidPc)?;
      self.execute(instruction)?;
    }
    Ok(ExecutionStatus::Finished)
  }
}

// Execution instructions
impl VirtualMachine {
  /// Execute a single bytecode instruction
  pub fn execute(&mut self, instruction: Instruction) -> Result<(), RuntimeError> {
    match SuperCode::from(instruction) {
      SuperCode::Misc => self.execute_misc(instruction),
      SuperCode::Index => self.execute_index(instruction),
      SuperCode::Comparison => self.execute_comparison(instruction),
      SuperCode::Arithmetic => self.execute_arithmetic(instruction),
    }
  }

  /// Execute an instruction from the misc category
  ///
  /// The 2 least significant bits have to match `bytecode::opcode::SuperCode::Misc`.
  fn execute_misc(&mut self, instruction: Instruction) -> Result<(), RuntimeError> {
    use misc::Subcode;
    match Subcode::from(instruction) {
      Subcode::Jump => self.execute_misc_jump(instruction)?,
      Subcode::Move => self.execute_misc_move(instruction)?,
      Subcode::Call => self.execute_misc_call(instruction)?,
      Subcode::Interrupt => unimplemented!(),
    }
    Ok(())
  }

  /// Execute a jump instruction from the misc category
  ///
  /// The 2 least significant bits have to match `bytecode::opcode::SuperCode::Misc` and the next 2 bits
  /// have to match `bytecode::opcode::misc::Subcode::Jump`.
  fn execute_misc_jump(&mut self, instruction: Instruction) -> Result<(), RuntimeError> {
    use misc::jump_subcode::{Reason, ConditionType};
    match Reason::from(instruction) {
      Reason::Special => {
        use misc::jump_subcode::Special;
        match Special::from(instruction) {
          Special::NoOp => (),
          Special::Return => {
            let stack_frame = self.call_stack.pop().ok_or_else(|| RuntimeError::EmptyCallStack)?;
            self.register_stack[stack_frame.return_index]
              = self.register_stack[stack_frame.register_start].clone();
            self.register_stack.resize(stack_frame.register_start, Value::Nil);
          }
          Special::Xa => unimplemented!(),
          Special::Xb => unimplemented!(),
        }
      }
      Reason::Always => self.set_pc(InstructionPointer::from_both_operands(instruction))?,
      reason @ (Reason::IfFalse | Reason::IfTrue) => {
        let flag = match ConditionType::from(instruction) {
          ConditionType::Register => self.register(RawRegister::from_destination(instruction))?.clone(),
          ConditionType::Global => todo!(),
        };
        if (matches!(reason, Reason::IfFalse) && flag == Value::Boolean(false)) ||
           (matches!(reason, Reason::IfTrue)  && flag == Value::Boolean(true)) {
          self.set_pc(InstructionPointer::from_both_operands(instruction))?;
        }
      }
    }
    Ok(())
  }

  /// Execute a move instruction from the misc category
  ///
  /// The 2 least significant bits have to match `bytecode::opcode::SuperCode::Misc` and the next 2 bits
  /// have to match `bytecode::opcode::misc::Subcode::Move`.
  fn execute_misc_move(&mut self, instruction: Instruction) -> Result<(), RuntimeError> {
    use misc::move_subcode::{DestinationType, SourceType};
    let source = match SourceType::from(instruction) {
      SourceType::Register => self.register(RawRegister::from_first(instruction))?.clone(),
      SourceType::Global => todo!(),
      SourceType::Immediate => Value::Integer(Immediate::from_first(instruction).0 as i64),
      SourceType::Constant => Value::from(self.constant(ConstantKey::from_first(instruction))?.clone()),
    };
    match DestinationType::from(instruction) {
      DestinationType::Register => *self.register_mut(RawRegister::from_destination(instruction))? = source,
      DestinationType::Global => todo!(),
    }
    Ok(())
  }

  /// Execute a call instruction from the misc category
  ///
  /// The 2 least significant bits have to match `bytecode::opcode::SuperCode::Misc` and the next 2 bits
  /// have to match `bytecode::opcode::misc::Subcode::Call`.
  fn execute_misc_call(&mut self, instruction: Instruction) -> Result<(), RuntimeError> {
    use misc::call_subcode::ArgCount;
    let arg_count = ArgCount::from(instruction);
    let return_register = RawRegister::from_destination(instruction);
    let function_key = FunctionKey::from_first(instruction);
    let arg_start = RawRegister::from_second(instruction);
    let arg_iter = (arg_start.0..(arg_start.0 + arg_count.0)).into_iter().map(|reg_num| RawRegister(reg_num));
    let stack_frame = self.call_stack.last().ok_or_else(|| RuntimeError::EmptyCallStack)?;
    let procedure_name = stack_frame.procedure.functions.get(function_key.0 as usize)
      .ok_or_else(|| RuntimeError::MissingFunction)?;
    let procedure = Rc::clone(self.functions.get(procedure_name).ok_or_else(|| RuntimeError::MissingFunction)?);
    let frame_base = self.register_stack.len();
    self.register_stack.resize(frame_base + procedure.register_count, Value::Nil);
    for (arg_index, arg_register) in (1..).into_iter().zip(arg_iter.into_iter()) {
      let arg_value = self.register(arg_register)?.clone();
      let register_slot = self.register_stack.get_mut(frame_base + arg_index)
        .ok_or_else(|| RuntimeError::InvalidRegister)?;
      *register_slot = arg_value;
    }
    self.call_stack.push(StackFrame {
      procedure,
      register_start: frame_base,
      pc: 0,
      return_index: stack_frame.register_start + return_register.0 as usize,
    });
    Ok(())
  }

  /// Execute an instruction from the index category
  ///
  /// The 2 least significant bits have to match `bytecode::opcode::SuperCode::Index`.
  fn execute_index(&mut self, instruction: Instruction) -> Result<(), RuntimeError> {
    use index::On;
    let decoded = index::decode(instruction);
    let destination_register = match decoded.destination {
      WildDestination::Register(register) => register,
      WildDestination::Global(_global) => todo!(),
    };
    let source_value = match decoded.source {
      WildSource::Register(register) => self.register(register)?.clone(),
      WildSource::Global(_global) => todo!(),
      WildSource::Immediate(immediate) => Value::Integer(immediate.0 as i64),
      WildSource::Constant(constant) => Value::from(self.constant(constant)?.clone()),
    };
    let index_value = match decoded.index {
      WildSource::Register(register) => self.register(register)?.clone(),
      WildSource::Global(_global) => todo!(),
      WildSource::Immediate(immediate) => Value::Integer(immediate.0 as i64),
      WildSource::Constant(constant) => Value::from(self.constant(constant)?.clone()),
    };
    match decoded.index_on {
      On::Source => {
        *self.register_mut(destination_register)? = source_value.get(index_value);
      }
      On::Destination => {
        let destination = self.register_mut(destination_register)?;
        destination.set(index_value, source_value);
      }
    }
    Ok(())
  }

  /// Execute an instruction from the comparison category
  ///
  /// The 2 least significant bits have to match `bytecode::opcode::SuperCode::Comparison`.
  fn execute_comparison(&mut self, instruction: Instruction) -> Result<(), RuntimeError> {
    use comparison::{Subcode, Sources};
    let decoded = comparison::decode(instruction);
    let (first, second) = match decoded.sources {
      Sources::FirstIsWild(first, second) => {
        let first = match first {
          WildSource::Register(register) => self.register(register)?.clone(),
          WildSource::Global(_global) => todo!(),
          WildSource::Immediate(immediate) => Value::Integer(immediate.0 as i64),
          WildSource::Constant(constant) => Value::from(self.constant(constant)?.clone()),
        };
        let second = self.register(second)?.clone();
        (first, second)
      }
      Sources::SecondIsWild(first, second) => {
        let second = match second {
          WildSource::Register(register) => self.register(register)?.clone(),
          WildSource::Global(_global) => todo!(),
          WildSource::Immediate(immediate) => Value::Integer(immediate.0 as i64),
          WildSource::Constant(constant) => Value::from(self.constant(constant)?.clone()),
        };
        let first = self.register(first)?.clone();
        (first, second)
      }
    };
    *self.register_mut(decoded.destination)? = match decoded.subcode {
      Subcode::Eq => Value::from(first == second),
      Subcode::Ne => Value::from(first != second),
      Subcode::Lt => Value::from(first < second),
      Subcode::Gt => Value::from(first > second),
      Subcode::Le => Value::from(first <= second),
      Subcode::Ge => Value::from(first >= second),
      Subcode::Xa => unimplemented!(),
      Subcode::Xb => unimplemented!(),
    };
    Ok(())
  }

  /// Execute an instruction from the arithmetic category
  ///
  /// The 2 least significant bits have to match `bytecode::opcode::SuperCode::Arithmetic`.
  fn execute_arithmetic(&mut self, instruction: Instruction) -> Result<(), RuntimeError> {
    use arithmetic::{Subcode, Sources};
    let decoded = arithmetic::decode(instruction);
    let (first, second) = match decoded.sources {
      Sources::FirstIsWild(first, second) => {
        let first = match first {
          WildSource::Register(register) => self.register(register)?.clone(),
          WildSource::Global(_global) => todo!(),
          WildSource::Immediate(immediate) => Value::Integer(immediate.0 as i64),
          WildSource::Constant(constant) => Value::from(self.constant(constant)?.clone()),
        };
        let second = self.register(second)?.clone();
        (first, second)
      }
      Sources::SecondIsWild(first, second) => {
        let second = match second {
          WildSource::Register(register) => self.register(register)?.clone(),
          WildSource::Global(_global) => todo!(),
          WildSource::Immediate(immediate) => Value::Integer(immediate.0 as i64),
          WildSource::Constant(constant) => Value::from(self.constant(constant)?.clone()),
        };
        let first = self.register(first)?.clone();
        (first, second)
      }
    };
    *self.register_mut(decoded.destination)? = match decoded.subcode {
      Subcode::Add => first + second,
      Subcode::Sub => first - second,
      Subcode::Mul => first * second,
      Subcode::Div => first / second,
      Subcode::Rem => first % second,
      Subcode::Pow => unimplemented!(),
      Subcode::Rot => unimplemented!(),
      Subcode::Log => unimplemented!(),
    };
    Ok(())
  }
}

