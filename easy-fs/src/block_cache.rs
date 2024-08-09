use crate::{BlockDevice, BLOCK_CACHE_NUM, BLOCK_SIZE};
use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::lazy_static;
use spin::Mutex;

pub struct BlockCache {
    cache: Box<[u8; BLOCK_SIZE]>,       // cache buffer in memory
    block_id: usize,                    // which block does this cache come from
    block_device: Arc<dyn BlockDevice>, // reference to low level block device for reading and writing.
    modified: bool,                     // modified in memory ?
}

impl BlockCache {
    /// Load a new BlockCache from disk.
    pub fn new(block_id: usize, block_device: Arc<dyn BlockDevice>) -> Self {
        let mut cache = Box::new([0u8; BLOCK_SIZE]);
        block_device.read_block(block_id, &mut *cache);
        Self {
            cache,
            block_id,
            block_device,
            modified: false,
        }
    }
    // turn the offset of cache into memory address
    fn addr_of_offset(&self, offset: usize) -> usize {
        &self.cache[offset] as *const _ as usize
    }
    /// Reinterpret data in cache memory from `offset` as `T` type object and return its reference.
    pub fn get_ref<T>(&self, offset: usize) -> &T
    where
        T: Sized,
    {
        let type_size = core::mem::size_of::<T>();
        assert!(offset + type_size <= BLOCK_SIZE);
        let addr = self.addr_of_offset(offset);
        unsafe { &*(addr as *const T) }
    }
    /// Reinterpret data in cache memory from `offset` as `T` type object and return its mutable reference.
    /// Using [`BlockCache::get_mut`] instead of [`BlockCache::get_ref`] implies modification.
    pub fn get_mut<T>(&mut self, offset: usize) -> &mut T
    where
        T: Sized,
    {
        let type_size = core::mem::size_of::<T>();
        assert!(offset + type_size <= BLOCK_SIZE);
        self.modified = true;
        let addr = self.addr_of_offset(offset);
        unsafe { &mut *(addr as *mut T) }
    }
    /// Bind closure `f` to a certain block cache and execute it.
    pub fn read<T, V>(&self, offset: usize, f: impl FnOnce(&T) -> V) -> V {
        f(self.get_ref(offset))
    }
    /// Bind closure `f` to a certain block cache and execute it.
    pub fn modify<T, V>(&mut self, offset: usize, f: impl FnOnce(&mut T) -> V) -> V {
        f(self.get_mut(offset))
    }
    /// Sync cache with block device.
    /// There should be a background process write cache back to block device every 30 seconds.
    pub fn sync(&mut self) {
        if self.modified {
            self.modified = false;
            self.block_device.write_block(self.block_id, &*self.cache);
        }
    }
}

/// RAII: write back to block device when dropping [`BlockCache`]
impl Drop for BlockCache {
    fn drop(&mut self) {
        self.sync()
    }
}

pub struct BlockCacheManager {
    // (cache-id, cache-ref)
    queue: VecDeque<(usize, Arc<Mutex<BlockCache>>)>,
}

impl BlockCacheManager {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }
    pub fn get_block_cache(
        &mut self,
        block_id: usize,
        block_device: Arc<dyn BlockDevice>,
    ) -> Arc<Mutex<BlockCache>> {
        if let Some(pair) = self.queue.iter().find(|pair| pair.0 == block_id) {
            Arc::clone(&pair.1)
        } else {
            // substitute, FIFO
            if self.queue.len() == BLOCK_CACHE_NUM {
                // from front to tail
                if let Some((idx, _)) = self
                    .queue
                    .iter()
                    .enumerate()
                    .find(|(_, pair)| Arc::strong_count(&pair.1) == 1)
                {
                    self.queue.drain(idx..=idx); // evacuate selected cache
                } else {
                    panic!("Run out of BlockCache!");
                }
            }
            // load block into memory and push back
            let block_cache = Arc::new(Mutex::new(BlockCache::new(
                block_id,
                Arc::clone(&block_device),
            )));
            self.queue.push_back((block_id, Arc::clone(&block_cache)));
            block_cache
        }
    }
}

lazy_static! {
    pub static ref BLOCK_CACHE_MANAGER: Mutex<BlockCacheManager> =
        Mutex::new(BlockCacheManager::new());
}
/// Get the block cache corresponding to the given block id and block device
pub fn get_block_cache(
    block_id: usize,
    block_device: Arc<dyn BlockDevice>,
) -> Arc<Mutex<BlockCache>> {
    BLOCK_CACHE_MANAGER
        .lock()
        .get_block_cache(block_id, block_device)
}

/// Sync all block cache to block device
pub fn block_cache_sync_all() {
    let manager = BLOCK_CACHE_MANAGER.lock();
    for (_, cache) in manager.queue.iter() {
        cache.lock().sync();
    }
}
