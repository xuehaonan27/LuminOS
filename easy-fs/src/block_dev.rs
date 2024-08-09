use core::any::Any;

/// Trait for block device
pub trait BlockDevice: Send + Sync + Any {
    /// Read a block according to `block_id`
    fn read_block(&self, block_id: usize, buf: &mut [u8]);
    /// Write a block according to `block_id`
    fn write_block(&self, block_id: usize, buf: &[u8]);
}
