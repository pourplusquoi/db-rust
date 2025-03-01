use crate::common::reinterpret;
use crate::logging::error_logging::ErrorLogging;
use crate::types::error::Error;
use crate::types::error::ErrorKind;
use crate::types::limits::*;
use crate::types::numeric_util::*;
use crate::types::types::Operation;
use crate::types::types::Str;
use crate::types::types::Types;
use crate::types::types::Varlen;
use crate::types::varlen_util::*;
use std::cmp::PartialEq;
use std::fmt::Debug;
use std::result::Result;

#[derive(Clone, Debug)]
pub struct Value<'a> {
    content: Types<'a>,
    size: usize,
}

impl<'a> Value<'a> {
    pub fn new(content: Types<'a>) -> Self {
        Value {
            size: get_size(&content),
            content: content,
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn borrow(&self) -> &'a Types {
        &self.content
    }

    pub fn borrow_mut(&mut self) -> &'a mut Types {
        &mut self.content
    }

    pub fn is_null(&self) -> bool {
        self.size == RSDB_VALUE_NULL as usize
    }

    pub fn is_numeric(&self) -> bool {
        match self.content {
            Types::TinyInt(_)
            | Types::SmallInt(_)
            | Types::Integer(_)
            | Types::BigInt(_)
            | Types::Decimal(_) => true,
            _ => false,
        }
    }

    pub fn is_integer(&self) -> bool {
        match self.content {
            Types::TinyInt(_) | Types::SmallInt(_) | Types::Integer(_) | Types::BigInt(_) => true,
            _ => false,
        }
    }

    pub fn is_comparable_to(&self, other: &Self) -> bool {
        match self.content {
            Types::Boolean(_) => match other.content {
                Types::Boolean(_) | Types::Varchar(_) => true,
                _ => false,
            },
            Types::TinyInt(_)
            | Types::SmallInt(_)
            | Types::Integer(_)
            | Types::BigInt(_)
            | Types::Decimal(_) => match other.content {
                Types::TinyInt(_)
                | Types::SmallInt(_)
                | Types::Integer(_)
                | Types::BigInt(_)
                | Types::Decimal(_)
                | Types::Varchar(_) => true,
                _ => false,
            },
            // Anything can be cast to a string!
            Types::Varchar(_) => true,
            _ => false,
        }
    }

    forward!(content, get_as_bool, Result<i8, Error>);
    forward!(content, get_as_i8, Result<i8, Error>);
    forward!(content, get_as_i16, Result<i16, Error>);
    forward!(content, get_as_i32, Result<i32, Error>);
    forward!(content, get_as_i64, Result<i64, Error>);
    forward!(content, get_as_u64, Result<u64, Error>);
    forward!(content, get_as_f64, Result<f64, Error>);
}

impl<'a> Operation for Value<'a> {
    fn eq(&self, other: &Self) -> Option<bool> {
        compare!(self, other, (|x, y| x == y), (|x| almost_zero(x)))
    }

    fn ne(&self, other: &Self) -> Option<bool> {
        compare!(self, other, (|x, y| x != y), (|x| !almost_zero(x)))
    }

    fn lt(&self, other: &Self) -> Option<bool> {
        compare!(self, other, (|x, y| x < y), (|x| x < 0.0))
    }

    fn le(&self, other: &Self) -> Option<bool> {
        compare!(self, other, (|x, y| x <= y), (|x| x <= 0.0))
    }

    fn gt(&self, other: &Self) -> Option<bool> {
        compare!(self, other, (|x, y| x > y), (|x| x > 0.0))
    }

    fn ge(&self, other: &Self) -> Option<bool> {
        compare!(self, other, (|x, y| x >= y), (|x| x >= 0.0))
    }

    fn add(&self, other: &Self) -> Result<Self, Error> {
        arithmetic!(self, other, (|x, y| add(x, y)))
    }

    fn subtract(&self, other: &Self) -> Result<Self, Error> {
        arithmetic!(self, other, (|x, y| subtract(x, y)))
    }

    fn multiply(&self, other: &Self) -> Result<Self, Error> {
        arithmetic!(self, other, (|x, y| multiply(x, y)))
    }

    fn divide(&self, other: &Self) -> Result<Self, Error> {
        arithmetic!(self, other, (|x, y| divide(x, y)))
    }

