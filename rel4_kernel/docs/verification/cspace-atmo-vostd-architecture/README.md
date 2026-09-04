# CSpace Atmo-Style Re-architecture

本目录记录 `sel4_cspace` 的新一轮重构方案。

这轮方案的出发点已经固定：

- 不再把“保留旧 `specs/cspace_ops/*` 证明农场”当作主线。
- 不再把 “尽量贴 `l4v` 语义分层” 当作架构约束。
- 直接把 `sel4_cspace` 视为一个类似 atmo 子模块的 verified subsystem。

也就是说，今后的主叙事应当是：

- `src/specs/abstract_cspace.rs` 提供抽象模型与全局不变量；
- `src/verified/{cap,mdb,slot}.rs` 提供 raw-backed 局部对象；
- `src/verified/cspace.rs` 提供 subsystem context 与全局恢复；
- `src/verified/{derive,resolve,insert}.rs` 只保留薄操作壳；
- `src/repr/*` 退到 view/result/helper；
- `src/specs/cspace_ops/*` 是过渡层，应持续变薄。

## 文档清单

1. [01-scope-and-principles.md](/workspace/rel4_kernel/docs/verification/cspace-atmo-vostd-architecture/01-scope-and-principles.md)
   说明目标、范围、原则和明确不做什么。

2. [02-evidence-and-references.md](/workspace/rel4_kernel/docs/verification/cspace-atmo-vostd-architecture/02-evidence-and-references.md)
   说明这套设计主要参考什么，哪些来源只是背景。

3. [03-target-architecture.md](/workspace/rel4_kernel/docs/verification/cspace-atmo-vostd-architecture/03-target-architecture.md)
   给出最终目标结构和模块职责。

4. [04-proof-obligations.md](/workspace/rel4_kernel/docs/verification/cspace-atmo-vostd-architecture/04-proof-obligations.md)
   说明 subsystem、对象层、patch 语义各自要证明什么。

5. [05-implementation-roadmap.md](/workspace/rel4_kernel/docs/verification/cspace-atmo-vostd-architecture/05-implementation-roadmap.md)
   给出按阶段落地的重构顺序。

6. [06-open-questions.md](/workspace/rel4_kernel/docs/verification/cspace-atmo-vostd-architecture/06-open-questions.md)
   只保留还没有定死的结构问题。

7. [TODO.md](/workspace/rel4_kernel/docs/verification/cspace-atmo-vostd-architecture/TODO.md)
   执行清单和阶段进度，以它为准持续维护。

## 现在的总判断

`sel4_cspace` 更像 atmo 里的一个 verified 子系统，而不是一个“先写厚 spec，再让对象层去满足 spec”的项目。

因此真正要学的不是某个具体函数的语法，而是下面这套组织方式：

- 小对象自己带 `view()/wf()`
- subsystem 自己带 `wf()`
- mutator 先变成局部 patch
- 全局恢复收口到 subsystem context
- 操作文件尽量薄

如果后续代码和文档发生冲突，以这个判断为准改文档和代码，而不是继续迁就旧分层。
