//! Semaphore

use crate::sync::UPSafeCell;
use crate::task::{block_current_and_run_next, current_task, wakeup_task, TaskControlBlock};
use alloc::{collections::VecDeque, sync::Arc};

/// semaphore Id
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct SemId(pub usize);

/// semaphore structure
pub struct Semaphore {
    /// semaphore id
    pub sem_id: SemId,
    /// semaphore inner
    pub inner: UPSafeCell<SemaphoreInner>,
}

pub struct SemaphoreInner {
    pub count: isize,
    pub wait_queue: VecDeque<Arc<TaskControlBlock>>,
}

impl Semaphore {
    /// Create a new semaphore
    pub fn new(sem_id: usize, res_count: usize) -> Self {
        trace!("kernel: Semaphore::new");
        Self {
            sem_id: SemId(sem_id),
            inner: unsafe {
                UPSafeCell::new(SemaphoreInner {
                    count: res_count as isize,
                    wait_queue: VecDeque::new(),
                })
            },
        }
    }

    /// up operation of semaphore
    pub fn up(&self) {
        trace!("kernel: Semaphore::up");
        let mut inner = self.inner.exclusive_access();
        inner.count += 1;

        let current_task = current_task().unwrap();
        let task_inner = current_task.inner_exclusive_access();

        if inner.count <= 0 {
            if let Some(task) = inner.wait_queue.pop_front() {
                wakeup_task(task);
            }
        }

        drop(task_inner);
        drop(current_task);

        if inner.count <= 0 {
            if let Some(task) = inner.wait_queue.pop_front() {
                let mut task_inner = task.inner_exclusive_access();

                if let Some((index, sem_count)) = task_inner
                    .need
                    .iter_mut()
                    .enumerate()
                    .find(|(_, (sem_id, _))| *sem_id == self.sem_id)
                {
                    sem_count.1 -= 1;
                    if sem_count.1 <= 0 {
                        task_inner.need.remove(index);
                    }
                } else {
                    panic!("there should be a need item be registed!");
                }

                if let Some((_, alloc_count)) = task_inner
                    .allocation
                    .iter_mut()
                    .find(|(sem_id, _)| *sem_id == self.sem_id)
                {
                    *alloc_count += 1;
                } else {
                    task_inner.allocation.push((self.sem_id, 1));
                }

                drop(task_inner);
                wakeup_task(task);
            }
        }
    }

    /// down operation of semaphore
    pub fn down(&self) {
        trace!("kernel: Semaphore::down");
        let mut inner = self.inner.exclusive_access();
        inner.count -= 1;

        let current_task = current_task().unwrap();
        let mut task_inner = current_task.inner_exclusive_access();

        if inner.count < 0 {
            if let Some(sem_count) = task_inner
                .need
                .iter_mut()
                .find(|(sem_id, _)| *sem_id == self.sem_id)
            {
                sem_count.1 += 1;
            } else {
                task_inner.need.push((self.sem_id.clone(), 1))
            }

            drop(task_inner);
            inner.wait_queue.push_back(current_task);
            drop(inner);
            block_current_and_run_next();
        } else {
            if let Some(alloc_count) = task_inner
                .allocation
                .iter_mut()
                .find(|(sem_id, _)| *sem_id == self.sem_id)
            {
                alloc_count.1 += 1;
            } else {
                task_inner.allocation.push((self.sem_id.clone(), 1));
            }
        }
    }
}
