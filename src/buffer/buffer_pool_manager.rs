use crate::buffer::lru_replacer::LRUReplacer;
use crate::buffer::replacer::Replacer;
use crate::common::config::PageId;
use crate::page::page::Page;
use std::hash::Hash;
use std::clone::Clone;
use std::cmp::Eq;
use std::collections::HashMap;
use std::collections::HashSet;

struct BufferPoolManager<T, R> where T: Page + Clone, R: Replacer<usize> {
  pool_size: usize,
  pages: Vec<T>,
  page_table: HashMap<PageId, usize>,
  replacer: R,
  free_list: HashSet<usize>,
}

impl<T> BufferPoolManager<T, LRUReplacer<usize>> where T: Page + Clone {

  pub fn new(size: usize) -> Self {
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
    match self.page_table.get(&page_id) {
      Some(&idx) => {
        let page = &mut self.pages[idx];
        page.pin();
        return Some(page);
      },
      None => (),
    }
    match self.free_list.iter().nth(0).map(|x| *x) {
      Some(idx) => {
        self.free_list.remove(&idx);
        Some(&mut self.pages[idx])
      },
      None => {
        match self.replacer.victim() {
          None => None,
          Some(idx) => {  // The idx of victim page.
            let page = &self.pages[idx];
            if self.pages[idx].is_dirty() {
              self.flush_page(page.page_id());
            }
            self.page_table.remove(&page.page_id());
            self.page_table.insert(page_id, idx);
            // TODO: Load the page from disk.
            Some(&mut self.pages[idx])
          },
        }
      },
    }
  }

  pub fn unpin_page(&self, page_id: PageId, is_dirty: bool) -> bool {
    // TODO
    false
  }

  pub fn flush_page(&self, page_id: PageId) -> bool {
    // TODO
    false
  }

  pub fn flush_all_pages(&self) {
    // TODO
  }

  pub fn delete_page(&self, page_id: PageId) -> bool {
    // TODO
    false
  }

  pub fn new_page(&mut self) -> Option<(PageId, &mut T)> {
    // TODO
    None
  }
}