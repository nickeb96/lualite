//! Top-level declaration parsers
//!
//! Parsers for function declarations and static variable/constant declarations.
//!
//! # Function Definition Example:
//! ```rust
//! let source_code = "\
//! function f(a, b)
//!   return (a + 1) * b
//! end
//! ";
//! # use lualite::parser::declaration::function_decl;
//! let (_remaining, parsed_ast) = function_decl(source_code).expect("parse error");
//!
//! # use lualite::ast::{FunctionDecl, Identifier, Statement, Expression, BinaryOperator, IntegerLiteral};
//! let expected_ast = FunctionDecl {
//!   name: Identifier("f".to_owned()),
//!   params: vec![Identifier("a".to_owned()), Identifier("b".to_owned())],
//!   body: vec![
//!     Statement::ReturnStatement(Some(Expression::Binary {
//!       left: Box::new(Expression::Binary {
//!         left: Box::new(Expression::Identifier(Identifier("a".to_owned()))),
//!         op: BinaryOperator::Add,
//!         right: Box::new(Expression::Integer(IntegerLiteral(1_i64))),
//!       }),
//!       op: BinaryOperator::Mul,
//!       right: Box::new(Expression::Identifier(Identifier("b".to_owned()))),
//!     })),
//!   ],
//! };
//!
//! assert_eq!(parsed_ast, expected_ast);
//! ```
//!
//! # Static Declaration Example:
//! ```rust
//! let source_code = "\
//! static SIZE = 512
//! ";
//! # use lualite::parser::declaration::static_decl;
//! let (_remaining, parsed_ast) = static_decl(source_code).expect("parse error");
//!
//! # use lualite::ast::{StaticDecl, Identifier, Expression, IntegerLiteral};
//! let expected_ast = StaticDecl {
//!   name: Identifier("SIZE".to_owned()),
//!   value: Some(Expression::Integer(IntegerLiteral(512_i64))),
//! };
//!
//! assert_eq!(parsed_ast, expected_ast);
//! ```
//!
//! The `declaration` parser can parse either declaration type and returns a
//! [`Declaration`] which is an enum of
//! either a function declaration or a static declaration.

use nom::{
  IResult,
  branch::alt,
  bytes::complete::tag,
  combinator::map,
  multi::separated_list0,
  sequence::{delimited, tuple},
};
use crate::ast::{Identifier, FunctionDecl, StaticDecl, Declaration};
use super::atomic::{identifier, keyword};
use super::statement::body;
use super::expression::expression;
use super::whitespace;

/// Parses any declaration
pub fn declaration(s: &str) -> IResult<&str, Declaration> {
  alt((
    map(function_decl, |fd| Declaration::Function(fd)),
    map(static_decl, |sd| Declaration::Static(sd)),
  ))(s)
}

/// Parses a function declaration
pub fn function_decl(s: &str) -> IResult<&str, FunctionDecl> {
  map(
    tuple((
      whitespace,
      keyword("function"),
      whitespace,
      identifier,
      whitespace,
      params_list,
      body,
      whitespace,
      keyword("end"),
    )),
    |(_, _function, _, name, _, params, body, _, _end)| FunctionDecl { name, params, body },
  )(s)
}

/// Parses a static declaration
pub fn static_decl(s: &str) -> IResult<&str, StaticDecl> {
  map(
    tuple((
      whitespace,
      keyword("static"),
      whitespace,
      identifier,
      whitespace,
      tag("="),
      whitespace,
      expression,
    )),
    |(_, _static, _, name, _, _equals, _, value)| StaticDecl { name, value: Some(value) },
  )(s)
}

fn params_list(s: &str) -> IResult<&str, Vec<Identifier>> {
  delimited(
    tag("("),
    separated_list0(
      tag(","),
      delimited(whitespace, identifier, whitespace),
    ),
    tag(")"),
  )(s)
}

