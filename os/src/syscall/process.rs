//! Process management syscalls
use crate::{ task::{change_program_brk, exit_current_and_run_next, get_syscall_count, suspend_current_and_run_next}, timer::get_time_us};

use crate::mm::{PageTable, VirtAddr, PhysAddr};
use crate::task::current_user_token;

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
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    // get vpn from ts
    
    let us = get_time_us();
   let time_val = TimeVal {
    sec: us / 1_000_000,
    usec: us % 1_000_000,
   };
    copy_to_user(current_user_token(), ts as usize, &time_val);
    0
}


fn copy_to_user(token: usize, dst: usize,  time_val: & TimeVal) {
    let start = dst;
    let start_va = VirtAddr::from(start);
    let start_vpn = start_va.floor();
    let page_table = PageTable::from_token(token);
    let ppn : usize = page_table.translate(start_vpn).unwrap().ppn().into();
    let pa = PhysAddr::from(ppn + start_va.page_offset());
    let ptr = pa.get_mut::<TimeVal>();
    
    *ptr = TimeVal {
        sec: time_val.sec,
        usec: time_val.usec,
    };
}



/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(trace_request: usize, id: usize, _data: usize) -> isize {
    trace!("kernel: sys_trace");
    match trace_request {
        2 => {
            get_syscall_count(id)
        }
        _ => {
            -1
        }
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    -1
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    -1
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
