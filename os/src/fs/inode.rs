//! `Arc<Inode>` -> `OSInodeInner`: In order to open files concurrently
//! we need to wrap `Inode` into `Arc`,but `Mutex` in `Inode` prevents
//! file systems from being accessed simultaneously
//!
//! `UPSafeCell<OSInodeInner>` -> `OSInode`: for static `ROOT_INODE`,we
//! need to wrap `OSInodeInner` into `UPSafeCell`
use super::File;
use crate::drivers::BLOCK_DEVICE;
use crate::mm::UserBuffer;
use crate::sync::UPSafeCell;
use alloc::sync::Arc;
use alloc::vec::Vec;
use bitflags::*;
use easy_fs::{EasyFileSystem, Inode, DiskInodeType};
use lazy_static::*;
use crate::fs::Stat;
use crate::fs::StatMode;
/// inode in memory
/// A wrapper around a filesystem inode
/// to implement File trait atop
pub struct OSInode {
    readable: bool,
    writable: bool,
    inner: UPSafeCell<OSInodeInner>,
}
/// The OS inode inner in 'UPSafeCell'
pub struct OSInodeInner {
    offset: usize,
    inode: Arc<Inode>,
}

impl OSInode {
    /// create a new inode in memory
    pub fn new(readable: bool, writable: bool, inode: Arc<Inode>) -> Self {
        Self {
            readable,
            writable,
            inner: unsafe { UPSafeCell::new(OSInodeInner { offset: 0, inode }) },
        }
    }
    /// read all data from the inode
    pub fn read_all(&self) -> Vec<u8> {
        let mut inner = self.inner.exclusive_access();
        let mut buffer: Vec<u8> = Vec::with_capacity(512);
        buffer.resize(512, 0);
        let mut v: Vec<u8> = Vec::new();
        loop {
            let len = inner.inode.read_at(inner.offset, &mut buffer);
            if len == 0 {
                break;
            }
            inner.offset += len;
            v.extend_from_slice(&buffer[..len]);
        }
        v
    }
}

lazy_static! {
    pub static ref ROOT_INODE: Arc<Inode> = {
        let efs = EasyFileSystem::open(BLOCK_DEVICE.clone());
        Arc::new(EasyFileSystem::root_inode(&efs))
    };
}

/// List all apps in the root directory
pub fn list_apps() {
    println!("/**** APPS ****");
    for app in ROOT_INODE.ls() {
        println!("{}", app);
    }
    println!("**************/");
}

bitflags! {
    ///  The flags argument to the open() system call is constructed by ORing together zero or more of the following values:
    pub struct OpenFlags: u32 {
        /// readyonly
        const RDONLY = 0;
        /// writeonly
        const WRONLY = 1 << 0;
        /// read and write
        const RDWR = 1 << 1;
        /// create new file
        const CREATE = 1 << 9;
        /// truncate file size to 0
        const TRUNC = 1 << 10;
    }
}

impl OpenFlags {
    /// Do not check validity for simplicity
    /// Return (readable, writable)
    pub fn read_write(&self) -> (bool, bool) {
        if self.is_empty() {
            (true, false)
        } else if self.contains(Self::WRONLY) {
            (false, true)
        } else {
            (true, true)
        }
    }
}

/// create a hardlink
pub fn create_hardlink(name: &str, target: &str) -> Option<Arc<OSInode>> {
    let inode = ROOT_INODE.linkat(name, DiskInodeType::HardLink, Some(target));
    let target_inode = ROOT_INODE.find(target).unwrap();
    target_inode.set_nlink(target_inode.get_nlink() + 1);
    inode.map(|inode| Arc::new(OSInode::new(true, true, inode)))
}
///unlink a file
pub fn unlink(name: &str) -> bool {
    ROOT_INODE.delete(name)
}

/// Open a file
pub fn open_file(name: &str, flags: OpenFlags) -> Option<Arc<OSInode>> {
    info!("open_file: {}", name);
    let (readable, writable) = flags.read_write();
    if flags.contains(OpenFlags::CREATE) {
        info!("create file");
        if let Some(inode) = ROOT_INODE.find(name) {
            // clear size
            inode.clear();
            Some(Arc::new(OSInode::new(readable, writable, inode)))
        } else {
            // create file
            ROOT_INODE
                .linkat(name, DiskInodeType::File, None)
                .map(|inode| Arc::new(OSInode::new(readable, writable, inode)))
        }
    } else {
        info!("open file");
        ROOT_INODE.find(name).map(|inode| {
            if flags.contains(OpenFlags::TRUNC) {
                inode.clear();
            }
            Arc::new(OSInode::new(readable, writable, inode))
        })
    }
}

impl File for OSInode {
    fn readable(&self) -> bool {
        self.readable
    }
    fn writable(&self) -> bool {
        self.writable
    }
    fn read(&self, mut buf: UserBuffer) -> usize {
        let mut inner = self.inner.exclusive_access();
        let mut total_read_size = 0usize;
        for slice in buf.buffers.iter_mut() {
            let read_size = inner.inode.read_at(inner.offset, *slice);
            if read_size == 0 {
                break;
            }
            inner.offset += read_size;
            total_read_size += read_size;
        }
        total_read_size
    }
    fn write(&self, buf: UserBuffer) -> usize {
        let mut inner = self.inner.exclusive_access();
        let mut total_write_size = 0usize;
        for slice in buf.buffers.iter() {
            let write_size = inner.inode.write_at(inner.offset, *slice);
            assert_eq!(write_size, slice.len());
            inner.offset += write_size;
            total_write_size += write_size;
        }
        total_write_size
    }

    fn fstat(&self) -> Stat {
        let inner = self.inner.exclusive_access();
        let ino = inner.inode.get_inode_id();
        let mode = match inner.inode.get_mode() {
            0 => StatMode::DIR,
            1 => StatMode::FILE,
            2 => StatMode::LINK,
            _ => StatMode::NULL,
        };
        let nlink = inner.inode.get_nlink();

        let mut stat = Stat::new();
        stat.ino = ino as u64;
        stat.mode = mode;
        stat.nlink = nlink as u32;
        stat
    }

}
