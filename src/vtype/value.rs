#![allow(dead_code)]

use crate::vtype::value_type::ValueType;

pub struct Value {
  value_type: ValueType,
  size: Option<u32>,  // Only set if |type| is Varchar.
}

impl Value {
  fn new(vtype: ValueType) -> Self {
    Value {
      size: match &vtype {
        // Assuming the length of string fits in u32.
        &ValueType::Varchar(ref val) => Some(val.len() as u32),
        _ => None,
      },
      value_type: vtype,
    }
  }
}