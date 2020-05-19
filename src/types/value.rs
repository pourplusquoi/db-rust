#![allow(dead_code)]

use crate::types::types::CmpBool;
use crate::types::types::Type;

pub struct Value {
  content: Type,
  size: Option<u32>,  // Only set iff |content| is Varchar.
}

pub fn cmp_bool(val: bool) -> CmpBool {
  if val {
    CmpBool::CmpTrue
  } else {
    CmpBool::CmpFalse
  }
}

impl Value {
  pub fn new(content: Type) -> Self {
    Value {
      size: match &content {
        // Assuming the length of string fits in u32.
        Type::Varchar(val) => Some(val.len() as u32),
        _ => None,
      },
      content: content,
    }
  }

  pub fn borrow(&self) -> &Type {
    &self.content
  }

  pub fn borrow_mut(&mut self) -> &mut Type {
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