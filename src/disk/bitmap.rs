use crate::common::config::CHECKSUM_SIZE;
use crate::disk::disk_manager::read;
use crate::disk::disk_manager::write;
use crate::logging::error_logging::ErrorLogging;
use std::collections::BTreeSet;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Seek;
use std::io::SeekFrom;
use std::ops::Drop;

// Using `u8` as word, which has 8 bytes.
pub const BITS_PER_WORD: usize = 8;
pub const FULL_WORD: u8 = 255;

pub struct Bitmap {
  file: File,
  cache: Vec<u8>,
  free: BTreeSet<usize>,
}

impl Drop for Bitmap {
  fn drop(&mut self) {
    // Unable to handle errors on destruction.
    self.sync().log();
  }
}

impl Bitmap {
  pub fn new(path: &str) -> std::io::Result<Self> {
    Ok(Bitmap {
      file: OpenOptions::new()
          .read(true)
          .write(true)
          .create(true)
          .open(path)?,
      cache: Vec::new(),
      free: BTreeSet::new(),
    }).and_then(|mut bitmap| {
      bitmap.init()?;
      Ok(bitmap)
    })
  }

  pub fn len(&self) -> usize {
    self.data().len()
  }

  pub fn find(&self) -> usize {
    match self.free.iter().nth(0) {
      Some(&word_idx) => {
        let word = self.data()[word_idx];
        // It is safe to unwrap here, because |word| < |FULL_WORD| always
        // holds.
        let bit_idx = (0..BITS_PER_WORD).rev()
                          .skip_while(|x| word & (1 << x) > 0)
                          .nth(0)
                          .map(|x| BITS_PER_WORD - 1 - x)
                          .unwrap();
        word_idx * BITS_PER_WORD + bit_idx
      },
      None => self.len() * BITS_PER_WORD,
    }
  }

  // Sets the bit at |idx|.
  pub fn set_bit(&mut self, idx: usize, bit: bool) {
    let word_idx = idx / BITS_PER_WORD;
    let bit_idx = idx % BITS_PER_WORD;
    let mask = 1 << (BITS_PER_WORD - 1 - bit_idx);
    self.grow(word_idx + 1);
    if bit {
      self.data_mut()[word_idx] |= mask;
    } else {
      self.data_mut()[word_idx] &= !mask;
    }
    // Update the free set.
    if self.data()[word_idx] < FULL_WORD {
      self.free.insert(word_idx);
    } else {
      self.free.remove(&word_idx);
    }
  }

  // Gets the bit at |idx|.
  pub fn get_bit(&self, idx: usize) -> bool {
    let word_idx = idx / BITS_PER_WORD;
    if word_idx >= self.len() {
      return false;
    }
    let bit_idx = idx % BITS_PER_WORD;
    let mask = 1 << (BITS_PER_WORD - 1 - bit_idx);
    self.data()[word_idx] & mask > 0
  }

  // Gets the word at |word_idx|.
  pub fn get_word(&self, word_idx: usize) -> u8 {
    if word_idx < self.len() {
      self.data()[word_idx]
    } else {
      0
    }
  }

  // Compacts and persists to disk.
  pub fn sync(&mut self) -> std::io::Result<()> {
    self.compact();
    let size = self.cache.len();
    self.file.set_len(size as u64)?;
    self.file.seek(SeekFrom::Start(0))?;
    write(&mut self.file, self.cache.as_mut(), size)?;
    Ok(())
  }

  // Truncates the tailing zeros.
  pub fn compact(&mut self) {
    while let Some(&word) = self.cache.last() {
      if word == 0 && self.data().len() > 0 {
        self.cache.pop();
        self.free.remove(&self.data().len());
      } else {
        break;
      }
    }
  }

  fn init(&mut self) -> std::io::Result<()> {
    let size = self.file.metadata()?.len() as usize;
    if size > 0 {
      self.cache = vec![0; size];
      read(&mut self.file, self.cache.as_mut(), size)?;
    } else {
      self.cache = vec![0; CHECKSUM_SIZE];
    }
    // Initialize the free set.
    let iter = self.cache.iter().skip(CHECKSUM_SIZE);
    for (word_idx, &word) in iter.enumerate() {
      if word < FULL_WORD {
        self.free.insert(word_idx);
      }
    }
    Ok(())
  }

