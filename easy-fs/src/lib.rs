//! An easy file system isolated from the kernel
#![no_std]
#![deny(missing_docs)]

extern crate alloc;

mod bitmap;
mod block_cache;
mod block_dev;
mod efs;
mod layout;
mod vfs;
// #[macro_use]
// mod console;

/// Size of a block, unit of file system management.
pub const BLOCK_SIZE: usize = 0x1000; // 4096 Bytes
/// Size of a sector, unit of read / write.
pub const SECTOR_SIZE: usize = 0x200; // 512 Bytes
/// Number of block caches that should stay in memory
const BLOCK_CACHE_NUM: usize = 128;
use bitmap::Bitmap;
use block_cache::{block_cache_sync_all, get_block_cache};
pub use block_dev::BlockDevice;
pub use efs::EasyFileSystem;
pub use layout::SuperBlock;
use layout::*;
pub use vfs::Inode;
