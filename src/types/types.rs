#![allow(unused_imports)]
#![allow(dead_code)]

use crate::types::limits::*;
use crate::types::value::Value;
use std::clone::Clone;

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
    Owned(Str<String>),
    Borrowed(Str<&'a str>),
}

#[derive(Clone)]
pub enum Str<T: Clone> {
    Val(T),
    MaxVal,
}

impl Str<String> {
    pub fn len(&self) -> usize {
        match self {
            Str::Val(s) => s.len(),
            Str::MaxVal => 0,
        }
    }

    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Str::Val(s) => Some(s.as_bytes()),
            Str::MaxVal => None,
        }
    }

    pub fn as_bytes_mut(&mut self) -> Option<&mut [u8]> {
        match self {
            Str::Val(s) => unsafe { Some(s.as_bytes_mut()) },
            Str::MaxVal => None,
        }
    }
}

impl Str<&str> {
    pub fn len(&self) -> usize {
        match self {
            Str::Val(s) => s.len(),
            Str::MaxVal => 0,
        }
    }

    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Str::Val(s) => Some(s.as_bytes()),
            Str::MaxVal => None,
        }
    }

    pub fn as_bytes_mut(&mut self) -> Option<&mut [u8]> {
        None
    }
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
                Varlen::Owned(s) => s.as_bytes(),
                Varlen::Borrowed(s) => s.as_bytes(),
            },
            _ => None,
        }
    }

    pub fn data_mut(&mut self) -> Option<&mut [u8]> {
        match self {
            Self::Varchar(varlen) => match varlen {
                Varlen::Owned(s) => s.as_bytes_mut(),
                Varlen::Borrowed(_) => None,
            },
            _ => None,
        }
    }

    pub fn is_coercable_from(&self, other: &Self) -> bool {
        match self {
            Self::Boolean(_) => true,
            Self::TinyInt(_)
            | Self::SmallInt(_)
            | Self::Integer(_)
            | Self::BigInt(_)
            | Self::Decimal(_) => match other {
                Self::TinyInt(_)
                | Self::SmallInt(_)
                | Self::Integer(_)
                | Self::BigInt(_)
                | Self::Decimal(_)
                | Self::Varchar(_) => true,
                _ => false,
            },
            Self::Timestamp(_) => match other {
                Self::Timestamp(_) | Self::Varchar(_) => true,
                _ => false,
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
        match self {
            Self::Boolean(_) => "BOOLEAN",
            Self::TinyInt(_) => "TINYINT",
            Self::SmallInt(_) => "SMALLINT",
            Self::Integer(_) => "INTEGER",
            Self::BigInt(_) => "BIGINT",
            Self::Decimal(_) => "DECIMAL",
            Self::Timestamp(_) => "TIMESTAMP",
            Self::Varchar(_) => "VARCHAR",
        }
        .to_string()
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

    pub fn owned(s: String) -> Self {
        Self::Varchar(Varlen::Owned(Str::Val(s)))
    }

    pub fn borrowed(s: &'a str) -> Self {
        Self::Varchar(Varlen::Borrowed(Str::Val(s)))
    }

    pub fn min_owned() -> Self {
        Self::Varchar(Varlen::Owned(Str::Val("".to_string())))
    }

    pub fn max_owned() -> Self {
        Self::Varchar(Varlen::Owned(Str::MaxVal))
    }

    pub fn min_borrowed() -> Self {
        Self::Varchar(Varlen::Borrowed(Str::Val("")))
    }

    pub fn max_borrowed() -> Self {
        Self::Varchar(Varlen::Borrowed(Str::MaxVal))
    }

    pub fn min_val(self) -> Self {
        match self {
            Self::Boolean(_) => Self::Boolean(0),
            Self::TinyInt(_) => Self::TinyInt(PELOTON_INT8_MIN),
            Self::SmallInt(_) => Self::SmallInt(PELOTON_INT16_MIN),
            Self::Integer(_) => Self::Integer(PELOTON_INT32_MIN),
            Self::BigInt(_) => Self::BigInt(PELOTON_INT64_MIN),
            Self::Decimal(_) => Self::Decimal(PELOTON_DECIMAL_MIN),
            Self::Timestamp(_) => Self::Timestamp(0),
            Self::Varchar(vc) => match vc {
                Varlen::Owned(_) => Self::min_owned(),
                Varlen::Borrowed(_) => Self::min_borrowed(),
            },
        }
    }

    pub fn max_val(self) -> Self {
        match self {
            Self::Boolean(_) => Self::Boolean(1),
            Self::TinyInt(_) => Self::TinyInt(PELOTON_INT8_MAX),
            Self::SmallInt(_) => Self::SmallInt(PELOTON_INT16_MAX),
            Self::Integer(_) => Self::Integer(PELOTON_INT32_MAX),
            Self::BigInt(_) => Self::BigInt(PELOTON_INT64_MAX),
            Self::Decimal(_) => Self::Decimal(PELOTON_DECIMAL_MAX),
            Self::Timestamp(_) => Self::Timestamp(PELOTON_TIMESTAMP_MAX),
            Self::Varchar(vc) => match vc {
                Varlen::Owned(_) => Self::max_owned(),
                Varlen::Borrowed(_) => Self::max_borrowed(),
            },
        }
    }
}

pub trait Operation: Sized {
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
