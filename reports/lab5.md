## 问答作业

1. 在我们的多线程实现中，当主线程 (即 0 号线程) 退出时，视为整个进程退出， 此时需要结束该进程管理的所有线程并回收其资源。 - 需要回收的资源有哪些？ - 其他线程的 TaskControlBlock 可能在哪些位置被引用，分别是否需要回收，为什么？
> 线程栈：每个线程都有自己的内核栈，主线程退出时需要释放这些栈占用的内存。
> `TCB`：每个线程的状态和上下文信息存储在`TCB`中，需要回收。
> 同步原语：所有被线程持有的同步原语，如锁、信号量、条件变量需要被释放。
> 文件：所有线程持有的文件需要关闭。

> `TCB`可能被一下这些位置引用
> 1. 调度器
> 2. 同步原语的等待队列
因为线程已经退出，保留这些引用会导致悬挂引用问题。


2. 对比以下两种 Mutex.unlock 的实现，二者有什么区别？这些区别可能会导致什么问题？

```rust
impl Mutex for Mutex1 {
    fn unlock(&self) {
        let mut mutex_inner = self.inner.exclusive_access();
        assert!(mutex_inner.locked);
        mutex_inner.locked = false;
        if let Some(waking_task) = mutex_inner.wait_queue.pop_front() {
            add_task(waking_task);
        }
    }
}

impl Mutex for Mutex2 {
    fn unlock(&self) {
        let mut mutex_inner = self.inner.exclusive_access();
        assert!(mutex_inner.locked);
        if let Some(waking_task) = mutex_inner.wait_queue.pop_front() {
            add_task(waking_task);
        } else {
            mutex_inner.locked = false;
        }
    }
}
```

在这两个`Mutex::unlock`实现中，存在一个关键的区别：
	1.	Mutex1::unlock 实现：
	•	直接将 locked 设置为 false。
	•	如果有等待任务，取出并唤醒第一个等待任务。
	2.	Mutex2::unlock 实现：
	•	先检查 wait_queue，如果有等待任务则取出并唤醒第一个任务。
	•	如果没有等待任务，将 locked 设置为 false。


在`Mutex1::unlock`中，`locked`总是会被设置为`false`，即使有等待任务。
而在`Mutex2::unlock`中，只有当`wait_queue`为空时，才会将`locked`设置为`false`。


•	任务调度问题：Mutex2 的实现会在有等待任务时不设置locked = false。这意味着下一个即将运行的任务在解锁后立即持有锁。这可以提高效率，因为避免了锁状态频繁切换，但也可能导致逻辑问题。如果这个任务尝试`lock mutex`，其他任务可能永远无法获取该锁，导致死锁。

•	锁的状态一致性问题：`Mutex1`保证在任何情况下都会将`locked`设置为 `false`，从而确保解锁操作完成后锁处于释放状态。但在`Mutex2`中，如果 `wait_queue`不为空，`locked`会保持`true`，可能引起对锁状态的误解。


# 荣誉准则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 **以下各位** 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

   > 无

2. 此外，我也参考了 **以下资料** ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

   > 无

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。