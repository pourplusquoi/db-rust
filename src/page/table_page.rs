// Slotted page format:
//  ---------------------------------------
// | HEADER | ... FREE SPACES ... | TUPLES |
//  ---------------------------------------
//                                 ^
//                         free space pointer
//
//  Header format (size in byte):
//  ---------------------------------------------------------------------------------------------
// | Checksum (8) | PageId (4) | LSN (4) | PrevPageId (4) | NextPageId (4) | FreeSpacePointer(8) |
//  ---------------------------------------------------------------------------------------------
//  --------------------------------------------------------------
// | TupleCount (8) | Tuple_1 offset (8) | Tuple_1 size (8) | ... |
//  --------------------------------------------------------------

use crate::common::config::PageId;
use crate::common::config::CHECKSUM_SIZE;
use crate::common::config::INVALID_PAGE_ID;
use crate::common::config::PAGE_SIZE;
use crate::common::reinterpret;
use crate::common::rid::Rid;
use crate::page::page::Page;
use crate::table::tuple::Tuple;
use std::clone::Clone;
use std::default::Default;

const PAGE_ID_OFFSET: usize = CHECKSUM_SIZE;
const PREV_PAGE_ID_OFFSET: usize = CHECKSUM_SIZE + 8;
const NEXT_PAGE_ID_OFFSET: usize = CHECKSUM_SIZE + 12;
const FREE_SPACE_PTR_OFFSET: usize = CHECKSUM_SIZE + 16;
const TUPLE_COUNT_OFFSET: usize = CHECKSUM_SIZE + 24;
const DATA_OFFSET: usize = CHECKSUM_SIZE + 32;

#[derive(Clone)]
pub struct TablePage {
    data: [u8; PAGE_SIZE],
    pin_count: i32,
    is_dirty: bool,
}

impl TablePage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn prev_page_id(&self) -> PageId {
        reinterpret::read_i32(&self.data[PREV_PAGE_ID_OFFSET..])
    }

    pub fn next_page_id(&self) -> PageId {
        reinterpret::read_i32(&self.data[NEXT_PAGE_ID_OFFSET..])
    }

    pub fn set_prev_page_id(&mut self, page_id: PageId) {
        reinterpret::write_i32(&mut self.data[PREV_PAGE_ID_OFFSET..], page_id);
    }

    pub fn set_next_page_id(&mut self, page_id: PageId) {
        reinterpret::write_i32(&mut self.data[NEXT_PAGE_ID_OFFSET..], page_id);
    }

    // TODO: Implement this.
    pub fn insert_tuple(&mut self, tuple: Tuple) -> Option<Rid> {
        false
    }

    // TODO: Implement this.
    pub fn mark_delete(&mut self, rid: &Rid) -> bool {
        false
    }

    // TODO: Implement this.
    pub fn replace_tuple(&mut self, rid: &Rid, tuple: Tuple) -> Option<Tuple> {
        None
    }

    // TODO: Implement this.
    pub fn apply_delete(&mut self, rid: &Rid) {}

    // TODO: Implement this.
    pub fn rollback_delete(&mut self, rid: &Rid) {}

    // TODO: Implement this.
    pub fn get_tuple(&self, rid: &Rid) -> Option<Tuple> {
        None
    }

    fn set_free_space_ptr(&mut self, ptr: usize) {
        reinterpret::write_u64(&mut self.data[FREE_SPACE_PTR_OFFSET..], ptr as u64);
    }

    fn set_tuple_count(&mut self, count: usize) {
        reinterpret::write_u64(&mut self.data[TUPLE_COUNT_OFFSET..], count as u64);
    }
}

impl Default for TablePage {
    fn default() -> Self {
        let mut page = TablePage {
            data: [0 as u8; PAGE_SIZE],
            pin_count: 0,
            is_dirty: false,
        };
        page.set_page_id(INVALID_PAGE_ID);
        page
    }
}

impl Page for TablePage {
    fn reset(&mut self) {
        self.set_prev_page_id(INVALID_PAGE_ID);
        self.set_next_page_id(INVALID_PAGE_ID);
        self.set_free_space_ptr(PAGE_SIZE);
        self.set_tuple_count(0);
        for byte in self.data.iter_mut().skip(DATA_OFFSET) {
            *byte = 0;
        }
    }

    fn page_id(&self) -> PageId {
        reinterpret::read_i32(&self.data[PAGE_ID_OFFSET..])
    }

    fn set_page_id(&mut self, page_id: PageId) {
        reinterpret::write_i32(&mut self.data[PAGE_ID_OFFSET..], page_id);
    }

    fn data(&self) -> &[u8; PAGE_SIZE] {
        &self.data
    }

    fn data_mut(&mut self) -> &mut [u8; PAGE_SIZE] {
        &mut self.data
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
