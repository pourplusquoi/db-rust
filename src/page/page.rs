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
  fn borrow(&self) -> &[char; PAGE_SIZE];
  fn borrow_mut(&mut self) -> &mut [char; PAGE_SIZE];
  fn page_id(&self) -> PageId;
  fn pin_count(&self) -> i32;
}