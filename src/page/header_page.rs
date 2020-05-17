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
use crate::common::newable::Newable;
use crate::page::page::Page;
use std::clone::Clone;

#[allow(dead_code)]
pub struct HeaderPage {
  data: [u8; PAGE_SIZE],
  page_id: PageId,
  pin_count: i32,
  is_dirty: bool,
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