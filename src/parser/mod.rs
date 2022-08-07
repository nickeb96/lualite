//! Parses source code text into [AST components](../ast/index.html)

pub mod declaration;
pub mod statement;
pub mod expression;
pub mod operator;
pub mod atomic;

use nom::{
  IResult,
  branch::alt,
  bytes::complete::{tag, take_until},
  combinator::map,
  multi::many0,
  sequence::{preceded, terminated},
};
use crate::ast::Declaration;
use self::declaration::declaration;

/// Parses line comments prefixed with `#`
pub fn comment(s: &str) -> IResult<&str, &str> {
  preceded(tag("#"), take_until("\n"))(s)
}

/// Parses spaces, newlines, and comments allowed between statements and declarations
pub fn whitespace(s: &str) -> IResult<&str, &str> {
  map(many0(alt((tag(" "), tag("\t"), tag("\n"), comment))), |_| "")(s)
}

/// Parses the contents of a file into a list of top-level `Declaration`s
pub fn parse_file(s: &str) -> IResult<&str, Vec<Declaration>> {
  terminated(many0(declaration), whitespace)(s)
}

