//! Process management syscalls
use crate::{
    config::{MAX_SYSCALL_NUM, PAGE_SIZE},
    task::{
        change_program_brk, current_user_token, exit_current_and_run_next, get_syscall_times, get_task_time_ms, suspend_current_and_run_next, TaskStatus,
        insert_framed_area, delete_framed_area
    },
    timer::get_time_us,
    mm:: {
        get_phys_addr,
        VirtAddr, VirtPageNum, StepByOne,
        PageTable,
    },
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
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
    let us: usize = get_time_us();
    let ts = get_phys_addr(current_user_token(), _ts as usize) as *mut TimeVal;
    unsafe {
        *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info NOT IMPLEMENTED YET!");
    let ti = get_phys_addr(current_user_token(), _ti as usize) as *mut TaskInfo;
    unsafe {
        let task_info = &mut *ti;
        task_info.status = TaskStatus::Running;
        task_info.time = get_task_time_ms();
        task_info.syscall_times = get_syscall_times();
    }
    0
}


// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    let start_va: VirtAddr = VirtAddr::from(_start);
    let end_va: VirtAddr = VirtAddr::from(_start + _len);

    if (!start_va.aligned())
        || (_port & !0x7 != 0)
        || (_port & 0x7 == 0) {
        return -1;
    }

    let mut vpn: VirtPageNum = start_va.floor();
    let pt = PageTable::from_token(current_user_token());
    for _ in 0 .. ((_len + (PAGE_SIZE - 1)) / PAGE_SIZE) {
        match pt.translate(vpn) {
            Some(pte) => {
                if pte.is_valid() {
                    return -1;
                }
            }
            None => {},
        }
        vpn.step();
    }

    insert_framed_area(start_va, end_va, _port);
    0
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    let start_va: VirtAddr = VirtAddr::from(_start);
    let end_va: VirtAddr = VirtAddr::from(_start + _len);

    if !start_va.aligned() {
        return -1;
    }
    let mut start_vpn = start_va.floor();
    let pt = PageTable::from_token(current_user_token());
    for _ in 0..((_len + (PAGE_SIZE - 1)) / PAGE_SIZE) {
        match pt.translate(start_vpn) {
            Some(pte) => {
                if !pte.is_valid() {
                    return -1;
                }
            }
            None => return -1,
        }
        start_vpn.step();
    }
    delete_framed_area(start_va, end_va);
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
