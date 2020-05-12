use std::hash::Hash;
use std::cmp::Eq;
use std::collections::BTreeMap;
use std::collections::HashMap;

trait Replacer<T> where T: Copy + Eq + Hash {
  fn insert(&mut self, val: T);
  fn erase(&mut self, val: &T) -> bool;
  fn victim(&mut self) -> Option<T>;
  fn size(&self) -> usize;
}

struct LRUReplacer<T> where T: Copy + Eq + Hash {
  forward: HashMap<T, u32>,
  backward: BTreeMap<u32, T>,
  clock: u32,
}

impl<T> LRUReplacer<T> where T: Copy + Eq + Hash {
  fn new() -> Self {
    LRUReplacer {
      forward: HashMap::new(),
      backward: BTreeMap::new(),
      clock: 0,
    }
  }
}

impl<T> Replacer<T> for LRUReplacer<T>
    where T: Copy + Eq + Hash {

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
      Some((&key, &val)) => Some((key, val)),
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

fn main() {}