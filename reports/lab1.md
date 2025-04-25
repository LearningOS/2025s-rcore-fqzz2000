# Report for Chapter 3 Programming Exercise

## Implemented Functionality
In this assignment, the main functionality is to count the number of syscalls to each applications respectively, and achieves a read/write byte function from a given memory address. 

For syscall counting, I added a nested array as a counting table in the TaskManagerInner and increase the corresponding count whenever the `syscall` function invoked. I first tried to add an array to each TCB struct but it causes weird bugs and  kernel crash for some unknown reasons. I may dig into it later. 

## Short Question

1. 这三个测试例应当会触发CPU的中断并被我们的trap hanlder捕获，容纳后panic退出
2.1 刚进入`__restore` 时，`sp`指向内核栈的栈顶。`__restore`会在触发trap(syscall)或在其它中断发生时(timer interrupt)在处理完syscall或其它中断后返回U态前被执行
2.2 L43-L48特殊处理了`sstatus`，`sepc`和`sscratch`寄存器的内容。将之前保存的值恢复到这三个寄存器中。其中`sstatus`保存了当前CPU运行的特权级，`sepc`保存了trap结束后需要继续执行官的地址，`sscratch` 保存了用户栈的栈顶，需要被恢复。
2.3 `x2`即为`sp`寄存器的别名，我们已经对它做过处理，`x4` 是`tp` 寄存器的别名，`tp`会指向`thread local storage`我们的trap中一般不会用到它
2.4 L60这条指令将内核栈顶和用户栈顶交换，交换后`sp` 保存用户栈顶的地址，`sscratch` 保存内核栈顶的地址。这样用户态程序可以继续运行不受影响。
3.5 `__restore` 中状态切换发生在`sret`后，这是RISCV提供的指令，可以设置返回上个特权级并且将`pc`切换到`sepc`中的值
3.6 这条指令和L60完全相同，依然是吧内核栈顶和用户栈顶交换，交换后`sp` 保存内核栈顶的地址，`sscratch` 保存用户栈顶的地址
3.7 发生在`__alltrap`被调用前的`ecall`指令

## Hornor Code

在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

Welkin-Y: 学习了它修改github workflow的方式从而使在测试失败的情况下依然可以打印出output

此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

rCore-Tutorial-Book-v3 3.6.0-alpha.1 文档 https://rcore-os.cn/rCore-Tutorial-Book-v3/chapter3/5exercise.html

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。