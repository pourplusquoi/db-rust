use crate::types::error::Error;
use crate::types::error::ErrorKind;
use std::clone::Clone;
use std::cmp::PartialEq;
use std::result::Result;

pub fn cast<T, U>(val: T) -> Result<U, Error>
where
    T: Clone + PartialEq + PrimitiveFrom<U>,
    U: Clone + PartialEq + PrimitiveFrom<T>,
{
    if T::from(U::from(val.clone())) != val {
        Err(Error::new(ErrorKind::Overflow, "Cast failure"))
    } else {
        Ok(U::from(val))
    }
}

pub fn force_cast<T, U>(val: T) -> U
where
    U: PrimitiveFrom<T>,
{
    U::from(val)
}

pub fn parse<T, U>(val: T) -> Result<U, Error>
where
    T: ParseInto<U>,
{
    val.into()
}

pub trait PrimitiveFrom<T> {
    fn from(val: T) -> Self;
}

pub trait ParseInto<T> {
    fn into(self) -> Result<T, Error>;
}

impl ParseInto<bool> for &str {
    fn into(self) -> Result<bool, Error> {
        if self == "true" || self == "1" || self == "t" {
            Ok(true)
        } else if self == "false" || self == "0" || self == "f" {
            Ok(false)
        } else {
            Err(Error::new(ErrorKind::CannotParse, "Parse boolean failure"))
        }
    }
}

parse_into!(i8);
parse_into!(i16);
parse_into!(i32);
parse_into!(i64);
parse_into!(u64);
parse_into!(f64);

primitive_from!(i8, i8);
primitive_from!(i8, i16);
primitive_from!(i8, i32);
primitive_from!(i8, i64);
primitive_from!(i8, f64);

primitive_from!(i16, i8);
primitive_from!(i16, i16);
primitive_from!(i16, i32);
primitive_from!(i16, i64);
primitive_from!(i16, f64);

primitive_from!(i32, i8);
primitive_from!(i32, i16);
primitive_from!(i32, i32);
primitive_from!(i32, i64);
primitive_from!(i32, f64);

primitive_from!(i64, i8);
primitive_from!(i64, i16);
primitive_from!(i64, i32);
primitive_from!(i64, i64);
primitive_from!(i64, f64);

primitive_from!(f64, i8);
primitive_from!(f64, i16);
primitive_from!(f64, i32);
primitive_from!(f64, i64);
primitive_from!(f64, f64);
