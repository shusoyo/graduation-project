# Verus: A Practical Foundation for Systems Verification 论文阅读笔记

## 摘要
形式化验证的用途是在运行期**前**发现错误。
在没有形式化验证的时候是如何保证安全的？论文说：1. 开发者“英雄式”的努力（我觉得是极高的技术实力，但是对于大部分系统软件来说，并不能保证所有的开发者都有这种实力），2. 特定的开发逻辑，3. 某种自动化缺丧失表达能力的技术。

## Verus 概览：面向系统的验证
> 跨层证明自动化

> 内存推理

得益于 rust 本身所有权机制，以及借用检查器所提供的内存安全，使用 verus 编写对 safe rust 的证明，不再通过手工开发者劳动或复杂编码来证明系统代码的内存安全。

而对于 unsafe rust，verus 扩展了对原始指针的支持， 可以通过 verus 来验证 unsafe 块的子集，同时 verus 提供了对 unsafe 快的自动证明。
 

> system idioms

verus 也提供了一些在 system idioms 中常用的对位操作和非线性算术的推理技术。

> multiple threads

verus 使用 VerusSync 来做对多线程的验证。

> 系统层推理


## Verus 设计中面向系统的关键方面

半自动程序验证器 vs 定理证明器


首先定理证明器有更强的表达力，但是半自动程序验证器更多的用来验证大型系统，主要的问题我还是觉得，自动程序验证器本身就深嵌入了霍尔逻辑和分离逻辑，可以很好的做命令式程序的验证，也能做精化证明，而且可以实现自动化证明，例如调用 smt 什么的。


Verus 的 spec 是纯函数，没有前置条件，也不能访问堆，所以更容易被 smt 处理（rust 语言特性使得很多情况下不需要像分离逻辑那样对堆操作，当必要时可以用 ghost mode）

> 量词与 SMT 求解


Verus 对量词的处理也更保守，量词在 smt 求解时一般采用 trigger 来实例化，verus 选择更谨慎的 trigger 或者鼓励用户检查对 trigger 的选择来提升 smt 的求解效率。

> 选择性使用 EPR 以获得完全自动化

TODO

> 非线性算术推理

> 位向量推理

可以通过 `by (annot)` 来改变 smt 的行为。

有 `integer_ring` `bit_vector`

> 3.4 多线程自动推理

TODO


