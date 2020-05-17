// Trait for replacer. Especially, the LRU replacer should implement those
// methods.

use crate::common::newable::Newable;
use std::clone::Clone;
use std::cmp::Eq;
use std::hash::Hash;

// Note: The type `T` should be cheap to clone; otherwise it would be
// expensive to use this trait.
pub trait Replacer<T> : Newable where T: Clone + Eq + Hash {
  fn insert(&mut self, val: T);
  fn erase(&mut self, val: &T) -> bool;
  fn victim(&mut self) -> Option<T>;
  fn size(&self) -> usize;
}