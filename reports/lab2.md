# Report for Chapter 4 Programming Exercise

## Implemented Functionality
I implemented `sys_get_time` `sys_trace` `sys_mmap` and `sys_munmap` respectively with virtual memory enabled. Most of the functionalities rely on  interfaces provided by MemorySet. 

## Short Question

1. 页表项是一个64bits的数据，其中63::54是reserved， 53：10是PPN，最低的8位为标志位U: 是否在U态可访问 W: 可写 X：可执行 R: 可读 V: 是否Valid， A: 记录页表项是否被访问过(猜测是用于Clock LRU？)， D记录页表内数据是否被更新
2.1 PageFault
2.2 `sscartch` `sepc` `sp`等等
2.3 这样可以减少程序开始的latency，并且可以优化程序的资源占用。因为可能很多数据我们在执行中并不需要。
2.4 布吉岛呀！
2.5 Lazy策略只需实现缺页异常，然后每次读取都调用`find_pte_create`并且立刻触发一次缺页异常。就可以实现lazy策略 
3.1 单页表的情况下更换页表的原理与双页表一样，只是无需先切换到内核页表。大致流程为: 切换到内核态，从内核态读取新应用的页表地址，将新应用的页表地址载入寄存器，然后回复到用户态
3.2 将内核态的页表项U都设置为0
3.3 单页表可以减少进行系统调用或者发生中断时切换页表的开销，因为每次切换页表都需要清空TCB
3.4 双页表情况下任何中断或者trap发生时都需要切换页表。而对于单页表操作系统，我们只需要在应用程序需要被抢占时切换页表。对于系统调用和中断都无需切换也不。

## Hornor Code

在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

Welkin-Y: 学习了它修改github workflow的方式从而使在测试失败的情况下依然可以打印出output

此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

rCore-Tutorial-Book-v3 3.6.0-alpha.1 文档 https://rcore-os.cn/rCore-Tutorial-Book-v3/chapter3/5exercise.html

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。