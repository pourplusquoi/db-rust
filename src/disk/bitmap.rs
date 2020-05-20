use std::fs::File;
use std::fs::OpenOptions;

struct Bitmap {
  file: File,
  cache: Vec<u8>,
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
    // TODO: Implement this.
    Ok(())
  }

  fn grow() {}

  fn truncate() {}
}