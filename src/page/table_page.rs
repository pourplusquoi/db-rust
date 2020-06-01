// Slotted page format:
//  ---------------------------------------
// | HEADER | ... FREE SPACES ... | TUPLES |
//  ---------------------------------------
//                                 ^
//                         free space pointer
//
//  Header format (size in byte):
//  ---------------------------------------------------------------------
// | PageId (4) | PrevPageId (4) | NextPageId (4) | FreeSpacePointer (4) |
//  ---------------------------------------------------------------------
//  --------------------------------------------------------------
// | TupleCount (4) | Tuple_1 offset (4) | Tuple_1 size (4) | ... |
//  --------------------------------------------------------------

use crate::common::config::PageId;
use crate::common::config::INVALID_PAGE_ID;
use crate::common::config::PAGE_SIZE;
use crate::page::page::Page;
use crate::table::tuple::Tuple;
use std::clone::Clone;
use std::default::Default;

#[allow(dead_code)]
#[derive(Clone)]
pub struct TablePage {
    data: [u8; PAGE_SIZE],
    page_id: PageId,
    pin_count: i32,
    is_dirty: bool,
}

impl TablePage {
    // TODO: Implement this.
    pub fn page_id() -> PageId {
        0
    }

    // TODO: Implement this.
    pub fn prev_page_id() -> PageId {
        0
    }

    // TODO: Implement this.
    pub fn next_page_id() -> PageId {
        0
    }

    // TODO: Implement this.
    pub fn set_prev_page_id(&mut self, page_id: PageId) {}

    // TODO: Implement this.
    pub fn set_next_page_id(&mut self, page_id: PageId) {}

    // TODO: Implement this.
    pub fn insert_tuple(&mut self) -> bool {
        false
    }

    // TODO: Implement this.
    pub fn mark_delete(&mut self) -> bool {
        false
    }

    // TODO: Implement this.
    pub fn update_tuple(&mut self) -> bool {
        false
    }

    // TODO: Implement this.
    pub fn apply_delete(&mut self) {}

    // TODO: Implement this.
    pub fn rollback_delete(&mut self) {}

    // TODO: Implement this.
    pub fn get_tuple(&self) -> Option<Tuple> {
        None
    }
}

impl Default for TablePage {
    fn default() -> Self {
        TablePage {
            data: [0 as u8; PAGE_SIZE],
            page_id: INVALID_PAGE_ID,
            pin_count: 0,
            is_dirty: false,
        }
    }
}

impl Page for TablePage {
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
mod tests {}
