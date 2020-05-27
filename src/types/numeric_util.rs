use crate::types::error::Error;
use crate::types::error::ErrorKind;
use std::cmp::PartialEq;
use std::cmp::PartialOrd;
use std::result::Result;

pub fn cast<T, U>(val: T) -> Result<U, Error>
where
    T: PartialEq + PrimitiveFrom<U>,
    U: PartialEq + PrimitiveFrom<T>,
{
    if T::from(&U::from(&val)) != val {
        Err(Error::new(ErrorKind::Overflow, "Cast failure"))
    } else {
        Ok(U::from(&val))
    }
}

pub fn loss_cast<T, U>(val: T) -> Result<U, Error>
where
    U: PrimitiveFrom<T> + HasLimits,
    T: PrimitiveFrom<U> + FloatNum,
{
    if val > T::from(&U::max()) || val < T::from(&U::min()) {
        Err(Error::new(ErrorKind::Overflow, "Cast failure"))
    } else {
        Ok(U::from(&val))
    }
}

pub fn parse<T, U>(val: T) -> Result<U, Error>
where
    T: ParseInto<U>,
{
    val.into()
}

pub fn add<T>(lhs: T, rhs: T) -> Result<T, Error>
where
    T: Arithmetic,
{
    let sum = lhs.add(&rhs);
    let zero = T::zero();
    if (lhs < zero && rhs < zero && sum > zero) || (lhs > zero && rhs > zero && sum < zero) {
        Err(Error::new(
            ErrorKind::Overflow,
            "Numeric value out of range",
        ))
    } else {
        Ok(sum)
    }
}

pub fn subtract<T>(lhs: T, rhs: T) -> Result<T, Error>
where
    T: Arithmetic,
{
    let diff = lhs.subtract(&rhs);
    let zero = T::zero();
    if (lhs > zero && rhs < zero && diff < zero) || (lhs < zero && rhs > zero && diff > zero) {
        Err(Error::new(
            ErrorKind::Overflow,
            "Numeric value out of range",
        ))
    } else {
        Ok(diff)
    }
}

pub fn multiply<T>(lhs: T, rhs: T) -> Result<T, Error>
where
    T: Arithmetic,
{
    let prod = lhs.multiply(&rhs);
    let zero = T::zero();
    if rhs != zero && prod.divide(&rhs) != lhs {
        Err(Error::new(
            ErrorKind::Overflow,
            "Numeric value out of range",
        ))
    } else {
        Ok(prod)
    }
}

pub fn divide<T>(lhs: T, rhs: T) -> Result<T, Error>
where
    T: Arithmetic,
{
    let zero = T::zero();
    if rhs == zero {
        Err(Error::new(ErrorKind::DivideByZero, "Division by zero"))
    } else {
        Ok(lhs.divide(&rhs))
    }
}

pub fn modulo<T>(lhs: T, rhs: T) -> Result<T, Error>
where
    T: Arithmetic,
{
    let zero = T::zero();
    if rhs == zero {
        Err(Error::new(ErrorKind::DivideByZero, "Division by zero"))
    } else {
        Ok(lhs.modulo(&rhs))
    }
}

pub trait PrimitiveFrom<T> {
    fn from(val: &T) -> Self;
}

pub trait ParseInto<T> {
    fn into(self) -> Result<T, Error>;
}

pub trait HasLimits {
    fn min() -> Self;
    fn max() -> Self;
}

pub trait FloatNum: PartialOrd {}

pub trait Arithmetic: PartialEq + PartialOrd {
    fn add(&self, other: &Self) -> Self;
    fn subtract(&self, other: &Self) -> Self;
    fn multiply(&self, other: &Self) -> Self;
    fn divide(&self, other: &Self) -> Self;
    fn modulo(&self, other: &Self) -> Self;
    fn zero() -> Self;
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

impl FloatNum for f64 {}

arithmetic_impl!(i8);
arithmetic_impl!(i16);
arithmetic_impl!(i32);
arithmetic_impl!(i64);
arithmetic_impl!(u64);
arithmetic_impl!(f64);

limits_impl!(i8, std::i8::MIN, std::i8::MAX);
limits_impl!(i16, std::i16::MIN, std::i16::MAX);
limits_impl!(i32, std::i32::MIN, std::i32::MAX);
limits_impl!(i64, std::i64::MIN, std::i64::MAX);
limits_impl!(u64, std::u64::MIN, std::u64::MAX);
limits_impl!(f64, std::f64::MIN, std::f64::MAX);

parse_into_impl!(i8);
parse_into_impl!(i16);
parse_into_impl!(i32);
parse_into_impl!(i64);
parse_into_impl!(u64);
parse_into_impl!(f64);

primitive_from_impl!(i8, i8);
primitive_from_impl!(i8, i16);
primitive_from_impl!(i8, i32);
primitive_from_impl!(i8, i64);
primitive_from_impl!(i8, f64);

primitive_from_impl!(i16, i8);
primitive_from_impl!(i16, i16);
primitive_from_impl!(i16, i32);
primitive_from_impl!(i16, i64);
primitive_from_impl!(i16, f64);

primitive_from_impl!(i32, i8);
primitive_from_impl!(i32, i16);
primitive_from_impl!(i32, i32);
primitive_from_impl!(i32, i64);
primitive_from_impl!(i32, f64);

primitive_from_impl!(i64, i8);
primitive_from_impl!(i64, i16);
primitive_from_impl!(i64, i32);
primitive_from_impl!(i64, i64);
primitive_from_impl!(i64, f64);

primitive_from_impl!(f64, i8);
primitive_from_impl!(f64, i16);
primitive_from_impl!(f64, i32);
primitive_from_impl!(f64, i64);
primitive_from_impl!(f64, f64);
