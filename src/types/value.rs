#![allow(unused_variables)]

use crate::types::limits::*;
use crate::types::types::Operation;
use crate::types::types::Str;
use crate::types::types::Types;
use crate::types::types::Varlen;
use crate::types::varlen_util::*;
use log::error;
use std::cmp::PartialEq;

#[derive(Clone)]
pub struct Value<'a> {
    content: Types<'a>,
    size: u32,
}

impl<'a> Value<'a> {
    pub fn new(content: Types<'a>) -> Self {
        Value {
            size: get_size(&content),
            content: content,
        }
    }

    pub fn len(&self) -> usize {
        self.size as usize
    }

    pub fn borrow(&self) -> &'a Types {
        &self.content
    }

    pub fn borrow_mut(&mut self) -> &'a mut Types {
        &mut self.content
    }

    pub fn is_null(&self) -> bool {
        self.size == PELOTON_VALUE_NULL
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

    forward!(content, get_as_bool, i8);
    forward!(content, get_as_i8, i8);
    forward!(content, get_as_i16, i16);
    forward!(content, get_as_i32, i32);
    forward!(content, get_as_i64, i64);
    forward!(content, get_as_u64, u64);
    forward!(content, get_as_f64, f64);
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

    fn add(&self, other: &Self) -> Self {
        arithmetic!(self, other, (|x, y| x + y))
    }

    fn subtract(&self, other: &Self) -> Self {
        arithmetic!(self, other, (|x, y| x - y))
    }

    fn multiply(&self, other: &Self) -> Self {
        arithmetic!(self, other, (|x, y| x * y))
    }

    fn divide(&self, other: &Self) -> Option<Self> {
        if other.is_zero() {
            error!("[divide] Divide by zero error");
            return None;
        }
        Some(arithmetic!(self, other, (|x, y| x / y)))
    }

    fn modulo(&self, other: &Self) -> Option<Self> {
        if other.is_zero() {
            error!("[modulo] Divide by zero error");
            return None;
        }
        match self.content {
            Types::Decimal(val) => Some(arithmetic_decimal!(
                val,
                other,
                (|x: f64, y: f64| x - (x / y).trunc() * y)
            )),
            _ => Some(arithmetic!(self, other, (|x, y| x % y))),
        }
    }

    fn sqrt(&self) -> Option<Self> {
        assert_numeric(self);
        if self.is_null() {
            let null = Types::decimal().null_val();
            return Some(Value::new(null));
        }
        let val = match self.content {
            Types::TinyInt(val) => val as f64,
            Types::SmallInt(val) => val as f64,
            Types::Integer(val) => val as f64,
            Types::BigInt(val) => val as f64,
            Types::Decimal(val) => val as f64,
            _ => panic!("Type error in sqrt"),
        };
        if val < 0.0 {
            None
        } else {
            Some(value!(val.sqrt(), Decimal))
        }
    }

    fn min(&self, other: &Self) -> Self {
        assert_comparable(self, other);
        if self.is_null() || other.is_null() {
            return self.null(other);
        }
        if self.le(other) == Some(true) {
            self.clone()
        } else {
            other.clone()
        }
    }

    fn max(&self, other: &Self) -> Self {
        assert_comparable(self, other);
        if self.is_null() || other.is_null() {
            return self.null(other);
        }
        if self.ge(other) == Some(true) {
            self.clone()
        } else {
            other.clone()
        }
    }

    fn null(&self, other: &Self) -> Self {
        match self.content {
            Types::TinyInt(_) => generate_match!(
                other.content,
                None,
                { [TinyInt], Some(nullas!(self)) },
                { [SmallInt, Integer, BigInt, Decimal], Some(nullas!(other)) }
            ),
            Types::SmallInt(_) => generate_match!(
                other.content,
                None,
                { [TinyInt, SmallInt], Some(nullas!(self)) },
                { [Integer, BigInt, Decimal], Some(nullas!(other)) }
            ),
            Types::Integer(_) => generate_match!(
                other.content,
                None,
                { [TinyInt, SmallInt, Integer], Some(nullas!(self)) },
                { [BigInt, Decimal], Some(nullas!(other)) }
            ),
            Types::BigInt(_) => generate_match!(
                other.content,
                None,
                { [TinyInt, SmallInt, Integer, BigInt], Some(nullas!(self)) },
                { [Decimal], Some(nullas!(other)) }
            ),
            Types::Decimal(_) => generate_match!(
                other.content,
                None,
                { [TinyInt, SmallInt, Integer, BigInt, Decimal], Some(nullas!(self)) }
            ),
            _ => None,
        }
        .expect("Type error for null")
    }

    fn is_zero(&self) -> bool {
        match self.content {
            Types::TinyInt(val) => val == 0,
            Types::SmallInt(val) => val == 0,
            Types::Integer(val) => val == 0,
            Types::BigInt(val) => val == 0,
            Types::Decimal(val) => almost_zero(val),
            _ => panic!("Type error for is_zero"),
        }
    }

    // Is the data inlined into this classes storage space, or must it be accessed
    // through an indirection/pointer?
    fn is_inlined(&self) -> bool {
        match self.content {
            Types::Varchar(_) => false,
            _ => true,
        }
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
            Types::Varchar(ref vc) => match vc {
                Varlen::Owned(Str::Val(val)) => val.clone(),
                Varlen::Borrowed(Str::Val(val)) => val.to_string(),
                _ => "varchar_max".to_string(),
            },
        }
    }

    // TODO: Implement this.
    fn serialize_to(&self, dst: &mut [u8]) {}

    // TODO: Implement this.
    fn deserialize_from(&mut self, src: &[u8]) {}

    // TODO: Implement this.
    fn cast_to(&self, dst: &mut Self) -> bool {
        false
    }
}

