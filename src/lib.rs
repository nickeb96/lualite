#![allow(unused)]
//! A basic scripting language with a Rust runtime

#[warn(unused)]
pub mod parser;
#[warn(unused)]
pub mod ast;
// TOOD: add unused lints
pub mod compiler;
#[warn(unused)]
pub mod bytecode;
#[warn(unused)]
pub mod runtime;

