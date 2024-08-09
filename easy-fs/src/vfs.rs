use core::fmt::Debug;

use alloc::{string::String, sync::Arc, vec::Vec};
use spin::{Mutex, MutexGuard};

use crate::{
    block_cache_sync_all, get_block_cache, BlockDevice, DirEntry, DiskInode, DiskInodeType,
    EasyFileSystem, DIRENT_SIZE,
};

/// Virtual filesystem layer over easy-fs
pub struct Inode {
    block_id: usize,
    block_offset: usize,
    fs: Arc<Mutex<EasyFileSystem>>,
    block_device: Arc<dyn BlockDevice>,
}

impl Debug for Inode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Inode {{ block_id: {}, block_offset: {}}}",
            self.block_id, self.block_offset
        )
    }
}

impl Inode {
    /// Create a new [`Inode`] at (`block_id`, `block_offset`) in `fs` on `block_device`
    pub fn new(
        block_id: u32,
        block_offset: usize,
        fs: Arc<Mutex<EasyFileSystem>>,
        block_device: Arc<dyn BlockDevice>,
    ) -> Self {
        Self {
            block_id: block_id as usize,
            block_offset,
            fs,
            block_device,
        }
    }
    fn read_disk_inode<V>(&self, f: impl FnOnce(&DiskInode) -> V) -> V {
        get_block_cache(self.block_id, Arc::clone(&self.block_device))
            .lock()
            .read(self.block_offset, f)
    }
    fn modify_disk_inode<V>(&self, f: impl FnOnce(&mut DiskInode) -> V) -> V {
        get_block_cache(self.block_id, Arc::clone(&self.block_device))
            .lock()
            .modify(self.block_offset, f)
    }
    fn is_dir(&self) -> bool {
        get_block_cache(self.block_id, Arc::clone(&self.block_device))
            .lock()
            .read(self.block_offset, |disk_inode: &DiskInode| {
                disk_inode.is_dir()
            })
    }
    fn is_file(&self) -> bool {
        get_block_cache(self.block_id, Arc::clone(&self.block_device))
            .lock()
            .read(self.block_offset, |disk_inode: &DiskInode| {
                disk_inode.is_file()
            })
    }
    /// Find a directory entry with `name`.
    /// Note: must assure that `self` correspond to a [`DiskInode`] with directory type.
    pub fn find(&self, name: &str) -> Option<Arc<Inode>> {
        assert!(self.is_dir());
        // lock to assure block cache exclusive accessing
        // avoid multiple cores accessing the filesystem concurrently
        let fs = self.fs.lock();
        self.read_disk_inode(|disk_inode| {
            self.find_inode_id(name, disk_inode).map(|inode_id| {
                let (block_id, block_offset) = fs.get_disk_inode_pos(inode_id);
                Arc::new(Self::new(
                    block_id,
                    block_offset,
                    self.fs.clone(),
                    self.block_device.clone(),
                ))
            })
        })
    }
    fn find_inode_id(&self, name: &str, disk_inode: &DiskInode) -> Option<u32> {
        // assert it is a directory
        assert!(disk_inode.is_dir());
        let file_count = (disk_inode.size as usize) / DIRENT_SIZE;
        let mut dirent = DirEntry::empty();
        for i in 0..file_count {
            let dirent_read_bytes =
                disk_inode.read_at(DIRENT_SIZE * i, dirent.as_bytes_mut(), &self.block_device);
            assert_eq!(dirent_read_bytes, DIRENT_SIZE);
            if dirent.name() == name {
                return Some(dirent.inode_number() as u32);
            }
        }
        None
    }
    /// Get all directory enties name.
    /// /// Note: must assure that `self` correspond to a [`DiskInode`] with directory type.
    pub fn ls(&self) -> Vec<String> {
        assert!(self.is_dir());
        let _fs = self.fs.lock();
        self.read_disk_inode(|disk_inode| {
            let file_count = (disk_inode.size as usize) / DIRENT_SIZE;
            let mut v: Vec<String> = Vec::new();
            for i in 0..file_count {
                let mut dirent = DirEntry::empty();
                let dirent_read_bytes =
                    disk_inode.read_at(DIRENT_SIZE * i, dirent.as_bytes_mut(), &self.block_device);
                assert_eq!(dirent_read_bytes, DIRENT_SIZE);
                v.push(String::from(dirent.name()));
            }
            v
        })
    }
    /// Create a new file as `name`.
    /// /// Note: must assure that `self` correspond to a [`DiskInode`] with directory type.
    pub fn create(&self, name: &str) -> Option<Arc<Inode>> {
        assert!(self.is_dir());
        let mut fs = self.fs.lock();
        if self
            .modify_disk_inode(|root_inode| {
                // assert it is a directory
                assert!(root_inode.is_dir());
                // has the file been created?
                self.find_inode_id(name, root_inode)
            })
            .is_some()
        {
            return None;
        }
        // create a new file
        // alloc a inode with an indirect block
        let new_inode_id = fs.alloc_inode();
        // initialize inode
        let (new_inode_block_id, new_inode_block_offset) = fs.get_disk_inode_pos(new_inode_id);
        get_block_cache(new_inode_block_id as usize, Arc::clone(&self.block_device))
            .lock()
            .modify(new_inode_block_offset, |new_inode: &mut DiskInode| {
                new_inode.initialize(DiskInodeType::File);
            });
        self.modify_disk_inode(|root_inode| {
            // append file in the dirent
            let file_count = (root_inode.size as usize) / DIRENT_SIZE;
            let new_size = (file_count + 1) * DIRENT_SIZE;
            // increase size
            self.increase_size(new_size as u32, root_inode, &mut fs);
            // write dirent
            let dirent = DirEntry::new(name, new_inode_id);
            root_inode.write_at(
                file_count * DIRENT_SIZE,
                dirent.as_bytes(),
                &self.block_device,
            );
        });
        let (block_id, block_offset) = fs.get_disk_inode_pos(new_inode_id);
        block_cache_sync_all();
        // return inode
        Some(Arc::new(Self::new(
            block_id,
            block_offset,
            Arc::clone(&self.fs),
            Arc::clone(&self.block_device),
        )))
        // release efs lock automatically by compiler
    }
    /// Clear contents of this inode.
    /// Note: must assure that `self` correspond to a [`DiskInode`] with directory type.
    pub fn clear(&self) {
        let mut fs = self.fs.lock();
        self.modify_disk_inode(|disk_inode| {
            let size = disk_inode.size;
            let data_blocks_dealloc = disk_inode.clear_size(&self.block_device);
            assert!(data_blocks_dealloc.len() == DiskInode::total_blocks(size) as usize);
            for data_block in data_blocks_dealloc.into_iter() {
                fs.dealloc_data(data_block);
            }
        });
        block_cache_sync_all();
    }
    /// Read data from current inode
    pub fn read_at(&self, offset: usize, buf: &mut [u8]) -> usize {
        assert!(self.is_file());
        let _fs = self.fs.lock();
        self.read_disk_inode(|disk_inode| disk_inode.read_at(offset, buf, &self.block_device))
    }
    /// Write data to current inode
    pub fn write_at(&self, offset: usize, buf: &[u8]) -> usize {
        assert!(self.is_file());
        let mut fs = self.fs.lock();
        let size = self.modify_disk_inode(|disk_inode| {
            self.increase_size((offset + buf.len()) as u32, disk_inode, &mut fs);
            disk_inode.write_at(offset, buf, &self.block_device)
        });
        block_cache_sync_all();
        size
    }
    fn increase_size(
        &self,
        new_size: u32,
        disk_inode: &mut DiskInode,
        fs: &mut MutexGuard<EasyFileSystem>,
    ) {
        if new_size < disk_inode.size {
            return;
        }
        let block_needed = disk_inode.blocks_num_needed(new_size);
        let mut v: Vec<u32> = Vec::new();
        for _ in 0..block_needed {
            v.push(fs.alloc_data());
        }
        disk_inode.increase_size(new_size, v, &self.block_device);
    }
}
