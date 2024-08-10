use core::fmt::Debug;

use crate::mm::UserBuffer;

pub mod inode;
pub mod pipe;
pub mod stdio;

pub use inode::{list_apps, open_file, OpenFlags};
pub use stdio::{Stderr, Stdin, Stdout};

pub trait File: Send + Sync + Debug {
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
    fn read(&self, buf: UserBuffer) -> usize;
    fn write(&self, buf: UserBuffer) -> usize;
}
