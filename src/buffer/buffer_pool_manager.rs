// Functionality: The simplified Buffer Manager interface allows a client to
// new/delete pages on disk, to read a disk page into the buffer pool and pin
// it, also to unpin a page in the buffer pool.

use crate::buffer::lru_replacer::LRUReplacer;
use crate::buffer::replacer::Replacer;
use crate::common::config::HEADER_PAGE_ID;
use crate::common::config::PageId;
use crate::common::error::*;
use crate::disk::disk_manager::DiskManager;
use crate::logging::error_logging::ErrorLogging;
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
    validate(page_id)?;
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
  // of this page. Returns |InvalidData| if the page pin count <= 0.
  pub fn unpin_page(&mut self,
                    page_id: PageId,
                    is_dirty: bool) -> std::io::Result<()> {
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

  // Flushes one page with specified |page_id| to disk. Returns |NotFound| if
  // no such page exists in |self.data.page_table|.
  pub fn flush_page(&mut self, page_id: PageId) -> std::io::Result<()> {
    info!("Flush page; page_id = {}", page_id);
    validate(page_id)?;
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
  pub fn delete_page(&mut self, page_id: PageId) -> std::io::Result<()> {
    info!("Delete page; page_id = {}", page_id);
    validate(page_id)?;
    match self.data.page_table.get(&page_id) {
      Some(&idx) => {
        // If a page is being deleted, there is no point of flushing it.
        let page = &mut self.data.pages[idx];
        if page.pin_count() > 0 {
          return Err(invalid_data("Cannot delete pinned page"));
        }
        page.set_is_dirty(false);
        self.data.free_list.push(idx);
        self.data.page_table.remove(&page_id);
      },
      None => (),
    }
    self.actor.disk_mgr.deallocate_page(page_id);
    Ok(())
  }

  // Creates a new page. User should call this method if one needs to create a
  // new page. This routine will call |self.actor.disk_mgr| to allocate a page.
  //
  // Note: This methods only returns the page in memory without syncing to
  // disk, the caller needs to flush it (even if no data are written to the
  // page) if one wish to read from it to avoid unexpected EOF.
  //
  // TODO: Update new page's metadata?
  pub fn new_page(&mut self) -> std::io::Result<&mut T> {
    info!("New page");
    Self::prepare_page(/*maybe_id=*/ None,
                       /*need_reset=*/ true,
                       &mut self.actor,
                       &mut self.data)
  }

  // Prepares and pins a new page and returns a (PageId, Page) pair. If
  // |maybe_id| is None, asks |actor.disk_mgr| to allocate a new page ID. If
  // |need_reset| is |true|, resets the page with 0's.
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
      // If flushing the old page fails, should not retry, because it may cause
      // stale data get flushed to disk.
      Some(idx) => {
        // Step 1: Remove the old page from page table if exists.
        let page = &mut data.pages[idx];
        data.page_table.remove(&page.page_id());
        // Step 2: Remove the old page from free list.
        data.free_list.pop();
        // Step 3: Flush the old page to disk.
        Self::flush_page_inl(&mut actor.disk_mgr, page).log();
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
    }.and_then(|page| {
      if need_reset {
        Self::reset_page(page);
      }
      page.pin();
      Ok(page)
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
        disk_mgr.write_page(page.page_id(), page.data_mut())?;
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

fn validate(page_id: PageId) -> std::io::Result<()> {
  if page_id < HEADER_PAGE_ID {
    return Err(invalid_input("Page ID is invalid"));
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use crate::common::reinterpret;
  use crate::page::table_page::TablePage;
  use crate::testing::file_deleter::FileDeleter;
  use super::*;

  type TestingBufferPoolManager = DefaultBufferPoolManager<TablePage>;

  #[test]
  fn buffer_pool_manager() {
    let file_path = "/tmp/buffer_pool_manager.1.testfile";

    // Test file deleter with RAII.
    let mut file_deleter = FileDeleter::new();
    file_deleter.push(&file_path);

    let result = TestingBufferPoolManager::new(10, file_path);
    assert!(result.is_ok(), "Failed to create");

    let mut bpm = result.unwrap();
    let maybe_page = bpm.new_page();
    assert!(maybe_page.is_ok());

    let page = maybe_page.unwrap();
    assert_eq!(1, page.page_id(), "Page 0 is reserved");

    // Change content in page one.
    reinterpret::write_str(&mut page.data_mut()[8..], "Hello");

    for i in 1..10 {
      assert_eq!(i + 1, bpm.new_page().unwrap().page_id());
    }
    // All the pages are pinned, the buffer pool is full.
    for _ in 10..15 {
      assert!(bpm.new_page().is_err());
    }

    // Upin the first five pages, add them to LRU list, set as dirty.
    for i in 1..6 {
      assert!(bpm.unpin_page(i, /*is_dirty=*/ true).is_ok());
    }
    // We have 5 empty slots in LRU list, evict page zero out of buffer pool.
    for i in 10..14 {
      assert_eq!(i + 1, bpm.new_page().unwrap().page_id());
    }

    // Fetch page one again.
    let maybe_page = bpm.fetch_page(1);
    assert!(maybe_page.is_ok());
    // Check read content.
    let page = maybe_page.unwrap();
    assert_eq!("Hello", reinterpret::read_str(&page.data()[8..]));
  }

  #[test]
  fn new_and_delete() {
    let file_path = "/tmp/buffer_pool_manager.2.testfile";

    // Test file deleter with RAII.
    let mut file_deleter = FileDeleter::new();
    file_deleter.push(&file_path);

    let mut bpm = TestingBufferPoolManager::new(10, file_path).unwrap();
    for i in 1..=10 {
      assert_eq!(i, bpm.new_page().unwrap().page_id());
    }
    assert!(bpm.new_page().is_err());
    assert!(bpm.delete_page(0).is_err());
    assert!(bpm.delete_page(1).is_err());

    // Unpin page 1 and it gets replaced to disk, but its page ID is still
    // occupied, therefore, page 11 is allocated.
    assert!(bpm.unpin_page(1, /*is_dirty=*/ true).is_ok());
    assert_eq!(11, bpm.new_page().unwrap().page_id());

    // Delete page 1 and unpin page 11, when |new_page| is called, page 11 gets
    // replaced to disk. Since page 1 is deallocated, its page ID is reused.
    assert!(bpm.delete_page(1).is_ok());
    assert!(bpm.unpin_page(11, /*is_dirty=*/ true).is_ok());
    assert_eq!(1, bpm.new_page().unwrap().page_id());

    assert!(bpm.delete_page(11).is_ok());
    for i in 6..=10 {
      assert!(bpm.unpin_page(i, /*is_dirty=*/ true).is_ok());
      assert!(bpm.delete_page(i).is_ok());
    }
    for i in 6..=10 {
      assert!(bpm.fetch_page(i).is_err());
      assert_eq!(i, bpm.new_page().unwrap().page_id());
    }
  }

  #[test]
  fn drop_flushes_all_pages() {
    let file_path = "/tmp/buffer_pool_manager.3.testfile";

    // Test file deleter with RAII.
    let mut file_deleter = FileDeleter::new();
    file_deleter.push(&file_path);

    {
      let mut bpm = TestingBufferPoolManager::new(10, file_path).unwrap();
      for id in 1..=10 as PageId {
        let page = bpm.new_page().unwrap();
        reinterpret::write_i32(&mut page.data_mut()[8..], id);
        // Only flush pages with |ID % 2 == 0|;
        assert_eq!(id, page.page_id());
        assert!(bpm.unpin_page(id, /*is_dirty=*/ true).is_ok());
      }
      // Delete pages with |ID > 5|.
      for id in 6..=10 {
        assert!(bpm.delete_page(id).is_ok());
      }
    }  // Drops bpm.

    {
      let mut bpm = TestingBufferPoolManager::new(10, file_path).unwrap();
      for id in 1..=5 as PageId {
        let page = bpm.fetch_page(id).unwrap();
        assert_eq!(id, reinterpret::read_i32(&page.data()[8..]));
      }
      for i in 6..=10 {
        assert!(bpm.fetch_page(i).is_err());
      }
    }  // Drops bpm.
  }
}