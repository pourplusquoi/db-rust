// Functionality: The simplified Buffer Manager interface allows a client to
// new/delete pages on disk, to read a disk page into the buffer pool and pin
// it, also to unpin a page in the buffer pool.

use crate::buffer::lru_replacer::LRUReplacer;
use crate::buffer::replacer::Replacer;
use crate::common::config::INVALID_PAGE_ID;
use crate::common::config::PageId;
use crate::common::error::*;
use crate::disk::disk_manager::DiskManager;
use crate::logging::error_logging::ErrorLogging;
use crate::page::table_page::TablePage;
use crate::page::page::Page;
use std::clone::Clone;
use std::collections::HashMap;
use std::ops::Drop;
use log::info;

// Struct members are split into |data| and |actor|, because this makes it
// possible to hold mutable borrow on |actor| while acquiring mutable/immutable
// borrow on |data|.
pub struct BufferPoolManager<T, R> where T: Page + Clone, R: Replacer<usize> {
  data: Data<T>,
  actor: Actor<R>,
}

// The default BufferPoolManager uses LRUReplacer.
pub type DefaultBufferPoolManager<T> = BufferPoolManager<T, LRUReplacer<usize>>;

impl<T, R> Drop for BufferPoolManager<T, R>
    where T: Page + Clone, R: Replacer<usize> {
  fn drop(&mut self) {
    // Unable to handle I/O errors on destruction.
    self.flush_all_pages().log();
  }
}

impl<T, R> BufferPoolManager<T, R> where T: Page + Clone, R: Replacer<usize> {
  pub fn new(size: usize, db_file: &str) -> std::io::Result<Self> {
    Ok(BufferPoolManager {
      data: Data::new(size),
      actor: Actor::new(db_file)?,
    }).and_then(|mut buffer_pool_mgr| {
      buffer_pool_mgr.init();
      Ok(buffer_pool_mgr)
    })
  }

  fn init(&mut self) {
    for i in 0..self.data.pool_size {
      self.data.free_list.push(i);
    }
  }

  // Fetches the page with specified |page_id|. Pins the page if it already
  // exists in |self.data.page_table|; otherwise, loads the page from disk.
  pub fn fetch_page(&mut self, page_id: PageId) -> std::io::Result<&mut T> {
    info!("Fetch page; page_id = {}", page_id);
    match self.data.page_table.get(&page_id) {
      Some(&idx) => {
        info!("Found page in table, will pin the page; idx = {}", idx);
        let page = &mut self.data.pages[idx];
        page.pin();
        return Ok(page);
      },
      None => (),
    }
    info!("Page not found in table, need to load from disk");
    let actor = &mut self.actor;
    let data = &mut self.data;
    Self::prepare_page(Some(page_id),
                       /*need_reset=*/ false,
                       actor, data)
        .and_then(|page| {
          info!("Loading the page from disk");
          Self::load_page_inl(&mut actor.disk_mgr, page).map(|_| page)
        })
  }

  // Unpins the page with specified |page_id|. |is_dirty| sets the dirty flag
  // of this page. Returns |false| if the page pin count <= 0.
  pub fn unpin_page(&mut self, page_id: PageId, is_dirty: bool) -> std::io::Result<()> {
    info!("Unpin page; page_id = {}", page_id);
    match self.data.page_table.get(&page_id) {
      Some(&idx) => {
        info!("Found page in table; idx = {}", idx);
        let page = &mut self.data.pages[idx];
        page.set_is_dirty(is_dirty);
        if page.unpin() {
          if page.pin_count() == 0 {
            info!("Insert page to replacer; idx = {}", idx);
            self.actor.replacer.insert(idx);
          }
          Ok(())
        } else {
          Err(invalid_data("Pin count <= 0, cannot be unpinned"))
        }
      },
      None => Err(not_found("Page not found in table")),
    }
  }

  // Flushes one page with specified |page_id| to disk. Returns |false| if no
  // such page exists in |self.data.page_table|.
  pub fn flush_page(&mut self, page_id: PageId) -> std::io::Result<()> {
    info!("Flush page; page_id = {}", page_id);
    if page_id == INVALID_PAGE_ID {
      return Err(invalid_input("Page ID is invalid"));
    }
    match self.data.page_table.get(&page_id) {
      Some(&idx) =>
          Self::flush_page_inl(&mut self.actor.disk_mgr,
                               &mut self.data.pages[idx]),
      None => Err(not_found("Page not found in table")),
    }
  }

  // Flushes if dirty all pages (i.e. |self.data.pages|) to disk. Finishes
  // flushing all pages regardless of I/O errors. Returns the first error
  // encountered.
  pub fn flush_all_pages(&mut self) -> std::io::Result<()> {
    let mut result = Ok(());
    for (page_id, &idx) in self.data.page_table.iter() {
      info!("Flush page; page_id = {}", page_id);
      let res = Self::flush_page_inl(&mut self.actor.disk_mgr,
                                     &mut self.data.pages[idx]);
      result = result.and(res);
    }
    result
  }

