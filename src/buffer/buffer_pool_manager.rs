use crate::buffer::lru_replacer::LRUReplacer;
use crate::buffer::replacer::Replacer;
use crate::common::config::INVALID_PAGE_ID;
use crate::common::config::PageId;
use crate::page::header_page::HeaderPage;
use crate::page::page::Page;
use std::hash::Hash;
use std::clone::Clone;
use std::cmp::Eq;
use std::collections::HashMap;
use std::collections::HashSet;
use log::info;
use log::warn;

struct BufferPoolManager<T, R> where T: Page + Clone, R: Replacer<usize> {
  pool_size: usize,
  pages: Vec<T>,
  page_table: HashMap<PageId, usize>,
  replacer: R,
  free_list: HashSet<usize>,
}

impl<T> BufferPoolManager<T, LRUReplacer<usize>> where T: Page + Clone {

  pub fn new(size: usize, db_file: &str) -> Self {
    let mut buffer_pool_mgr = BufferPoolManager {
      pool_size: size,
      pages: vec![T::new(); size],
      page_table: HashMap::new(),
      replacer: LRUReplacer::new(),
      free_list: HashSet::new(),
    };
    buffer_pool_mgr.init();
    buffer_pool_mgr
  }

  fn init(&mut self) {
    for i in 0..self.pool_size {
      self.free_list.insert(i);
    }
  }

  pub fn fetch_page(&mut self, page_id: PageId) -> Option<&mut T> {
    info!("Fetch page; page_id = {}", page_id);
    match self.page_table.get(&page_id) {
      Some(&idx) => {
        info!("Found page in table, will pin the page; idx = {}", idx);
        let page = &mut self.pages[idx];
        page.pin();
        return Some(page);
      },
      None => (),
    }
    info!("Page not found in table, need to load from disk");
    let maybe_page = match self.free_list.iter().nth(0).map(|x| *x) {
      Some(idx) => {
        info!("Free page avaible, will use it");
        self.free_list.remove(&idx);
        let page = &mut self.pages[idx];
        Some(page)
      },
      None => {
        info!("Free page unavaible, finding replacement");
        match self.replacer.victim() {
          None => {
            warn!("Replacer cannot find a victim");
            None
          },
          Some(idx) => {  // The idx of victim page.
            info!("Found victim page; idx = {}", idx);
            let page = &self.pages[idx];
            self.flush_page_inl(page);
            self.page_table.remove(&page.page_id());
            self.page_table.insert(page_id, idx);
            let page = &mut self.pages[idx];
            Some(page)
          },
        }
      },
    };
    info!("Loading the page from disk");
    // TODO: Load the page from disk.
    maybe_page
  }

  pub fn unpin_page(&mut self, page_id: PageId, is_dirty: bool) -> bool {
    info!("Unpin page; page_id = {}", page_id);
    match self.page_table.get(&page_id) {
      Some(&idx) => {
        info!("Found page in table; idx = {}", idx);
        let page = &mut self.pages[idx];
        page.set_dirty(is_dirty);
        if page.unpin() {
          if page.pin_count() == 0 {
            info!("Insert page to replacer; idx = {}", idx);
            self.replacer.insert(idx);
          }
          true
        } else {
          warn!("Pin count <= 0, cannot be unpinned");
          false
        }
      },
      None => {
        warn!("Page not found in table");
        false
      },
    }
  }

  pub fn flush_page(&self, page_id: PageId) -> bool {
    info!("Flush page; page_id = {}", page_id);
    if (page_id == INVALID_PAGE_ID) {
      warn!("Page ID is invalid");
      return false;
    }
    match self.page_table.get(&page_id) {
      None => {
        warn!("Page not found in table");
        false
      },
      Some(&idx) => self.flush_page_inl(&self.pages[idx]),
    }
  }

  pub fn flush_all_pages(&self) {
    for (page_id, &idx) in self.page_table.iter() {
      info!("Flush page; page_id = {}", page_id);
      self.flush_page_inl(&self.pages[idx]);
    }
  }

  pub fn delete_page(&self, page_id: PageId) -> bool {
    // TODO: Implement this.
    false
  }

  pub fn new_page(&mut self) -> Option<(PageId, &mut T)> {
    // TODO: Implement this.
    None
  }

  fn flush_page_inl(&self, page: &T) -> bool {
    if page.is_dirty() {
      info!("Page is dirty, will write it to disk");
      // TODO: Write page back to disk.
    }
    true
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn buffer_pool_manager() {
    let mut bpm: BufferPoolManager<HeaderPage, _> =
        BufferPoolManager::new(10, "test.db");

    let maybe_page = bpm.new_page();
    assert!(maybe_page.is_some());

    let (page_id, page) = maybe_page.unwrap();
    assert_eq!(0, page_id);

    let data = page.borrow_mut();
  }
}