fn almost_zero(val: f64) -> bool {
    val <= std::f64::EPSILON && val >= -std::f64::EPSILON
}

fn assert_numeric(val: &Value) {
    if !val.is_numeric() {
        panic!("Non numeric");
    }
}

fn assert_comparable(lhs: &Value, rhs: &Value) {
    if !lhs.is_comparable_to(rhs) {
        panic!("Cannot compare");
    }
}

fn varlen_value_cmp(lhs: &Varlen, rhs: &Value) -> i8 {
    match rhs.content {
        Types::Varchar(ref varlen) => varlen_cmp(lhs, varlen),
        _ => varlen_cmp(lhs, &rhs.content.to_varlen()),
    }
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

fn get_size<'a>(content: &Types<'a>) -> u32 {
    match content {
        Types::Boolean(val) => compute_size(*val, PELOTON_BOOLEAN_NULL),
        Types::TinyInt(val) => compute_size(*val, PELOTON_INT8_NULL),
        Types::SmallInt(val) => compute_size(*val, PELOTON_INT16_NULL),
        Types::Integer(val) => compute_size(*val, PELOTON_INT32_NULL),
        Types::BigInt(val) => compute_size(*val, PELOTON_INT64_NULL),
        Types::Timestamp(val) => compute_size(*val, PELOTON_TIMESTAMP_NULL),
        Types::Decimal(val) => compute_size(*val, PELOTON_DECIMAL_NULL),
        // Assuming the length of string fits in u32.
        Types::Varchar(val) => val.len() as u32,
    }
}

fn compute_size<T: PartialEq>(val: T, null: T) -> u32 {
    if val == null {
        PELOTON_VALUE_NULL
    } else {
        0
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

        assert_eq!(Some(true), int1.add(&int1).eq(&value!(4, TinyInt)));
        assert_eq!(Some(true), int1.add(&int2).eq(&value!(5, SmallInt)));

        assert_eq!(Some(true), int2.subtract(&int3).eq(&value!(-2, Integer)));
        assert_eq!(Some(true), dec1.subtract(&int3).eq(&value!(5.0, Decimal)));

        assert_eq!(Some(true), int3.multiply(&int4).eq(&value!(35, BigInt)));
        assert_eq!(Some(true), dec1.multiply(&int4).eq(&value!(70.0, Decimal)));
        assert_eq!(Some(true), int3.multiply(&dec1).eq(&value!(50.0, Decimal)));

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

        assert!(int4.divide(&int5).is_none());
        assert!(int4.divide(&dec2).is_none());
        assert!(int2.modulo(&int5).is_none());
        assert!(int2.modulo(&dec2).is_none());
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
        assert!(int3.sqrt().is_none());

        assert_eq!(Some(true), dec1.sqrt().unwrap().eq(&value!(0.0, Decimal)));
        assert_eq!(Some(true), dec2.sqrt().unwrap().eq(&value!(4.0, Decimal)));
        assert!(dec3.sqrt().is_none());
    }

    #[test]
    fn null_and_checks() {
        let nullint = Value::new(Types::integer().null_val());
        let nulldec = Value::new(Types::decimal().null_val());
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
        assert!(num1.null(&num2).is_null());
        assert!(num2.null(&num1).is_null());
    }

    #[test]
    fn min_and_max() {
        let int1 = value!(0, Integer);
        let int2 = value!(9, Integer);
        let int3 = value!(-9, Integer);
        let dec1 = value!(1.0, Decimal);
        let dec2 = value!(16.0, Decimal);
        let dec3 = value!(-16.0, Decimal);
        let nullint = Value::new(Types::integer().null_val());
        let nulldec = Value::new(Types::decimal().null_val());

        assert_eq!(Some(true), int1.min(&int3).eq(&int3));
        assert_eq!(Some(true), int2.max(&int3).eq(&int2));
        assert_eq!(Some(true), dec1.min(&dec2).eq(&dec1));
        assert_eq!(Some(true), dec1.max(&dec3).eq(&dec1));
        assert_eq!(Some(true), int1.min(&dec1).eq(&int1));
        assert_eq!(Some(true), int1.max(&dec1).eq(&dec1));
    }

    #[test]
    fn serialize_and_deserialize() {}

    #[test]
    fn cast_test() {}
}
