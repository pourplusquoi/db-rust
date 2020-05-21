// Disk manager takes care of the allocation and deallocation of pages within a
// database. It also performs read and write of pages to and from disk, and
// provides a logical file layer within the context of a database management
// system. Page ID is allocated from 0.

use crate::common::config::PAGE_SIZE;
use crate::common::config::PageId;
use crate::common::error::*;
use crate::common::reinterpret;
use crate::disk::bitmap::Bitmap;
use std::collections::hash_map::DefaultHasher;
use std::fs::File;
use std::fs::OpenOptions;
use std::hash::Hash;
use std::hash::Hasher;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;

pub const BITMAP_FILE_SUFFIX: &'static str = ".bitmap";

// TODO: Right now, DiskManager does not support creating directories, i.e.
// the |db_file| being passed to |DiskManager::new| has to be under an existing
// directory. However, it might not be the DiskManager's responsibility to
// create directories.

pub struct DiskManager {
  db_io: File,
  bitmap: Bitmap,
}

impl DiskManager {
  pub fn new(db_file: &str) -> std::io::Result<Self> {
    let bitmap_file = db_file.to_string() + BITMAP_FILE_SUFFIX;
    Ok(DiskManager {
      db_io: OpenOptions::new()
          .read(true)
          .write(true)
          .create(true)
          .open(db_file)?,
      bitmap: Bitmap::new(&bitmap_file)?,
    })
  }

  // Writes data to page with the specified page ID on disk.
  // The caller needs to ensure that page_id >= 1 and is valid.
  pub fn write_page(&mut self,
                    page_id: PageId,
                    data: &mut [u8]) -> std::io::Result<()> {
    let offset = (page_id as u64) * (PAGE_SIZE as u64);
    self.db_io.seek(SeekFrom::Start(offset))?;
    write(&mut self.db_io, data, PAGE_SIZE)?;
    self.db_io.sync_data()?;
    Ok(())
  }

  // Reads data from page with the specified page ID on disk.
  // The caller needs to ensure that page_id >= 1 and is valid.
  pub fn read_page(&mut self,
                   page_id: PageId,
                   data: &mut [u8]) -> std::io::Result<()> {
    if !self.bitmap.get_bit(page_id as usize) {
      return Err(invalid_input(
          &format!("The page is not allocated; page_id = {}", page_id)));
    }

    // Extend the file length when the page is at the tail.
    let offset = (page_id as u64) * (PAGE_SIZE as u64);
    if offset == self.db_io.metadata()?.len() {
      self.db_io.set_len(offset + PAGE_SIZE as u64)?;
    }

    self.db_io.seek(SeekFrom::Start(offset))?;
    read(&mut self.db_io, data, PAGE_SIZE)?;
    Ok(())
  }

  pub fn allocate_page(&mut self) -> PageId {
    let idx = self.bitmap.find();
    self.bitmap.set_bit(idx, true);
    idx as PageId
  }

  // |HEADER_PAGE_ID| is the smallest possible page ID. Therefore, the caller
  // needs to ensure that |page_id| >= |HEADER_PAGE_ID|.
  pub fn deallocate_page(&mut self, page_id: PageId) {
    self.bitmap.set_bit(page_id as usize, false);
  }
}

