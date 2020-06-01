// Trait for page. All page classes need to implement these methods.
//
// Wrapper around actual data page in main memory and also contains bookkeeping
// information used by buffer pool manager like pin_count/dirty_flag/page_id.
// Use page as a basic unit within the database system.

use crate::common::config::PageId;
use crate::common::config::PAGE_SIZE;
use std::default::Default;

pub trait Page: Default {
    fn data(&self) -> &[u8; PAGE_SIZE];
    fn data_mut(&mut self) -> &mut [u8; PAGE_SIZE];
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
