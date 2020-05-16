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
use std::ops::Drop;
use log::info;
use log::warn;

struct Data<T> where T: Page + Clone {
  pages: Vec<T>,
  page_table: HashMap<PageId, usize>,
  free_list: HashSet<usize>,
}

impl<T> Data<T> where T: Page + Clone {
  pub fn new(size: usize) -> Self {
    Data {
      pages: vec![T::new(); size],
      page_table: HashMap::new(),
      free_list: HashSet::new(),
    }
  }
}

struct Reactor<R> where R: Replacer<usize> {
  replacer: R,
  disk_mgr: DiskManager,
}

impl<R> Reactor<R> where R: Replacer<usize> {
  pub fn new(db_file: &str) -> std::io::Result<Self> {
    let rector = Reactor {
      replacer: R::new(),
      disk_mgr: DiskManager::new(db_file)?,
    };
    Ok(rector)
  }
}

pub struct BufferPoolManager<T, R> where T: Page + Clone, R: Replacer<usize> {
  pool_size: usize,
  data: Data<T>,
  reactor: Reactor<R>,  // Maybe mutable
}

// The default BufferPoolManager uses LRUReplacer.
pub type DefaultBufferPoolManager<T> = BufferPoolManager<T, LRUReplacer<usize>>;

impl<T, R> Drop for BufferPoolManager<T, R>
    where T: Page + Clone, R: Replacer<usize> {
  fn drop(&mut self) {
    self.flush_all_pages();
  }
}

impl<T, R> BufferPoolManager<T, R> where T: Page + Clone, R: Replacer<usize> {

  pub fn new(size: usize, db_file: &str) -> std::io::Result<Self> {
    let mut buffer_pool_mgr = BufferPoolManager {
      pool_size: size,
      data: Data::new(size),
      reactor: Reactor::new(db_file)?,
    };
    buffer_pool_mgr.init();
    Ok(buffer_pool_mgr)
  }

  fn init(&mut self) {
    for i in 0..self.pool_size {
      self.data.free_list.insert(i);
    }
  }

  pub fn fetch_page(&mut self, page_id: PageId) -> Option<&mut T> {
    info!("Fetch page; page_id = {}", page_id);
    match self.data.page_table.get(&page_id) {
      Some(&idx) => {
        info!("Found page in table, will pin the page; idx = {}", idx);
        let page = &mut self.data.pages[idx];
        page.pin();
        return Some(page);
      },
      None => (),
    }
    info!("Page not found in table, need to load from disk");
    let reactor = &mut self.reactor;
    Self::prepare_page(Some(page_id),
                       /*need_reset=*/ false,
                       reactor,
                       &mut self.data)
        .and_then(|(_, page)| {
          info!("Loading the page from disk");
          Self::load_page_inl(&mut reactor.disk_mgr, page).map(|_| page).ok()
        })
  }

  pub fn unpin_page(&mut self, page_id: PageId, is_dirty: bool) -> bool {
    info!("Unpin page; page_id = {}", page_id);
    match self.data.page_table.get(&page_id) {
      Some(&idx) => {
        info!("Found page in table; idx = {}", idx);
        let page = &mut self.data.pages[idx];
        page.set_is_dirty(is_dirty);
        if page.unpin() {
          if page.pin_count() == 0 {
            info!("Insert page to replacer; idx = {}", idx);
            self.reactor.replacer.insert(idx);
          }
          true
        } else {
          warn!("Pin count <= 0, cannot be unpinned");
          false
        }
      },
      None => false,  // Page not found in table.
    }
  }

  pub fn flush_page(&mut self, page_id: PageId) -> bool {
    info!("Flush page; page_id = {}", page_id);
    if page_id == INVALID_PAGE_ID {
      warn!("Page ID is invalid");
      return false;
    }
    match self.data.page_table.get(&page_id) {
      Some(&idx) => Self::flush_page_inl(&mut self.reactor.disk_mgr,
                                         &mut self.data.pages[idx]).is_ok(),
      None => false,  // Page not found in table.
    }
  }

  pub fn flush_all_pages(&mut self) {
    for (page_id, &idx) in self.data.page_table.iter() {
      info!("Flush page; page_id = {}", page_id);
      Self::flush_page_inl(&mut self.reactor.disk_mgr,
                           &mut self.data.pages[idx]);
    }
  }

  pub fn delete_page(&self, page_id: PageId) -> bool {
    info!("Delete page; page_id = {}", page_id);
    // TODO: Implement this. (Need to reset pin_count & is_dirty!)?
    // If the page is found within page table, but pin_count != 0, return false.
    false
  }

  pub fn new_page(&mut self) -> Option<(PageId, &mut T)> {
    info!("New page");
    Self::prepare_page(/*maybe_id=*/ None,
                       /*need_reset=*/ true,
                       &mut self.reactor,
                       &mut self.data)
        .map(|(page_id, page)| {
          // TODO: Update new page's metadata.
          (page_id, page)
        })
  }

  fn prepare_page<'a>(
      maybe_id: Option<PageId>,
      need_reset: bool,
      reactor: &mut Reactor<R>,
      data: &'a mut Data<T>) -> Option<(PageId, &'a mut T)> {
    match data.free_list.iter().nth(0).map(|x| *x) {
      Some(idx) => Self::page_with_idx(idx, maybe_id, reactor, data),
      None => {
        info!("Free page unavaible, finding replacement");
        match reactor.replacer.victim() {
          // The idx of victim page.
          Some(idx) => Self::page_with_idx(idx, maybe_id, reactor, data),
          None => None,  // Replacer cannot find a victim.
        }
      },
    }.map(|(page_id, page)| {
      if need_reset {
        Self::reset_page(page);
      }
      (page_id, page)
    })
  }

  fn page_with_idx<'a>(
      idx: usize,
      maybe_id: Option<PageId>,
      rector: &mut Reactor<R>,
      data: &'a mut Data<T>) -> Option<(PageId, &'a mut T)> {
    let allocate = || {
      info!("Allocate page ID");
      rector.disk_mgr.allocate_page()
    };
    let page_id = maybe_id.unwrap_or_else(allocate);
    info!("Found free page; page_id = {}; idx = {}", page_id, idx);
    let page = &mut data.pages[idx];
    // Flush the old page to disk.
    Self::flush_page_inl(&mut rector.disk_mgr, page);
    // Update the page table.
    data.page_table.remove(&page.page_id());
    data.page_table.insert(page_id, idx);
    // Update the page ID.
    page.set_page_id(page_id);
    Some((page_id, page))
  }

  fn flush_page_inl(disk_mgr: &mut DiskManager,
                    page: &mut T) -> std::io::Result<()> {
    if page.is_dirty() {
      info!("Page is dirty, will write it to disk");
      disk_mgr.write_page(page.page_id(), page.borrow())?;
      page.set_is_dirty(false);
    }
    Ok(())
  }

  fn load_page_inl(disk_mgr: &mut DiskManager,
                   page: &mut T) -> std::io::Result<()> {
    page.set_is_dirty(false);
    disk_mgr.read_page(page.page_id(), page.borrow_mut())?;
    Ok(())
  }

  fn reset_page(page: &mut T) {
    info!("Reset page");
    for byte in page.borrow_mut().iter_mut() {
      *byte = 0;
    }
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