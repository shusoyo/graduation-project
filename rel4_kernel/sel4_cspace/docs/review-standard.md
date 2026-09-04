# `sel4_cspace` Review Standard

本文档定义 `sel4_cspace` 在 layer reset 之后的默认 review 标准。

当前项目不再把主要讨论组织在旧的 `mdb_patch` 叙事上，而是以新的分层设计为中心：

- `cap`
- `mdb_node`
- `cte_t`
- `mdb`
- `cdt`
- `cspace`

review、设计讨论、进度判断，都默认围绕这套层次展开。

## Core Principle

默认先看三件事，再给结论：

1. exec 语义是否仍然对
2. layer boundary 是否真的清楚
3. proof 是否在正确的层支付了复杂度

不要只看 proof 能不能过，也不要只看文件拆得是否更细。

## Methodology Baseline

当前项目继续使用下面的方法论：

- Verus 组织方式优先参考 `atmo`
- 语义和 contract 强度优先参考 `l4v`
- 当前阶段不把显式 `l4v`-style refinement tower 当成默认目标

也就是说：

- `l4v` 校准语义目标、不变量强度、`requires/ensures` 强度
- `atmo` 校准分层、ghost 组织、exec-proof 对齐方式

## Review Order

### 1. Exec First

先看 runtime 主体，再看 lemma。

默认动作：

1. 找当前 operation 的 canonical exec
2. 对照 `sel4_cspace/reference_0ca248f/src/cte.rs` 或用户指定 old path
3. 看控制流、局部 staging、patch 顺序、空槽检查、链路修补顺序
4. mentally inline helper，确认没有把 runtime 意图藏掉

评判标准：

- 不要求表面语法接近
- 但要求主语义和关键 patch 顺序仍然能和旧实现对上

### 2. Layer Boundary Second

当前项目的首要架构目标，是把 proof 和语义复杂度放到正确层次。

review 时默认检查：

- `cspace::cte` 里的 `cte_t` 是否真的是 slot object，而不是纯 wrapper
- `cspace::mdb` 是否只负责 MDB 图和相关 capability 投影语义
- `cspace::cdt` 是否单独负责 derivation tree，而不是混进 `mdb`
- `cspace::manager` 是否只做组合，而不是重新吞掉所有下层细节

如果某个 helper 名字上属于下层，但 contract 实际仍在打包上层完整语义，要把它当作架构风险点。

### 3. Proof Third

proof review 的重点仍然不是“能不能过”，而是结构是否稳定。

优先检查：

- 前提是否清楚、稳定、属于正确层
- 后置是否表达了真正重要的局部 post
- frame 条件是否明确
- 最终 `wf` 是否通过分层组合收回，而不是在每层都硬拼完整 `wf`

偏好的形状是：

- runtime mutation
- local post
- layer-local frame / obligation
- higher-layer composition
- final `wf` combiner

当前 mutation proof 的默认主骨架是：

- 单一最终后状态
- owner-local preservation lemma
- manager-level composition

默认避免：

- 为每个微步中间态单独建一轮 frame proof
- 用 `unchanged` vocabulary 直接承担 semantic recovery 的主体工作

需要警惕的形状是：

- 把跨层语义硬塞进低层 helper
- 反复在 operation proof 中手工重建整包 `wf`
- helper 名义上是 structural，但 contract 实际暗含 semantic 或 derivation 结论
- `cte_t` method 仍然只是 new manager 再转发

## Layer-Specific Expectations

### `cap`

负责 capability 自身语义：tag、object、rights、badge、zombie、cnode 参数、derive/finalise 相关语义。

不应默认承担 MDB 图或 CDT parent 语义。

### `mdb_node`

负责最底层链字段表示：`prev`、`next`、`revocable`、`first_badged`。

它是数据层，不是全局语义 owner。

### `cte_t`

负责 slot object 抽象：`cap + mdb_node` 的局部读写、局部 view、entry-level 约束。

在模块归属上，`cte_t` 默认属于 `cspace::cte`，而不是顶层孤立模块，也不是 `manager` 的内部细节。

默认期望：

- 可以承载 slot-local verified method
- 不再只是 compatibility wrapper
- 不承担 whole-MDB 或 whole-CDT 全局 invariant owner 职责

### `mdb`

负责 MDB 图建模和全局验证：

- `prev/next` 图关系
- no self link / no two cycle
- incoming parent/badge/untyped edge 语义
- patch / recovery / frame vocabulary

这里允许使用 capability 的投影语义，但不应吞掉完整 capability 语义层。

额外检查：

- `mdb` 的 proof API 是否以 graph primitive 命名，而不是直接以 `insert/move/swap/delete` 命名。
- `rank` 若被引入，是否只用于 acyclic / topological-order，而不是被扩张成替代全部 semantic edge reasoning。
- `unchanged` 是否仍只作为薄 frame 工具，而不是重新长回主证明骨架。

### `cdt`

负责 capability derivation tree：

- `cdt_parent`
- `is_original`
- `spec_should_be_parent_of(...)`
- derivation tree 层的 `wf`

默认不把它混进 `mdb`。

### `cspace`

负责组合上面几层，再接 resolve/cnode/zombie 和更高层操作 contract。

默认模块划分是：`cspace::cte`、`cspace::mdb`、`cspace::cdt`、`cspace::manager`、`cspace::resolve`。

默认不再把所有细节都直接压回一个大而全的 operation proof。

## Completion Language

讨论“完成度”时，继续区分三层：

### Layer-Core Level

某一层自己的 contract、proof 组织、`wf` 已经稳定。

### Manager-Level CSpace Core

`cspace` 组合层内部主操作已经 verified，并能通过分层组合收回总 `wf`。

### Whole-Kernel / `l4v`-Level

已经接到更强的系统不变量或 refinement story。

当前 reset 之后，默认目标仍然是第二层，不默认声称第三层。

## Current Reset Rule

旧的文档、旧的 `mdb_patch` 路线说明、旧的 replacement 叙事，已经归档到：

- `sel4_cspace/docs/archive/2026-05-13-layer-reset/`

后续设计讨论默认以新分层文档为准。只有在需要回溯旧证明、旧 residual、旧路线时，才回 archive 查历史。
