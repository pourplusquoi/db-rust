use crate::common::config::PageId;
use crate::common::config::INVALID_PAGE_ID;
use std::clone::Clone;
use std::cmp::Eq;
use std::cmp::PartialEq;
use std::fmt::Debug;

#[derive(Clone, Debug)]
pub struct Rid {
    page_id: PageId,
    slot_num: usize,
}

impl Rid {
    pub fn new(page_id: PageId, slot_num: usize) -> Self {
        Rid {
            page_id: page_id,
            slot_num: slot_num,
        }
    }

    pub fn page_id(&self) -> PageId {
        self.page_id
    }

    pub fn slot_num(&self) -> usize {
        self.slot_num
    }

    pub fn to_string(&self) -> String {
        format!(
            "Rid[page_id: {}, slot_num: {}]",
            self.page_id, self.slot_num
        );
    }
}

impl Default for Rid {
    fn default() -> Self {
        Rid {
            page_id: INVALID_PAGE_ID,
            slot_num: 0,
        }
    }
}

impl PartialEq for Rid {
    fn eq(&self, other: &Self) -> bool {
        self.page_id == other.page_id && self.slot_num == other.slot_num
    }
}

impl Eq for Rid {}
