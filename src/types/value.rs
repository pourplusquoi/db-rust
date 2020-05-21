#![allow(dead_code)]
#![allow(unused_variables)]

use crate::types::types::Operation;
use crate::types::types::Types;

pub struct Value<'a> {
  content: Types<'a>,
  size: Option<u32>,  // Only set iff |content| is Varchar.
}

impl<'a> Value<'a> {
  pub fn new(content: Types<'a>) -> Self {
    Value {
      size: match &content {
        // Assuming the length of string fits in u32.
        Types::Varchar(val) => Some(val.len() as u32),
        _ => None,
      },
      content: content,
    }
  }

  pub fn borrow(&self) -> &'a Types {
    &self.content
  }

  pub fn borrow_mut(&mut self) -> &'a mut Types {
    &mut self.content
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

impl<'a> Operation for Value<'a> {
  fn eq(&self, other: &Self) -> Option<bool> {None}
  fn ne(&self, other: &Self) -> Option<bool> {None}
  fn lt(&self, other: &Self) -> Option<bool> {None}
  fn le(&self, other: &Self) -> Option<bool> {None}
  fn gt(&self, other: &Self) -> Option<bool> {None}
  fn ge(&self, other: &Self) -> Option<bool> {None}
  fn add(&self, other: &Self) -> Option<Self> {None}
  fn subtract(&self, other: &Self) -> Option<Self> {None}
  fn multiply(&self, other: &Self) -> Option<Self> {None}
  fn divide(&self, other: &Self) -> Option<Self> {None}
  fn modulo(&self, other: &Self) -> Option<Self> {None}
  fn min(&self, other: &Self) -> Option<Self> {None}
  fn max(&self, other: &Self) -> Option<Self> {None}
  fn sqrt(&self, other: &Self) -> Option<Self> {None}
  fn null(&self, other: &Self) -> Option<Self> {None}
  fn is_zero(&self) -> bool {false}
  fn is_inlined(&self) -> bool {false}
  fn to_string(&self) -> String {String::from("")}
  fn serialize_to(&self, dst: &mut [u8]) {}
  fn deserialize_from(&mut self, src: &[u8]) {}
  fn cast(&self, dst: &mut Self) -> bool {false}
}