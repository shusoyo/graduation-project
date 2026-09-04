# CSpace 论文文字草稿

状态：写作草稿，2026-04-27

## 1. 说明

这份文档服务于论文写作，不再重复记录工程推进过程。

- 技术口径、已证范围、未证范围与 TCB 以 `cspace-verification-plan.md` 为准。
- 推进时间线与阶段判断以 `cspace-session-log.md` 为准。
- 本文档只负责把当前已经稳定下来的口径，压成可以直接进入论文正文的文字素材。

## 2. 题目候选

- 基于 Verus 的 seL4 Rust 重写中 CSpace 子系统局部形式化验证
- 面向 seL4 Rust 重写的 CSpace 子系统局部 Refinement 验证研究
- 复用 l4v 语义路线的 Rust CSpace 子系统 Verus 验证方法

推荐选择原则：

- 题目里优先体现 `CSpace`、`Rust`、`Verus`、`局部验证/Refinement`。
- 不要写成“Rust 版 seL4 完整验证”。

## 3. 中文摘要终稿

本文面向 seL4 的 Rust 重写，研究如何在显式可信边界内对 CSpace 子系统建立局部形式化保证。与整系统验证不同，本文将底层 bitfield 访问、指针与内存读写、FFI、arch-specific capability 细节以及非 CSpace 子系统前提视为 trusted boundary，只对 CSpace 子集本身的语义、局部不变量和关键原语的 refinement 关系进行证明。语义来源上，本文尽量复用 l4v 对 `sameRegionAs`、`sameObjectAs`、`isCapRevocable`、`deriveCap`、`resolveAddressBits`、`cteInsert`、`cteMove`、`cteSwap`、`isMDBParentOf`、`ensureNoChildren` 和 `isFinalCapability` 的既有定义与证明分解方式；工程实现上，则采用 Verus-native 的抽象模型、bridge 分层与签名式契约组织。当前结果表明：选定的 Rust CSpace 子集已经获得与 l4v 路线一致的局部语义与 refinement 保证，其中 capability query 三项已成为真实的 Verus 接口，其余核心原语则通过 refined wrapper 证明满足抽象 specification。本文展示了在不回避可信边界的前提下，将 l4v 语义路线迁移到 Rust/Verus 验证工程中的一种可行方法。

关键词建议：

- seL4
- Rust
- Verus
- CSpace
- 形式化验证
- Refinement

## 4. 英文摘要草稿

This work studies how to establish local formal guarantees for the CSpace subsystem in a Rust rewrite of seL4 under an explicit trusted boundary. Instead of aiming at whole-kernel verification, we treat low-level bitfield access, pointer and memory manipulation, FFI, architecture-specific capability details, and assumptions from non-CSpace subsystems as trusted components, and focus only on the semantics, local invariants, and refinement properties of a selected CSpace subset. On the semantic side, we reuse as much as possible the l4v design for `sameRegionAs`, `sameObjectAs`, `isCapRevocable`, `deriveCap`, `resolveAddressBits`, `cteInsert`, `cteMove`, `cteSwap`, `isMDBParentOf`, `ensureNoChildren`, and `isFinalCapability`. On the proof-engineering side, we adopt a Verus-native organization based on abstract models, bridge layers, and signature-style contracts. The current result establishes local semantic and refinement guarantees for the selected Rust CSpace subset that are aligned with the l4v proof route: three capability-query operations have already been turned into genuine Verus interfaces, while the remaining core primitives are verified through refined wrappers over opaque execution bodies. This work demonstrates a practical path for migrating l4v-style CSpace reasoning into a Rust/Verus verification workflow without hiding the trusted boundary.

Suggested keywords:

- seL4
- Rust
- Verus
- CSpace
- formal verification
- refinement

## 5. 研究问题

建议在绪论或方法章节前部显式写出 3 到 4 个研究问题，避免全文后面变成“做了很多事，但主问题不够清楚”。

推荐版本：

- `RQ1`：在不追求整系统验证的前提下，如何为 Rust 重写中的 `CSpace` 子系统划定一个可接受且可解释的 trusted boundary？
- `RQ2`：在选定的 `CSpace` 子集上，如何复用 l4v 已有的语义定义与证明分解方式，同时用 Verus-native 的形式重新组织抽象模型、局部 invariant 和 primitive specification？
- `RQ3`：如何把现有 Rust 实现中的 concrete `cap/cte/ret` 结构映射到抽象 `CSpace` 模型，并证明关键接口满足对应抽象合同？
- `RQ4`：在当前阶段，哪些入口已经获得“真实 Verus 接口”级别的验证，哪些仍处于“opaque body + refined wrapper”级别，这种分层如何在论文中被诚实表达？

