# 总结

在本次的实验中，实现了sys_task_info系统调用，用于获取当前任务的信息。我的实现是在TCB结构体中添加了一些成员变量记录一些信息。代码如下：

```rust
pub struct TaskControlBlock {
    /// The task status in it's lifecycle
    pub task_status: TaskStatus,
    /// The task context
    pub task_cx: TaskContext,
    /// The numbers of syscall called by task
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    /// first scheduled time (ms)
    pub time: Option<usize>,
}

```
# 简答

1. rustsbi版本：0.3.0-alpha.2

出错行为

```
[kernel] PageFault in application, bad addr = 0x0, bad instruction = 0x804003a4, kernel killed it.
[kernel] IllegalInstruction in application, kernel killed it.
[kernel] IllegalInstruction in application, kernel killed it.
```


2. 
   1. 首次进入`__restore`函数是由于`__switch`切换内核线程的上下文导致`restore`下一个线程的`ra`寄存器，在这个过程中没有修改过`a0`寄存器，所以此时的`a0`寄存器保存的是`__switch`函数的第一个参数即当前内核线程的`taskcontext`
      1. `__restore`在用户程序第一次被调度时，充当一个跳板的作用，返回用户空间。
      2. 当发生中断或者异常时，可以用来`restore`用户上下文信息。
   2. 从内核栈中读取之前保存的特殊寄存器的值，防止发生Trap嵌套的时候，会覆盖掉 `scause/sscratch/sepc`的值。
      1. `sepc`保存了被中断时的`pc`值。
      2. `sstatus`保存了被中断时`CPU`处于用户态还是内核态。
      3. `sscratch`保存了用户态的栈顶指针。

3. 在这部分的代码之中，是恢复在内核栈上存放的用户态程序的上下文信息，`x2`也就是`sp`寄存器，实际上它在`L48`之中已经从`t2`恢复到了` sscratch`之中，这里没有必要再进行相关的恢复操作。而`x4`是栈帧寄存器，没有用到。
4. 此时，`sp`是用户态程序的栈顶指针，而`sscratch`存放的是内核栈栈顶指针。
5. `sret`指令，会根据`sstatus`中的字段选择恢复到那个特权级。
6. `ecall`指令，或者由于在用户态发生了时钟中断以及外设中断。





# 荣誉准则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 **以下各位** 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

   > 无

2. 此外，我也参考了 **以下资料** ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

   > 无

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。