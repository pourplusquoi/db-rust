#![allow(unused_imports)]
#![allow(dead_code)]

use crate::vtype::value::Value;

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