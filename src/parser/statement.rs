//! Statement Parsers
//!
//! Parsers for each of the different statement types.
//!
//! # Example:
//! ```rust
//! let source_code = "
//! while y > 5 do
//!   f(y)
//!   y = y - g(x, y)
//! end
//! ";
//! # use lualite::parser::statement::statement;
//! let (_remaining, parsed_ast) = statement(source_code).expect("parse error");
//!
//! # use lualite::ast::{Statement, Expression, Identifier, BinaryOperator, IntegerLiteral};
//! let expected_ast = Statement::WhileStatement {
//!   condition: Expression::Binary {
//!     left: Box::new(Expression::Identifier(Identifier("y".to_owned()))),
//!     op: BinaryOperator::Gt,
//!     right: Box::new(Expression::Integer(IntegerLiteral(5_i64))),
//!   },
//!   body: vec![
//!     Statement::SingleStatement(Expression::FunctionCall {
//!       left: Box::new(Expression::Identifier(Identifier("f".to_owned()))),
//!       args: vec![Expression::Identifier(Identifier("y".to_owned()))],
//!     }),
//!     Statement::AssignStatement(Identifier("y".to_owned()), Expression::Binary {
//!       left: Box::new(Expression::Identifier(Identifier("y".to_owned()))),
//!       op: BinaryOperator::Sub,
//!       right: Box::new(Expression::FunctionCall {
//!         left: Box::new(Expression::Identifier(Identifier("g".to_owned()))),
//!         args: vec![
//!           Expression::Identifier(Identifier("x".to_owned())),
//!           Expression::Identifier(Identifier("y".to_owned())),
//!         ],
//!       }),
//!     })
//!   ],
//! };
//!
//! assert_eq!(parsed_ast, expected_ast);
//! ```

use nom::{
  IResult,
  branch::alt,
  bytes::complete::tag,
  character::complete::space0,
  combinator::{map, opt},
  multi::many0,
  sequence::{delimited, preceded, tuple},
};
use crate::ast::{Statement, Expression};
use super::atomic::{identifier, keyword};
use super::expression::expression;
use super::whitespace;

/// Body of a loop, function, etc.
pub fn body(s: &str) -> IResult<&str, Vec<Statement>> {
  many0(statement)(s)
}

/// Any statement
pub fn statement(s: &str) -> IResult<&str, Statement> {
  preceded(
    whitespace,
    alt((
      if_statement,
      while_statement,
      return_statement,
      index_assign_statement,
      assign_statement,
      single_statement,
    )),
  )(s)
}

/// An expression treated as a single statement
pub fn single_statement(s: &str) -> IResult<&str, Statement> {
  map(expression, |expr| Statement::SingleStatement(expr))(s)
}

/// Assignment from an expression to an identifier
pub fn assign_statement(s: &str) -> IResult<&str, Statement> {
  map(
    tuple((
      identifier,
      delimited(space0, tag("="), space0),
      expression,
    )),
    |(lhs, _equals, rhs)| {
      Statement::AssignStatement(lhs, rhs)
    },
  )(s)
}

/// Assignment from an expression into an indexed container
pub fn index_assign_statement(s: &str) -> IResult<&str, Statement> {
  map(
    tuple((
      identifier,
      delimited(tag("["), expression, tag("]")),
      delimited(space0, tag("="), space0),
      expression,
    )),
    |(table, index, _equals, value)| {
      Statement::IndexAssignStatement { table: Expression::Identifier(table), index, value }
    },
  )(s)
}

/// Return statement with optional return expression
pub fn return_statement(s: &str) -> IResult<&str, Statement> {
  map(
    tuple((
      keyword("return"),
      opt(preceded(space0, expression)),
    )),
    |(_return, maybe_expression)| {
      Statement::ReturnStatement(maybe_expression)
    },
  )(s)
}

/// While loop
pub fn while_statement(s: &str) -> IResult<&str, Statement> {
  map(
    tuple((
      keyword("while"),
      delimited(
        space0,
        expression,
        space0,
      ),
      keyword("do"),
      body,
      preceded(whitespace, keyword("end")),
    )),
    |(_while, condition, _do, body, _end)| {
      Statement::WhileStatement { condition, body }
    }
  )(s)
}

/// If statement with optional else clause and elseif clauses
pub fn if_statement(s: &str) -> IResult<&str, Statement> {
  map(
    tuple((
      keyword("if"),
      delimited(
        space0,
        expression,
        space0,
      ),
      keyword("then"),
      body,
      alt((
        else_if_clause,
        else_clause,
        map(preceded(whitespace, keyword("end")), |_end| None),
      )),
    )),
    |(_if, condition, _then, body, else_body)| {
      Statement::IfStatement { condition, body, else_body }
    }
  )(s)
}

fn else_if_clause(s: &str) -> IResult<&str, Option<Vec<Statement>>> {
  map(
    tuple((
      preceded(whitespace, keyword("elseif")),
      delimited(
        space0,
        expression,
        space0,
      ),
      keyword("then"),
      body,
      alt((
        else_if_clause,
        else_clause,
        map(preceded(whitespace, keyword("end")), |_end| None),
      )),
    )),
    |(_elseif, condition, _then, body, else_body)| {
      Some(vec![Statement::IfStatement { condition, body, else_body }])
    },
  )(s)
}

fn else_clause(s: &str) -> IResult<&str, Option<Vec<Statement>>> {
  map(
    tuple((
      preceded(whitespace, keyword("else")),
      body,
      preceded(whitespace, keyword("end")),
    )),
    |(_else, body, _end)| Some(body),
  )(s)
}

