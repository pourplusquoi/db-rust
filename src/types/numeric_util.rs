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
    if T::primitive_from(U::primitive_from(val.clone())) != val {
        Err(Error::new(ErrorKind::Overflow, "Cast failure"))
    } else {
        Ok(U::primitive_from(val))
    }
}

pub fn loss_cast<T>(val: f64) -> Result<T, Error>
where
    T: PrimitiveFrom<f64> + HasLimits,
    f64: PrimitiveFrom<T>,
{
    if val > f64::primitive_from(T::max()) || val < f64::primitive_from(T::min()) {
        Err(Error::new(ErrorKind::Overflow, "Cast failure"))
    } else {
        Ok(T::primitive_from(val))
    }
}

pub fn parse<T, U>(val: T) -> Result<U, Error>
where
    T: ParseInto<U>,
{
    val.parse_into()
}

pub trait PrimitiveFrom<T> {
    fn primitive_from(val: T) -> Self;
}

pub trait ParseInto<T> {
    fn parse_into(self) -> Result<T, Error>;
}

pub trait HasLimits {
    fn min() -> Self;
    fn max() -> Self;
}

impl ParseInto<bool> for &str {
    fn parse_into(self) -> Result<bool, Error> {
        if self == "true" || self == "1" || self == "t" {
            Ok(true)
        } else if self == "false" || self == "0" || self == "f" {
            Ok(false)
        } else {
            Err(Error::new(ErrorKind::CannotParse, "Parse boolean failure"))
        }
    }
}

limits!(i8, std::i8::MIN, std::i8::MAX);
limits!(i16, std::i16::MIN, std::i16::MAX);
limits!(i32, std::i32::MIN, std::i32::MAX);
limits!(i64, std::i64::MIN, std::i64::MAX);
limits!(u64, std::u64::MIN, std::u64::MAX);
limits!(f64, std::f64::MIN, std::f64::MAX);

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
