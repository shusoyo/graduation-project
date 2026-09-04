# 论文参考文献与 PDF 清单

本目录整理了适合当前毕业设计的核心参考文献，重点覆盖 4 个方向：

1. seL4 / reL4 与微内核验证背景
2. CSpace / capability / access control 相关验证
3. Rust / Verus 形式化验证方法
4. 向 syscall 级证明扩展时可引用的系统文档

## 推荐优先阅读顺序

### 先读这 4 篇

1. `01-sel4-formal-verification-os-kernel-sosp2009.pdf`
   - 作用：绪论和相关工作里的“经典起点”
   - 你可以用来说明：内核完全验证为什么重要，seL4 为什么是微内核验证的标志性工作

2. `02-comprehensive-formal-verification-os-microkernel-tocs2014.pdf`
   - 作用：相关工作和背景章节里的“综合性总述”
   - 你可以用来说明：功能正确性证明之外，还可以进一步扩展到访问控制、二进制正确性等

3. `05-verus-linear-ghost-types-arxiv.pdf`
   - 作用：方法章节
   - 你可以用来说明：为什么选择 Verus，以及 Verus 如何结合 Rust 类型系统与 SMT 验证

4. `04-sel4-manual-latest.pdf`
   - 作用：写对象语义、syscall 接口和术语定义时查规范
   - 你可以用来核对 CNode / capability / syscall 的正式接口表述

### 第二层阅读

5. `08-sel4-enforces-integrity-esop2012.pdf`
   - 作用：安全性质章节
   - 你可以用来说明 capability 系统不仅要“功能正确”，还与 integrity / authority confinement 密切相关

6. `07-translation-validation-verified-os-kernel-sosp2013.pdf`
   - 作用：讨论完整验证边界
   - 你可以用来说明从源码级证明继续走向二进制级保证的研究路径

7. `03-from-l3-to-sel4-tosp2013.pdf`
   - 作用：系统演化背景
   - 你可以用来交代 L4/seL4 设计思想、为何能力系统和微内核结构适合形式化验证

8. `06-rustbelt-popl2018.pdf`
   - 作用：Rust 语言形式化背景
   - 你可以用来说明 Rust 语言自身的安全语义基础，与 Verus 所做工作之间的关系

## 如何在论文里使用这些文献

### 绪论

- `01` 用来介绍 seL4 验证的开创性意义
- `02` 用来介绍“完整 assurance story” 的扩展方向
- `03` 用来介绍 L4 到 seL4 的设计演进

### 相关工作

- `01`、`02`、`07`、`08` 组成 seL4 方向的相关工作主线
- `05`、`06` 组成 Rust/Verus 方向的相关工作主线

### 方法章节

- `05` 用来介绍 Verus 的设计理念和证明方式
- `06` 用来说明 Rust 语言本身的形式化基础

### syscall / 接口章节

- `04` 用来引用 CNode、capability、syscall 相关规范定义

## 已下载 PDF 文件

- `pdfs/01-sel4-formal-verification-os-kernel-sosp2009.pdf`
- `pdfs/02-comprehensive-formal-verification-os-microkernel-tocs2014.pdf`
- `pdfs/03-from-l3-to-sel4-tosp2013.pdf`
- `pdfs/04-sel4-manual-latest.pdf`
- `pdfs/05-verus-linear-ghost-types-arxiv.pdf`
- `pdfs/06-rustbelt-popl2018.pdf`
- `pdfs/07-translation-validation-verified-os-kernel-sosp2013.pdf`
- `pdfs/08-sel4-enforces-integrity-esop2012.pdf`

## 使用建议

- 先把 `01`、`02`、`05` 的摘要和引言通读一遍，再开始写绪论和相关工作。
- 写到 capability 派生、访问控制或 integrity 时，再补 `08`。
- 写到 syscall、CNode 接口或对象定义时，随时翻 `04`。
- 如果导师要求“参考文献别太少”，这 8 篇已经足够组成一版质量不错的基础清单，之后可以再按需要扩到 12 到 15 篇。
