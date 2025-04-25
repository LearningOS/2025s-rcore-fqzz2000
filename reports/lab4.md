# Report for Chapter 6 Programming Exercise

## Implemented Functionality
Implemented link and unlink function for the file systems port spawn, mmap, munmap and get time to chapter 6. Initially I create an additional inodefor each hardlink, which later been proved redundant. The final deisgn came up after a discussion with Welkin
## Short Question
1 root node记录了整个文件系统各个段的位置。文件系统需要通过root node找到各个区域的起始点然后根据offset访问文件。如果root node丢失文件系统将无法访问文件
2. pipe的实际使用例子 `netstat -tnlp | grep nginx` 找到nginx监听的端口
3. 可以使用mmap共享一个文件作为通信的内容，并把文件作为ring buffer来实现读写 
## Hornor Code

在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

Welkin-Y: 学习了它修改github workflow的方式从而使在测试失败的情况下依然可以打印出output
Welkin-Y: 参考了他的spawn实现修改了没有stdout的错误
Welkin-Y: 参考了unlink时删除dirent的实现

此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

rCore-Tutorial-Book-v3 3.6.0-alpha.1 文档 https://rcore-os.cn/rCore-Tutorial-Book-v3/chapter3/5exercise.html

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。