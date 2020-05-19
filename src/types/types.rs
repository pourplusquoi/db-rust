#![allow(unused_imports)]
#![allow(dead_code)]

use crate::types::value::Value;

#[derive(Clone)]
pub enum Types<'a> {
  Boolean(i8),
  TinyInt(i8),
  SmallInt(i16),
  Integer(i32),
  BigInt(i64),
  Decimal(f64),
  Timestamp(u64),
  Varchar(Varlen<'a>),
}

#[derive(Clone)]
pub enum Varlen<'a> {
  Owned(String),
  Borrowed(&'a str),
}

impl<'a> Varlen<'a> {
  pub fn len(&self) -> usize {
    match self {
      Varlen::Owned(s) => s.len(),
      Varlen::Borrowed(s) => s.len(),
    }
  }
}

impl<'a> Types<'a> {
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

  pub fn is_coercable_from(&self, other: &Self) -> bool {
    match self {
      Self::Boolean(_) => true,
      Self::TinyInt(_) | Self::SmallInt(_) |
      Self::Integer(_) | Self::BigInt(_) |
      Self::Decimal(_) => match other {
        Self::TinyInt(_) | Self::SmallInt(_) |
        Self::Integer(_) | Self::BigInt(_) |
        Self::Decimal(_) | Self::Varchar(_) => true,
        _ => false,
      },
      Self::Timestamp(_) => {
        match other {
          Self::Timestamp(_) | Self::Varchar(_) => true,
          _ => false,
        }
      },
      Self::Varchar(_) => true,
    }
  }

  pub fn type_size(&self) -> usize {
    match self {
      Self::Boolean(_) => 1,
      Self::TinyInt(_) => 1,
      Self::SmallInt(_) => 2,
      Self::Integer(_) => 4,
      Self::BigInt(_) => 8,
      Self::Decimal(_) => 8,
      Self::Timestamp(_) => 8,
      Self::Varchar(_) => 0,
    }
  }

  pub fn type_id(&self) -> String {
    String::from(match self {
      Self::Boolean(_) => "BOOLEAN",
      Self::TinyInt(_) => "TINYINT",
      Self::SmallInt(_) => "SMALLINT",
      Self::Integer(_) => "INTEGER",
      Self::BigInt(_) => "BIGINT",
      Self::Decimal(_) => "DECIMAL",
      Self::Timestamp(_) => "TIMESTAMP",
      Self::Varchar(_) => "VARCHAR",
    })
  }

  pub fn boolean() -> Self {
    Self::Boolean(0)
  }

  pub fn tinyint() -> Self {
    Self::TinyInt(0)
  }

  pub fn smallint() -> Self {
    Self::SmallInt(0)
  }

  pub fn integer() -> Self {
    Self::Integer(0)
  }

  pub fn bigint() -> Self {
    Self::BigInt(0)
  }

  pub fn decimal() -> Self {
    Self::Decimal(0.0)
  }

  pub fn timestamp() -> Self {
    Self::Timestamp(0)
  }

  // pub fn varchar_owned() -> Self {
  //   Self::Varchar(Varlen::Owned(String::from("")))
  // }

  // pub fn min_val(Self) -> Self {}

  // pub fn max_val(Self) -> Self {}
}

pub trait Operation : Sized {
  fn eq(&self, other: &Self) -> Option<bool>;
  fn ne(&self, other: &Self) -> Option<bool>;
  fn lt(&self, other: &Self) -> Option<bool>;
  fn le(&self, other: &Self) -> Option<bool>;
  fn gt(&self, other: &Self) -> Option<bool>;
  fn ge(&self, other: &Self) -> Option<bool>;
  fn add(&self, other: &Self) -> Option<Self>;
  fn subtract(&self, other: &Self) -> Option<Self>;
  fn multiply(&self, other: &Self) -> Option<Self>;
  fn divide(&self, other: &Self) -> Option<Self>;
  fn modulo(&self, other: &Self) -> Option<Self>;
  fn min(&self, other: &Self) -> Option<Self>;
  fn max(&self, other: &Self) -> Option<Self>;
  fn sqrt(&self, other: &Self) -> Option<Self>;
  fn null(&self, other: &Self) -> Option<Self>;
  fn is_zero(&self) -> bool;
  fn is_inlined(&self) -> bool;
  fn to_string(&self) -> String;
  fn serialize_to(&self, dst: &mut [u8]);
  fn deserialize_from(&mut self, src: &[u8]);
  fn cast(&self, dst: &mut Self) -> bool;
}