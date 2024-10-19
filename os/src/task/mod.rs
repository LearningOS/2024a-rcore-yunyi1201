//! Task management implementation
//!
//! Everything about task management, like starting and switching tasks is
//! implemented here.
//!
//! A single global instance of [`TaskManager`] called `TASK_MANAGER` controls
//! all the tasks in the whole operating system.
//!
//! A single global instance of [`Processor`] called `PROCESSOR` monitors running
//! task(s) for each core.
//!
//! A single global instance of `PID_ALLOCATOR` allocates pid for user apps.
//!
//! Be careful when you see `__switch` ASM function in `switch.S`. Control flow around this function
//! might not be what you expect.
mod context;
mod id;
mod manager;
mod processor;
mod switch;
#[allow(clippy::module_inception)]
mod task;

use crate::loader::get_app_data_by_name;
use crate::loader::{get_app_data, get_num_app};
use crate::mm::MapPermission;
use crate::sync::UPSafeCell;
use crate::syscall::TaskInfo;
use crate::timer::get_time_ms;
use crate::trap::TrapContext;
use alloc::sync::Arc;
use alloc::vec::Vec;
use lazy_static::*;
pub use manager::{fetch_task, TaskManager};
use switch::__switch;
pub use task::{TaskControlBlock, TaskStatus};

pub use context::TaskContext;
pub use id::{kstack_alloc, pid_alloc, KernelStack, PidHandle};
pub use manager::add_task;
pub use processor::{
    current_task, current_trap_cx, current_user_token, run_tasks, schedule, take_current_task,
    Processor,
};
/// Suspend the current 'Running' task and run the next task in task list.
pub fn suspend_current_and_run_next() {
    // There must be an application running.
    let task = take_current_task().unwrap();

    // ---- access current TCB exclusively
    let mut task_inner = task.inner_exclusive_access();
    let task_cx_ptr = &mut task_inner.task_cx as *mut TaskContext;
    // Change status to Ready
    task_inner.task_status = TaskStatus::Ready;
    drop(task_inner);
    // ---- release current PCB

    // push back to ready queue.
    add_task(task);
    // jump to scheduling cycle
    schedule(task_cx_ptr);
}

/// pid of usertests app in make run TEST=1
pub const IDLE_PID: usize = 0;

/// Exit the current 'Running' task and run the next task in task list.
pub fn exit_current_and_run_next(exit_code: i32) {
    // take from Processor
    let task = take_current_task().unwrap();

    let pid = task.getpid();
    if pid == IDLE_PID {
        println!(
            "[kernel] Idle process exit with exit_code {} ...",
            exit_code
        );
        panic!("All applications completed!");
    }

    // **** access current TCB exclusively
    let mut inner = task.inner_exclusive_access();
    // Change status to Zombie
    inner.task_status = TaskStatus::Zombie;
    // Record exit code
    inner.exit_code = exit_code;
    // do not move to its parent but under initproc

    // ++++++ access initproc TCB exclusively
    {
        let mut initproc_inner = INITPROC.inner_exclusive_access();
        for child in inner.children.iter() {
            child.inner_exclusive_access().parent = Some(Arc::downgrade(&INITPROC));
            initproc_inner.children.push(child.clone());
        }
    }
    // ++++++ release parent PCB

    inner.children.clear();
    // deallocate user space
    inner.memory_set.recycle_data_pages();
    drop(inner);
    // **** release current PCB
    // drop task manually to maintain rc correctly
    drop(task);
    // we do not have to save task context
    let mut _unused = TaskContext::zero_init();
    schedule(&mut _unused as *mut _);
}

// lazy_static! {
//     /// Creation of initial process
//     ///
//     /// the name "initproc" may be changed to any other app name like "usertests",
//     /// but we have user_shell, so we don't need to change it.
//     pub static ref INITPROC: Arc<TaskControlBlock> = Arc::new(TaskControlBlock::new(
//         get_app_data_by_name("ch5b_initproc").unwrap()
//     ));
//     /// Generally, the first task in task list is an idle task (we call it zero process later).
//     /// But in ch4, we load apps statically, so the first task is a real app.
//     fn run_first_task(&self) -> ! {
//         let mut inner = self.inner.exclusive_access();
//         let next_task = &mut inner.tasks[0];
//         next_task.task_status = TaskStatus::Running;
//         next_task.first_scheduled = Some(get_time_ms());
//         let next_task_cx_ptr = &next_task.task_cx as *const TaskContext;
//         drop(inner);
//         let mut _unused = TaskContext::zero_init();
//         // before this, we should drop local variables that must be dropped manually
//         unsafe {
//             __switch(&mut _unused as *mut _, next_task_cx_ptr);
//         }
//         panic!("unreachable in run_first_task!");
//     }

//     /// Change the status of current `Running` task into `Ready`.
//     fn mark_current_suspended(&self) {
//         let mut inner = self.inner.exclusive_access();
//         let cur = inner.current_task;
//         inner.tasks[cur].task_status = TaskStatus::Ready;
//     }

