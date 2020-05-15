use crate::common::config::INVALID_PAGE_ID;
use crate::common::config::PAGE_SIZE;
use crate::common::config::PageId;
use crate::page::page::Page;
use std::clone::Clone;

#[allow(dead_code)]
pub struct HeaderPage {
  data: [char; PAGE_SIZE],
  page_id: PageId,
  pin_count: i32,
  is_dirty: bool,
}

// impl HeaderPage {
//   pub fn new() -> Self {
//     HeaderPage {
//       data: [0 as char; PAGE_SIZE],
//       page_id: INVALID_PAGE_ID,
//       pin_count: 0,
//       is_dirty: false,
//     }
//   }
// }

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

impl Page for HeaderPage {
  fn new() -> Self {
    HeaderPage {
      data: [0 as char; PAGE_SIZE],
      page_id: INVALID_PAGE_ID,
      pin_count: 0,
      is_dirty: false,
    }
  }

  fn borrow(&self) -> &[char; PAGE_SIZE] {
    &self.data
  }

  fn borrow_mut(&mut self) -> &mut [char; PAGE_SIZE] {
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