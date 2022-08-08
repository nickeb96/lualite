//! A basic scripting language with a Rust runtime

pub mod parser;
pub mod ast;
#[allow(unused)] // TOOD: remove after compiler refactor
pub mod compiler;
pub mod bytecode;
pub mod runtime;

