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
use crate::common::newable::Newable;
use crate::page::page::Page;
use std::clone::Clone;

#[allow(dead_code)]
pub struct TablePage {
    data: [u8; PAGE_SIZE],
    page_id: PageId,
    pin_count: i32,
    is_dirty: bool,
}

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

impl Newable for TablePage {
    fn new() -> Self {
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
