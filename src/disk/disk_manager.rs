// Disk manager takes care of the allocation and deallocation of pages within a
// database. It also performs read and write of pages to and from disk, and
// provides a logical file layer within the context of a database management
// system. Page ID is allocated from 1, the page 0 is reserved.

use crate::common::config::PAGE_SIZE;
use crate::common::config::PageId;
use crate::common::config::INVALID_PAGE_ID;
use crate::common::error::*;
use crate::common::reinterpret;
use crate::logging::error_logging::ErrorLogging;
use crate::page::reserved_page::ReservedPage;
use std::collections::HashSet;
use std::collections::hash_map::DefaultHasher;
use std::fs;
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
use std::ops::Drop;

// TODO: Right now, DiskManager does not support creating directories, i.e.
// the |db_file| being passed to |DiskManager::new| has to be under an existing
// directory. However, it might not be the DiskManager's responsibility to
// create directories.

pub struct DiskManager {
  db_io: File,
  metadata: Metadata,
}

impl Drop for DiskManager {
  fn drop(&mut self) {
    self.metadata.drop();
    // Unable to handle errors on destruction.
    self.db_io.seek(SeekFrom::Start(0)).log();
    Self::write_inl(&mut self.db_io,
                    self.metadata.reserved.data_mut(),
                    PAGE_SIZE).log();
  }
}

impl DiskManager {
  pub fn new(db_file: &str) -> std::io::Result<Self> {
    Ok(DiskManager {
      db_io: OpenOptions::new()
          .read(true)
          .write(true)
          .create(true)
          .open(db_file)?,
      metadata: Metadata::new(),
    }).and_then(|mut disk_mgr| {
      disk_mgr.init(db_file)?;
      Ok(disk_mgr)
    })
  }

  fn init(&mut self, db_file: &str) -> std::io::Result<()> {
    // Read only if the file is not empty.
    if fs::metadata(db_file)?.len() > 0 {
      Self::read_inl(&mut self.db_io,
                     self.metadata.reserved.data_mut(),
                     PAGE_SIZE)?;
    }
    let mut records = self.metadata.reserved.read_records();
    if records.is_empty() {
      self.metadata.next_free = 1;
    } else {
      self.metadata.next_free = records.pop().unwrap();
    }
    self.metadata.free_pages = records.drain(..).collect();
    Ok(())
  }

  // Writes data to page with the specified page ID on disk.
  // The caller needs to ensure that page_id >= 1 and is valid.
  pub fn write_page(&mut self,
                    page_id: PageId,
                    data: &mut [u8]) -> std::io::Result<()> {
    let offset = (page_id as usize) * PAGE_SIZE;
    self.db_io.seek(SeekFrom::Start(offset as u64))?;
    Self::write_inl(&mut self.db_io, data, PAGE_SIZE)?;
    self.db_io.sync_data()?;
    Ok(())
  }

  // Reads data from page with the specified page ID on disk.
  // The caller needs to ensure that page_id >= 1 and is valid.
  pub fn read_page(&mut self,
                   page_id: PageId,
                   data: &mut [u8]) -> std::io::Result<()> {
    let offset = (page_id as usize) * PAGE_SIZE;
    self.db_io.seek(SeekFrom::Start(offset as u64))?;
    Self::read_inl(&mut self.db_io, data, PAGE_SIZE)?;
    Ok(())
  }

  pub fn allocate_page(&mut self) -> PageId {
    self.metadata.find_free()
  }

  pub fn deallocate_page(&mut self, page_id: PageId) {
    self.metadata.insert_free(page_id);
  }

  fn write_inl(file: &mut File,
               data: &mut [u8],
               size: usize) -> std::io::Result<()> {
    update_checksum(data);
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
      if bytes_read == 0 {
        return Err(Error::new(ErrorKind::UnexpectedEof,
                              "I/O error: read 0 byte"));
      }
      pos += bytes_read;
    }
    validate_checksum(data)?;
    Ok(())
  }
}

struct Metadata {
  free_pages: HashSet<PageId>,
  next_free: PageId,
  reserved: ReservedPage,
}

impl Metadata {
  pub fn new() -> Self {
    Metadata {
      free_pages: HashSet::new(),
      next_free: INVALID_PAGE_ID,
      reserved: ReservedPage::new(),
    }
  }

