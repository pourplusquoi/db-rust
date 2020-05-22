#![allow(dead_code)]
#![allow(unused_variables)]

use crate::types::limits::*;
use crate::types::types::Operation;
use crate::types::types::Types;
use crate::types::types::Varlen;
use crate::types::varlen_util::*;

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

    fn get_as_bool(&self) -> i8 {
        self.content.get_as_bool()
    }

    fn get_as_i8(&self) -> i8 {
        self.content.get_as_i8()
    }

    fn get_as_i16(&self) -> i16 {
        self.content.get_as_i16()
    }

    fn get_as_i32(&self) -> i32 {
        self.content.get_as_i32()
    }

    fn get_as_i64(&self) -> i64 {
        self.content.get_as_i64()
    }

    fn get_as_u64(&self) -> u64 {
        self.content.get_as_u64()
    }

    fn get_as_f64(&self) -> f64 {
        self.content.get_as_f64()
    }

    // pub fn data() -> {}
}

impl<'a> Operation for Value<'a> {
    fn eq(&self, other: &Self) -> Option<bool> {
        compare!(self, other, (|x, y| x == y), (|x: Value| x.is_zero()))
    }

    fn ne(&self, other: &Self) -> Option<bool> {
        compare!(self, other, (|x, y| x != y), (|x: Value| !x.is_zero()))
    }

    fn lt(&self, other: &Self) -> Option<bool> {
        compare!(
            self,
            other,
            (|x, y| x < y),
            (|x: Value| x.get_as_f64() < 0.0)
        )
    }

    fn le(&self, other: &Self) -> Option<bool> {
        compare!(
            self,
            other,
            (|x, y| x <= y),
            (|x: Value| x.get_as_f64() <= 0.0)
        )
    }

    fn gt(&self, other: &Self) -> Option<bool> {
        compare!(
            self,
            other,
            (|x, y| x > y),
            (|x: Value| x.get_as_f64() > 0.0)
        )
    }

    fn ge(&self, other: &Self) -> Option<bool> {
        compare!(
            self,
            other,
            (|x, y| x >= y),
            (|x: Value| x.get_as_f64() >= 0.0)
        )
    }

    // TODO: Implement this.
    fn add(&self, other: &Self) -> Self {
        self.clone()
    }

    // TODO: Implement this.
    fn subtract(&self, other: &Self) -> Self {
        self.clone()
    }

    // TODO: Implement this.
    fn multiply(&self, other: &Self) -> Self {
        self.clone()
    }

    // TODO: Implement this.
    fn divide(&self, other: &Self) -> Self {
        self.clone()
    }

    // TODO: Implement this.
    fn modulo(&self, other: &Self) -> Self {
        self.clone()
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

    // TODO: Implement this.
    fn sqrt(&self) -> Self {
        self.clone()
    }

    fn null(&self, other: &Self) -> Self {
        match self.content {
            Types::TinyInt(_) => match other.content {
                Types::TinyInt(_)
                | Types::SmallInt(_)
                | Types::Integer(_)
                | Types::BigInt(_)
                | Types::Decimal(_) => Some(Value::new(other.content.clone())),
                _ => None,
            },
            Types::SmallInt(_) => match other.content {
                Types::TinyInt(_) => Some(Value::new(self.content.clone())),
                Types::SmallInt(_) | Types::Integer(_) | Types::BigInt(_) | Types::Decimal(_) => {
                    Some(Value::new(other.content.clone()))
                }
                _ => None,
            },
            Types::Integer(_) => match other.content {
                Types::TinyInt(_) | Types::SmallInt(_) => Some(Value::new(self.content.clone())),
                Types::Integer(_) | Types::BigInt(_) | Types::Decimal(_) => {
                    Some(Value::new(other.content.clone()))
                }
                _ => None,
            },
            Types::BigInt(_) => match other.content {
                Types::TinyInt(_) | Types::SmallInt(_) | Types::Integer(_) => {
                    Some(Value::new(self.content.clone()))
                }
                Types::BigInt(_) | Types::Decimal(_) => Some(Value::new(other.content.clone())),
                _ => None,
            },
            Types::Decimal(_) => match other.content {
                Types::TinyInt(_)
                | Types::SmallInt(_)
                | Types::Integer(_)
                | Types::BigInt(_)
                | Types::Decimal(_) => Some(Value::new(self.content.clone())),
                _ => None,
            },
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
            _ => {
                panic!("Type error for is_zero");
            }
        }
    }

    // TODO: Implement this.
    fn is_inlined(&self) -> bool {
        false
    }

    // TODO: Implement this.
    fn to_string(&self) -> String {
        String::from("")
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

fn get_size<'a>(content: &Types<'a>) -> u32 {
    match content {
        Types::Boolean(val) => {
            if *val == PELOTON_BOOLEAN_NULL {
                PELOTON_VALUE_NULL
            } else {
                0
            }
        }
        Types::TinyInt(val) => {
            if *val == PELOTON_INT8_NULL {
                PELOTON_VALUE_NULL
            } else {
                0
            }
        }
        Types::SmallInt(val) => {
            if *val == PELOTON_INT16_NULL {
                PELOTON_VALUE_NULL
            } else {
                0
            }
        }
        Types::Integer(val) => {
            if *val == PELOTON_INT32_NULL {
                PELOTON_VALUE_NULL
            } else {
                0
            }
        }
        Types::BigInt(val) => {
            if *val == PELOTON_INT64_NULL {
                PELOTON_VALUE_NULL
            } else {
                0
            }
        }
        Types::Timestamp(val) => {
            if *val == PELOTON_TIMESTAMP_NULL {
                PELOTON_VALUE_NULL
            } else {
                0
            }
        }
        Types::Decimal(val) => {
            if *val == PELOTON_DECIMAL_NULL {
                PELOTON_VALUE_NULL
            } else {
                0
            }
        }
        // Assuming the length of string fits in u32.
        Types::Varchar(val) => val.len() as u32,
    }
}
