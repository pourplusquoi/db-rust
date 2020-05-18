// Format (size in byte):
//  -----------------------------------------------------------
// | Checksum (8) | RecordCount (4) | Entry_1 free_id (4) ... |
//  -----------------------------------------------------------

use crate::common::config::PAGE_SIZE;
use crate::common::config::PageId;
use crate::common::reinterpret;

pub struct ReservedPage {
  data: [u8; PAGE_SIZE],
  page_id: PageId,
}

impl ReservedPage {
  pub fn new() -> Self {
    ReservedPage {
      data: [0 as u8; PAGE_SIZE],
      page_id: 0,
    }
  }

  pub fn page_id(&self) -> PageId {
    self.page_id
  }

  pub fn data(&self) -> &[u8; PAGE_SIZE] {
    &self.data
  }

  pub fn data_mut(&mut self) -> &mut [u8; PAGE_SIZE] {
    &mut self.data
  }

  pub fn record_count(&self) -> usize {
    unsafe {
      reinterpret::read_u32(&self.data[8..]) as usize
    }
  }

  pub fn write_records(&mut self, free_ids: &Vec<PageId>) {
    unsafe {
      let mut offset = 8;
      // Assmusing |free_ids.len()| fits into u32.
      reinterpret::write_u32(&mut self.data[offset..], free_ids.len() as u32);
      // print!("Writing ");
      for &id in free_ids.iter() {
        // print!("id={} ", id);
        offset += 4;
        reinterpret::write_i32(&mut self.data[offset..], id);
      }
      // println!("");
    }
  }

  pub fn read_records(&self) -> Vec<PageId> {
    let mut free_ids = Vec::new();
    unsafe {
      let mut offset = 8;
      let size = reinterpret::read_u32(&self.data[offset..]);
      // print!("Reading ");
      for _ in 0..size {
        offset += 4;
        // print!("id={} ", reinterpret::read_i32(&self.data[offset..]));
        free_ids.push(reinterpret::read_i32(&self.data[offset..]));
      }
      // println!("");
    }
    free_ids
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn write_and_read() {
    let mut page = ReservedPage::new();
    assert_eq!(0, page.record_count());

    let vec1 = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    page.write_records(&vec1);
    assert_eq!(10, page.record_count());
    assert_eq!(vec1, page.read_records());
  
    let vec2 = vec![2048, 9999, -1, 0];
    page.write_records(&vec2);
    assert_eq!(4, page.record_count());
    assert_eq!(vec2, page.read_records());
  }
}