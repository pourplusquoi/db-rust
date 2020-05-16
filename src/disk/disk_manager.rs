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

pub struct DiskManager {
  db_io: File,
  next_page_id: AtomicPageId,
}

impl DiskManager {

  pub fn new(db_file: &str) -> std::io::Result<Self> {
    let disk_mgr = DiskManager {
      db_io: OpenOptions::new().write(true).create(true).open(db_file)?,
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
    let next_page_id = self.next_page_id.fetch_add(1, Ordering::SeqCst);
    next_page_id - 1
  }

  pub fn deallocate_page(&self, page_id: PageId) {
    // TODO: Deallocate page (operations like drop index/table)
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
      if bytes_read == 0 {
        return Err(Error::new(ErrorKind::UnexpectedEof,
                              "I/O error: read 0 byte"));
      }
      pos += bytes_read;
    }
    Ok(())
  }
}