  fn grow(&mut self, to: usize) {
    if self.len() < to {
      self.cache.resize(to + CHECKSUM_SIZE, 0);
    }
  }

  fn data(&self) -> &[u8] {
    &self.cache[CHECKSUM_SIZE..]
  }

  fn data_mut(&mut self) -> &mut [u8] {
    &mut self.cache[CHECKSUM_SIZE..]
  }
}

#[cfg(test)]
mod tests {
  use crate::testing::file_deleter::FileDeleter;
  use super::*;

  #[test]
  fn set_and_get_bit() {
    let path = "/tmp/testfile.bitmap.1.db";

    // Test file deleter with RAII.
    let mut file_deleter = FileDeleter::new();
    file_deleter.push(&path);

    let result = Bitmap::new(&path);
    assert!(result.is_ok(), "Failed to create Bitmap");

    let mut bitmap = result.unwrap();
    assert_eq!(false, bitmap.get_bit(1234));
    assert_eq!(false, bitmap.get_bit(4321));
    assert_eq!(false, bitmap.get_bit(1024));

    bitmap.set_bit(1234, true);
    bitmap.set_bit(4321, true);
    bitmap.set_bit(1024, true);
    assert_eq!(true, bitmap.get_bit(1234));
    assert_eq!(true, bitmap.get_bit(4321));
    assert_eq!(true, bitmap.get_bit(1024));

    bitmap.set_bit(1234, false);
    bitmap.set_bit(1024, false);
    assert_eq!(false, bitmap.get_bit(1234));
    assert_eq!(true, bitmap.get_bit(4321));
    assert_eq!(false, bitmap.get_bit(1024));

    assert_eq!(541, bitmap.len());
  }

  #[test]
  fn len_and_compact() {
    let path = "/tmp/testfile.bitmap.2.db";

    // Test file deleter with RAII.
    let mut file_deleter = FileDeleter::new();
    file_deleter.push(&path);

    let result = Bitmap::new(&path);
    assert!(result.is_ok(), "Failed to create Bitmap");

    let mut bitmap = result.unwrap();
    bitmap.set_bit(1234, true);
    bitmap.set_bit(4321, true);
    assert_eq!(541, bitmap.len());

    bitmap.set_bit(4321, false);
    assert_eq!(541, bitmap.len());
    bitmap.compact();
    assert_eq!(155, bitmap.len());

    bitmap.set_bit(1234, false);
    assert_eq!(155, bitmap.len());
    bitmap.compact();
    assert_eq!(0, bitmap.len());
  }

  #[test]
  fn drop_new() {
    let path = "/tmp/testfile.bitmap.3.db";

    // Test file deleter with RAII.
    let mut file_deleter = FileDeleter::new();
    file_deleter.push(&path);

    {
      let result = Bitmap::new(&path);
      assert!(result.is_ok(), "Failed to create Bitmap");

      let mut bitmap = result.unwrap();
      bitmap.set_bit(1234, true);
      bitmap.set_bit(4321, true);
      assert_eq!(541, bitmap.len());

      bitmap.set_bit(4321, false);
      assert_eq!(541, bitmap.len());
    }  // Drops bitmap: compacts and persist to disk.

    {
      let result = Bitmap::new(&path);
      assert!(result.is_ok(), "Failed to create Bitmap");
  
      let mut bitmap = result.unwrap();
      assert_eq!(155, bitmap.len());
      assert_eq!(true, bitmap.get_bit(1234));
      assert_eq!(false, bitmap.get_bit(4321));

      bitmap.set_bit(1234, false);
      assert_eq!(155, bitmap.len());
    }  // Drops bitmap: compacts and persist to disk.

    {
      let result = Bitmap::new(&path);
      assert!(result.is_ok(), "Failed to create Bitmap");
  
      let bitmap = result.unwrap();
      assert_eq!(0, bitmap.len());
      assert_eq!(false, bitmap.get_bit(1234));
      assert_eq!(false, bitmap.get_bit(4321));
    }
  }
}