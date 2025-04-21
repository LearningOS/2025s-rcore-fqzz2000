use super::{
    block_cache_sync_all, get_block_cache, BlockDevice, DirEntry, DiskInode, DiskInodeType,
    EasyFileSystem, DIRENT_SZ,
};
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::{Mutex, MutexGuard};
/// Virtual filesystem layer over easy-fs
pub struct Inode {
    block_id: usize,
    block_offset: usize,
    fs: Arc<Mutex<EasyFileSystem>>,
    block_device: Arc<dyn BlockDevice>,
    deleted: bool,
}

impl Inode {
    /// Create a vfs inode
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
            deleted: false,
        }
    }
    /// Get the number of hard links
    pub fn get_nlink(&self) -> u32 {
        if self.read_disk_inode(|disk_inode| disk_inode.is_file()) {
            self.read_disk_inode(|disk_inode| disk_inode.nlink)
        } else {
            let target_inode_id = self.read_disk_inode(|disk_inode| disk_inode.direct[0]);
            let (target_block_id, target_block_offset) = self.fs.lock().get_disk_inode_pos(target_inode_id);
            get_block_cache(target_block_id as usize, Arc::clone(&self.block_device))
                .lock()
                .read(target_block_offset, |target_inode: &DiskInode| {
                    target_inode.nlink
                })
        }
    }
    /// Set the number of hard links
    pub fn set_nlink(&self, nlink: u32) {
        self.modify_disk_inode(|disk_inode| disk_inode.nlink = nlink);
    }
    /// Get the mode of the inode
    pub fn get_mode(&self) -> isize {
        self.read_disk_inode(|disk_inode| {
            if disk_inode.is_dir() {
                return 0 as isize
            } else if disk_inode.is_file() {
                return 1 as isize
            } else if disk_inode.is_hard_link() {
                return 2 as isize
            }
            return -1 as isize
        })
    }

    /// Get the inode id
    pub fn get_inode_id(&self) -> u32 {
        if self.read_disk_inode(|disk_inode| disk_inode.is_file()) {
            self.read_disk_inode(|disk_inode| disk_inode.inode_id)
        } else {
            self.read_disk_inode(|disk_inode| disk_inode.direct[0])
        }
    }
    /// Call a function over a disk inode to read it
    fn read_disk_inode<V>(&self, f: impl FnOnce(&DiskInode) -> V) -> V {
        get_block_cache(self.block_id, Arc::clone(&self.block_device))
            .lock()
            .read(self.block_offset, f)
    }
    /// Call a function over a disk inode to modify it
    fn modify_disk_inode<V>(&self, f: impl FnOnce(&mut DiskInode) -> V) -> V {
        get_block_cache(self.block_id, Arc::clone(&self.block_device))
            .lock()
            .modify(self.block_offset, f)
    }
    /// Find inode under a disk inode by name
    fn find_inode_id(&self, name: &str, disk_inode: &DiskInode) -> Option<u32> {
        log::info!("find_inode_id: {}", name);
        // assert it is a directory
        assert!(disk_inode.is_dir());
        let file_count = (disk_inode.size as usize) / DIRENT_SZ;
        let mut dirent = DirEntry::empty();
        for i in 0..file_count {
            assert_eq!(
                disk_inode.read_at(DIRENT_SZ * i, dirent.as_bytes_mut(), &self.block_device,),
                DIRENT_SZ,
            );
            if dirent.name() == name {
                log::info!("find_inode_id: {} found", name);
                return Some(dirent.inode_id() as u32)
              }
        }
        log::info!("find_inode_id: {} not found", name);
        None
    }
   
    /// Find inode under current inode by name
    pub fn find(&self, name: &str) -> Option<Arc<Inode>> {
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
    /// Increase the size of a disk inode
    fn increase_size(
        &self,
        new_size: u32,
        disk_inode: &mut DiskInode,
        fs: &mut MutexGuard<EasyFileSystem>,
    ) {
        if new_size < disk_inode.size {
            return;
        }
        let blocks_needed = disk_inode.blocks_num_needed(new_size);
        let mut v: Vec<u32> = Vec::new();
        for _ in 0..blocks_needed {
            v.push(fs.alloc_data());
        }
        disk_inode.increase_size(new_size, v, &self.block_device);
    }

    /// Create inode under current inode by name
    pub fn  create(&self, name: &str) -> Option<Arc<Inode>> {
        self.linkat(name, DiskInodeType::File, None)
    }

    /// Create inode under current inode by name
    pub fn linkat(&self, name: &str, inode_type: DiskInodeType, target_file_name: Option<&str>) -> Option<Arc<Inode>> {
        log::info!("create: {}", name);
        let mut fs = self.fs.lock();
        let op = |root_inode: &DiskInode| {
            // assert it is a directory
            assert!(root_inode.is_dir());
            // has the file been created?
            self.find_inode_id(name, root_inode)
        };
        if self.read_disk_inode(op).is_some() {
            return None;
        }
        // create a new file
        // alloc a inode with an indirect block
        let new_inode_id = fs.alloc_inode();
        let mut target_inode_id: Option<u32> = None;
        if let Some(target) = target_file_name {
            // find the target file inode id
            target_inode_id = self.read_disk_inode(|root_inode| {
                self.find_inode_id(target, root_inode)
            });
            if target_inode_id.is_none() {
                return None;
            } 
        }
        // initialize inode
        let (new_inode_block_id, new_inode_block_offset) = fs.get_disk_inode_pos(new_inode_id);
        get_block_cache(new_inode_block_id as usize, Arc::clone(&self.block_device))
            .lock()
            .modify(new_inode_block_offset, |new_inode: &mut DiskInode| {
                
                // if it is a hard link, we need to set the target file name
                if inode_type == DiskInodeType::HardLink {
                    new_inode.initialize(DiskInodeType::HardLink, new_inode_id);
                    new_inode.direct[0] = target_inode_id.unwrap();
                } else {
                    new_inode.initialize(DiskInodeType::File, new_inode_id);
                }
            });
        self.modify_disk_inode(|root_inode| {
            // append file in the dirent
            let file_count = (root_inode.size as usize) / DIRENT_SZ;
            let new_size = (file_count + 1) * DIRENT_SZ;
            // increase size
            self.increase_size(new_size as u32, root_inode, &mut fs);
            // write dirent
            let dirent = DirEntry::new(name, new_inode_id);
            root_inode.write_at(
                file_count * DIRENT_SZ,
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
            self.fs.clone(),
            self.block_device.clone(),
        )))
        // release efs lock automatically by compiler
    }

    /// Delete a file
    pub fn delete(&self, name: &str) -> bool {
        // reverse the create process
        let mut fs = self.fs.lock();
        let op = |root_inode: &DiskInode| {
            self.find_inode_id(name, root_inode)
        };
        if self.read_disk_inode(op).is_none() {
            return false;
        }
        
        let to_delete_inode_id = self.read_disk_inode(op).unwrap();
        let (block_id, block_offset) = fs.get_disk_inode_pos(to_delete_inode_id);
        let is_file = get_block_cache(block_id as usize, Arc::clone(&self.block_device))
            .lock()
            .read(block_offset, |disk_inode: &DiskInode| disk_inode.is_file());
        if is_file {
            get_block_cache(block_id as usize, Arc::clone(&self.block_device))
                .lock()
                .modify(block_offset, |disk_inode: &mut DiskInode| disk_inode.nlink -= 1);
        } else {
            let target_inode_id = get_block_cache(block_id as usize, Arc::clone(&self.block_device))
                .lock()
                .read(block_offset, |disk_inode: &DiskInode| disk_inode.direct[0]);
            let (target_block_id, target_block_offset) = fs.get_disk_inode_pos(target_inode_id);
            get_block_cache(target_block_id as usize, Arc::clone(&self.block_device))
                .lock()
                .modify(target_block_offset, |target_inode: &mut DiskInode| target_inode.nlink -= 1);
        }

        // delete the inode
        self.modify_disk_inode(|root_inode| {
            let file_count = (root_inode.size as usize) / DIRENT_SZ;
            let mut new_entries = Vec::new();
            // match filename
            for i in 0..file_count {
                let mut new_entry = DirEntry::empty();
                root_inode.read_at(i * DIRENT_SZ, new_entry.as_bytes_mut(), &self.block_device);
                if new_entry.name() != name {
                    new_entries.push(new_entry);
                }
            }
            // remove old entries
            root_inode.increase_size(
                (new_entries.len() * DIRENT_SZ) as u32,
                Vec::new(),
                &self.block_device,
            );
            // write new entries
            for (i, entry) in new_entries.iter().enumerate() {
                root_inode.write_at(i * DIRENT_SZ, entry.as_bytes(), &self.block_device);
            }
            // write new entries
        });

        block_cache_sync_all();
        true
    }
    /// List inodes under current inode
    pub fn ls(&self) -> Vec<String> {
        let _fs = self.fs.lock();
        self.read_disk_inode(|disk_inode| {
            let file_count = (disk_inode.size as usize) / DIRENT_SZ;
            let mut v: Vec<String> = Vec::new();
            for i in 0..file_count {
                let mut dirent = DirEntry::empty();
                assert_eq!(
                    disk_inode.read_at(i * DIRENT_SZ, dirent.as_bytes_mut(), &self.block_device,),
                    DIRENT_SZ,
                );
                v.push(String::from(dirent.name()));
            }
            v
        })
    }
    /// Read data from current inode
    pub fn read_at(&self, offset: usize, buf: &mut [u8]) -> usize {
        let fs = self.fs.lock();
        let is_file = self.read_disk_inode(|disk_inode| disk_inode.is_file());
        if is_file {
            self.read_disk_inode(|disk_inode| disk_inode.read_at(offset, buf, &self.block_device))
        } else {
            // let target_inode_id = self.read_disk_inode(|disk_inode| disk_inode.direct[0]);
            // let (target_block_id, target_block_offset) = fs.get_disk_inode_pos(target_inode_id);
            // get_block_cache(target_block_id as usize, Arc::clone(&self.block_device))
            //     .lock()
            //     .read(target_block_offset, |target_inode: &DiskInode| {
            //         target_inode.read_at(offset, buf, &self.block_device)
            //     })
            offset
        }
    }
    /// Write data to current inode
    pub fn write_at(&self, offset: usize, buf: &[u8]) -> usize {
        let mut fs = self.fs.lock();
        let is_file = self.read_disk_inode(|disk_inode| disk_inode.is_file());
        if is_file {
            self.modify_disk_inode(|disk_inode| {
                self.increase_size((offset + buf.len()) as u32, disk_inode, &mut fs);
                disk_inode.write_at(offset, buf, &self.block_device)
            })
        } else {
            let target_inode_id = self.read_disk_inode(|disk_inode| disk_inode.direct[0]);
            let (target_block_id, target_block_offset) = fs.get_disk_inode_pos(target_inode_id);
            get_block_cache(target_block_id as usize, Arc::clone(&self.block_device))
                .lock()
                .modify(target_block_offset, |target_inode: &mut DiskInode| {
                    self.increase_size((offset + buf.len()) as u32, target_inode, &mut fs);
                    target_inode.write_at(offset, buf, &self.block_device)
                })
        }
    }
    /// Clear the data in current inode
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
}
