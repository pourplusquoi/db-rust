use crate::common::config::PageId;
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

impl PartialEq for Rid {
    fn eq(&self, other: &Self) -> bool {
        self.page_id == other.page_id && self.slot_num == other.slot_num
    }
}

impl Eq for Rid {}