  pub fn insert_free(&mut self, id: PageId) {
    self.free_pages.insert(id);
    while self.free_pages.remove(&(self.next_free - 1)) {
      self.next_free -= 1;
    }
  }

  pub fn find_free(&mut self) -> PageId {
    match self.free_pages.iter().nth(0).map(|x| *x) {
      Some(id) => {
        self.free_pages.remove(&id);
        id
      },
      None => {
        let id = self.next_free;
        self.next_free += 1;
        id
      },
    }
  }

  fn drop(&mut self) {
    let mut vec: Vec<_> = self.free_pages.drain().collect();
    vec.push(self.next_free);
    self.reserved.write_records(&vec);
  }
}

fn update_checksum(data: &mut [u8]) {
  reinterpret::write_u64(data, compute_checksum(&data[8..]));
}

fn validate_checksum(data: &[u8]) -> std::io::Result<()> {
  match reinterpret::read_u64(data) == compute_checksum(&data[8..]) {
    true => Ok(()),
    false => Err(invalid_data("Page corrupted")),
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
    let file_path = "/tmp/disk_manager.1.testfile";

    // Test file deleter with RAII.
    let mut file_deleter = FileDeleter::new();
    file_deleter.push(&file_path);

    let result = DiskManager::new(&file_path);
    assert!(result.is_ok(), "Failed to create DiskManager");

    let mut disk_mgr = result.unwrap();
    let page_id = disk_mgr.allocate_page();
    assert_eq!(1, page_id, "Page 0 is reserved");

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
    let file_path = "/tmp/disk_manager.2.testfile";

    // Test file deleter with RAII.
    let mut file_deleter = FileDeleter::new();
    file_deleter.push(&file_path);

    let page_id = 1;
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
    let file_path = "/tmp/disk_manager.3.testfile";

    // Test file deleter with RAII.
    let mut file_deleter = FileDeleter::new();
    file_deleter.push(&file_path);

    {
      let result = DiskManager::new(&file_path);
      assert!(result.is_ok(), "Failed to create DiskManager");

      let mut disk_mgr = result.unwrap();
      assert_eq!(1, disk_mgr.allocate_page());
      assert_eq!(2, disk_mgr.allocate_page());
      assert_eq!(3, disk_mgr.allocate_page());
      assert_eq!(4, disk_mgr.allocate_page());
      assert_eq!(5, disk_mgr.allocate_page());

      disk_mgr.deallocate_page(2);
      assert_eq!(2, disk_mgr.allocate_page());
      disk_mgr.deallocate_page(3);
      disk_mgr.deallocate_page(3);
      assert_eq!(3, disk_mgr.allocate_page());
      assert_eq!(6, disk_mgr.allocate_page());

      disk_mgr.deallocate_page(1);
      disk_mgr.deallocate_page(2);
      disk_mgr.deallocate_page(3);
      disk_mgr.deallocate_page(4);
      disk_mgr.deallocate_page(5);
      disk_mgr.deallocate_page(6);
      assert_eq!(1, disk_mgr.allocate_page());
      assert_eq!(2, disk_mgr.allocate_page());
      assert_eq!(3, disk_mgr.allocate_page());
      assert_eq!(4, disk_mgr.allocate_page());
      assert_eq!(5, disk_mgr.allocate_page());
    }  // Drops disk_mgr.

    {
      let result = DiskManager::new(&file_path);
      assert!(result.is_ok(), "Failed to create DiskManager");

      let mut disk_mgr = result.unwrap();
      assert_eq!(6, disk_mgr.allocate_page());
      assert_eq!(7, disk_mgr.allocate_page());
      assert_eq!(8, disk_mgr.allocate_page());

      disk_mgr.deallocate_page(1);
      disk_mgr.deallocate_page(8);
      disk_mgr.deallocate_page(7);
      disk_mgr.deallocate_page(6);
    }  // Drops disk_mgr.

    {
      let result = DiskManager::new(&file_path);
      assert!(result.is_ok(), "Failed to create DiskManager");

      let mut disk_mgr = result.unwrap();
      assert_eq!(1, disk_mgr.allocate_page());
      assert_eq!(6, disk_mgr.allocate_page());
      assert_eq!(7, disk_mgr.allocate_page());
      assert_eq!(8, disk_mgr.allocate_page());
      assert_eq!(9, disk_mgr.allocate_page());
    }  // Drops disk_mgr.
  }
}