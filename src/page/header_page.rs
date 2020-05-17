// Database use the first page (page_id = 0) as header page to store metadata, in
// our case, we will contain information about table/index name (length less than
// 32 bytes) and their corresponding root_id
//
// Format (size in byte):
//  -----------------------------------------------------------------
// | RecordCount (4) | Entry_1 name (32) | Entry_1 root_id (4) | ... |
//  -----------------------------------------------------------------

use crate::common::config::INVALID_PAGE_ID;
use crate::common::config::PAGE_SIZE;
use crate::common::config::PageId;
use crate::common::error::*;
use crate::common::newable::Newable;
use crate::page::page::Page;
use crate::page::reinterpret;
use std::clone::Clone;

#[allow(dead_code)]
pub struct HeaderPage {
  data: [u8; PAGE_SIZE],
  page_id: PageId,
  pin_count: i32,
  is_dirty: bool,
}

impl HeaderPage {
  pub fn init(&mut self) {
    self.set_record_count(0);
  }

  pub fn insert_record(&mut self, name: &str,
                       root_id: PageId) -> std::io::Result<()> {
    Self::validate_name(name)?;
    if self.find_record(name).is_ok() {
      return Err(already_exists(&format!("Record exists; name = {}", name)));
    }
    let count = self.get_record_count();
    let offset = 4 + count * 36;
    unsafe {
      reinterpret::write_str(&mut self.data_mut()[offset..], name);
      reinterpret::write_i32(&mut self.data_mut()[(offset + 32)..], root_id);
    }
    self.set_record_count(count + 1);
    Ok(())
  }

  pub fn delete_record(&mut self, name: &str) -> std::io::Result<()> {
    Self::validate_name(name)?;
    let idx = self.find_record(name)?;
    let count = self.get_record_count();
    let offset = 4 + idx * 36;
    let n = (count - idx - 1) * 36;
    unsafe {
      let ptr = self.data_mut().as_mut_ptr().add(offset);
      for i in 0..n {
        *ptr.add(i) = *ptr.add(i + 36);
      }
    }
    self.set_record_count(count - 1);
    Ok(())
  }

  pub fn update_record(&mut self, name: &str,
                       root_id: PageId) -> std::io::Result<()> {
    Self::validate_name(name)?;
    let idx = self.find_record(name)?;
    let offset = 4 + idx * 36;
    unsafe {
      reinterpret::write_i32(&mut self.data_mut()[(offset + 32)..], root_id);
    }
    Ok(())
  }

  pub fn get_root_id(&self, name: &str) -> std::io::Result<i32> {
    Self::validate_name(name)?;
    let idx = self.find_record(name)?;
    let offset = (idx + 1) * 36;
    let root_id = unsafe {
      reinterpret::read_i32(&self.data[offset..])
    };
    Ok(root_id)
  }

  pub fn get_record_count(&self) -> usize {
    unsafe {
      reinterpret::read_u32(self.data()) as usize
    }
  }

  fn find_record(&self, name: &str) -> std::io::Result<usize> {
    for i in 0..self.get_record_count() {
      unsafe {
        let raw_name = reinterpret::read_str(&self.data()[(4 + i * 36)..]);
        if raw_name == name {
          return Ok(i);
        }
      }
    }
    Err(not_found("Record not found"))
  }

  fn set_record_count(&mut self, record_count: usize) {
    unsafe {
      // Assuming |record_count| fits in u32.
      reinterpret::write_u32(self.data_mut(), record_count as u32);
    }
  }

  fn validate_name(name: &str) -> std::io::Result<()> {
    if name.len() > 32 {
      Err(invalid_input("Name length should be <= 32"))
    } else {
      Ok(())
    }
  }
}

impl Clone for HeaderPage {
  fn clone(&self) -> Self {
    HeaderPage {
      data: self.data,
      page_id: self.page_id,
      pin_count: self.pin_count,
      is_dirty: self.is_dirty,
    }
  }
}

impl Newable for HeaderPage {
  fn new() -> Self {
    HeaderPage {
      data: [0 as u8; PAGE_SIZE],
      page_id: INVALID_PAGE_ID,
      pin_count: 0,
      is_dirty: false,
    }
  }
}

impl Page for HeaderPage {
  fn data(&self) -> &[u8; PAGE_SIZE] {
    &self.data
  }

  fn data_mut(&mut self) -> &mut [u8; PAGE_SIZE] {
    &mut self.data
  }

  fn page_id(&self) -> PageId {
    self.page_id
  }

  fn page_id_mut(&mut self) -> &mut PageId {
    &mut self.page_id
  }

  fn pin_count(&self) -> i32 {
    self.pin_count
  }

  fn pin_count_mut(&mut self) -> &mut i32 {
    &mut self.pin_count
  }

  fn is_dirty(&self) -> bool {
    self.is_dirty
  }

  fn is_dirty_mut(&mut self) -> &mut bool {
    &mut self.is_dirty
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn header_page_test() {
    let mut header_page = HeaderPage::new();
    assert_eq!(0, header_page.get_record_count());
    assert!(header_page.get_root_id("Table A").is_err());

    assert!(header_page.insert_record("Table A", 12).is_ok());
    assert!(header_page.insert_record("Table B", 0).is_ok());
    assert!(header_page.insert_record("Table C", -1).is_ok());
    assert_eq!(12, header_page.get_root_id("Table A").unwrap());
    assert_eq!(0, header_page.get_root_id("Table B").unwrap());
    assert_eq!(-1, header_page.get_root_id("Table C").unwrap());
    assert_eq!(3, header_page.get_record_count());

    assert!(header_page.insert_record("Table A", 25).is_err());
    assert!(header_page.update_record("Table D", 7).is_err());

    assert!(header_page.update_record("Table A", 27).is_ok());
    assert!(header_page.update_record("Table B", 50).is_ok());
    assert!(header_page.update_record("Table C", 94).is_ok());
    assert_eq!(27, header_page.get_root_id("Table A").unwrap());
    assert_eq!(50, header_page.get_root_id("Table B").unwrap());
    assert_eq!(94, header_page.get_root_id("Table C").unwrap());
    assert_eq!(3, header_page.get_record_count());

    assert!(header_page.delete_record("Table A").is_ok());
    assert!(header_page.delete_record("Table B").is_ok());
    assert!(header_page.delete_record("Table D").is_err());
    assert!(header_page.get_root_id("Table A").is_err());
    assert!(header_page.get_root_id("Table B").is_err());
    assert_eq!(94, header_page.get_root_id("Table C").unwrap());
    assert_eq!(1, header_page.get_record_count());

    assert!(header_page.insert_record("Table A", 64).is_ok());
    assert_eq!(64, header_page.get_root_id("Table A").unwrap());
    assert_eq!(2, header_page.get_record_count());
  }
}