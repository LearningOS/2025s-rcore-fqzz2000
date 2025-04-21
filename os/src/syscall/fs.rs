//! File and filesystem-related syscalls
use crate::fs::{open_file, OpenFlags, Stat, create_hardlink, unlink};
use crate::mm::{translated_byte_buffer, translated_str, UserBuffer};
use crate::task::{current_task, current_user_token};
use crate::mm::{PageTable, VirtAddr, PhysAddr};

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel:pid[{}] sys_write", current_task().unwrap().pid.0);
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        if !file.writable() {
            return -1;
        }
        let file = file.clone();
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.write(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel:pid[{}] sys_read", current_task().unwrap().pid.0);
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        if !file.readable() {
            return -1;
        }
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        trace!("kernel: sys_read .. file.read");
        file.read(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    trace!("kernel:pid[{}] sys_open", current_task().unwrap().pid.0);
    let task = current_task().unwrap();
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(inode) = open_file(path.as_str(), OpenFlags::from_bits(flags).unwrap()) {
        let mut inner = task.inner_exclusive_access();
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    trace!("kernel:pid[{}] sys_close", current_task().unwrap().pid.0);
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

fn copy_to_user(token: usize, dst: usize,  stat: & Stat)  {
    let start = dst;
    let start_va = VirtAddr::from(start);
    let start_vpn = start_va.floor();
    let page_table = PageTable::from_token(token);
    let ppn : PhysAddr = page_table.translate(start_vpn).unwrap().ppn().into();
    let ppn_usize : usize = ppn.into();
    let pa : PhysAddr = PhysAddr::from(ppn_usize + start_va.page_offset());
    let ptr = pa.get_mut::<Stat>();
    
    *ptr = Stat {
        dev: stat.dev,
        ino: stat.ino,
        mode: stat.mode,
        nlink: stat.nlink,
        pad: stat.pad,
    };
    
}

/// YOUR JOB: Implement fstat.
pub fn sys_fstat(fd: usize, st: *mut Stat) -> isize {
    trace!(
        "kernel:pid[{}] sys_fstat NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        drop(inner);
        let stat = file.fstat();
        copy_to_user(token, st as usize, &stat);
        0
    } else {
        -1
    }
    
   
    
}

/// YOUR JOB: Implement linkat.
pub fn sys_linkat(old_name: *const u8, new_name: *const u8) -> isize {
    info!("sys_linkat called");
    // check if the old name and new name are the same
    if old_name == new_name {
        return -1;
    }
    // let task = current_task().unwrap();
    let token = current_user_token();
    let old_name = translated_str(token, old_name);
    let new_name = translated_str(token, new_name);
    info!("old_name: {}", old_name);
    info!("new_name: {}", new_name);

    // check if old name and new name are the same
    if old_name.as_str() == new_name.as_str() {
        return -1;
    }
    info!("create_hardlink called");
    if let Some(_inode) = create_hardlink( new_name.as_str(), old_name.as_str()) {
        // let mut inner = task.inner_exclusive_access();
        // let fd = inner.alloc_fd();
        // inner.fd_table[fd] = Some(inode);
        // fd as isize
        0
    } else {
        -1
    }
}

/// YOUR JOB: Implement unlinkat.
pub fn sys_unlinkat(_name: *const u8) -> isize {
    info!("sys_unlinkat called");
    let token = current_user_token();
    let name = translated_str(token, _name);
    info!("name: {}", name);
    if unlink(name.as_str()) {
        0
    } else {
        -1
    }
}
