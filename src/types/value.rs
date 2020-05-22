#![allow(dead_code)]
#![allow(unused_variables)]

use crate::types::limits::*;
use crate::types::types::Operation;
use crate::types::types::Types;

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

    // pub fn data() -> {}
}

impl<'a> Operation for Value<'a> {
    fn eq(&self, other: &Self) -> Option<bool> {
        None
    }

    fn ne(&self, other: &Self) -> Option<bool> {
        None
    }

    fn lt(&self, other: &Self) -> Option<bool> {
        None
    }

    fn le(&self, other: &Self) -> Option<bool> {
        None
    }

    fn gt(&self, other: &Self) -> Option<bool> {
        None
    }

    fn ge(&self, other: &Self) -> Option<bool> {
        None
    }

    fn add(&self, other: &Self) -> Option<Self> {
        None
    }

    fn subtract(&self, other: &Self) -> Option<Self> {
        None
    }

    fn multiply(&self, other: &Self) -> Option<Self> {
        None
    }

    fn divide(&self, other: &Self) -> Option<Self> {
        None
    }

    fn modulo(&self, other: &Self) -> Option<Self> {
        None
    }

    fn min(&self, other: &Self) -> Option<Self> {
        if !self.is_comparable_to(other) {
            panic!("Cannot compare");
        }
        None
    }

    fn max(&self, other: &Self) -> Option<Self> {
        if !self.is_comparable_to(other) {
            panic!("Cannot compare");
        }
        None
    }

    fn sqrt(&self) -> Option<Self> {
        None
    }

    fn null(&self, other: &Self) -> Option<Self> {
        None
    }

    fn is_zero(&self) -> bool {
        false
    }

    fn is_inlined(&self) -> bool {
        false
    }

    fn to_string(&self) -> String {
        String::from("")
    }

    fn serialize_to(&self, dst: &mut [u8]) {}

    fn deserialize_from(&mut self, src: &[u8]) {}

    fn cast_to(&self, dst: &mut Self) -> bool {
        false
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
