use crate::sync::{Condvar, Mutex, MutexBlocking, MutexSpin, Semaphore};
use crate::task::{block_current_and_run_next, current_process, current_task};
use crate::timer::{add_timer, get_time_ms};
use alloc::sync::Arc;
/// sleep syscall
pub fn sys_sleep(ms: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_sleep",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let expire_ms = get_time_ms() + ms;
    let task = current_task().unwrap();
    add_timer(expire_ms, task);
    block_current_and_run_next();
    0
}
/// mutex create syscall
pub fn sys_mutex_create(blocking: bool) -> isize {
    info!(
        "kernel:pid[{}] tid[{}] sys_mutex_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mutex: Option<Arc<dyn Mutex>> = if !blocking {
        Some(Arc::new(MutexSpin::new()))
    } else {
        Some(Arc::new(MutexBlocking::new()))
    };
    let mut process_inner = process.inner_exclusive_access();
    if let Some(id) = process_inner
        .mutex_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.mutex_list[id] = mutex;
        process_inner.mutex_available_resources[id] = 1;
        id as isize
    } else {
        process_inner.mutex_list.push(mutex);
        process_inner.mutex_available_resources.push(1);
        // push to all vectors in allocation matrix and need matrix
        process_inner.mutex_allocation_matrix.iter_mut().for_each(|v| v.push(0));
        process_inner.mutex_need_matrix.iter_mut().for_each(|v| v.push(0));
        process_inner.mutex_list.len() as isize - 1
    }
}
/// mutex lock syscall
pub fn sys_mutex_lock(mutex_id: usize) -> isize {
    info!(
        "kernel:pid[{}] tid[{}] sys_mutex_lock",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    info!("try lock mutex {}", mutex_id);
    let try_lock = process.try_lock_mutex(mutex_id);
    info!("try lock mutex done" );
    if try_lock.is_err() {
        info!("try lock mutex failed");
        return -0xdead;
    }
    let mut process_inner = process.inner_exclusive_access();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
        // update available resources and allocation matrix and need matrix
    process_inner.mutex_available_resources[mutex_id] -= 1;
    process_inner.mutex_allocation_matrix[current_task().unwrap().inner_exclusive_access().res.as_ref().unwrap().tid][mutex_id] += 1;
    process_inner.mutex_need_matrix[current_task().unwrap().inner_exclusive_access().res.as_ref().unwrap().tid][mutex_id] += 1;

    drop(process_inner);
    drop(process);
    mutex.lock();

    0
}
/// mutex unlock syscall
pub fn sys_mutex_unlock(mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_unlock",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    // update available resources and allocation matrix and n   eed matrix
    process_inner.mutex_available_resources[mutex_id] += 1;
    process_inner.mutex_allocation_matrix[current_task().unwrap().inner_exclusive_access().res.as_ref().unwrap().tid][mutex_id] -= 1;
    process_inner.mutex_need_matrix[current_task().unwrap().inner_exclusive_access().res.as_ref().unwrap().tid][mutex_id] -= 1;
    drop(process_inner);
    drop(process);
    mutex.unlock();
    0
}
/// semaphore create syscall
pub fn sys_semaphore_create(res_count: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .semaphore_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.semaphore_list[id] = Some(Arc::new(Semaphore::new(res_count)));
        process_inner.semaphore_available_resources[id] = res_count;
        process_inner.semaphore_allocation_matrix[current_task().unwrap().inner_exclusive_access().res.as_ref().unwrap().tid][id] = 0;
        process_inner.semaphore_need_matrix[current_task().unwrap().inner_exclusive_access().res.as_ref().unwrap().tid][id] = 0;
        id
    } else {
        process_inner
            .semaphore_list
            .push(Some(Arc::new(Semaphore::new(res_count))));
        process_inner.semaphore_available_resources.push(res_count);
        process_inner.semaphore_max_list.push(res_count);
        process_inner.semaphore_allocation_matrix.iter_mut().for_each(|v| v.push(0));
        process_inner.semaphore_need_matrix.iter_mut().for_each(|v| v.push(0));
        process_inner.semaphore_list.len() - 1
    };
    id as isize
}
/// semaphore up syscall
pub fn sys_semaphore_up(sem_id: usize) -> isize {
    info!(
        "kernel:pid[{}] tid[{}] sys_semaphore_up {}",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid,
        sem_id
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    process_inner.semaphore_available_resources[sem_id] += 1;
    process_inner.semaphore_allocation_matrix[current_task().unwrap().inner_exclusive_access().res.as_ref().unwrap().tid][sem_id] -= 1;
    
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    drop(process_inner);
    drop(process);
    sem.up();
    0
}
/// semaphore down syscall
pub fn sys_semaphore_down(sem_id: usize) -> isize {
    info!(
        "kernel:pid[{}] tid[{}] sys_semaphore_down {}",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid,
        sem_id
    );
    let process = current_process();

    match process.try_lock_semaphore(sem_id) {
        Err(e) => {
            info!("try lock semaphore failed: {}", e);
            return -0xdead as isize;
        }
        Ok(true) => {

        let mut process_inner = process.inner_exclusive_access();
        process_inner.semaphore_available_resources[sem_id] -= 1;
        process_inner.semaphore_allocation_matrix[current_task().unwrap().inner_exclusive_access().res.as_ref().unwrap().tid][sem_id] += 1;
        drop(process_inner);
    }
    Ok(false) => {
        info!("try lock semaphore wait");
    }
    }
    let process_inner = process.inner_exclusive_access();
    info!("semaphore_available_resources: {} for {}  ", process_inner.semaphore_available_resources[sem_id], sem_id);
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    drop(process_inner);
    drop(process);
    sem.down();
    0
}
/// condvar create syscall
pub fn sys_condvar_create() -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .condvar_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.condvar_list[id] = Some(Arc::new(Condvar::new()));
        id
    } else {
        process_inner
            .condvar_list
            .push(Some(Arc::new(Condvar::new())));
        process_inner.condvar_list.len() - 1
    };
    id as isize
}
/// condvar signal syscall
pub fn sys_condvar_signal(condvar_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_signal",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    drop(process_inner);
    condvar.signal();
    0
}
/// condvar wait syscall
pub fn sys_condvar_wait(condvar_id: usize, mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_wait",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    condvar.wait(mutex);
    0
}
/// enable deadlock detection syscall
///
/// YOUR JOB: Implement deadlock detection, but might not all in this syscall
pub fn sys_enable_deadlock_detect(enabled: usize) -> isize {
    info!("kernel: sys_enable_deadlock_detect enabled: {}", enabled);

    let process = current_process();
    match enabled {
        0 => {
            process.inner_exclusive_access().deadlock_detection = false;
        }
        1 => {
            process.inner_exclusive_access().deadlock_detection = true;
        }
        _ => {
            return -1;
        }
    }
    0
}