    fn modulo(&self, other: &Self) -> Result<Self, Error> {
        arithmetic!(self, other, (|x, y| modulo(x, y)))
    }

    fn sqrt(&self) -> Result<Self, Error> {
        assert_numeric(self)?;
        if self.is_null() {
            let null = Types::decimal().null_val()?;
            return Ok(Value::new(null));
        }
        let val = match self.content {
            Types::TinyInt(val) => val as f64,
            Types::SmallInt(val) => val as f64,
            Types::Integer(val) => val as f64,
            Types::BigInt(val) => val as f64,
            Types::Decimal(val) => val as f64,
            _ => Err(unsupported!("Invalid type for `sqrt`"))?,
        };
        if val < 0.0 {
            Err(unsupported!("Cannot take `sqrt` on negative value"))
        } else {
            Ok(value!(val.sqrt(), Decimal))
        }
    }

    fn min(&self, other: &Self) -> Result<Self, Error> {
        assert_comparable(self, other)?;
        if self.is_null() || other.is_null() {
            return self.null(other);
        }
        if self.le(other) == Some(true) {
            Ok(self.clone())
        } else {
            Ok(other.clone())
        }
    }

    fn max(&self, other: &Self) -> Result<Self, Error> {
        assert_comparable(self, other)?;
        if self.is_null() || other.is_null() {
            return self.null(other);
        }
        if self.ge(other) == Some(true) {
            Ok(self.clone())
        } else {
            Ok(other.clone())
        }
    }

    fn null(&self, other: &Self) -> Result<Self, Error> {
        match self.content {
            Types::TinyInt(_) => genmatch!(
                other.content,
                Err(unsupported!("Invalid type for `null` on TinyInt")),
                { [TinyInt], nullas!(self) },
                { [SmallInt, Integer, BigInt, Decimal], nullas!(other) }
            ),
            Types::SmallInt(_) => genmatch!(
                other.content,
                Err(unsupported!("Invalid type for `null` on SmallInt")),
                { [TinyInt, SmallInt], nullas!(self) },
                { [Integer, BigInt, Decimal], nullas!(other) }
            ),
            Types::Integer(_) => genmatch!(
                other.content,
                Err(unsupported!("Invalid type for `null` on Integer")),
                { [TinyInt, SmallInt, Integer], nullas!(self) },
                { [BigInt, Decimal], nullas!(other) }
            ),
            Types::BigInt(_) => genmatch!(
                other.content,
                Err(unsupported!("Invalid type for `null` on BigInt")),
                { [TinyInt, SmallInt, Integer, BigInt], nullas!(self) },
                { [Decimal], nullas!(other) }
            ),
            Types::Decimal(_) => genmatch!(
                other.content,
                Err(unsupported!("Invalid type for `null` on Decimal")),
                { [TinyInt, SmallInt, Integer, BigInt, Decimal], nullas!(self) }
            ),
            _ => Err(unsupported!("Invalid type for `null`")),
        }
    }

    fn is_zero(&self) -> Result<bool, Error> {
        let res = match self.content {
            Types::TinyInt(val) => val == 0,
            Types::SmallInt(val) => val == 0,
            Types::Integer(val) => val == 0,
            Types::BigInt(val) => val == 0,
            Types::Decimal(val) => almost_zero(val),
            _ => Err(unsupported!("Invalid type for `is_zero`"))?,
        };
        Ok(res)
    }

    // Is the data inlined into this classes storage space, or must it be accessed
    // through an indirection/pointer?
    fn is_inlined(&self) -> bool {
        self.content.is_inlined()
    }

    fn to_string(&self) -> String {
        match self.content {
            Types::Boolean(val) => {
                if val == 0 {
                    "false".to_string()
                } else if val == 1 {
                    "true".to_string()
                } else {
                    "boolean_null".to_string()
                }
            }
            Types::TinyInt(_) => string!(self, "tinyint"),
            Types::SmallInt(_) => string!(self, "smallint"),
            Types::Integer(_) => string!(self, "integer"),
            Types::BigInt(_) => string!(self, "bigint"),
            Types::Decimal(_) => string!(self, "decimal"),
            Types::Timestamp(val) => string!(self, human_readable(val)),
            Types::Varchar(ref varlen) => match varlen {
                Varlen::Owned(Str::Val(val)) => val.clone(),
                Varlen::Borrowed(Str::Val(val)) => val.to_string(),
                _ => "varchar_max".to_string(),
            },
        }
    }