如果你想更精简，可以压成 3 个问题：

- 如何固定 `CSpace` 局部验证的可信边界？
- 如何在语义上对齐 l4v、在工程上采用 Verus？
- 如何为现有 Rust `CSpace` 接口建立可复用的 refinement 证明入口？

## 6. 引言终稿草稿

### 6.1 背景与问题

高可信操作系统内核的形式化验证长期以来主要围绕 seL4 及其 l4v 证明体系展开。随着 Rust 在系统软件中的广泛应用，围绕现有高可信内核设计进行 Rust 重写，并尝试将既有验证经验迁移到新的实现载体上，成为一个具有研究意义的问题。相比从零开始为一个全新系统设计验证体系，这一路径的价值在于：一方面可以复用 seL4 在对象模型、能力系统和证明结构上的成熟经验，另一方面也可以检验 Verus 等新一代 Rust 验证工具在真实系统代码上的适用性。

然而，直接复刻 l4v 的整系统证明并不现实。当前 Rust 重写代码仍包含大量底层位域操作、指针转换、FFI 调用以及尚未纳入统一抽象模型的外部子系统语义。如果忽略这些现实约束而简单声称“验证整个 Rust 版 seL4”，既不准确，也不利于形成可持续推进的验证工程。因此，本文选择一种更审慎的局部验证策略：先把验证目标收敛到 CSpace 子系统本身，只证明其局部语义、局部不变量与关键原语的 refinement 关系，并将其余底层实现细节与外部依赖显式纳入 trusted boundary。

### 6.2 为什么选择 CSpace

选择 CSpace 作为切入点有两方面原因。首先，能力系统是 seL4 内核设计的核心组成部分，`sameRegionAs`、`sameObjectAs`、`deriveCap`、`cteInsert`、`resolveAddressBits` 等操作直接决定了能力派生、查找与局部更新的语义正确性。其次，l4v 已经为这些操作提供了成熟的定义与 correspondence 证明路线，使得“证明什么、如何分解证明任务”并非完全无据可依。本文的目标不是机械翻译 Isabelle 证明脚本，而是在语义内容上尽量对齐 l4v，在表示方式、模块结构与证明组织上采用更适合 Verus/Rust 的工程形态。

### 6.3 本文方法

基于这一目标，本文首先建立 Rust CSpace 子集对应的抽象 capability 与抽象状态模型，随后提炼只依赖 CSpace 本身的局部 invariant，再为关键原语定义 preservation-first specification，并通过 bridge 层把 concrete `cap/cte/ret` 与抽象模型连接起来。在此基础上，本文分别证明 capability query、lookup 原语以及若干局部修改原语满足相应抽象合同，最终形成一份明确区分“已证入口”“未覆盖范围”与“trusted boundary”的本地证明台账。

因此，本文的核心目标不是给出 Rust 版 seL4 的整系统完备证明，而是在明确可信边界的基础上，为 CSpace 子系统建立一条可持续扩展的局部验证主线。围绕这一目标，本文一方面复用 l4v 已经成熟回答的“证明什么”这一问题，尽量保持对能力关系、派生规则、查找语义和局部更新原语的语义口径一致；另一方面使用 Verus-native 的方式重新组织“如何证明”，将抽象建模、局部 invariant、primitive specification、bridge 映射与 Rust refinement 证明整合到统一工程中。这样的设计既保留了 l4v 的理论来源，也为后续继续局部化 trusted helper、扩展到更多 Rust exec 实现提供了稳定起点。

## 7. 贡献终稿草稿

本文的主要贡献如下：

1. 提出了一种面向 seL4 Rust 重写 `CSpace` 子系统的局部验证路线，在显式 trusted boundary 下把验证目标收敛为“局部语义、局部不变量与 Rust refinement”，从而使已证范围、未证范围与可信边界都能够被清晰描述。
2. 在语义上复用 l4v 的 CSpace 设计，在工程上采用 Verus-native 的抽象模型、bridge 分层与签名式契约组织方式，形成了“l4v 提供语义基线，Verus 承担实现与证明工程”的迁移框架。
3. 为选定的 Rust CSpace 子集建立了可复用的规格与证明入口，其中 capability query 三项已经成为真实的 Verus 接口，其余关键原语则通过 refined wrapper 证明满足抽象 specification，并进一步整理出已证范围、未证范围与 TCB 台账，为后续继续收紧 trusted boundary 和替换更多 opaque exec body 提供了基线。

