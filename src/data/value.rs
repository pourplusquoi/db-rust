#![allow(dead_code)]

use crate::data::value_type::CmpBool;
use crate::data::value_type::ValueType;

pub struct Value {
  data: ValueType,
  size: Option<u32>,  // Only set iff |type| is Varchar.
}

pub fn cmp_bool(val: bool) -> CmpBool {
  if val {
    CmpBool::CmpTrue
  } else {
    CmpBool::CmpFalse
  }
}

impl Value {
  pub fn new(data: ValueType) -> Self {
    Value {
      size: match &data {
        // Assuming the length of string fits in u32.
        ValueType::Varchar(val) => Some(val.len() as u32),
        _ => None,
      },
      data: data,
    }
  }

  pub fn borrow(&self) -> &ValueType {
    &self.data
  }

  pub fn borrow_mut(&mut self) -> &mut ValueType {
    &mut self.data
  }

  // TODO: Implement this.
  pub fn is_integer(&self) -> bool {false}

  // TODO: Implement this.
  pub fn is_comparable(&self) -> bool {false}

  pub fn len(&self) -> Option<usize> {
    self.size.map(|x| x as usize)
  }

  // pub fn data() -> {}
}