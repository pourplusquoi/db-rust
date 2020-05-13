pub const INVALID_PAGE_ID: i32 = -1;         // representing an invalid page id
pub const INVALID_TRANSACTION_ID: i32 = -1;  // representing an invalid txn id
pub const HEADER_PAGE_ID: i32 = 0;           // the header page id
pub const PAGE_SIZE: usize = 4096;           // size of a data page in byte
pub const BUCKET_SIZE:usize = 50;            // size of extendible hash bucket

pub type PageId = i32;
pub type TransactionId = i32;