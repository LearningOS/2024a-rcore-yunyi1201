1. stride 算法原理非常简单，但是有一个比较大的问题。例如两个 pass = 10 的进程，使用 8bit 无符号整形储存 stride， p1.stride = 255, p2.stride = 250，在 p2 执行一个时间片后，理论上下一次应该 p1 执行。

- 实际情况是轮到 p1 执行吗？为什么？
> 不是，当进程的`stride`值溢出时，可能在下个时间片仍然是`p2`的`stride`比较小

我们之前要求进程优先级 >= 2 其实就是为了解决这个问题。可以证明， 在不考虑溢出的情况下 , 在进程优先级全部 >= 2 的情况下，如果严格按照算法执行，那么 STRIDE_MAX – STRIDE_MIN <= BigStride / 2。

- 为什么？尝试简单说明（不要求严格证明）。
当优先级至少为2时，`stride`值范围限制在$[2, \text{BigStride}]$。保证最大差值不超过 $\frac{\text{BigStride}}{2}$的原因是在每次调度选择时，较小的`stride`值和较大的`stride`值之间的差不会超过$\text{BigStride} /2$。

已知以上结论，考虑溢出的情况下，可以为 Stride 设计特别的比较器，让 BinaryHeap<Stride> 的 pop 方法能返回真正最小的 Stride。补全下列代码中的 partial_cmp 函数，假设两个 Stride 永远不会相等。


```rust
use core::cmp::Ordering;

struct Stride(u64);

impl PartialOrd for Stride {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let max_stride = 65546u64;      
        let half_max_stride = max_stride / 2;
        let self_val = self.0 & max_stride;
        let other_val = other.0 & max_stride;
        if self_val < other_val && other_val - self_val <= half_max_stride {
            Some(Ordering::Less)
        } else if self_val > other_val && self_val - other_val <= half_max_stride {
            Some(Ordering::Greater)
        } else if self_val < other_val {
            Some(Ordering::Greater)
        } else {
            Some(Ordering::Less)
        }
    }
}

impl PartialEq for Stride {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}

```