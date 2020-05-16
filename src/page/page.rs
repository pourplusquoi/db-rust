use crate::common::config::PAGE_SIZE;
use crate::common::config::PageId;

// #[allow(dead_code)]
// pub struct Page {
//   data: [u8; PAGE_SIZE],
//   page_id: PageId,
//   pin_count: i32,
//   is_dirty: bool,
// }

pub trait Page {
  fn new() -> Self;
  fn borrow(&self) -> &[u8; PAGE_SIZE];
  fn borrow_mut(&mut self) -> &mut [u8; PAGE_SIZE];
  fn page_id(&self) -> PageId;
  fn page_id_mut(&mut self) -> &mut PageId;
  fn pin_count(&self) -> i32;
  fn pin_count_mut(&mut self) -> &mut i32;
  fn is_dirty(&self) -> bool;
  fn is_dirty_mut(&mut self) -> &mut bool;

  // Pins the page, increment the pin count by 1.
  fn pin(&mut self) {
    *self.pin_count_mut() += 1;
  }

  // Unpins the page, decrement the pin count by 1.
  // Returns false iff the pin count <= 0, which means that the page may not
  // be unpinned.
  fn unpin(&mut self) -> bool {
    if self.pin_count() <= 0 {
      false
    } else {
      *self.pin_count_mut() -= 1;
      true
    }
  }

  fn set_page_id(&mut self, page_id: PageId) {
    *self.page_id_mut() = page_id;
  }

  fn set_is_dirty(&mut self, is_dirty: bool) {
    *self.is_dirty_mut() = is_dirty;
  }
}