//     /// Change the status of current `Running` task into `Exited`.
//     fn mark_current_exited(&self) {
//         let mut inner = self.inner.exclusive_access();
//         let cur = inner.current_task;
//         inner.tasks[cur].task_status = TaskStatus::Exited;
//     }

//     /// Find next task to run and return task id.
//     ///
//     /// In this case, we only return the first `Ready` task in task list.
//     fn find_next_task(&self) -> Option<usize> {
//         let inner = self.inner.exclusive_access();
//         let current = inner.current_task;
//         (current + 1..current + self.num_app + 1)
//             .map(|id| id % self.num_app)
//             .find(|id| inner.tasks[*id].task_status == TaskStatus::Ready)
//     }

//     /// Get the current 'Running' task's token.
//     fn get_current_token(&self) -> usize {
//         let inner = self.inner.exclusive_access();
//         inner.tasks[inner.current_task].get_user_token()
//     }

//     /// Get the current 'Running' task's trap contexts.
//     fn get_current_trap_cx(&self) -> &'static mut TrapContext {
//         let inner = self.inner.exclusive_access();
//         inner.tasks[inner.current_task].get_trap_cx()
//     }

//     /// Change the current 'Running' task's program break
//     pub fn change_current_program_brk(&self, size: i32) -> Option<usize> {
//         let mut inner = self.inner.exclusive_access();
//         let cur = inner.current_task;
//         inner.tasks[cur].change_program_brk(size)
//     }

//     /// Switch current `Running` task to the task we have found,
//     /// or there is no `Ready` task and we can exit with all applications completed
//     fn run_next_task(&self) {
//         if let Some(next) = self.find_next_task() {
//             let mut inner = self.inner.exclusive_access();
//             let current = inner.current_task;
//             inner.tasks[next].task_status = TaskStatus::Running;
//             if inner.tasks[next].first_scheduled.is_none() {
//                 inner.tasks[next].first_scheduled = Some(get_time_ms());
//             }
//             inner.current_task = next;
//             let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut TaskContext;
//             let next_task_cx_ptr = &inner.tasks[next].task_cx as *const TaskContext;
//             drop(inner);
//             // before this, we should drop local variables that must be dropped manually
//             unsafe {
//                 __switch(current_task_cx_ptr, next_task_cx_ptr);
//             }
//             // go back to user mode
//         } else {
//             panic!("All applications completed!");
//         }
//     }
//     /// record syscall
//     fn record_syscall(&self, syscall_number: usize) {
//         let mut inner = self.inner.exclusive_access();
//         let cur = inner.current_task;
//         inner.tasks[cur].syscall_times[syscall_number] += 1;
//     }

//     /// get take info
//     fn get_current_task_info(&self, info: &mut TaskInfo) {
//         let inner = self.inner.exclusive_access();
//         let cur = inner.current_task;
//         *info = TaskInfo {
//             status: TaskStatus::Running,
//             syscall_times: inner.tasks[cur].syscall_times,
//             time: get_time_ms() - inner.tasks[cur].first_scheduled.unwrap(),
//         }
//     }

//     /// check virtual page is maped in current task
//     fn is_mapped(&self, va: crate::mm::VirtAddr) -> bool {
//         let inner = self.inner.exclusive_access();
//         let cur = inner.current_task;
//         inner.tasks[cur].memory_set.is_mapped(va.into())
//     }

//     /// map area structure, controls a contiguous piece of virtual memory
//     fn map_area(&self, start: usize, end: usize, permission: usize) {
//         let mut inner = self.inner.exclusive_access();
//         let cur = inner.current_task;
//         inner.tasks[cur].memory_set.insert_framed_area(
//             start.into(),
//             end.into(),
//             MapPermission::from(permission) | MapPermission::U,
//         );
//     }

//     /// unmap area structure, controls a contiguous piece of virtual memory
//     fn unmap_area(&self, start: usize, end: usize) {
//         let mut inner = self.inner.exclusive_access();
//         let cur = inner.current_task;
//         inner.tasks[cur].memory_set.unmap_area(start, end);
//     }
// }

///Add init process to the manager
pub fn add_initproc() {
    add_task(INITPROC.clone());
}

/// record syscall invoke
pub fn record_syscall(syscall_number: usize) {
    TASK_MANAGER.record_syscall(syscall_number);
}

/// get current task infomation
pub fn get_current_task_info(info: &mut TaskInfo) {
    TASK_MANAGER.get_current_task_info(info);
}

/// check virtual page is maped in current task
pub fn is_mapped(va: crate::mm::VirtAddr) -> bool {
    TASK_MANAGER.is_mapped(va)
}

/// map a contiguous piece of virtual memory in current task
pub fn map_area(start: usize, end: usize, permission: usize) {
    TASK_MANAGER.map_area(start, end, permission);
}

/// unmap a contiguous piece of virtual memory in current task
pub fn unmap_area(start: usize, end: usize) {
    TASK_MANAGER.unmap_area(start, end);
}