如果需要更工程化的表达版本，可以改写为：

1. 完成了 `sel4_cspace` 子集的抽象建模、primitive specification 与 refinement 主线搭建。
2. 打通了 capability query、lookup 和局部更新原语的 Verus 证明入口，并将部分真实 Rust 接口直接提升为 Verus contract 形式。
3. 形成了显式区分“真实接口已证”“refined wrapper 已证”“未覆盖范围”“trusted boundary”的论文口径与工程台账。

## 8. 局限性与边界

需要强调的是，本文当前并未证明整个 kernel state 的全局一致性，也未验证所有 arch-specific capability 细节、bitfield 生成代码、底层内存模型和 FFI 内部实现。因此，本文的结论应被理解为：在显式 trusted boundary 之内，选定的 Rust CSpace 子集已经获得与 l4v 语义路线一致的局部形式化保证，而不是对整个系统行为给出端到端完备证明。

写作时建议显式保留下面三点：

- 本文是 `CSpace` 子系统局部验证，而不是整系统验证。
- 当前证明结果同时包含“真实 Verus 接口已验证”和“refined wrapper 已验证”两种完成度。
- trusted boundary 是显式前提，不应在摘要、引言和结论中被静默省略。

## 9. 术语统一建议

全文建议统一使用下面这些术语，尽量不要来回切换：

| 推荐术语 | 建议含义 | 不建议混用的近义表述 |
| --- | --- | --- |
| `trusted boundary` | 本文显式承认但不在当前轮次内证明的边界 | 可信部分、默认正确部分、黑盒部分 |
| `TCB` | trusted boundary 中需要单列说明的核心可信项 | 可信代码、底层假设（除非专门解释） |
| `局部验证` | 只针对 `CSpace` 子系统的语义与局部 invariant | 子系统完整验证、模块完备验证 |
| `抽象模型` | `CapSpec / CSpaceState / wf` 这一层 | 规范模型、理论模型（除非上下文明确） |
| `bridge` | concrete 表示到 abstract 模型的最小映射层 | 中间语义层、第二套规格层 |
| `refined wrapper` | 对 opaque exec body 给出抽象合同的 Verus 证明入口 | 真实接口、最终接口 |
| `真实 Verus 接口` | 在 `feature=verify` 下函数签名本身带 contract 的入口 | refined wrapper |
| `l4v 对齐` | 语义内容和证明分解方式的来源基线 | 完全复刻 l4v、逐行翻译 l4v |

推荐固定写法：

- 写“与 l4v 语义路线一致”比写“完全按照 l4v 实现”更准确。
- 写“建立局部语义与 refinement 保证”比写“证明该系统正确”更稳妥。
- 写“opaque exec body + refined wrapper”比写“该函数已经完全验证”更诚实。

## 10. 章节安排建议

如果你按标准毕业论文结构写，推荐下面这个展开方式：

1. 绪论
   说明研究背景、问题、动机、贡献与论文结构。
2. 背景与相关基础
   介绍 seL4、l4v、Rust 重写背景，以及 Verus 的基本验证机制。
3. 验证范围与可信边界
   固定本文的验证对象、已纳入范围、未纳入范围和 TCB。
4. CSpace 抽象模型与规格设计
   介绍 `CapSpec`、`CSpaceState`、局部 invariant、primitive specification。
5. bridge 与 Rust refinement 证明
   介绍 concrete `cap/cte/ret` 如何映射到抽象模型，以及关键 refined entry 的证明主线。
6. 当前实现结果与案例分析
   汇总 capability query、lookup、局部更新原语的已证情况，并说明真实接口与 refined wrapper 的差异。
7. 局限性、后续工作与总结
   说明当前 remaining TCB、未覆盖范围，以及后续 trusted boundary 收紧和扩展方向。

如果学校要求“相关工作”单列，可把第 2 章拆成：

2. 背景基础
3. 相关工作

## 11. 答辩口径

### 11.1 一句话版本

