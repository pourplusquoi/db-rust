use crate::common::config::PAGE_SIZE;
use crate::common::config::PageId;

// #[allow(dead_code)]
// pub struct Page {
//   data: [char; PAGE_SIZE],
//   page_id: PageId,
//   pin_count: i32,
//   is_dirty: bool,
// }

pub trait Page {
  fn new() -> Self;
  fn borrow(&self) -> &[char; PAGE_SIZE];
  fn borrow_mut(&mut self) -> &mut [char; PAGE_SIZE];
  fn page_id(&self) -> PageId;
  fn pin_count(&self) -> i32;
  fn pin_count_mut(&mut self) -> &mut i32;
  fn is_dirty(&self) -> bool;
  fn is_dirty_mut(&mut self) -> &mut bool;

  fn pin(&mut self) {
    *self.pin_count_mut() += 1;
  }

  fn unpin(&mut self) {
    *self.pin_count_mut() -= 1;
  }

  fn set_dirty(&mut self, is_dirty: bool) {
    *self.is_dirty_mut() = is_dirty;
  }
}