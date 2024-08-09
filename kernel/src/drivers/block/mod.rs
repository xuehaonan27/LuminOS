mod virtio_blk;

pub use virtio_blk::VirtIOBlock;

use crate::board::BlockDeviceImpl;
use alloc::sync::Arc;
use easy_fs::BlockDevice;
use lazy_static::*;

lazy_static! {
    pub static ref BLOCK_DEVICE: Arc<dyn BlockDevice> = Arc::new(BlockDeviceImpl::new());
}

#[allow(unused)]
pub fn block_device_test() {
    use easy_fs::BLOCK_SIZE;
    let block_device = BLOCK_DEVICE.clone();
    let mut write_buffer = [0u8; BLOCK_SIZE];
    let mut read_buffer: [u8; 4096] = [0u8; BLOCK_SIZE];
    for i in 0..BLOCK_SIZE {
        for byte in write_buffer.iter_mut() {
            *byte = i as u8;
        }
        block_device.write_block(i as usize, &write_buffer);
        block_device.read_block(i as usize, &mut read_buffer);
        assert_eq!(write_buffer, read_buffer);
    }
    kprintln!("block device test passed!");
}