  // Deletes a page. User should call this method for deleting a page. This
  // routine will call |self.actor.disk_mgr| to deallocate the page.
  pub fn delete_page(&self, page_id: PageId) -> bool {
    info!("Delete page; page_id = {}", page_id);
    // TODO: Implement this. (Need to reset pin_count & is_dirty!)?  If the
    // page is found within page table, but pin_count != 0, return false.
    false
  }

  // Creates a new page. User should call this method if one needs to create a
  // new page. This routine will call |self.actor.disk_mgr| to allocate a page.
  pub fn new_page(&mut self) -> std::io::Result<&mut T> {
    info!("New page");
    Self::prepare_page(/*maybe_id=*/ None,
                       /*need_reset=*/ true,
                       &mut self.actor,
                       &mut self.data)
        .map(|page| {
          // TODO: Update new page's metadata.
          page
        })
  }

  // Prepares a new page and returns a (PageId, Page) pair. If |maybe_id| is
  // None, asks |actor.disk_mgr| to allocate a new page ID. If |need_reset| is
  // |true|, resets the page with 0's.
  fn prepare_page<'a>(
      maybe_id: Option<PageId>,
      need_reset: bool,
      actor: &mut Actor<R>,
      data: &'a mut Data<T>) -> std::io::Result<&'a mut T> {
    if data.free_list.is_empty() {
      info!("Free page unavaible, finding replacement");
      match actor.replacer.victim() {
        Some(idx) => {
          data.free_list.push(idx);
        },
        None => {
          return Err(not_found("Replacer cannot find a victim"));
        },
      }
    }
    match data.free_list.last().map(|x| *x) {
      // If flushing the old page fails, the following operations will stop
      // early, in which case, the old page remains dirty and inside
      // |data.free_list|. Will retry flushing next time when it is selected
      // for accommodation for a new page.
      Some(idx) => {
        // Step 1: Remove the old page from page table if exists.
        let page = &mut data.pages[idx];
        data.page_table.remove(&page.page_id());
        // Step 2: Flush the old page to disk.
        Self::flush_page_inl(&mut actor.disk_mgr, page)?;
        // Step 3: Remove the old page from free list.
        data.free_list.pop();
        // Step 4: Update the page ID.
        let allocate = || {
          info!("Allocate page ID");
          actor.disk_mgr.allocate_page()
        };
        page.set_page_id(maybe_id.unwrap_or_else(allocate));
        // Step 5: Insert the new page ID into page table.
        data.page_table.insert(page.page_id(), idx);
        Ok(page)
      },
      None => Err(invalid_data("Should not reach here")),
    }.map(|page| {
      if need_reset {
        Self::reset_page(page);
      }
      page
    })
  }

  // Flushes the specified page to disk manager iff the page is dirty, resets
  // the dirty flag. |page.data()| stores the data being written to disk.
  //
  // Note: If the page is not dirty, calling this is a no-op.
  fn flush_page_inl(disk_mgr: &mut DiskManager,
                    page: &mut T) -> std::io::Result<()> {
    match page.is_dirty() {
      true => {
        info!("Page is dirty, flushiung to disk");
        disk_mgr.write_page(page.page_id(), page.data())?;
        page.set_is_dirty(false);
      },
      false => { info!("Page is not dirty, skipping"); }
    }
    Ok(())
  }

  // Loads the specified page from disk manager. |page.data_mut()| is the place
  // where the data being read will be stored.
  //
  // Note: It is not allowed to load page when the current page is dirty.
  fn load_page_inl(disk_mgr: &mut DiskManager,
                   page: &mut T) -> std::io::Result<()> {
    match page.is_dirty() {
      true => Err(invalid_data("Cannot load while current page is dirty")),
      false => {
        info!("Loading page from disk");
        disk_mgr.read_page(page.page_id(), page.data_mut())?;
        Ok(())
      },
    }
  }

  // Resets the specified page by writing 0's to its content.
  fn reset_page(page: &mut T) {
    info!("Reset page");
    for byte in page.data_mut().iter_mut() {
      *byte = 0;
    }
  }
}

struct Data<T> where T: Page + Clone {
  pool_size: usize,
  pages: Vec<T>,
  page_table: HashMap<PageId, usize>,
  free_list: Vec<usize>,
}

impl<T> Data<T> where T: Page + Clone {
  pub fn new(size: usize) -> Self {
    Data {
      pool_size: size,
      pages: vec![T::new(); size],
      page_table: HashMap::new(),
      free_list: Vec::new(),
    }
  }
}

struct Actor<R> where R: Replacer<usize> {
  replacer: R,
  disk_mgr: DiskManager,
}

impl<R> Actor<R> where R: Replacer<usize> {
  pub fn new(db_file: &str) -> std::io::Result<Self> {
    let actor = Actor {
      replacer: R::new(),
      disk_mgr: DiskManager::new(db_file)?,
    };
    Ok(actor)
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
    assert!(maybe_page.is_ok());

    let page = maybe_page.unwrap();
    assert_eq!(0, page.page_id());

    let _data = page.data_mut();
  }
}