use std::cmp::Eq;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::hash::Hash;

// Note: The type `T` should be cheap to clone; otherwise it would be
// expensive to use this trait.
pub trait Replacer<T> where T: Clone + Eq + Hash {
  fn insert(&mut self, val: T);
  fn erase(&mut self, val: &T) -> bool;
  fn victim(&mut self) -> Option<T>;
  fn size(&self) -> usize;
}

pub struct LRUReplacer<T> where T: Clone + Eq + Hash {
  forward: HashMap<T, u32>,
  backward: BTreeMap<u32, T>,
  clock: u32,
}

impl<T> LRUReplacer<T> where T: Clone + Eq + Hash {
  pub fn new() -> Self {
    LRUReplacer {
      forward: HashMap::new(),
      backward: BTreeMap::new(),
      clock: 0,
    }
  }
}

impl<T> Replacer<T> for LRUReplacer<T>
    where T: Clone + Eq + Hash {

  fn insert(&mut self, val: T) {
    match self.forward.get(&val) {
      None => (),
      Some(c) => {
        self.backward.remove(c);
      }
    }
    self.forward.insert(val.clone(), self.clock);
    self.backward.insert(self.clock, val);
    self.clock += 1;
  }

  fn erase(&mut self, val: &T) -> bool {
    match self.forward.remove(val) {
      None => false,
      Some(ref c) => {
        self.backward.remove(c);
        true
      },
    }
  }

  fn victim(&mut self) -> Option<T> {
    let first = match self.backward.iter().nth(0) {
      None => None,
      Some((key, val)) => Some((*key, val.clone())),
    };
    match first {
      None => None,
      Some((key, val)) => {
        self.backward.remove(&key);
        self.forward.remove(&val);
        Some(val)
      },
    }
  }

  fn size(&self) -> usize {
    self.forward.len()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn lru_replacer_i32() {
    let mut lru = LRUReplacer::new();
    
    // Push element into replacer.
    lru.insert(1);
    lru.insert(2);
    lru.insert(3);
    lru.insert(4);
    lru.insert(5);
    lru.insert(6);
    lru.insert(1);
    assert_eq!(6, lru.size());
    
    // Pop element from replacer.
    assert_eq!(Some(2), lru.victim());
    assert_eq!(Some(3), lru.victim());
    assert_eq!(Some(4), lru.victim());
    
    // Remove element from replacer.
    assert_eq!(false, lru.erase(&4));
    assert_eq!(true, lru.erase(&6));
    assert_eq!(2, lru.size());
    
    // Pop element from replacer after removal.
    assert_eq!(Some(5), lru.victim());
    assert_eq!(Some(1), lru.victim());
    assert_eq!(0, lru.size());

    // Pop when empty.
    assert_eq!(None, lru.victim());
    assert_eq!(0, lru.size());

    // Erase when empty.
    assert_eq!(false, lru.erase(&1));
    assert_eq!(false, lru.erase(&2));
    assert_eq!(0, lru.size());
  }

  #[test]
  fn lru_replacer_string() {
    let mut lru = LRUReplacer::new();

    lru.insert(String::from("hello"));
    lru.insert(String::from("world"));
    lru.insert(String::from("hello"));
    assert_eq!(2, lru.size());

    assert_eq!(Some(String::from("world")), lru.victim());
    assert_eq!(false, lru.erase(&String::from("world")));
    assert_eq!(1, lru.size());

    lru.insert(String::from("hello"));
    assert_eq!(1, lru.size());

    lru.insert(String::from("world"));
    assert_eq!(2, lru.size());

    assert_eq!(Some(String::from("hello")), lru.victim());
    assert_eq!(Some(String::from("world")), lru.victim());
    assert_eq!(0, lru.size());
  }
}