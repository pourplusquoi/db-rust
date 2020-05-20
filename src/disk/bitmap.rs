#![allow(dead_code)]

use crate::logging::error_logging::ErrorLogging;
use std::fs::File;
use std::fs::OpenOptions;
use std::ops::Drop;

struct Bitmap {
  file: File,
  cache: Vec<u8>,
}

impl Drop for Bitmap {
  fn drop(&mut self) {
    self.truncate();
    self.sync().log();
  }
}

impl Bitmap {
  pub fn new(path: &str) -> std::io::Result<Self> {
    // TODO: Implement this.
    Ok(Bitmap {
      file: OpenOptions::new()
          .read(true)
          .write(true)
          .create(true)
          .open(path)?,
      cache: Vec::new(),
    }).and_then(|mut bitmap| {
      bitmap.init()?;
      Ok(bitmap)
    })
  }

  fn init(&mut self) -> std::io::Result<()> {
    // TODO: Implement this. Read all.
    Ok(())
  }

  pub fn set_bit(idx: usize, bit: bool) {
    // TODO: Implement this.
  }

  pub fn get_bit() -> bool {
    // TODO: Implement this.
    false
  }

  pub fn sync(&self) -> std::io::Result<()> {
    // TODO: Implement this. Persist to disk.
    Ok(())
  }

  fn grow(&mut self) {
    // TODO: Implement this.
  }

  fn truncate(&mut self) {
    // TODO: Implement this. Truncates the tailing zeros.
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
  }

  #[test]
  fn drop_new() {
    let path = "/tmp/testfile.bitmap.1.db";

    // Test file deleter with RAII.
    let mut file_deleter = FileDeleter::new();
    file_deleter.push(&path);

    {
      let result = Bitmap::new(&path);
      assert!(result.is_ok(), "Failed to create Bitmap");

      let mut bitmap = result.unwrap();
    }  // Drops bitmap.

    {
      let result = Bitmap::new(&path);
      assert!(result.is_ok(), "Failed to create Bitmap");
  
      let mut bitmap = result.unwrap();
    }  // Drops bitmap.
  }
}