    // The caller needs to make sure that |dst| has enough space to hold data.
    fn serialize_to(&self, dst: &mut [u8]) {
        match self.content {
            Types::Boolean(val) => reinterpret::write_i8(dst, val),
            Types::TinyInt(val) => reinterpret::write_i8(dst, val),
            Types::SmallInt(val) => reinterpret::write_i16(dst, val),
            Types::Integer(val) => reinterpret::write_i32(dst, val),
            Types::BigInt(val) => reinterpret::write_i64(dst, val),
            Types::Decimal(val) => reinterpret::write_f64(dst, val),
            Types::Timestamp(val) => reinterpret::write_u64(dst, val),
            Types::Varchar(ref varlen) => match varlen {
                Varlen::Owned(Str::Val(val)) => {
                    reinterpret::write_i8(dst, 0);
                    reinterpret::write_str(&mut dst[1..], val);
                }
                Varlen::Borrowed(Str::Val(val)) => {
                    reinterpret::write_i8(dst, 0);
                    reinterpret::write_str(&mut dst[1..], val);
                }
                _ => reinterpret::write_i8(dst, 1),
            },
        }
    }

    // The caller needs to make sure that |src| is valid.
    fn deserialize_from(&mut self, src: &[u8]) {
        match &mut self.content {
            Types::Boolean(val) => *val = reinterpret::read_i8(src),
            Types::TinyInt(val) => *val = reinterpret::read_i8(src),
            Types::SmallInt(val) => *val = reinterpret::read_i16(src),
            Types::Integer(val) => *val = reinterpret::read_i32(src),
            Types::BigInt(val) => *val = reinterpret::read_i64(src),
            Types::Decimal(val) => *val = reinterpret::read_f64(src),
            Types::Timestamp(val) => *val = reinterpret::read_u64(src),
            Types::Varchar(vc) => {
                let byte = reinterpret::read_i8(src);
                if byte == 0 {
                    let s = reinterpret::read_str(&src[1..]).to_string();
                    *vc = Varlen::Owned(Str::Val(s));
                } else {
                    *vc = Varlen::Owned(Str::MaxVal);
                }
            }
        }
    }

    fn cast_to(&self, dst: &mut Self) -> Result<(), Error> {
        match self.content {
            Types::Boolean(src) => match &mut dst.content {
                Types::Boolean(val) => *val = src,
                Types::Varchar(val) => *val = Varlen::Owned(Str::Val(src.to_string())),
                _ => Err(unsupported!("Cannot cast boolean to given type"))?,
            },
            Types::TinyInt(src) => castnum!(dst.content, src, cast, "tinyint"),
            Types::SmallInt(src) => castnum!(dst.content, src, cast, "smallint"),
            Types::Integer(src) => castnum!(dst.content, src, cast, "integer"),
            Types::BigInt(src) => castnum!(dst.content, src, cast, "bigint"),
            Types::Decimal(src) => castnum!(dst.content, src, loss_cast, "decimal"),
            Types::Timestamp(src) => match &mut dst.content {
                Types::Timestamp(val) => *val = src,
                Types::Varchar(val) => *val = Varlen::Owned(Str::Val(src.to_string())),
                _ => Err(unsupported!("Cannot cast boolean to given type"))?,
            },
            Types::Varchar(ref varlen) => match &mut dst.content {
                Types::Boolean(val) => *val = parse::<_, bool>(varlen.borrow()?)? as i8,
                Types::TinyInt(val) => *val = parse(varlen.borrow()?)?,
                Types::SmallInt(val) => *val = parse(varlen.borrow()?)?,
                Types::Integer(val) => *val = parse(varlen.borrow()?)?,
                Types::BigInt(val) => *val = parse(varlen.borrow()?)?,
                Types::Decimal(val) => *val = parse(varlen.borrow()?)?,
                Types::Timestamp(val) => *val = parse(varlen.borrow()?)?,
                Types::Varchar(val) => *val = varlen.clone(),
            },
        }
        Ok(())
    }
}

fn almost_zero(val: f64) -> bool {
    val <= std::f64::EPSILON && val >= -std::f64::EPSILON
}

fn assert_numeric(val: &Value) -> Result<(), Error> {
    if !val.is_numeric() {
        Err(unsupported!("Non numeric"))
    } else {
        Ok(())
    }
}