pub fn write(file: &mut File,
             data: &mut [u8],
             size: usize) -> std::io::Result<()> {
  update_checksum(data)?;
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

pub fn read(file: &mut File,
            data: &mut [u8],
            size: usize) -> std::io::Result<()> {
  let mut pos = 0;
  while pos < size {
    let bytes_read = file.read(&mut data[pos..])?;
    if bytes_read == 0 {
      return Err(Error::new(ErrorKind::UnexpectedEof,
                            "I/O error: read 0 byte"));
    }
    pos += bytes_read;
  }
  validate_checksum(data)?;
  Ok(())
}

fn update_checksum(data: &mut [u8]) -> std::io::Result<()> {
  if data.len() < 8 {
    return Err(invalid_input("Data length should >= 8"));
  }
  reinterpret::write_u64(data, compute_checksum(&data[8..]));
  Ok(())
}

fn validate_checksum(data: &[u8]) -> std::io::Result<()> {
  if data.len() < 8 {
    return Err(invalid_input("Data length should >= 8"));
  }
  let checksum = reinterpret::read_u64(data);
  if checksum == 0 {
    return Ok(());  // The page is empty, it is a success.
  }
  match checksum == compute_checksum(&data[8..]) {
    true => Ok(()),
    false => Err(invalid_data("Data corrupted")),
  }
}

fn compute_checksum(data: &[u8]) -> u64 {
  let mut hasher = DefaultHasher::new();
  data.hash(&mut hasher);
  hasher.finish()
}

#[cfg(test)]
mod tests {
  use crate::testing::file_deleter::FileDeleter;
  use super::*;

  #[test]
  fn disk_manager() {
    let file_path = "/tmp/testfile.disk_manager.1.db";
    let bitmap_path = file_path.to_string() + BITMAP_FILE_SUFFIX;

    // Test file deleter with RAII.
    let mut file_deleter = FileDeleter::new();
    file_deleter.push(&file_path);
    file_deleter.push(&bitmap_path);

    let result = DiskManager::new(&file_path);
    assert!(result.is_ok(), "Failed to create DiskManager");

    let mut disk_mgr = result.unwrap();
    let page_id = disk_mgr.allocate_page();
    assert_eq!(0, page_id, "Page should start from 0");

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

    unsafe {
      // Write the data to page on disk with specified page ID.
      let data_write: &mut [u8] = data.as_bytes_mut();
      assert!(disk_mgr.write_page(page_id, data_write).is_ok());
      // Reads the data from page on disk with the same page ID.
      let data_read: &mut [u8] = buffer.as_bytes_mut();
      assert!(disk_mgr.read_page(page_id, data_read).is_ok());
    }

    // Make sure that the data written and the data read match.
    assert_eq!(data[8..], buffer[8..],
               "Data read differ from the data written");
    assert_eq!(reinterpret::read_u64(buffer[0..8].as_bytes()),
               compute_checksum(data[8..].as_bytes()),
               "Checksum is set incorrectly");
  }

  #[test]
  fn drop_new() {
    let file_path = "/tmp/testfile.disk_manager.2.db";
    let bitmap_path = file_path.to_string() + BITMAP_FILE_SUFFIX;

    // Test file deleter with RAII.
    let mut file_deleter = FileDeleter::new();
    file_deleter.push(&file_path);
    file_deleter.push(&bitmap_path);

    let page_id: PageId;
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

    {
      let result = DiskManager::new(&file_path);
      assert!(result.is_ok(), "Failed to create DiskManager");

      let mut disk_mgr = result.unwrap();
      page_id = disk_mgr.allocate_page();
      unsafe {
        // Write the data to page on disk with specified page ID.
        let data_write: &mut [u8] = data.as_bytes_mut();
        assert!(disk_mgr.write_page(page_id, data_write).is_ok());
      }
    }  // Drops disk_mgr.

    {
      let result = DiskManager::new(&file_path);
      assert!(result.is_ok(), "Failed to create DiskManager");

      let mut disk_mgr = result.unwrap();
      unsafe {
        // Reads the data from page on disk with the same page ID.
        let data_read: &mut [u8] = buffer.as_bytes_mut();
        assert!(disk_mgr.read_page(page_id, data_read).is_ok());
      }

      // Make sure that the data written and the data read match.
      assert_eq!(data[8..], buffer[8..],
                 "Data read differ from the data written");
      assert_eq!(reinterpret::read_u64(buffer[0..8].as_bytes()),
                 compute_checksum(data[8..].as_bytes()),
                 "Checksum is set incorrectly");
    }  // Drops disk_mgr.
  }

  #[test]
  fn allocate_deallocate() {
    let file_path = "/tmp/testfile.disk_manager.3.db";
    let bitmap_path = file_path.to_string() + BITMAP_FILE_SUFFIX;

    // Test file deleter with RAII.
    let mut file_deleter = FileDeleter::new();
    file_deleter.push(&file_path);
    file_deleter.push(&bitmap_path);

    {
      let result = DiskManager::new(&file_path);
      assert!(result.is_ok(), "Failed to create DiskManager");

      let mut disk_mgr = result.unwrap();
      assert_eq!(0, disk_mgr.allocate_page());
      assert_eq!(1, disk_mgr.allocate_page());
      assert_eq!(2, disk_mgr.allocate_page());
      assert_eq!(3, disk_mgr.allocate_page());
      assert_eq!(4, disk_mgr.allocate_page());

      disk_mgr.deallocate_page(2);
      assert_eq!(2, disk_mgr.allocate_page());
      disk_mgr.deallocate_page(3);
      disk_mgr.deallocate_page(3);
      assert_eq!(3, disk_mgr.allocate_page());
      assert_eq!(5, disk_mgr.allocate_page());

      disk_mgr.deallocate_page(0);
      disk_mgr.deallocate_page(4);
      assert_eq!(0, disk_mgr.allocate_page());
      assert_eq!(4, disk_mgr.allocate_page());
      disk_mgr.deallocate_page(1);
      disk_mgr.deallocate_page(2);
      disk_mgr.deallocate_page(3);
      disk_mgr.deallocate_page(5);
      assert_eq!(1, disk_mgr.allocate_page());
      assert_eq!(2, disk_mgr.allocate_page());
      assert_eq!(3, disk_mgr.allocate_page());
    }  // Drops disk_mgr.

    {
      let result = DiskManager::new(&file_path);
      assert!(result.is_ok(), "Failed to create DiskManager");

      let mut disk_mgr = result.unwrap();
      assert_eq!(5, disk_mgr.allocate_page());
      assert_eq!(6, disk_mgr.allocate_page());
      assert_eq!(7, disk_mgr.allocate_page());

      disk_mgr.deallocate_page(0);
      disk_mgr.deallocate_page(7);
      disk_mgr.deallocate_page(6);
      disk_mgr.deallocate_page(5);
    }  // Drops disk_mgr.

    {
      let result = DiskManager::new(&file_path);
      assert!(result.is_ok(), "Failed to create DiskManager");

      let mut disk_mgr = result.unwrap();
      assert_eq!(0, disk_mgr.allocate_page());
      assert_eq!(5, disk_mgr.allocate_page());
      assert_eq!(6, disk_mgr.allocate_page());
      assert_eq!(7, disk_mgr.allocate_page());
      assert_eq!(8, disk_mgr.allocate_page());
    }  // Drops disk_mgr.
  }
}