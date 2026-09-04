# 02 Evidence And References

## 1. 这份文档回答什么

这里只回答一件事：

**为什么新的 `sel4_cspace` 架构要按 subsystem/object/patch 这条线走。**

## 2. 主要依据的优先级

这轮设计的依据，优先级已经改成下面这样：

1. 当前 `sel4_cspace` 代码里已经存在并正在验证的结构
2. atmo 的 verified subsystem 组织方式
3. `vostd/ostd` 的对象式 `view()/wf()` 语言
4. 历史上的 `l4v` / seL4 CSpace 语义背景

其中第 4 项不再是本轮架构的主驱动力。

## 3. 当前仓库里真正的直接事实

### 3.1 抽象模型已经在 crate 内

当前真正的抽象模型，不应该再表述成“来自外部旧 spec”，而应直接认定为：

- [abstract_cspace.rs](/workspace/rel4_kernel/sel4_cspace/src/specs/abstract_cspace.rs)

这里已经定义了：

- `CapSpec`
- `SlotEntrySpec`
- `CSpaceState`
- `valid_cap`
- `slot_*` 关系语义
- `valid_slots`
- `mdb_prev_next_consistent`
- `cnode_*_wf`
- `cspace_roots_wf`
- `wf`

对当前项目来说，这就是 subsystem 数学模型层。

### 3.2 subsystem 结构已经落到了代码上

当前代码已经明确出现了 subsystem-centered 结构：

- [verified/cspace.rs](/workspace/rel4_kernel/sel4_cspace/src/verified/cspace.rs)
  里面已经有：
  - `CspaceCtx`
  - `PatchTouchedSlots`
  - `CspacePatchSpec`
  - `cte_insert_patch`
  - `insert_new_cap_patch`
- [verified/slot.rs](/workspace/rel4_kernel/sel4_cspace/src/verified/slot.rs)
  里面已经有 patch 局部 post helper
- [verified/insert.rs](/workspace/rel4_kernel/sel4_cspace/src/verified/insert.rs)
  里面已经是 thin shell 风格

这说明新架构并不是纯计划，而是已经有代码落点。

### 3.3 raw-backed 对象也是既成事实

当前对象层已经不是纯 view façade，而是 raw-backed：

- [verified/cap.rs](/workspace/rel4_kernel/sel4_cspace/src/verified/cap.rs)
- [verified/mdb.rs](/workspace/rel4_kernel/sel4_cspace/src/verified/mdb.rs)
- [verified/slot.rs](/workspace/rel4_kernel/sel4_cspace/src/verified/slot.rs)

这正是 atmo 风格里最关键的一步：对象贴着运行时真相，而不是先绕到厚中间层。

## 4. atmo 提供的真正参考

### 4.1 要学的是“模块组织方式”，不是照抄函数

对 `sel4_cspace` 最有价值的 atmo 经验，不是某个具体 API，而是：

- 局部对象自己带 ghost 视图
- subsystem 自己带全局 `wf()`
- 操作证明围绕对象和 subsystem 展开
- spec 文件不承担 theorem farm

### 4.2 `sel4_cspace` 更像 atmo 的一个子模块

从粒度上看，`sel4_cspace` 不该类比 atmo 的整个 kernel，而应类比：

- pagetable 一类的 verified subsystem
- memory/process manager 里带局部对象与全局约束的子模块

因此合理映射是：

- `CapRef` / `MdbRef` / `SlotRef`
  对应 atmo 里的局部 verified object
- `CspaceCtx`
  对应 atmo 里的 subsystem object
- `insert/derive/resolve`
  对应 subsystem 上的方法或很薄的操作壳

### 4.3 为什么不能继续走厚 spec

atmo 风格下，spec 层之所以不会爆炸，是因为：

- 对象自身已经吸收很多局部语义；
- subsystem 自身已经吸收很多恢复结论；
- mutator 先是对象更新，再是 subsystem 恢复；
- 不是把所有东西都挤到一个操作 spec 文件里。

这正是当前 `insert.rs` 过厚的反例来源。

## 5. `vostd/ostd` 提供的语言依据

`vostd/ostd` 给我们的主要不是“大架构答案”，而是对象式 Verus 语言：

- `view()`
- `wf()`
- object-local spec
- 在对象和 owner/context 上挂前后条件

所以这轮重构吸收的是它的表达方式，而不是一定要完整复制某套 owner 结构。

## 6. `l4v` 的现在定位

这轮文档里，`l4v` 只保留成下面这个地位：

- 历史背景
- 可能的概念祖先
- 论文里可讨论的语义渊源

但它不再是：

- 当前模块分层的硬约束
- 当前命名设计的硬约束
- 当前 proof organization 的硬约束

如果将来写论文，需要非常严格地区分两件事：

- “当前 Verus 证明直接基于什么定义”
- “这些定义在概念上与哪些 seL4/l4v 语义同源”

前者今天可以直接指向本 crate；后者需要单独做对照。
