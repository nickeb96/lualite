
pub mod function;
pub mod temporary;

use std::fmt::Write;
use std::borrow::Borrow;
use crate::ast::{FunctionDecl, Declaration, Statement};
use crate::bytecode::Procedure;
use function::FunctionCompiler;

pub fn compile_function(function: &FunctionDecl) -> Procedure {
  let mut fc = FunctionCompiler::with_parameters(&function.params);
  for statement in function.body.iter() {
    fc.compile_statement(statement);
  }
  // add implicit return if it doesn't already exist
  match function.body.last() {
    Some(Statement::ReturnStatement(_)) => (),
    _ => fc.compile_statement(&Statement::ReturnStatement(None)),
  }
  fc.finish()
}

pub fn compile_declarations<I, D>(declarations: I) -> Vec<(String, Procedure)>
where
  I: IntoIterator<Item=D>,
  D: Borrow<Declaration>,
{
  let mut functions = Vec::new();
  for declaration in declarations.into_iter() {
    match declaration.borrow() {
      Declaration::Function(fn_decl) => {
        let name = fn_decl.name.0.clone();
        let procedure = compile_function(&fn_decl);
        functions.push((name, procedure));
      }
      _ => (),
    }
  }
  functions
}
