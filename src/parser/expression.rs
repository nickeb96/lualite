//! Expression parsers
//!
//! The primary parser to use in this module is [`expression`], which will parse any
//! expression.  It's used in the examples below.  The other parsers in this module
//! will only parse a specific type of expression.  For example [`function_call`] will
//! only parse an expressions where a function call is the root of the expression tree.
//! # Examples:
//! ```rust
//! let source_code = "x + 1.0 * y";
//! # use lualite::parser::expression::expression;
//! let (_remaining, parsed_ast) = expression(source_code).expect("parse error");
//!
//! # use lualite::ast::{Expression, Identifier, BinaryOperator, FloatLiteral};
//! let expected_ast = Expression::Binary {
//!   left: Box::new(Expression::Identifier(Identifier("x".to_owned()))),
//!   op: BinaryOperator::Add,
//!   right: Box::new(Expression::Binary {
//!     left: Box::new(Expression::Float(FloatLiteral(1.0_f64))),
//!     op: BinaryOperator::Mul,
//!     right: Box::new(Expression::Identifier(Identifier("y".to_owned()))),
//!   })
//! };
//!
//! assert_eq!(parsed_ast, expected_ast);
//! ```
//! Order of operations can be changed with parentheses:
//! ```rust
//! let source_code = "(x + 1.0) * y";
//! # use lualite::parser::expression::expression;
//! let (_remaining, parsed_ast) = expression(source_code).expect("parse error");
//!
//! # use lualite::ast::{Expression, Identifier, BinaryOperator, FloatLiteral};
//! let expected_ast = Expression::Binary {
//!   left: Box::new(Expression::Binary {
//!     left: Box::new(Expression::Identifier(Identifier("x".to_owned()))),
//!     op: BinaryOperator::Add,
//!     right: Box::new(Expression::Float(FloatLiteral(1.0_f64))),
//!   }),
//!   op: BinaryOperator::Mul,
//!   right: Box::new(Expression::Identifier(Identifier("y".to_owned()))),
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
  sequence::{delimited, pair, tuple},
  multi::{many0, separated_list0},
};
use crate::ast::{Expression, BooleanLiteral};
use super::atomic::{identifier, integer, float, string, keyword};
use super::{operator, whitespace};

/// Any possible expression with arbitrary nesting
///
/// Used like so:
/// ```rust
/// # use lualite::parser::expression::expression;
/// let (_, ast) = expression("f(x + 1, true)").expect("parse error");
/// let (_, ast) = expression("array[i - 1] * 2").expect("parse error");
/// let (_, ast) = expression("(a / 2) + (b / 3)").expect("parse error");
/// let (_, ast) = expression("a < b and b < c").expect("parse error");
/// ```
pub fn expression(s: &str) -> IResult<&str, Expression> {
  comparison_expression(s)
}

/// Expressions with the highest precedence
///
/// Consists of literals, identifiers, and parenthesized expressions.
pub fn leaf_expression(s: &str) -> IResult<&str, Expression> {
  alt((
    map(identifier, |ident| Expression::Identifier(ident)),
    map(float, |flt| Expression::Float(flt)),
    map(integer, |int| Expression::Integer(int)),
    map(string, |s| Expression::String(s)),
    map(keyword("true"), |_| Expression::Boolean(BooleanLiteral(true))),
    map(keyword("false"), |_| Expression::Boolean(BooleanLiteral(false))),
    parenthesized,
  ))(s)
}

/// Post-fix operators (call and index)
pub fn postfix_expression(s: &str) -> IResult<&str, Expression> {
  alt((
    function_call,
    index_expression,
    leaf_expression,
  ))(s)
}

/// Function call expression
pub fn function_call(s: &str) -> IResult<&str, Expression> {
  map(
    tuple((
      leaf_expression,
      space0,
      arg_list,
    )),
    |(function, _, args)| Expression::FunctionCall {
      left: Box::new(function),
      args,
    },
  )(s)
}

/// Parenthesized expression to modify operator precedence order
pub fn parenthesized(s: &str) -> IResult<&str, Expression> {
  delimited(
    tag("("),
    expression,
    tag(")"),
  )(s)
}

/// Argument list for a function call
pub fn arg_list(s: &str) -> IResult<&str, Vec<Expression>> {
  delimited(
    tag("("),
    separated_list0(
      tag(","),
      delimited(whitespace, expression, whitespace),
    ),
    tag(")"),
  )(s)
}

/// Indexed container as an r-value
pub fn index_expression(s: &str) -> IResult<&str, Expression> {
  map(
    tuple((
      leaf_expression,
      space0,
      tag("["),
      expression,
      tag("]"),
    )),
    |(array, _, _, index, _)| Expression::Index {
      left: Box::new(array),
      index: Box::new(index),
    },
  )(s)
}

/// An expression raised to the power of another expression
pub fn power_expression(s: &str) -> IResult<&str, Expression> {
  map(
    pair(postfix_expression, opt(pair(operator::power, leaf_expression))),
    |(base, maybe_exponent)| {
      match maybe_exponent {
        Some((op, exponent)) => Expression::Binary {
          left: Box::new(base),
          op,
          right: Box::new(exponent),
        },
        None => base,
      }
    },
  )(s)
}

/// Unary prefix operator expressions
pub fn unary_expression(s: &str) -> IResult<&str, Expression> {
  map(
    pair(many0(operator::unary), power_expression),
    |(op_stack, last)| {
      let mut expr = last;
      for op in op_stack.into_iter().rev() {
        expr = Expression::Unary {
          op,
          right: Box::new(expr),
        };
      }
      expr
    },
  )(s)
}

/// Multiplicative binary operator expressions (*, /, %)
pub fn multiplicative_expression(s: &str) -> IResult<&str, Expression> {
  map(
    pair(power_expression, many0(pair(operator::multiplicative, unary_expression))),
    |(first, remaining)| {
      let mut expr = first;
      for (op, right) in remaining {
        expr = Expression::Binary {
          left: Box::new(expr),
          op,
          right: Box::new(right),
        };
      }
      expr
    }
  )(s)
}

/// Additive binary operator expressions (+, -)
pub fn additive_expression(s: &str) -> IResult<&str, Expression> {
  map(
    pair(multiplicative_expression, many0(pair(operator::additive, multiplicative_expression))),
    |(first, remaining)| {
      let mut expr = first;
      for (op, right) in remaining {
        expr = Expression::Binary {
          left: Box::new(expr),
          op,
          right: Box::new(right),
        };
      }
      expr
    }
  )(s)
}

/// Comparison expressions (==, !=, <, >=, etc.)
pub fn comparison_expression(s: &str) -> IResult<&str, Expression> {
  map(
    pair(additive_expression, opt(pair(operator::comparison, additive_expression))),
    |(left, maybe_right)| {
      match maybe_right {
        Some((op, right)) => Expression::Binary {
          left: Box::new(left),
          op,
          right: Box::new(right),
        },
        None => left,
      }
    },
  )(s)
}

