
use std::fmt;
use crate::ast::{IntegerLiteral, FloatLiteral, BooleanLiteral, StringLiteral};
use crate::runtime;

/// Constant literals in a function
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ConstantValue {
  Integer(i64),
  Float(f64),
  Boolean(bool),
  String(String),
}

impl From<IntegerLiteral> for ConstantValue {
  fn from(integer: IntegerLiteral) -> Self {
    Self::Integer(integer.0)
  }
}

impl From<FloatLiteral> for ConstantValue {
  fn from(float: FloatLiteral) -> Self {
    Self::Float(float.0)
  }
}

impl From<BooleanLiteral> for ConstantValue {
  fn from(boolean: BooleanLiteral) -> Self {
    Self::Boolean(boolean.0)
  }
}

impl From<StringLiteral> for ConstantValue {
  fn from(string: StringLiteral) -> Self {
    Self::String(string.0)
  }
}

impl fmt::Display for ConstantValue {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      ConstantValue::Integer(integer) => {
        integer.fmt(f)?;
        f.write_str("_i64")
      }
      ConstantValue::Float(float) => {
        float.fmt(f)?;
        f.write_str("_f64")
      }
      ConstantValue::Boolean(boolean) => {
        boolean.fmt(f)
      }
      ConstantValue::String(string) => {
        f.write_str("\"")?;
        string.fmt(f)?;
        f.write_str("\"")
      }
    }
  }
}

impl From<ConstantValue> for runtime::Value {
  fn from(constant_value: ConstantValue) -> runtime::Value {
    match constant_value {
      ConstantValue::Integer(integer) => runtime::Value::from(integer),
      ConstantValue::Float(float) => runtime::Value::from(float),
      ConstantValue::Boolean(boolean) => runtime::Value::from(boolean),
      ConstantValue::String(string) => runtime::Value::from(string),
    }
  }
}

