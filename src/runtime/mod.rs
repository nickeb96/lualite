//! Bytecode runtime
//!
//! Create a [`VirtualMachine`] and attach compiled functions to it.  Begin execution
//! with [`run`], or use [`initialize_with_values`] and [`execution_loop`] for more fine grained
//! control.
//!
//! [`run`]: VirtualMachine::run
//! [`initialize_with_values`]: VirtualMachine::initialize_with_values
//! [`execution_loop`]: VirtualMachine::execution_loop
//!
//! # Example:
//! ```rust
//! # use std::rc::Rc;
//! # use lualite::{parser, compiler, runtime::{VirtualMachine, Value, InstructionCount}};
//! let trisum_source_code = r"function trisum(a, b, c) return a + b + c end";
//! let (_, trisum_fn_decl) = parser::declaration::function_decl(trisum_source_code).unwrap();
//! let trisum_procedure = compiler::compile_function(&trisum_fn_decl);
//!
//! let mut vm = VirtualMachine::new();
//!
//! vm.initialize_with_values(Rc::new(trisum_procedure), [10.into(), 20.into(), 30.into()]);
//! vm.execution_loop(InstructionCount::Unlimited);
//!
//! assert_eq!(vm.get_result(), Value::Integer(60));
//! ```

mod virtual_machine;
mod value;
mod error;

pub use value::Value;
pub use virtual_machine::VirtualMachine;
pub use error::RuntimeError;

#[derive(Debug)]
pub enum InstructionCount {
  Limited(usize),
  Unlimited,
}

#[derive(Debug)]
pub enum ExecutionStatus {
  Finished,
  Unfinished,
}
