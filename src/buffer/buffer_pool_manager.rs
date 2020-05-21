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
  // TODO: Update new page's metadata?
  pub fn new_page(&mut self) -> std::io::Result<&mut T> {
    info!("New page");
    Self::prepare_page(/*maybe_id=*/ None,
                       /*need_reset=*/ true,
                       &mut self.actor,
                       &mut self.data)
  }

  // Prepares and pins a new page and returns a (PageId, Page) pair.
  // If |maybe_id| is None, asks |actor.disk_mgr| to allocate a new page ID.
  // If |need_reset| is |true|, resets the page with 0's. Returns error if the
  // old page fails to be flushed to disk.
  fn prepare_page<'a>(
      maybe_id: Option<PageId>,
      need_reset: bool,
      actor: &mut Actor<R>,
      data: &'a mut Data<T>) -> std::io::Result<&'a mut T> {
    let either = match data.free_list.last().map(|x| *x) {
      Some(idx) => Ok(Either::FromFreeList(idx)),
      None => {
        info!("Free page unavaible, finding replacement");
        match actor.replacer.victim() {
          Some(idx) => Ok(Either::FromReplacer(idx)),
          None => Err(not_found("Replacer cannot find a victim")),
        }
      }
    }?;
    let idx = *either.borrow();
    let page = &mut data.pages[idx];
    match Self::flush_page_inl(&mut actor.disk_mgr, page) {
      Ok(()) => {  // On flush success.
        match either {
          Either::FromFreeList(_) => {
            data.free_list.pop();
          },
          Either::FromReplacer(_) => {
            data.page_table.remove(&page.page_id());
          },
        }
        let allocate = || {
          info!("Allocate page ID");
          actor.disk_mgr.allocate_page()
        };
        page.set_page_id(maybe_id.unwrap_or_else(allocate));
        data.page_table.insert(page.page_id(), idx);
        Ok(page)
      },
      Err(e) => {  // On flush failure.
        match either {
          Either::FromFreeList(_) => (),
          Either::FromReplacer(idx) => {
            // Insert page back to replacer if flush fails. 
            actor.replacer.insert(idx);
          },
        }
        Err(e)
      },
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

enum Either<T> {
  FromFreeList(T),
  FromReplacer(T),
}

impl<T> Either<T> {
  pub fn borrow(&self) -> &T {
    match self {
      &Self::FromFreeList(ref v) => v,
      &Self::FromReplacer(ref v) => v,
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
  use crate::common::config::BITMAP_FILE_SUFFIX;
  use crate::common::config::CHECKSUM_SIZE;
  use crate::common::reinterpret;
  use crate::page::table_page::TablePage;
  use crate::testing::file_deleter::FileDeleter;
  use super::*;

  type TestingBufferPoolManager = DefaultBufferPoolManager<TablePage>;

  #[test]
  fn buffer_pool_manager() {
    let file_path = "/tmp/testfile.buffer_pool_manager.1.db";
    let bitmap_path = file_path.to_string() + BITMAP_FILE_SUFFIX;

    // Test file deleter with RAII.
    let mut file_deleter = FileDeleter::new();
    file_deleter.push(&file_path);
    file_deleter.push(&bitmap_path);

    let result = TestingBufferPoolManager::new(10, file_path);
    assert!(result.is_ok(), "Failed to create");

    let mut bpm = result.unwrap();
    let maybe_page = bpm.new_page();
    assert!(maybe_page.is_ok());

    let page = maybe_page.unwrap();
    assert_eq!(HEADER_PAGE_ID, page.page_id());

    // Change content in page one.
    reinterpret::write_str(&mut page.data_mut()[CHECKSUM_SIZE..], "Hello");

    // Create 9 new pages.
    for i in 1..10 {
      assert_eq!(i + HEADER_PAGE_ID, bpm.new_page().unwrap().page_id());
    }

    // All the pages are pinned, the buffer pool is full.
    for _ in 10..15 {
      assert!(bpm.new_page().is_err());
    }

    // Upin the first five pages, add them to LRU list, set as dirty.
    for i in 0..5 {
      assert!(bpm.unpin_page(i + HEADER_PAGE_ID, /*is_dirty=*/ true).is_ok());
    }

    // We have 5 empty slots in LRU list, evict page zero out of buffer pool.
    for i in 10..14 {
      assert_eq!(i + HEADER_PAGE_ID, bpm.new_page().unwrap().page_id());
    }

    // Fetch page one again.
    let maybe_page = bpm.fetch_page(HEADER_PAGE_ID);
    assert!(maybe_page.is_ok());

    // Check read content.
    let page = maybe_page.unwrap();
    assert_eq!("Hello", reinterpret::read_str(&page.data()[CHECKSUM_SIZE..]));
  }

  #[test]
  fn new_and_delete() {
    let file_path = "/tmp/testfile.buffer_pool_manager.2.db";
    let bitmap_path = file_path.to_string() + BITMAP_FILE_SUFFIX;

    // Test file deleter with RAII.
    let mut file_deleter = FileDeleter::new();
    file_deleter.push(&file_path);
    file_deleter.push(&bitmap_path);

    let mut bpm = TestingBufferPoolManager::new(10, file_path).unwrap();
    for i in 0..10 {
      assert_eq!(i + HEADER_PAGE_ID, bpm.new_page().unwrap().page_id());
    }
    assert!(bpm.new_page().is_err());
    assert!(bpm.delete_page(HEADER_PAGE_ID - 1).is_err());
    assert!(bpm.delete_page(HEADER_PAGE_ID).is_err());

    // Unpin page |HEADER_PAGE_ID| and it gets replaced to disk, but its page
    // ID is still occupied, therefore, page |10 + HEADER_PAGE_ID| is
    // allocated.
    assert!(bpm.unpin_page(HEADER_PAGE_ID, /*is_dirty=*/ true).is_ok());
    assert_eq!(10 + HEADER_PAGE_ID, bpm.new_page().unwrap().page_id());

    // Delete page |HEADER_PAGE_ID| and unpin page |10 + HEADER_PAGE_ID|, when
    // |new_page| is called, page |10 + HEADER_PAGE_ID| gets replaced to disk.
    // Since page |HEADER_PAGE_ID| is deallocated, its page ID is reused.
    assert!(bpm.delete_page(HEADER_PAGE_ID).is_ok());
    assert!(bpm.unpin_page(10 + HEADER_PAGE_ID, /*is_dirty=*/ true).is_ok());
    assert_eq!(HEADER_PAGE_ID, bpm.new_page().unwrap().page_id());

    assert!(bpm.delete_page(10 + HEADER_PAGE_ID).is_ok());
    for i in 5..10 {
      assert!(bpm.unpin_page(i + HEADER_PAGE_ID, /*is_dirty=*/ true).is_ok());
      assert!(bpm.delete_page(i + HEADER_PAGE_ID).is_ok());
    }
    for i in 5..10 {
      assert!(bpm.fetch_page(i + HEADER_PAGE_ID).is_err());
      assert_eq!(i + HEADER_PAGE_ID, bpm.new_page().unwrap().page_id());
    }
  }

  #[test]
  fn drop_flushes_all_pages() {
    let file_path = "/tmp/testfile.buffer_pool_manager.3.db";
    let bitmap_path = file_path.to_string() + BITMAP_FILE_SUFFIX;

    // Test file deleter with RAII.
    let mut file_deleter = FileDeleter::new();
    file_deleter.push(&file_path);
    file_deleter.push(&bitmap_path);

    {
      let mut bpm = TestingBufferPoolManager::new(10, file_path).unwrap();
      for idx in 0..10 as PageId {
        let id = idx + HEADER_PAGE_ID;
        let page = bpm.new_page().unwrap();
        assert_eq!(id, page.page_id());

        // Only flush pages with |ID % 2 == 0|;
        if id % 2 == 0 {
          reinterpret::write_i32(&mut page.data_mut()[CHECKSUM_SIZE..], id);
        }
        assert!(bpm.unpin_page(id, /*is_dirty=*/ id % 2 == 0).is_ok());
      }

      // Delete pages with |ID >= 5 + HEADER_PAGE_ID|.
      for idx in 5..10 {
        assert!(bpm.delete_page(idx + HEADER_PAGE_ID).is_ok());
      }
    }  // Drops bpm.

    {
      let mut bpm = TestingBufferPoolManager::new(10, file_path).unwrap();
      for idx in 0..5 as PageId {
        let id = idx + HEADER_PAGE_ID;
        let page = bpm.fetch_page(id).unwrap();
        assert_eq!(if id % 2 == 0 {id} else {0},
                   reinterpret::read_i32(&page.data()[CHECKSUM_SIZE..]));
      }
      for idx in 5..10 {
        assert!(bpm.fetch_page(idx + HEADER_PAGE_ID).is_err());
      }
    }  // Drops bpm.
  }
}