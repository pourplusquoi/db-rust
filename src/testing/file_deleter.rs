use std::fs;
use std::ops::Drop;

pub struct FileDeleter<'a> {
  file_paths: Vec<&'a str>,
}

impl<'a> FileDeleter<'a> {
  pub fn new() -> Self {
    FileDeleter {
      file_paths: Vec::new(),
    }
  }

  pub fn push(&mut self, path: &'a str) {
    self.file_paths.push(path);
  }
}

impl<'a> Drop for FileDeleter<'a> {
  fn drop(&mut self) {
    for path in self.file_paths.iter() {
      // Ignore errors.
      fs::remove_file(path);
    }
  }
}