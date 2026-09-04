# 提交版论文配套说明

## 中文摘要压缩版

微内核因结构精简、可信计算基较小和安全边界清晰而成为形式化验证研究的重要对象。能力空间（CSpace）作为能力系统微内核中的核心子系统，负责能力槽位组织、派生关系维护、对象访问路径解析以及删除与回收等关键操作，其正确性直接影响访问控制、能力传播和系统调用处理。由于 CSpace 操作通常表现为局部状态更新，但必须保持 MDB 链接、派生关系、badge 语义和空槽约束等全局性质，因此有必要采用形式化方法对其进行系统化分析与验证。

本文以 reL4 内核中的 `sel4_cspace` 模块为研究对象，研究基于 Verus 的 capability-space 核心层形式化建模与验证方法。本文构建了 capability、槽位视图与 CSpaceManager 的统一抽象模型，设计了覆盖 capability 内容、MDB 链接、parent-child 派生关系、badge 派生关系、untyped 子关系以及 zombie 相关约束的一组核心不变量，并采用 changed-slots 与 frame 条件驱动的证明组织方式，对 `resolve_address_bits`、`cte_insert`、`insert_new_cap`、`cte_move`、`cte_swap`、`delete_one`、`delete_all`、`set_empty`、`reduce_zombie` 和 `revoke` 等关键路径展开规约设计与主体实现。

当前结果表明，reL4 CSpace 核心层已经形成较完整的 capability-space 级验证闭环：`resolve_address_bits` 已建立显式规约、循环不变式、终止性约束和 refinement；`insert`、`move`、`swap` 以及 delete core 路径已形成较系统的 CSpaceManager 内部验证收口。本文工作为后续 syscall 连接、public wrapper 对齐和 trusted boundary 工程化收缩提供了基础，也表明在 Rust/Verus 生态下推进微内核关键子系统验证具有现实可行性。

关键词：形式化验证；微内核；能力空间；Verus

## 英文摘要压缩版

Microkernels are a major target of formal verification because of their small trusted computing base and explicit protection boundaries. In a capability-based microkernel, the Capability Space (CSpace) is responsible for capability-slot organization, derivation maintenance, object-path resolution, and deletion-related behaviors. Since CSpace operations are typically local state mutations that must preserve global properties such as MDB consistency, derivation relations, badge semantics, and empty-slot constraints, a systematic formal treatment is necessary.

This thesis studies the `sel4_cspace` component in the reL4 kernel and develops a Verus-based methodology for modeling and verifying the core capability-space layer. We construct a unified abstract model for capabilities, slot views, and the global CSpaceManager state, define a family of invariants covering capability contents, MDB links, derivation relations, badge properties, untyped-child constraints, and zombie-related conditions, and organize the proofs of `resolve_address_bits`, `cte_insert`, `insert_new_cap`, `cte_move`, `cte_swap`, `delete_one`, `delete_all`, `set_empty`, `reduce_zombie`, and `revoke` around changed-slot sets and frame conditions.

The current result shows that the reL4 CSpace core already forms a substantial capability-space level verification closeout. `resolve_address_bits` provides explicit specifications, loop invariants, termination arguments, and refinement, while `insert`, `move`, `swap`, and delete-core paths form a systematic manager-level verification structure. These results provide a foundation for syscall integration, public-wrapper alignment, and further trusted-boundary engineering, and demonstrate the practical feasibility of verified microkernel subsystems in the Rust/Verus ecosystem.

KEY WORDS: Formal Verification; Microkernel; Capability Space; Verus

## 参考文献补强建议

当前提交版主稿已经具备 20 条参考文献，可以继续按下面方向增强“参考痕迹”。

1. 在第 1 章绪论中增加对 `seL4 Manual` 的一处显式定义性引用，用于支撑 capability、CNode 和 CSpace 的正式术语来源。

2. 在第 2 章 Verus 与方法部分增加一句对 `Verus: Verifying Rust Programs using Linear Ghost Types` 与 `Verus: A Practical Foundation for Systems Verification` 的并列说明，强调前者提供工具基础，后者提供系统实践证据。

3. 在第 2 章或第 7 章增加对 `Atmosphere`、`CortenMM` 和 `PoWER` 的并列比较，用于说明 Verus 生态已经覆盖内核组件、内存管理和存储系统三个方向。

4. 在第 7 章增加对 `IronFleet` 或 `IronSync` 的一句对照说明，突出“本文虽然对象不同，但同属复杂系统级验证实践”。

## 学校模板落地提醒

1. 最终排版时，把“目录”部分改成学校要求的页码形式，不直接保留 markdown 风格目录。

2. 正文可直接使用 [docs/thesis_submission_draft.md](/workspace/rel4_kernel/docs/thesis_submission_draft.md) 作为主线底稿，再把本文件中的摘要压缩版替换进模板首页。

3. 若导师希望“摘要再短一点”，优先删减背景句，不删成果句和方法句。

4. 若导师希望“结论再保守一点”，只需弱化“已完成”这一类动词，不必重写整篇正文。
