
use std::ops::{Add, Sub, Mul, Div, Rem};
use std::cmp::Ordering;
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;

//pub static NIL: Value = Value::Nil;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
  Nil,
  Boolean(bool),
  Integer(i64),
  Float(f64),
  ShortStr {
    buf: [u8; 14],
    len: u8,
  },
  LongStr(Rc<String>),
  Array(Rc<RefCell<Vec<Value>>>),
}

impl Default for Value {
  fn default() -> Self {
    Self::Nil
  }
}

impl From<bool> for Value {
  fn from(b: bool) -> Self {
    Self::Boolean(b)
  }
}

impl From<i64> for Value {
  fn from(int: i64) -> Self {
    Self::Integer(int)
  }
}

impl From<i32> for Value {
  fn from(int: i32) -> Self {
    Self::Integer(int.into())
  }
}

impl From<i16> for Value {
  fn from(int: i16) -> Self {
    Self::Integer(int.into())
  }
}

impl From<i8> for Value {
  fn from(int: i8) -> Self {
    Self::Integer(int.into())
  }
}

impl TryFrom<u64> for Value {
  type Error = std::num::TryFromIntError;
  fn try_from(int: u64) -> Result<Self, Self::Error> {
    Ok(Self::Integer(int.try_into()?))
  }
}

impl From<u32> for Value {
  fn from(int: u32) -> Self {
    Self::Integer(int.into())
  }
}

impl From<u16> for Value {
  fn from(int: u16) -> Self {
    Self::Integer(int.into())
  }
}

impl From<u8> for Value {
  fn from(int: u8) -> Self {
    Self::Integer(int.into())
  }
}

impl TryFrom<isize> for Value {
  type Error = std::num::TryFromIntError;
  fn try_from(int: isize) -> Result<Self, Self::Error> {
    Ok(Self::Integer(int.try_into()?))
  }
}

impl TryFrom<usize> for Value {
  type Error = std::num::TryFromIntError;
  fn try_from(int: usize) -> Result<Self, Self::Error> {
    Ok(Self::Integer(int.try_into()?))
  }
}

impl From<f64> for Value {
  fn from(flt: f64) -> Self {
    Self::Float(flt)
  }
}

impl From<f32> for Value {
  fn from(flt: f32) -> Self {
    Self::Float(flt.into())
  }
}

impl From<&str> for Value {
  fn from(string: &str) -> Self {
    if string.len() <= 14 {
      let mut buf = [0; 14];
      buf[..string.len()].copy_from_slice(string.as_bytes());
      Value::ShortStr {
        buf,
        len: string.len() as u8,
      }
    } else {
      Self::LongStr(Rc::new(string.to_owned()))
    }
  }
}

impl From<String> for Value {
  fn from(string: String) -> Self {
    if string.len() <= 14 {
      let mut buf = [0; 14];
      buf[..string.len()].copy_from_slice(string.as_bytes());
      Value::ShortStr {
        buf,
        len: string.len() as u8,
      }
    } else {
      Self::LongStr(Rc::new(string))
    }
  }
}

impl<V> FromIterator<V> for Value
  where V: Into<Value>
{
  fn from_iter<I>(iter: I) -> Value
    where I: IntoIterator<Item=V>
  {
    let inner_vec: Vec<Value> = iter.into_iter().map(|item| item.into()).collect();
    Value::Array(Rc::new(RefCell::new(inner_vec)))
  }
}

impl fmt::Display for Value {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Value::Nil => write!(f, "nil"),
      Value::Integer(int) => write!(f, "{int}"),
      Value::Float(float) => write!(f, "{float}"),
      Value::ShortStr { buf, len } => write!(f, "\"{}\"", std::str::from_utf8(&buf[..*len as usize]).unwrap()),
      Value::LongStr(string) => write!(f, "\"{string}\""),
      Value::Array(array) => {
        let array = array.borrow();
        write!(f, "[")?;
        let mut iter = array.iter();
        if let Some(first) = iter.next() {
          write!(f, "{first}")?;
        }
        for element in iter {
          write!(f, ", {element}")?;
        }
        write!(f, "]")?;
        Ok(())
      }
      Value::Boolean(true) => write!(f, "true"),
      Value::Boolean(false) => write!(f, "false"),
    }
  }
}

impl PartialOrd for Value {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    match (self, other) {
      (Value::Integer(left), Value::Integer(right)) => left.partial_cmp(&right),
      _ => None,
    }
  }
}

impl Add for Value {
  type Output = Value;
  fn add(self, other: Self) -> Self::Output {
    match (self, other) {
      (Value::Integer(left), Value::Integer(right)) => Value::Integer(left + right),
      (Value::Float(left), Value::Float(right)) => Value::Float(left + right),
      _ => Value::Nil,
    }
  }
}

impl Sub for Value {
  type Output = Value;
  fn sub(self, other: Self) -> Self::Output {
    match (self, other) {
      (Value::Integer(left), Value::Integer(right)) => Value::Integer(left - right),
      (Value::Float(left), Value::Float(right)) => Value::Float(left - right),
      _ => Value::Nil,
    }
  }
}

impl Mul for Value {
  type Output = Value;
  fn mul(self, other: Self) -> Self::Output {
    match (self, other) {
      (Value::Integer(left), Value::Integer(right)) => Value::Integer(left * right),
      (Value::Float(left), Value::Float(right)) => Value::Float(left * right),
      _ => Value::Nil,
    }
  }
}

impl Div for Value {
  type Output = Value;
  fn div(self, other: Self) -> Self::Output {
    match (self, other) {
      (Value::Integer(left), Value::Integer(right)) => Value::Integer(left / right),
      (Value::Float(left), Value::Float(right)) => Value::Float(left / right),
      _ => Value::Nil,
    }
  }
}

impl Rem for Value {
  type Output = Value;
  fn rem(self, other: Self) -> Self::Output {
    match (self, other) {
      (Value::Integer(left), Value::Integer(right)) => Value::Integer(left % right),
      _ => Value::Nil,
    }
  }
}

impl Value {
  pub fn get(&self, key: Value) -> Value {
    match (self, key) {
      (Value::Array(array), Value::Integer(num)) => {
        let index: usize = match num.try_into() {
          Ok(as_usize) => as_usize,
          _ => return Value::Nil,
        };
        let array = array.borrow();
        array.get(index).unwrap_or(&Value::Nil).clone()
      }
      _ => todo!(),
    }
  }

  pub fn set(&mut self, key: Value, value: Value) {
    match (self, key) {
      (Value::Array(array), Value::Integer(num)) => {
        let index: usize = match num.try_into() {
          Ok(as_usize) => as_usize,
          _ => return,
        };
        let mut array = array.borrow_mut();
        if let Some(element) = array.get_mut(index) {
          *element = value;
        } else if index == array.len() {
          array.push(value);
        }
      }
      _ => todo!(),
    }
  }
}

