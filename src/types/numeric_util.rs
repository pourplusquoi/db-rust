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

pub trait PrimitiveFrom<T> {
    fn from(val: T) -> Self;
}

primitive_from!(i8, i16);
primitive_from!(i16, i8);

primitive_from!(i8, i32);
primitive_from!(i32, i8);

primitive_from!(i8, i64);
primitive_from!(i64, i8);

primitive_from!(i16, i32);
primitive_from!(i32, i16);

primitive_from!(i16, i64);
primitive_from!(i64, i16);

primitive_from!(i32, i64);
primitive_from!(i64, i32);
