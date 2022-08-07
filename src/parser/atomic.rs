//! Building blocks for expressions, statements, and declarations

use nom::{
  IResult,
  branch::alt,
  bytes::complete::{tag, take_until, take_while1},
  character::{self, complete::satisfy},
  combinator::{map, not},
  number,
  sequence::delimited,
};
use crate::ast::{Identifier, IntegerLiteral, FloatLiteral, StringLiteral};

/// Parser builder for making keyword parsers
///
/// `keyword` is not a parser on its own.  Instead it can be called to create and
/// return parsers that can be used to match keywords.
/// # Example:
/// ```rust
/// # use lualite::parser::atomic::keyword;
/// let function_kw_parser = keyword("function");
/// let while_kw_parser = keyword("while");
///
/// assert!(function_kw_parser("function f(x) ...").is_ok()); // succeeds, "function" is matched
/// assert!(while_kw_parser("function f(x) ...").is_err()); // fails, "while" not matched
/// assert!(function_kw_parser("x + 1").is_err()); // fails, "function" not matched
/// ```
/// # Lifetime:
/// **Note**: The lifetime of the resulting parsers are bound by the `'kw` lifetime of the
/// input string.
/// ```rust compile_fail
/// # use lualite::parser::atomic::keyword;
/// let test_parser = {
///   let test_kw: String = "test".to_owned(); // test_kw is dropped at the end of this block
///   keyword(&test_kw) // Borrow check error here: the returned parser would outlive `test_kw`
/// };
/// test_parser("test");
/// ```
pub fn keyword<'kw>(kw: &'kw str) -> impl Fn(&str) -> IResult<&str, &str> + 'kw {
  move |s| {
    let (remaining, matched) = word(s)?;
    if matched == kw {
      Ok((remaining, matched))
    } else {
      Err(nom::Err::Error(nom::error::Error { input: s, code: nom::error::ErrorKind::Tag }))
    }
  }
}

/// Matches any keyword
///
/// # Example:
/// ```rust
/// # use lualite::parser::atomic::any_keyword;
/// assert!(any_keyword("if").is_ok()); // succeeds
/// assert!(any_keyword("hello").is_err()); // fails, "hello" is not a keyword
/// assert!(any_keyword("1234").is_err()); // fails, 1234 is not a word
/// ```
pub fn any_keyword(s: &str) -> IResult<&str, &str> {
  alt((
    keyword("end"), keyword("function"), keyword("return"), keyword("if"), keyword("then"),
    keyword("elseif"), keyword("else"), keyword("while"), keyword("do"), keyword("for"),
    keyword("in"), keyword("nil"), keyword("and"), keyword("or"), keyword("not"), 
    keyword("true"), keyword("false")
  ))(s)
}

/// Parses a string consisting of only letters, numbers, and underscores
///
/// First character cannot be a digit.
pub fn word(s: &str) -> IResult<&str, &str> {
  satisfy(|c: char| c.is_alphabetic() || c == '_')(s)?;
  take_while1(|c: char| c.is_alphanumeric() || c == '_')(s)
}

/// Parser for identifiers
///
/// Ensures the first character is not a digit and the identifier is not a keyword.
/// # Example:
/// ```rust
/// # use lualite::parser::atomic::identifier;
/// assert!(identifier("x").is_ok()); // succeeds
/// assert!(identifier("foo_bar3").is_ok()); // succeeds
/// assert!(identifier("_unused_var").is_ok()); // succeeds
/// assert!(identifier("while").is_err()); // fails (while is a keyword)
/// assert!(identifier("7hello").is_err()); // fails (treated as integer literal)
/// ```
pub fn identifier(s: &str) -> IResult<&str, Identifier> {
  not(any_keyword)(s)?;
  map(word, |ident: &str| Identifier(ident.to_owned()))(s)
}

/// Parser for signed 64-bit integer literals
///
/// # Example:
/// ```rust
/// # use lualite::parser::atomic::integer;
/// use lualite::ast::IntegerLiteral;
///
/// assert_eq!(integer("537"), Ok(("", IntegerLiteral(537_i64))));
/// assert!(integer("abcd").is_err());
/// assert_eq!(integer("-11"), Ok(("", IntegerLiteral(-11_i64))));
/// ```
pub fn integer(s: &str) -> IResult<&str, IntegerLiteral> {
  map(character::complete::i64, |int| IntegerLiteral(int))(s)
}

/// Parser for double-precision floating-point literals
///
/// Input must contain a `.` for this parser to succeed.  Doing so prevents it from
/// matching integers.
pub fn float(s: &str) -> IResult<&str, FloatLiteral> {
  let (_remaining, flt_str) = number::complete::recognize_float(s)?;
  // without this check, float would match integers as well
  if flt_str.contains('.') {
    map(number::complete::double, |flt| FloatLiteral(flt))(s)
  } else {
    Err(nom::Err::Error(nom::error::Error { input: s, code: nom::error::ErrorKind::Float }))
  }
}

/// Parser for string literals
pub fn string(s: &str) -> IResult<&str, StringLiteral> {
  map(delimited(tag("\""), take_until("\""), tag("\"")), |string: &str| StringLiteral(string.to_owned()))(s)
}

