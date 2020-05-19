#![allow(unused_imports)]
#![allow(dead_code)]

use crate::data::value::Value;

pub enum ValueType {
  Boolean(i8),
  TinyInt(i8),
  SmallInt(i16),
  Integer(i32),
  BigInt(i64),
  Decimal(f64),
  Timestamp(u64),
  Varchar(Varlen),
}

pub enum Varlen {
  Owned(String),
  Borrowed(&'static str),
}

pub enum CmpBool {
  CmpTrue,
  CmpFalse,
  CmpNull,
}

impl Varlen {
  pub fn len(&self) -> usize {
    match self {
      &Varlen::Owned(ref s) => s.len(),
      &Varlen::Borrowed(ref s) => s.len(),
    }
  }
}

impl ValueType {
  pub fn data(&self) -> Option<&[u8]> {
    match self {
      Self::Varchar(varlen) => match varlen {
        Varlen::Owned(s) => Some(s.as_bytes()),
        Varlen::Borrowed(s) => Some(s.as_bytes()),
      },
      _ => None,
    }
  }

  pub fn data_mut(&mut self) -> Option<&mut [u8]> {
    match self {
      Self::Varchar(varlen) => match varlen {
        Varlen::Owned(s) => unsafe {
          Some(s.as_bytes_mut())
        },
        Varlen::Borrowed(_) => None,
      },
      _ => None,
    }
  }
}