fn assert_comparable(lhs: &Value, rhs: &Value) -> Result<(), Error> {
    if !lhs.is_comparable_to(rhs) {
        Err(unsupported!("Cannot compare"))
    } else {
        Ok(())
    }
}

fn varlen_value_cmp(lhs: &Varlen, rhs: &Value) -> Result<i8, Error> {
    let res = match rhs.content {
        Types::Varchar(ref varlen) => varlen_cmp(lhs, varlen),
        _ => varlen_cmp(lhs, &rhs.content.to_varlen()?),
    };
    Ok(res)
}

fn human_readable(mut tm: u64) -> String {
    let micro = (tm % 1000000) as u32;
    tm /= 1000000;
    let mut second = (tm % 100000) as u32;
    let sec = (second % 60) as u16;
    second /= 60;
    let min = (second % 60) as u16;
    second /= 60;
    let hour = (second % 24) as u16;
    tm /= 100000;
    let year = (tm % 10000) as u16;
    tm /= 10000;
    let mut tz = (tm % 27) as i32;
    tz -= 12;
    tm /= 27;
    let day = (tm % 32) as u16;
    tm /= 32;
    let month = tm as u16;
    let mut s = format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:06}",
        year, month, day, hour, min, sec, micro
    );
    if tz >= 0 {
        s.push('+');
    } else {
        s.push('-');
    }
    if tz < 0 {
        tz = -tz;
    }
    s.push_str(&format!("{:02}", tz));
    s
}

fn get_size<'a>(content: &Types<'a>) -> usize {
    let size = content.size();
    match content {
        Types::Boolean(val) => choose_size(val, &RSDB_BOOLEAN_NULL, size),
        Types::TinyInt(val) => choose_size(val, &RSDB_INT8_NULL, size),
        Types::SmallInt(val) => choose_size(val, &RSDB_INT16_NULL, size),
        Types::Integer(val) => choose_size(val, &RSDB_INT32_NULL, size),
        Types::BigInt(val) => choose_size(val, &RSDB_INT64_NULL, size),
        Types::Timestamp(val) => choose_size(val, &RSDB_TIMESTAMP_NULL, size),
        Types::Decimal(val) => choose_size(val, &RSDB_DECIMAL_NULL, size),
        Types::Varchar(val) => val.len(),
    }
}

