//! Process management syscalls
use crate::config::PAGE_SIZE;
use crate::task::{change_program_brk, exit_current_and_run_next, get_syscall_count, suspend_current_and_run_next};
use crate::timer::get_time_us;
use crate::mm::{MapPermission, PageTable, PageTableEntry, PhysAddr, VirtAddr};
use crate::task::{map_memory, current_user_token, unmap_memory};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");    
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    // get vpn from ts
    
    let us = get_time_us();
   let time_val = TimeVal {
    sec: us / 1_000_000,
    usec: us % 1_000_000,
   };
    copy_to_user(current_user_token(), _ts as usize, &time_val);
    0
}


fn copy_to_user(token: usize, dst: usize,  time_val: & TimeVal) {
    let start = dst;
    let start_va = VirtAddr::from(start);
    let start_vpn = start_va.floor();
    let page_table = PageTable::from_token(token);
    let ppn : PhysAddr = page_table.translate(start_vpn).unwrap().ppn().into();
    let ppn_usize : usize = ppn.into();
    let pa : PhysAddr = PhysAddr::from(ppn_usize + start_va.page_offset());
    let ptr = pa.get_mut::<TimeVal>();
    
    *ptr = TimeVal {
        sec: time_val.sec,
        usec: time_val.usec,
    };
}

fn pte_to_pa(pte: &PageTableEntry, offset: usize) -> PhysAddr {
    let ppn : PhysAddr = pte.ppn().into();
    let ppn_usize : usize = ppn.into();
    let pa : PhysAddr = PhysAddr::from(ppn_usize + offset);
    pa
}


/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");
    match trace_request {
        0 => {
            // read from id
            info!("read from id: {}", id);
            if id > (1 << 39 - 1) {
                return -1;
            }
            let page_table = PageTable::from_token(current_user_token());
            let va = VirtAddr::from(id);
            if let Some(pte) = page_table.translate(va.floor()) {
                if pte.readable() && pte.is_valid() {
                    info!("pte is readable");
                    // todo
                    let pa = pte_to_pa(&pte, va.page_offset());
                    unsafe {
                        // read from pa
                        *(pa.0 as *mut u8) as isize
                    }

                } else {
                    info!("pte is not readable");
                    -1
                }
            } else {
                info!("pte is not found");
                -1
            }
        }
        1 => {
            // write to id
            info!("write to id: {}", id);
            if id > (1 << 39 - 1) {
                return -1;
            }
            let page_table = PageTable::from_token(current_user_token());
            let va = VirtAddr::from(id);
            if let Some(pte) = page_table.translate(va.floor()) {
                if pte.writable() && pte.is_valid() {
                    // todo
                    let pa = pte_to_pa(&pte, va.page_offset());
                    unsafe {
                        // write to pa
                        *(pa.0 as *mut u8) = data as u8;
                    }
                    return 0;
                } else {
                    -1
                }
            } else {
                -1
            }
        }
        2 => {
            get_syscall_count(id)
        }
        _ => {
            -1
        }
    }
}

fn is_overlap(start: &usize, len: &usize, page_table: &PageTable) -> bool {
    let start_va = VirtAddr::from(*start);
    let mut start_vpn = start_va.floor();

    let end_vpn = VirtAddr::from(start + len).ceil();
    let end_vpn_usize : usize = end_vpn.into();
    let mut start_vpn_usize : usize = start_vpn.into();

    while start_vpn_usize < end_vpn_usize {
        // check if given vpn in page table
        if let Some(_pte) = page_table.translate(start_vpn) {
            if _pte.is_valid() {
                info!("vpn: {} is mapped", start_vpn.0);
                return true;
            }
        }
        start_vpn.0 += 1;
        start_vpn_usize = start_vpn.into();
    }
    false    
}

fn all_overlap(start: &usize, len: &usize, page_table: &PageTable) -> bool {
    let start_va = VirtAddr::from(*start);
    let mut start_vpn = start_va.floor();

    let end_vpn = VirtAddr::from(start + len).ceil();
    let end_vpn_usize : usize = end_vpn.into();
    let mut start_vpn_usize : usize = start_vpn.into();
    while start_vpn_usize < end_vpn_usize {
        // check if given vpn in page table
        if let Some(_pte) = page_table.translate(start_vpn) {
            if !_pte.is_valid() {
                return false;
            }
        } else {
            return false;
        }
        start_vpn.0 += 1;
        start_vpn_usize = start_vpn.into();
    }
    true    
    
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _prot: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    // check prot
    if _prot & !0x7 != 0 || _prot & 0x7 == 0 {
        error!("failed due to prot");
        return -1;
    }
    // check align
    let va = VirtAddr::from(_start);
    if !va.aligned() {
        error!("failed due to alignment");
        return -1;
    }
    // check len
    if _len == 0 {
        return 0;
    }
    // check if any pages already mapped
    let page_table = PageTable::from_token(current_user_token());
    if is_overlap(&_start, &_len, &page_table) {
        error!("failed due to overlap");
        return -1;
    }
    // map pages
    let mut permission = MapPermission::U;
    if _prot & 0x1 != 0 {
        permission.insert(MapPermission::R);
    }
    if _prot & 0x2 != 0 {
        permission.insert(MapPermission::W);
    }
    if _prot & 0x4 != 0 {
        permission.insert(MapPermission::X);
    }

    map_memory(va, VirtAddr::from(_start + _len), permission);
    0

}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    if _start % PAGE_SIZE != 0 {
        return -1; 
    }
    let page_table = PageTable::from_token(current_user_token());
    if !all_overlap(&_start, &_len, &page_table) {
        return -1;
    }
    unmap_memory(_start, _len);
    0
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
