//! File and filesystem-related syscalls
use crate::fs::ROOT_INODE;
use crate::fs::{open_file, OpenFlags, Stat};
use crate::mm::copy_to_user;
use crate::mm::{translated_byte_buffer, translated_str, UserBuffer};
use crate::task::{current_task, current_user_token, get_current_task_fd_stat};

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

/// YOUR JOB: Implement fstat.
pub fn sys_fstat(fd: usize, st: *mut Stat) -> isize {
    trace!("kernel: sys_fstat");
    if let Some(stat) = get_current_task_fd_stat(fd) {
        let buffers = translated_byte_buffer(
            current_user_token(),
            st as *const u8,
            core::mem::size_of::<Stat>(),
        );
        return unsafe {
            copy_to_user(
                buffers,
                core::slice::from_raw_parts(
                    &stat as *const _ as *const u8,
                    core::mem::size_of::<Stat>(),
                ),
            )
        };
    }
    -1
}

/// YOUR JOB: Implement linkat.
pub fn sys_linkat(old_name: *const u8, new_name: *const u8) -> isize {
    trace!("kernel: link");
    let token = current_user_token();
    let old_path = translated_str(token, old_name);
    let new_path = translated_str(token, new_name);

    if old_path == new_path {
        return -1;
    }
    if ROOT_INODE.link(&old_path.as_str(), new_path.as_str()) {
        return 0;
    }
    -1
}

/// YOUR JOB: Implement unlinkat.
pub fn sys_unlinkat(name: *const u8) -> isize {
    trace!("kernel: unlink");
    let token = current_user_token();
    let path = translated_str(token, name);
    ROOT_INODE.unlink(path.as_str())
}