这项工作的重点不是证明整个 Rust 版 seL4，而是在显式可信边界内，把 l4v 对 CSpace 的证明思路迁移到 Verus，并为 Rust CSpace 子集建立一条已经跑通的局部 refinement 验证主线。

### 11.2 30 秒版本

我的工作不是去证明整个 Rust 版 seL4，而是在显式可信边界内，选取其中的 CSpace 子系统，复用 l4v 已有的语义路线，用 Verus 为 capability relation、lookup 和若干局部更新原语建立抽象规格与 refinement 证明。当前已经跑通了一个局部验证闭环，并整理出已证范围、未证范围和 TCB 台账，为后续继续扩展提供了基线。

## 12. 图表建议

- 表：已证 Rust 入口与抽象规格对应关系
- 表：CSpace 子集的 l4v 语义来源与 Verus 证明入口对应表
- 表：当前 trusted boundary（TCB）分类与保留原因
- 表：当前未覆盖范围与后续扩展方向
- 图：从 concrete CSpace 到 abstract CSpace 的 bridge/refinement 结构

## 13. 使用建议

- 如果现在开始正式写论文，优先从本文件复制摘要、引言和贡献小节。
- 如果学校要求英文摘要，可以先直接从本文件的英文摘要草稿出发，再按学校格式润色。
- 如果开始写正文，先确定 `研究问题` 和 `章节安排`，再展开细节，会比直接堆技术内容更稳。
- 如果需要核对任何“这句话是否说过头”，回到 `cspace-verification-plan.md` 检查已证范围、未证范围与 TCB 台账。
- 如果需要回顾为什么会形成当前口径，回到 `cspace-session-log.md` 查看阶段演化记录。

## 14. 结论与后续工作草稿

### 14.1 结论草稿

可直接作为结论主体的版本：

本文面向 seL4 的 Rust 重写，在显式 trusted boundary 内对 CSpace 子系统进行了局部形式化验证。与整系统验证不同，本文将 bitfield 访问、指针与内存读写、FFI、arch-specific capability 细节以及非 CSpace 子系统前提视为当前轮次之外的可信边界，只对选定 CSpace 子集的能力关系、查找语义、局部不变量以及关键原语的 refinement 关系进行证明。围绕这一目标，本文复用了 l4v 中关于 capability relation、derive、lookup 与局部更新原语的既有语义路线，同时采用 Verus-native 的抽象模型、bridge 分层与签名式契约组织方式，建立了从 concrete Rust 实现到 abstract CSpace specification 的一条局部证明主线。当前结果表明：在所固定的边界之内，选定的 Rust CSpace 子集已经获得与 l4v 语义路线一致的局部语义与 refinement 保证，其中 capability query 三项已经成为真实的 Verus 接口，其余核心原语则通过 refined wrapper 获得抽象合同保证。

如果需要更短版本，可压成：

本文完成的不是 Rust 版 seL4 的整系统验证，而是在显式可信边界内，为其 `CSpace` 子系统建立了一条与 l4v 语义路线一致、并以 Verus 为实现载体的局部 refinement 验证主线。

### 14.2 后续工作草稿

可直接作为“后续工作”小节的版本：

尽管当前工作已经为 `CSpace` 子系统建立了稳定的局部验证基线，但仍存在若干值得继续推进的方向。首先，在 trusted boundary 收缩方面，`trusted_range_top_u128_if_small`、concrete view 提取器以及若干 pointer/object-local observer 仍然保留在 TCB 内，后续可以继续把这些 helper 收紧为更小的机器数值连接点或更细粒度的结构观察器。其次，在实现形态方面，当前除 capability query 三项外，若干核心原语仍采取“opaque exec body + refined wrapper”的完成方式；后续可继续选择 `cte_insert`、`derive_cap`、`ensure_no_children` 等入口，逐步替换为签名式的 Verus 方法体。再次，在证明范围方面，本文目前只覆盖 `CSpace` 局部 invariant，而尚未连接到更大范围的 kernel invariant，也未纳入删除、revoke、finalise 主路径以及更多 arch-specific capability 细节。因而，本文更适合作为 Rust/Verus 版 `CSpace` 验证主线的起点，而不是终点。

答辩时如果只需要一句话概括后续工作，可以写成：

后续工作的重点，一是继续缩小 trusted boundary，二是逐步用 fully verified Verus body 替换当前仍为 opaque 的 Rust exec 实现，三是把当前的 `CSpace` 局部证明继续向更完整的删除链路和更大范围的系统不变量连接。
