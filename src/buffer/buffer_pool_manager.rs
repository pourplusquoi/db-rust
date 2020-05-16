use crate::buffer::lru_replacer::LRUReplacer;
use crate::buffer::replacer::Replacer;
use crate::common::config::INVALID_PAGE_ID;
use crate::common::config::PageId;
use crate::disk::disk_manager::DiskManager;
use crate::page::table_page::TablePage;
use crate::page::page::Page;
use std::clone::Clone;
use std::collections::HashMap;
use std::collections::HashSet;
use log::info;
use log::warn;

type DefaultBufferPoolManager<T> = BufferPoolManager<T, LRUReplacer<usize>>;

struct BufferPoolManager<T, R> where T: Page + Clone, R: Replacer<usize> {
  pool_size: usize,
  pages: Vec<T>,
  disk_mgr: DiskManager,
  page_table: HashMap<PageId, usize>,
  replacer: R,
  free_list: HashSet<usize>,
}

impl<T> BufferPoolManager<T, LRUReplacer<usize>> where T: Page + Clone {

  pub fn new(size: usize, db_file: &str) -> std::io::Result<Self> {
    let mut buffer_pool_mgr = BufferPoolManager {
      pool_size: size,
      pages: vec![T::new(); size],
      disk_mgr: DiskManager::new(db_file)?,
      page_table: HashMap::new(),
      replacer: LRUReplacer::new(),
      free_list: HashSet::new(),
    };
    buffer_pool_mgr.init();
    Ok(buffer_pool_mgr)
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
    let maybe_page =
        self.prepare_page(|| page_id, /*need_reset=*/ false)
            .map(|(_, page)| page);
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
    if page_id == INVALID_PAGE_ID {
      warn!("Page ID is invalid");
      return false;
    }
    match self.page_table.get(&page_id) {
      Some(&idx) => self.flush_page_inl(&self.pages[idx]),
      None => {
        warn!("Page not found in table");
        false
      },
    }
  }

  pub fn flush_all_pages(&self) {
    for (page_id, &idx) in self.page_table.iter() {
      info!("Flush page; page_id = {}", page_id);
      self.flush_page_inl(&self.pages[idx]);
    }
  }

  pub fn delete_page(&self, page_id: PageId) -> bool {
    info!("Delete page; page_id = {}", page_id);
    // TODO: Implement this.
    false
  }

  pub fn new_page(&mut self) -> Option<(PageId, &mut T)> {
    info!("New page");
    self.prepare_page(|| self.disk_mgr.allocate_page(), /*need_reset=*/ true)
        .map(|(page_id, page)| {
          // TODO: Update new page's metadata.
          (page_id, page)
        })
  }

  fn flush_page_inl(&self, page: &T) -> bool {
    if page.is_dirty() {
      info!("Page is dirty, will write it to disk");
      // TODO: Write page back to disk.
    }
    true
  }

  fn prepare_page<F>(&mut self,
                     page_id_supplier: F,
                     need_reset: bool) -> Option<(PageId, &mut T)>
      where F: FnOnce() -> PageId {
    match self.free_list.iter().nth(0).map(|x| *x) {
      Some(idx) => {
        let page_id = page_id_supplier();
        info!("Free page avaible, will use it; page_id = {}", page_id);
        self.free_list.remove(&idx);
        self.page_table.insert(page_id, idx);
        let page = &mut self.pages[idx];
        Some((page_id, page))
      },
      None => {
        info!("Free page unavaible, finding replacement");
        match self.replacer.victim() {
          Some(idx) => {  // The idx of victim page.
            let page_id = page_id_supplier();
            info!("Found victim page; page_id = {}; idx = {}", page_id, idx);
            let page = &self.pages[idx];
            self.flush_page_inl(page);
            self.page_table.remove(&page.page_id());
            self.page_table.insert(page_id, idx);
            let page = &mut self.pages[idx];
            Some((page_id, page))
          },
          None => {
            warn!("Replacer cannot find a victim");
            None
          },
        }
      },
    }.map(|(page_id, page)| {
      if need_reset {
        Self::reset_page(page);
      }
      (page_id, page)
    })
  }

  fn reset_page(page: &mut T) {
    info!("Reset page");
    // TODO: Implement this.
  }
}

#[cfg(test)]
mod tests {
  use crate::testing::file_deleter::FileDeleter;
  use super::*;

  type TestingBufferPoolManager = DefaultBufferPoolManager<TablePage>;

  #[test]
  fn buffer_pool_manager() {
    // Test file deleter with RAII.
    let mut file_deleter = FileDeleter::new();

    let file_path = "/tmp/testfile";
    file_deleter.push(&file_path);
    let result = TestingBufferPoolManager::new(10, file_path);
    assert!(result.is_ok(), "Failed to create");

    let mut bpm = result.unwrap();
    let maybe_page = bpm.new_page();
    assert!(maybe_page.is_some());

    let (page_id, page) = maybe_page.unwrap();
    assert_eq!(0, page_id);

    let data = page.borrow_mut();
  }
}