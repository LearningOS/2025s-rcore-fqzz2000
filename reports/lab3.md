# Report for Chapter 5 Programming Exercise

## Implemented Functionality
port `mmap` `munmap` `get_time` from Chapter 4. Implemented spawn which is essentially a combination of `fork() + exec()` without memory copy. Implemented Stride scheduling algorithm. 

## Short Question

1.理论上下一次应该是p1执行，因为p1的stride值大，应该让更小的执行。但实际情况是：不会轮到p1执行。
原因是：当使用8bit无符号整型时，p2执行后，它的stride会增加BigStride/priority = 255/10 ≈ 26，变成250+26=276。但是276超过了8bit无符号整型能表示的最大值255，所以会发生溢出，实际存储的值是276-256=20。因此比较时，p2.stride(20) < p1.stride(255)，所以下一次仍然会执行p2。

每个进程每次执行后stride增加的值为BigStride/priority
当优先级>=2时，每次stride增加的值<=BigStride/2 调度算法总是选择stride最小的进程执行 当一个进程的stride比其他所有进程都小时，它会被选中执行，然后增加stride
如果增加后仍是最小的，它会再次执行 但由于每次增加值<=BigStride/2，所以经过足够多次执行后，它的stride值一定会超过其他进程
这保证了所有进程都有机会执行，不会有一个进程独占CPU 而且可以证明，任意两个进程的stride差值不会超过BigStride/2

如果要实现比较器，当差值大于BigStride/2是，需要将结果调换
## Hornor Code

在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

Welkin-Y: 学习了它修改github workflow的方式从而使在测试失败的情况下依然可以打印出output

此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

rCore-Tutorial-Book-v3 3.6.0-alpha.1 文档 https://rcore-os.cn/rCore-Tutorial-Book-v3/chapter3/5exercise.html

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。