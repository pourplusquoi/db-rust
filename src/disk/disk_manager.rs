use crate::common::config::AtomicPageId;
use crate::common::config::PAGE_SIZE;
use crate::common::config::PageId;
use std::sync::atomic::Ordering;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;

// TODO: Right now, DiskManager does not support creating directories, i.e.
// the |db_file| being passed to |DiskManager::new| has to be under an existing
// directory. However, it might not be the DiskManager's responsibility to
// create directories.

pub struct DiskManager {
  db_io: File,
  next_page_id: AtomicPageId,
}

impl DiskManager {

  pub fn new(db_file: &str) -> std::io::Result<Self> {
    let disk_mgr = DiskManager {
      db_io: OpenOptions::new()
          .read(true)
          .write(true)
          .create(true)
          .open(db_file)?,
      next_page_id: AtomicPageId::new(0),
    };
    Ok(disk_mgr)
  }

  // Writes data to page with the specified page ID on disk.
  // The caller needs to ensure that page_id >= 0 and is valid.
  pub fn write_page(&mut self,
                    page_id: PageId,
                    data: &[u8]) -> std::io::Result<()> {
    let offset = (page_id as usize) * PAGE_SIZE;
    self.db_io.seek(SeekFrom::Start(offset as u64))?;
    Self::write_inl(&mut self.db_io, data, PAGE_SIZE)?;
    self.db_io.sync_data()?;
    Ok(())
  }

  // Reads data from page with the specified page ID on disk.
  // The caller needs to ensure that page_id >= 0 and is valid.
  pub fn read_page(&mut self,
                   page_id: PageId,
                   data: &mut [u8]) -> std::io::Result<()> {
    let offset = (page_id as usize) * PAGE_SIZE;
    self.db_io.seek(SeekFrom::Start(offset as u64))?;
    Self::read_inl(&mut self.db_io, data, PAGE_SIZE)?;
    Ok(())
  }

  pub fn allocate_page(&mut self) -> PageId {
    self.next_page_id.fetch_add(1, Ordering::SeqCst)
    // TODO: Allocate new page (operations like create index/table).
    // For now just keep an increasing counter.
  }

  pub fn deallocate_page(&mut self, page_id: PageId) {
    // TODO: Deallocate page (operations like drop index/table).
    // Need bitmap in header page for tracking pages.
  }

  fn write_inl(file: &mut File,
               data: &[u8],
               size: usize) -> std::io::Result<()> {
    let mut pos = 0;
    while pos < size {
      let bytes_written = file.write(&data[pos..])?;
      if bytes_written == 0 {
        return Err(Error::new(ErrorKind::WriteZero,
                              "I/O error: wrote 0 byte"));
      }
      pos += bytes_written;
    }
    Ok(())
  }

  fn read_inl(file: &mut File,
              data: &mut [u8],
              size: usize) -> std::io::Result<()> {
    let mut pos = 0;
    while pos < size {
      let bytes_read = file.read(&mut data[pos..])?;
      println!("Read {} bytes", bytes_read);
      if bytes_read == 0 {
        return Err(Error::new(ErrorKind::UnexpectedEof,
                              "I/O error: read 0 byte"));
      }
      pos += bytes_read;
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use crate::testing::file_deleter::FileDeleter;
  use super::*;

  #[test]
  fn disk_manager() {
    // Test file deleter with RAII.
    let mut file_deleter = FileDeleter::new();

    let file_path = "/tmp/testfile";
    file_deleter.push(&file_path);
    let result = DiskManager::new(&file_path);
    assert!(result.is_ok(), "Failed to create DiskManager");

    let mut disk_mgr = result.unwrap();
    let page_id = disk_mgr.allocate_page();
    assert_eq!(0, page_id);

    let mut data = String::with_capacity(PAGE_SIZE);
    let mut buffer = String::with_capacity(PAGE_SIZE);
    for i in 0..PAGE_SIZE {
      // Write some random bytes into `data`.
      data.push((i % 26 + 97) as u8 as char);
      // Reset the buffer.
      buffer.push('\0');
    }
    assert_eq!(PAGE_SIZE, data.len());
    assert_eq!(PAGE_SIZE, buffer.len());

    // Write the data to page on disk with specified page ID.
    let data_write: &[u8] = data.as_bytes();
    assert!(disk_mgr.write_page(page_id, data_write).is_ok());
    unsafe {
      // Reads the data from page on disk with the same page ID.
      let data_read: &mut [u8] = buffer.as_bytes_mut();
      assert!(disk_mgr.read_page(page_id, data_read).is_ok());
      // Make sure that the data written and the data read match.
      assert_eq!(data, buffer, "Data read differ from the data written");
    }
  }}
