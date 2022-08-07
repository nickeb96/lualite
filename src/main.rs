#![allow(unused)]

mod parser;
mod ast;
mod compiler;
mod bytecode;
mod runtime;


use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::env;
use std::path::Path;
use std::fs::File;
use std::io::{self, prelude::*};
use crate::ast::Identifier;
use crate::runtime::{VirtualMachine, Value, RuntimeError};


#[derive(Debug)]
enum InterpreterError {
  NoFilesGiven,
  FileCouldNotBeOpened,
  Io(io::Error),
  Runtime(RuntimeError),
  Unknown,
}

impl From<io::Error> for InterpreterError {
  fn from(io_error: io::Error) -> Self {
    Self::Io(io_error)
  }
}

impl From<RuntimeError> for InterpreterError {
  fn from(runtime_error: RuntimeError) -> Self {
    Self::Runtime(runtime_error)
  }
}

fn get_args_content() -> Result<String, InterpreterError> {
  let mut ret = String::new();
  let args_iter = env::args().skip(1);
  if args_iter.len() == 0 {
    return Err(InterpreterError::NoFilesGiven);
  }
  for arg in args_iter {
    let path = Path::new(&arg);
    let mut file = File::open(path)?;
    file.read_to_string(&mut ret)?;
    ret.push('\n');
  }
  Ok(ret)
}

fn main() -> Result<(), InterpreterError> {
  let source_code = get_args_content()?;
  let parse_result = parser::parse_file(&source_code);
  let declarations = match parse_result {
    Ok((_, declarations)) => declarations,
    Err(error) => {
      println!("parse error: {error:?}");
      return Err(InterpreterError::Unknown);
    }
  };
  let functions = compiler::compile_declarations(declarations.iter());

  /*
  // debug output
  for (name, procedure) in functions.iter() {
    println!("{name:?}:");
    println!("{procedure}");
  }
  */

  let mut vm = VirtualMachine::with_functions(functions);
  let output = vm.run("main", [])?;
  println!("lualite result: {output}");
  Ok(())
}

