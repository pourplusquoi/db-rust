use crate::types::error::Error;
use crate::types::error::ErrorKind;
use crate::types::limits::*;
use std::clone::Clone;
use std::result::Result;

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
            Str::MaxVal => 0 as usize,
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

    pub fn owned() -> Self {
        Self::Varchar(Varlen::Owned(Str::MaxVal))
    }

    pub fn borrowed() -> Self {
        Self::Varchar(Varlen::Borrowed(Str::MaxVal))
    }

    pub fn min_val(mut self) -> Self {
        match &mut self {
            Self::Boolean(val) => *val = 0,
            Self::TinyInt(val) => *val = RSDB_INT8_MIN,
            Self::SmallInt(val) => *val = RSDB_INT16_MIN,
            Self::Integer(val) => *val = RSDB_INT32_MIN,
            Self::BigInt(val) => *val = RSDB_INT64_MIN,
            Self::Decimal(val) => *val = RSDB_DECIMAL_MIN,
            Self::Timestamp(val) => *val = 0,
            Self::Varchar(vc) => match vc {
                Varlen::Owned(val) => *val = Str::Val("".to_string()),
                Varlen::Borrowed(val) => *val = Str::Val(""),
            },
        }
        self
    }

    pub fn max_val(mut self) -> Self {
        match &mut self {
            Self::Boolean(val) => *val = 1,
            Self::TinyInt(val) => *val = RSDB_INT8_MAX,
            Self::SmallInt(val) => *val = RSDB_INT16_MAX,
            Self::Integer(val) => *val = RSDB_INT32_MAX,
            Self::BigInt(val) => *val = RSDB_INT64_MAX,
            Self::Decimal(val) => *val = RSDB_DECIMAL_MAX,
            Self::Timestamp(val) => *val = RSDB_TIMESTAMP_MAX,
            Self::Varchar(vc) => match vc {
                Varlen::Owned(val) => *val = Str::MaxVal,
                Varlen::Borrowed(val) => *val = Str::MaxVal,
            },
        }
        self
    }

    pub fn null_val(mut self) -> Result<Self, Error> {
        match &mut self {
            Self::Boolean(val) => *val = RSDB_BOOLEAN_NULL,
            Self::TinyInt(val) => *val = RSDB_INT8_NULL,
            Self::SmallInt(val) => *val = RSDB_INT16_NULL,
            Self::Integer(val) => *val = RSDB_INT32_NULL,
            Self::BigInt(val) => *val = RSDB_INT64_NULL,
            Self::Decimal(val) => *val = RSDB_DECIMAL_NULL,
            Self::Timestamp(val) => *val = RSDB_TIMESTAMP_NULL,
            _ => Err(Error::new(
                ErrorKind::NotSupported,
                "Invalid type for `null_val`",
            ))?,
        };
        Ok(self)
    }

    pub fn to_varlen(&self) -> Result<Varlen, Error> {
        let varlen = match self {
            Self::Boolean(val) => Varlen::Owned(Str::Val(val.to_string())),
            Self::TinyInt(val) => Varlen::Owned(Str::Val(val.to_string())),
            Self::SmallInt(val) => Varlen::Owned(Str::Val(val.to_string())),
            Self::Integer(val) => Varlen::Owned(Str::Val(val.to_string())),
            Self::BigInt(val) => Varlen::Owned(Str::Val(val.to_string())),
            Self::Decimal(val) => Varlen::Owned(Str::Val(val.to_string())),
            Self::Timestamp(val) => Varlen::Owned(Str::Val(val.to_string())),
            _ => Err(unsupported!("Type error for to_varlen"))?,
        };
        Ok(varlen)
    }

    pub fn get_as_bool(&self) -> Result<i8, Error> {
        let res = match self {
            Self::Boolean(val) => *val as i8,
            _ => Err(unsupported!("Invalid type for `get_as_bool`"))?,
        };
        Ok(res)
    }

    pub fn get_as_i8(&self) -> Result<i8, Error> {
        let res = match self {
            Self::TinyInt(val) => *val as i8,
            _ => Err(unsupported!("Invalid type for `get_as_i8`"))?,
        };
        Ok(res)
    }

    pub fn get_as_i16(&self) -> Result<i16, Error> {
        let res = match self {
            Self::TinyInt(val) => *val as i16,
            Self::SmallInt(val) => *val as i16,
            _ => Err(unsupported!("Invalid type for `get_as_i16`"))?,
        };
        Ok(res)
    }

    pub fn get_as_i32(&self) -> Result<i32, Error> {
        let res = match self {
            Self::TinyInt(val) => *val as i32,
            Self::SmallInt(val) => *val as i32,
            Self::Integer(val) => *val as i32,
            _ => Err(unsupported!("Invalid type for `get_as_i32`"))?,
        };
        Ok(res)
    }

    pub fn get_as_i64(&self) -> Result<i64, Error> {
        let res = match self {
            Self::TinyInt(val) => *val as i64,
            Self::SmallInt(val) => *val as i64,
            Self::Integer(val) => *val as i64,
            Self::BigInt(val) => *val as i64,
            _ => Err(unsupported!("Invalid type for `get_as_i64`"))?,
        };
        Ok(res)
    }

    pub fn get_as_u64(&self) -> Result<u64, Error> {
        let res = match self {
            Self::Timestamp(val) => *val as u64,
            _ => Err(unsupported!("Invalid type for `get_as_u64`"))?,
        };
        Ok(res)
    }

    pub fn get_as_f64(&self) -> Result<f64, Error> {
        let res = match self {
            Self::Decimal(val) => *val as f64,
            _ => Err(unsupported!("Invalid type for `get_as_f64`"))?,
        };
        Ok(res)
    }
}

pub trait Operation: Sized {
    fn eq(&self, other: &Self) -> Option<bool>;
    fn ne(&self, other: &Self) -> Option<bool>;
    fn lt(&self, other: &Self) -> Option<bool>;
    fn le(&self, other: &Self) -> Option<bool>;
    fn gt(&self, other: &Self) -> Option<bool>;
    fn ge(&self, other: &Self) -> Option<bool>;
    fn add(&self, other: &Self) -> Result<Self, Error>;
    fn subtract(&self, other: &Self) -> Result<Self, Error>;
    fn multiply(&self, other: &Self) -> Result<Self, Error>;
    fn divide(&self, other: &Self) -> Result<Self, Error>;
    fn modulo(&self, other: &Self) -> Result<Self, Error>;
    fn sqrt(&self) -> Result<Self, Error>;
    fn min(&self, other: &Self) -> Result<Self, Error>;
    fn max(&self, other: &Self) -> Result<Self, Error>;
    fn null(&self, other: &Self) -> Result<Self, Error>;
    fn is_zero(&self) -> bool;
    fn is_inlined(&self) -> bool;
    fn to_string(&self) -> String;
    fn serialize_to(&self, dst: &mut [u8]);
    fn deserialize_from(&mut self, src: &[u8]);
    fn cast_to(&self, dst: &mut Self) -> bool;
}