fn choose_size<T: PartialEq>(val: &T, null: &T, size: usize) -> usize {
    if val == null {
        RSDB_VALUE_NULL as usize
    } else {
        size
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::types::Str;

    #[test]
    fn numeric_comparison() {
        let int1 = Value::new(Types::TinyInt(42));
        let int2 = Value::new(Types::SmallInt(42));
        let int3 = Value::new(Types::Integer(42));
        let int4 = Value::new(Types::Integer(100));
        let int5 = Value::new(Types::BigInt(42));
        assert_eq!(Some(true), int1.eq(&int2));
        assert_eq!(Some(true), int1.eq(&int3));
        assert_eq!(Some(false), int1.eq(&int4));
        assert_eq!(Some(true), int1.eq(&int5));
    }

    #[test]
    fn string_comparison() {
        let str1 = Value::new(Types::Varchar(Varlen::Owned(Str::Val("hello".to_string()))));
        let str2 = Value::new(Types::Varchar(Varlen::Borrowed(Str::Val("hello"))));
        let str3 = Value::new(Types::Varchar(Varlen::Owned(Str::MaxVal)));
        let str4 = Value::new(Types::Varchar(Varlen::Borrowed(Str::MaxVal)));
        assert_eq!(Some(true), str1.eq(&str2));
        assert_eq!(Some(false), str1.ne(&str2));
        assert_eq!(Some(true), str1.lt(&str3));
        assert_eq!(Some(true), str1.le(&str3));
        assert_eq!(Some(false), str1.gt(&str3));
        assert_eq!(Some(false), str1.ge(&str3));
        assert_eq!(Some(true), str1.lt(&str4));
        assert_eq!(Some(true), str1.le(&str4));
        assert_eq!(Some(false), str1.gt(&str4));
        assert_eq!(Some(false), str1.ge(&str4));
        assert_eq!(Some(true), str3.eq(&str4));
        assert_eq!(Some(false), str3.ne(&str4));
    }

    #[test]
    fn numeric_arithmetic() {
        let int1 = Value::new(Types::TinyInt(2));
        let int2 = Value::new(Types::SmallInt(3));
        let int3 = Value::new(Types::Integer(5));
        let int4 = Value::new(Types::BigInt(7));
        let int5 = Value::new(Types::Integer(0));
        let dec1 = Value::new(Types::Decimal(10.0));
        let dec2 = Value::new(Types::Decimal(0.0));

        assert_eq!(Some(true), int1.add(&int1).unwrap().eq(&value!(4, TinyInt)));
        assert_eq!(
            Some(true),
            int1.add(&int2).unwrap().eq(&value!(5, SmallInt))
        );

        assert_eq!(
            Some(true),
            int2.subtract(&int3).unwrap().eq(&value!(-2, Integer))
        );
        assert_eq!(
            Some(true),
            dec1.subtract(&int3).unwrap().eq(&value!(5.0, Decimal))
        );

        assert_eq!(
            Some(true),
            int3.multiply(&int4).unwrap().eq(&value!(35, BigInt))
        );
        assert_eq!(
            Some(true),
            dec1.multiply(&int4).unwrap().eq(&value!(70.0, Decimal))
        );
        assert_eq!(
            Some(true),
            int3.multiply(&dec1).unwrap().eq(&value!(50.0, Decimal))
        );

        assert_eq!(
            Some(true),
            int3.divide(&int4).unwrap().eq(&value!(0, BigInt))
        );
        assert_eq!(
            Some(true),
            int4.divide(&int1).unwrap().eq(&value!(3, BigInt))
        );
        assert_eq!(
            Some(true),
            int5.divide(&int3).unwrap().eq(&value!(0, Integer))
        );
        assert_eq!(
            Some(true),
            dec1.divide(&int3).unwrap().eq(&value!(2.0, Decimal))
        );
        assert_eq!(
            Some(true),
            int1.divide(&dec1).unwrap().eq(&value!(0.2, Decimal))
        );

        assert_eq!(
            Some(true),
            int4.modulo(&int2).unwrap().eq(&value!(1, BigInt))
        );
        assert_eq!(
            Some(true),
            int5.modulo(&int3).unwrap().eq(&value!(0, Integer))
        );
        assert_eq!(
            Some(true),
            dec1.modulo(&int1).unwrap().eq(&value!(0.0, Decimal))
        );

        assert!(int4.divide(&int5).is_err());
        assert!(int4.divide(&dec2).is_err());
        assert!(int2.modulo(&int5).is_err());
        assert!(int2.modulo(&dec2).is_err());
    }

    #[test]
    fn sqrt_test() {
        let int1 = value!(0, Integer);
        let int2 = value!(9, Integer);
        let int3 = value!(-9, Integer);
        let dec1 = value!(0.0, Decimal);
        let dec2 = value!(16.0, Decimal);
        let dec3 = value!(-16.0, Decimal);

        assert_eq!(Some(true), int1.sqrt().unwrap().eq(&value!(0.0, Decimal)));
        assert_eq!(Some(true), int2.sqrt().unwrap().eq(&value!(3.0, Decimal)));
        assert!(int3.sqrt().is_err());

        assert_eq!(Some(true), dec1.sqrt().unwrap().eq(&value!(0.0, Decimal)));
        assert_eq!(Some(true), dec2.sqrt().unwrap().eq(&value!(4.0, Decimal)));
        assert!(dec3.sqrt().is_err());
    }

    #[test]
    fn null_and_checks() {
        let nullint = Value::new(Types::integer().null_val().unwrap());
        let nulldec = Value::new(Types::decimal().null_val().unwrap());
        assert!(nullint.is_integer());
        assert!(!nulldec.is_integer());
        assert!(nullint.is_numeric());
        assert!(nulldec.is_numeric());
        assert!(nullint.is_null());
        assert!(nulldec.is_null());

        // Calling compare on null returns None.
        assert!(nullint.sqrt().unwrap().eq(&nullint).is_none());
        assert!(nulldec.sqrt().unwrap().eq(&nulldec).is_none());

        let num1 = value!(0, Integer);
        let num2 = value!(0, BigInt);
        assert!(num1.null(&num2).unwrap().is_null());
        assert!(num2.null(&num1).unwrap().is_null());
    }

    #[test]
    fn min_and_max() {
        let int1 = value!(0, Integer);
        let int2 = value!(9, Integer);
        let int3 = value!(-9, Integer);
        let dec1 = value!(1.0, Decimal);
        let dec2 = value!(16.0, Decimal);
        let dec3 = value!(-16.0, Decimal);
        assert_eq!(Some(true), int1.min(&int3).unwrap().eq(&int3));
        assert_eq!(Some(true), int2.max(&int3).unwrap().eq(&int2));
        assert_eq!(Some(true), dec1.min(&dec2).unwrap().eq(&dec1));
        assert_eq!(Some(true), dec1.max(&dec3).unwrap().eq(&dec1));
        assert_eq!(Some(true), int1.min(&dec1).unwrap().eq(&int1));
        assert_eq!(Some(true), int1.max(&dec1).unwrap().eq(&dec1));

        let nullint = Value::new(Types::integer().null_val().unwrap());
        let nulldec = Value::new(Types::decimal().null_val().unwrap());
        assert!(nullint.min(&int1).unwrap().is_null());
        assert!(nullint.max(&int2).unwrap().is_null());
        assert!(int2.min(&nullint).unwrap().is_null());
        assert!(int1.max(&nullint).unwrap().is_null());
        assert!(nulldec.min(&dec1).unwrap().is_null());
        assert!(nulldec.max(&dec2).unwrap().is_null());
        assert!(dec2.min(&nulldec).unwrap().is_null());
        assert!(dec1.max(&nulldec).unwrap().is_null());
    }

    #[test]
    fn serialize_and_deserialize() {
        let mut buffer = [0; 100];

        let intw = value!(123454321, BigInt);
        let mut intr = Value::new(Types::bigint());
        intw.serialize_to(&mut buffer);
        intr.deserialize_from(&buffer);
        assert_eq!(123454321, intr.get_as_i64().unwrap());

        let strw = value!(
            Varlen::Borrowed(Str::Val("oranges are not the only fruit")),
            Varchar
        );
        let mut strr = Value::new(Types::borrowed());
        strw.serialize_to(&mut buffer);
        strr.deserialize_from(&buffer);
        match strr.content {
            Types::Varchar(Varlen::Owned(Str::Val(s))) => {
                assert_eq!("oranges are not the only fruit", s)
            }
            _ => panic!("fail"),
        }

        let strw = value!(Varlen::Owned(Str::MaxVal), Varchar);
        let mut strr = Value::new(Types::owned());
        strw.serialize_to(&mut buffer);
        strr.deserialize_from(&buffer);
        match strr.content {
            Types::Varchar(Varlen::Owned(Str::MaxVal)) => (),
            _ => panic!("fail"),
        }
    }

    #[test]
    fn cast_test() {
        let integer = value!(66666, Integer);
        let mut bigint = Value::new(Types::bigint());
        assert!(integer.cast_to(&mut Value::new(Types::tinyint())).is_err());
        assert!(integer.cast_to(&mut Value::new(Types::smallint())).is_err());
        assert!(integer.cast_to(&mut bigint).is_ok());
        assert_eq!(66666, bigint.get_as_i64().unwrap());

        let decimal = value!(314.15927, Decimal);
        let mut bigint = Value::new(Types::bigint());
        assert!(decimal.cast_to(&mut Value::new(Types::tinyint())).is_err());
        assert!(decimal.cast_to(&mut Value::new(Types::smallint())).is_ok());
        assert!(decimal.cast_to(&mut bigint).is_ok());
        assert_eq!(314, bigint.get_as_i64().unwrap());

        let string1 = value!(Varlen::Borrowed(Str::Val("1234")), Varchar);
        let mut smallint = Value::new(Types::bigint());
        assert!(string1.cast_to(&mut smallint).is_ok());
        assert_eq!(1234, smallint.get_as_i16().unwrap());

        let string2 = value!(Varlen::Borrowed(Str::Val("12.34")), Varchar);
        let mut tinyint = Value::new(Types::tinyint());
        let mut decimal = Value::new(Types::decimal());
        assert!(string2.cast_to(&mut tinyint).is_err());
        assert!(string2.cast_to(&mut decimal).is_ok());
        assert_eq!(12.34, decimal.get_as_f64().unwrap());

        let invalid = value!(Varlen::Borrowed(Str::Val("invalid")), Varchar);
        let mut integer = Value::new(Types::integer());
        let mut decimal = Value::new(Types::decimal());
        assert!(invalid.cast_to(&mut integer).is_err());
        assert!(invalid.cast_to(&mut decimal).is_err());
    }
}
