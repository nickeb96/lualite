//! Operator Symbols grouped by precedence

use nom::{
  IResult,
  branch::alt,
  bytes::complete::tag,
  character::complete::space0,
  combinator::map,
  sequence::delimited,
};
use crate::ast::{UnaryOperator, BinaryOperator};

pub fn unary(s: &str) -> IResult<&str, UnaryOperator> {
  delimited(
    space0,
    map(tag("-"), |_| UnaryOperator::Neg),
    space0,
  )(s)
}

pub fn power(s: &str) -> IResult<&str, BinaryOperator> {
  delimited(
    space0,
    map(tag("^"), |_| BinaryOperator::Pow),
    space0,
  )(s)
}

pub fn multiplicative(s: &str) -> IResult<&str, BinaryOperator> {
  delimited(
    space0,
    alt((
      map(tag("*"), |_| BinaryOperator::Mul),
      map(tag("/"), |_| BinaryOperator::Div),
      map(tag("%"), |_| BinaryOperator::Rem),
    )),
    space0,
  )(s)
}

pub fn additive(s: &str) -> IResult<&str, BinaryOperator> {
  delimited(
    space0,
    alt((
      map(tag("+"), |_| BinaryOperator::Add),
      map(tag("-"), |_| BinaryOperator::Sub),
    )),
    space0,
  )(s)
}

pub fn comparison(s: &str) -> IResult<&str, BinaryOperator> {
  delimited(
    space0,
    alt((
      map(tag("=="), |_| BinaryOperator::Eq),
      map(tag("!="), |_| BinaryOperator::Ne),
      map(tag("<="), |_| BinaryOperator::Le),
      map(tag(">="), |_| BinaryOperator::Ge),
      map(tag("<"), |_| BinaryOperator::Lt),
      map(tag(">"), |_| BinaryOperator::Gt),
    )),
    space0,
  )(s)
}

