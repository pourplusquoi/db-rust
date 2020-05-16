use crate::common::config::INVALID_PAGE_ID;
use crate::common::config::PAGE_SIZE;
use crate::common::config::PageId;
use crate::page::page::Page;
use std::clone::Clone;

#[allow(dead_code)]
pub struct TablePage {
  data: [u8; PAGE_SIZE],
  page_id: PageId,
  pin_count: i32,
  is_dirty: bool,
}

// impl TablePage {
//   pub fn new() -> Self {
//     TablePage {
//       data: [0 as u8; PAGE_SIZE],
//       page_id: INVALID_PAGE_ID,
//       pin_count: 0,
//       is_dirty: false,
//     }
//   }
// }

impl Clone for TablePage {
  fn clone(&self) -> Self {
    TablePage {
      data: self.data,
      page_id: self.page_id,
      pin_count: self.pin_count,
      is_dirty: self.is_dirty,
    }
  }
}

impl Page for TablePage {
  fn new() -> Self {
    TablePage {
      data: [0 as u8; PAGE_SIZE],
      page_id: INVALID_PAGE_ID,
      pin_count: 0,
      is_dirty: false,
    }
  }

  fn borrow(&self) -> &[u8; PAGE_SIZE] {
    &self.data
  }

  fn borrow_mut(&mut self) -> &mut [u8; PAGE_SIZE] {
    &mut self.data
  }

  fn page_id(&self) -> PageId {
    self.page_id
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