# 01 Scope And Principles

## 1. 目标

这轮重构的目标不是“给现有 `sel4_cspace` 补更多 theorem”，而是：

**把 `sel4_cspace` 原地重构成一个 atmo 风格的 verified subsystem。**

这里的 atmo 风格，指的是下面几件事：

- 运行时真相仍然是 raw 结构；
- 证明主语是对象和 subsystem，而不是操作 spec 文件；
- ghost 语义尽量收进对象和 context；
- mutator 先表达成 patch，再由 subsystem 恢复全局不变量。

## 2. 范围

本计划覆盖 `sel4_cspace` 的核心验证骨架，包括：

- 抽象模型与全局不变量：
  [sel4_cspace/src/specs/abstract_cspace.rs](/workspace/rel4_kernel/sel4_cspace/src/specs/abstract_cspace.rs)
- 局部对象层：
  [cap.rs](/workspace/rel4_kernel/sel4_cspace/src/verified/cap.rs),
  [mdb.rs](/workspace/rel4_kernel/sel4_cspace/src/verified/mdb.rs),
  [slot.rs](/workspace/rel4_kernel/sel4_cspace/src/verified/slot.rs)
- subsystem context：
  [cspace.rs](/workspace/rel4_kernel/sel4_cspace/src/verified/cspace.rs)
- 操作薄壳：
  [derive.rs](/workspace/rel4_kernel/sel4_cspace/src/verified/derive.rs),
  [resolve.rs](/workspace/rel4_kernel/sel4_cspace/src/verified/resolve.rs),
  [insert.rs](/workspace/rel4_kernel/sel4_cspace/src/verified/insert.rs)
- view/helper 层：
  `src/repr/*`
- 兼容边界：
  `cte.rs`、`interface.rs`、`compatibility.rs`、`refinement_bridge.rs`

## 3. 非目标

当前明确不把下面这些事情当主目标：

- 继续维护“操作一个大 spec 文件 + 一组 preservation theorems”的旧节奏。
- 为了论文口径强行贴齐 `l4v` 的模块划分。
- 当前阶段验证 `sel4_common` 本身。
- 当前阶段重写外部 runtime API。
- 继续扩张 `refinement_bridge.rs`。

## 4. 核心判断

### 4.1 `sel4_cspace` 对应的是 atmo 的“verified 子系统”

从组织方式上看，`sel4_cspace` 不该对标 atmo 的整个 kernel，而更接近：

- 一个有自己抽象模型的 verified 子模块；
- 里面有局部对象；
- 也有自己的 subsystem invariant；
- 主要操作在这个 subsystem 内部闭合证明。

所以正确问题不是“`cte_insert` 在 atmo 里对应哪个函数”，而是：

- `cap/mdb/slot` 对应 atmo 里的局部对象；
- `cspace` 对应 atmo 里的 subsystem context；
- `insert/derive/resolve` 对应挂在 subsystem 或对象上的操作。

### 4.2 当前真正的问题不是语义来源，而是语义主语

过去代码变厚，不是因为 Verus 天生就需要那么多行，而是因为主语放错了：

- 把语义中心放在 `specs/cspace_ops/*`
- 把操作写成全局 `Map` 重写脚本
- 把恢复结论写成 theorem farm

这样就算语义是对的，也会自然长成厚 spec。

## 5. 设计原则

### 5.1 对象优先

局部语义先挂在对象上：

- `CapRef`
- `MdbRef`
- `SlotRef`

对象自己负责：

- `view()`
- `wf()`
- query
- 局部 post 语义

### 5.2 subsystem 唯一负责全局恢复

全局不变量只由 `CspaceCtx` 收口。

也就是说：

- `valid_slots`
- `mdb_prev_next_consistent`
- `cnode_lookup_wf`
- `cnode_slots_wf`
- `cspace_roots_wf`
- 总 `wf()`

都应该由 [verified/cspace.rs](/workspace/rel4_kernel/sel4_cspace/src/verified/cspace.rs) 成为主语。

### 5.3 patch-centered mutator

mutator 不直接从“整张 `Map` 怎么改”开始讲，而是先说：

- 改了哪些 slot
- 每个 touched slot 的 post view 是什么
- 哪些 context field 保持不变

也就是先形成 `CspacePatchSpec`，再谈恢复。

### 5.4 `specs/cspace_ops/*` 只保留薄语义锚点

这些文件长期只能承担：

- precondition
- 抽象效果摘要
- 少量 bridge/helper

不能继续承担：

- 主 proof body
- 大段 rewiring 展开
- 每个全局不变量的完整恢复证明

### 5.5 简单优先

如果一个 ghost 参数、owner 抽象、wrapper struct 不是当前证明真正需要的，就不引入。

优先级固定为：

1. 简单但 sound
2. 容易统一风格
3. 再考虑更强抽象

## 6. 成功标准

这轮重构完成时，应同时满足下面几点：

- `verified/cspace.rs` 是全局不变量恢复的唯一中心。
- `verified/{cap,mdb,slot}.rs` 是局部语义中心。
- `verified/{derive,resolve,insert}.rs` 明显变薄。
- `src/specs/cspace_ops/*` 不再是主证明中心。
- `refinement_bridge.rs` 退回边界层。
- 文档与代码都不再把 `l4v` 作为主架构约束。
