// Database system configuration.

pub const INVALID_PAGE_ID: i32 = -1; // Represents an invalid page ID.
pub const INVALID_TRANSACTION_ID: i32 = -1; // Represents an invalid tansaction ID.
pub const HEADER_PAGE_ID: i32 = 0; // The header page ID.
pub const PAGE_SIZE: usize = 4096; // Size of a data page in bytes.
pub const CHECKSUM_SIZE: usize = 8; // Size of the checksum overhead.

pub type PageId = i32;
pub type TransactionId = i32;
