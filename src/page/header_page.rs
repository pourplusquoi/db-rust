use crate::common::config::INVALID_PAGE_ID;
use crate::common::config::PAGE_SIZE;
use crate::common::config::PageId;
use crate::page::page::Page;

#[allow(dead_code)]
struct HeaderPage {
  data: [char; PAGE_SIZE],
  page_id: PageId,
  pin_count: i32,
  is_dirty: bool,
}

impl HeaderPage {
  pub fn new() -> Self {
    HeaderPage {
      data: [0 as char; PAGE_SIZE],
      page_id: INVALID_PAGE_ID,
      pin_count: 0,
      is_dirty: false,
    }
  }
}

impl Page for HeaderPage {
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
}