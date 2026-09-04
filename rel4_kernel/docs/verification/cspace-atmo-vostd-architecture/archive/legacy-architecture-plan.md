# CSpace 新架构方案

## 1. 方案目标

本文档基于三类材料共同设计 `sel4_cspace` 的下一代验证架构：

1. 当前分支中的 `sel4_cspace` 证明代码；
2. `/workspace/aux/vostd-main` 中 `vostd` / `ostd` 的 owner/view/wf 风格；
3. `/workspace/aux/atmosphere-main` 中 `atmo` 的 kernel/object invariant 风格。

目标不是简单模仿其中任一方，而是在 `sel4_common` 由 generated bitfield 结构主导的现实约束下，给 `sel4_cspace` 设计一套：

- 保留当前语义资产；
- 消除显式大 bridge 依赖；
- 引入对象内生规约；
- 允许未来继续扩展为更大 verified object system

的验证架构。

---

## 2. 三方风格对照

### 2.1 当前 `sel4_cspace` 的风格

当前证明代码的中心结构非常清楚：

- [abstract_cspace.rs](/workspace/rel4_kernel/sel4_cspace/specs/abstract_cspace.rs)  
  定义 `CSpaceState`、`CapSpec`、`SlotEntrySpec` 等抽象世界。

- [refinement_bridge.rs](/workspace/rel4_kernel/sel4_cspace/src/refinement_bridge.rs)  
  负责 raw `cap` / `cte_t` / 返回值 与抽象语义的桥接。

- [interface.rs](/workspace/rel4_kernel/sel4_cspace/src/interface.rs)  
  作为外部 verify-facing wrapper，对 `cte.rs` 的 `*_refined` 证明入口做稳定暴露。

这套结构的本质是：

**抽象状态在对象之外，精化关系在对象之外，方法最后证明“满足外置 spec”。**

### 2.2 `vostd` / `ostd` 的风格

在 `vostd` / `ostd` 里，更关键的是这些对象关系：

- `Inv`
- `InvView`
- `OwnerOf`
- `ModelOf`
- `wf(self, owner)`

可以直接参考：

- [ownership.rs](/workspace/aux/vostd-main/vstd_extra/src/ownership.rs)
- [demo/src/lib.rs](/workspace/aux/vostd-main/demo/src/lib.rs)

这套风格的核心不是“没有 spec”，而是：

- 规约更多内嵌在对象与 owner 的关系里；
- `view()` 和 `wf()` 是第一等公民；
- 具体方法主要在 `self + owner` 关系上证明，而不是先跳到一个外置全局 bridge 状态机。

`ostd` 的更重型实例还显示出：

- 可以在复杂数据结构中引入 owner 树；
- 可以让 cursor/item/page-table entry 等对象分别拥有局部 `wf()`；
- 可以用 tracked owner 在多层结构中递归传播局部一致性。

### 2.3 `atmo` 的风格

`atmo` 的特点不是 owner/view 语法本身，而是：

- 整个 kernel 或 subsystem 本身就是一个 verified object；
- `Kernel::wf()` 之类全局不变量非常强；
- 每个 syscall 的证明都在“更新组件后恢复全局不变量”这个框架里展开。

可参考：

- [kernel/spec.rs](/workspace/aux/atmosphere-main/kernel/verified/kernel/spec.rs)
- [syscall_new_endpoint.rs](/workspace/aux/atmosphere-main/kernel/verified/kernel/syscall_new_endpoint.rs)

这说明如果 `sel4_cspace` 要继续演化成更对象化的系统，最终也需要：

- 某种 subsystem-level owner/context；
- 某种全局 `wf()`；
- 若干 object-level `wf()` 组成全局 `wf()`。

---

## 3. 结论：新架构不该照搬哪一边

### 3.1 不能继续以当前架构为中心小修小补

因为当前架构即使已经比旧版更 Verus-native，核心组织方式仍然是：

- `specs/*`
- `repr/bridge`
- `*_refined -> interface wrapper`

它仍然会把“对象自身是什么”这个问题外包给 bridge/state。

### 3.2 也不能直接照搬 `atmo`

`atmo` 是一个更大粒度的 verified kernel object graph。

`sel4_cspace` 当前的问题规模更像：

- 一个带 bitfield/raw layout 的低层能力系统模块；
- 不宜一开始就强行构造完整的 `Kernel` 式大对象图；
- 也不宜直接要求所有 mutator 都走全局 tracked resource choreography。

### 3.3 最适合的方向：以 `vostd/ostd` 为主，借 `atmo` 的 subsystem 组织思路

因此新架构应当：

- 以 `vostd/ostd` 的 `OwnerOf` / `ModelOf` / `view()` / `wf()` 风格为基础；
- 以 `atmo` 的 subsystem-level `wf()` / invariant composition 为远期目标；
- 保留当前 `specs/*` 作为语义锚点和迁移对照，而不是第一步就删除。

---

## 4. 新架构的核心设计

### 4.1 四层，而不是旧三层

新的建议分层不是：

- spec
- refinement_bridge
- impl

而是：

1. raw repr 层
2. repr/view 层
3. verified object / owner / wf 层
4. subsystem model / operation spec 层

#### 第一层：raw repr

包括：

- `sel4_common::structures_gen::cap`
- `sel4_common::structures_gen::mdb_node`
- `sel4_cspace::cte::cte_t`

它们保留为 runtime representation，不做验证主语言。

#### 第二层：repr/view

这一层解释 raw 值，但不承担完整操作语义。

建议逻辑对象：

- `CapView`
- `MdbView`
- `SlotView`
- `ResolveRetView`

这一层的职责是：

- 吸收 bitfield getter/setter 细节；
- 提供稳定对象视图；
- 为下一层 `wf()` 提供语义材料。

#### 第三层：verified object / owner / wf

这是新架构的核心。

建议对象：

- `CapRef`
- `SlotRef`
- `SlotMut`
- `MdbRef`
- `CNodeRef`
- `CspaceCtx`
- `CspaceOwner`

这一层的职责是：

- 让 raw 对象第一次“带着规约活着”；
- 让 query/mutator 的前后条件转到对象本身；
- 让一致性证明集中为 `wf()` 与 owner 保持。

#### 第四层：subsystem model / operation spec

这一层不是 bridge，而是新的高层模型层。

建议保留并重构：

- `CSpaceState` 继续存在，但逐步从“所有 public proof 的主语言”退化为“subsystem model”
- `spec_cte_insert_post` 等 high-level operation spec 继续存在，但主要作用变成：
  - 解释 object-level update 的全局意义；
  - 提供 old/new 体系对照；
  - 承接未来 subsystem-level invariant theorem。

---

## 5. 具体对象方案

### 5.1 `CapView`

职责：

- 取代今天很多 `CapSpec` 直接穿透到 bridge 的用法；
- 作为 raw `cap` 的稳定逻辑读视图；
- 支持 query 级方法，例如 `same_region_as`、`same_object_as`。

建议：

- `CapView` 在语义上接近今天的 `CapSpec`；
- 但命名与位置上更明确为 object view，而不是系统外部 spec state 的一个片段。

### 5.2 `MdbView`

职责：

- 承载 `mdb_prev`、`mdb_next`、`mdb_revocable`、`mdb_first_badged`；
- 成为 slot 级关系判断的局部语义载体。

它的价值在于把今天很多“slot entry 才能读到的 mdb 语义”拆得更细。

### 5.3 `SlotView`

职责：

- 组合 `CapView + MdbView`；
- 成为 slot query 与 mutator 的主要逻辑对象；
- 替代今天直接拿 `SlotEntrySpec` 与 heap observer 对照的写法。

### 5.4 `SlotRef` / `SlotMut`

职责：

- `SlotRef`：只读观察 + `view()` + 局部 `wf()`
- `SlotMut`：可更新对象 + update 后恢复 `wf()`

这是最值得优先构建的 wrapper，因为 CSpace 的绝大多数操作最后都会汇聚到 slot。

### 5.5 `CspaceCtx` / `CspaceOwner`

职责：

- 表达“这一组 slot / root / cnode lookup 处于一致状态”；
- 提供 subsystem-level 前提；
- 成为未来 mutator 恢复全局一致性的证明落点。

它借的是 `atmo` 的 subsystem 不变量思路，但粒度只做到 CSpace 子系统，不扩成整内核。

---

## 6. 文件级重组方案

### 6.1 保留 `repr/*`

当前分支里已经有：

- `sel4_cspace/src/repr/*`

这是非常好的起点，应保留并强化。

未来要求：

- 所有 bitfield 投影尽量都经由 `repr/*` 进入逻辑世界；
- 不在 `cte.rs` / `interface.rs` 里散落一堆 raw getter 对齐证明。

### 6.2 `refinement_bridge.rs` 的最终命运

当前它仍然过重。

未来应只保留：

- external type spec
- 小 constructor witness
- 少量底层 observer

不再承担：

- object-level semantic helper
- local transition glue
- query/mutator public proof surface

### 6.3 新增 `verified/*`

建议新增目录：

- `sel4_cspace/src/verified/mod.rs`
- `sel4_cspace/src/verified/cap.rs`
- `sel4_cspace/src/verified/mdb.rs`
- `sel4_cspace/src/verified/slot.rs`
- `sel4_cspace/src/verified/cspace.rs`

职责划分：

- `verified/cap.rs`  
  `CapView`、`CapRef`、cap-local query

- `verified/mdb.rs`  
  `MdbView`、mdb relation helper

- `verified/slot.rs`  
  `SlotView`、`SlotRef`、`SlotMut`、slot-local query/mutator

- `verified/cspace.rs`  
  `CspaceCtx`、`CspaceOwner`、subsystem-level invariants

### 6.4 `cte.rs` 的新角色

未来 [cte.rs](/workspace/rel4_kernel/sel4_cspace/src/cte.rs) 不应继续成为“全部证明 helper 的终点仓库”。

建议它逐步只保留：

- runtime body
- 少量直接贴实现的局部证明

而把对象化规约迁到 `verified/*`。

### 6.5 `interface.rs` 的新角色

未来 [interface.rs](/workspace/rel4_kernel/sel4_cspace/src/interface.rs) 应退为：

- facade
- re-export
- 非核心的稳定入口

不再承担“证明语言设计中心”。

---

## 7. 新架构下的证明组织

### 7.1 query

以 `same_region_as`、`same_object_as` 为例：

当前写法更像：

- raw cap
- bridge to `CapSpec`
- 证明 equals abstract predicate

新写法应更像：

- `CapRef::view() -> CapView`
- query 直接定义在 `CapView` 或 `CapRef` 上
- 由 `CapRef.wf(owner)` 保证 raw 值与 view 一致

### 7.2 derived query

以 `is_final_cap`、`ensure_no_children` 为例：

当前写法依赖：

- `CSpaceState`
- slot id
- heap observer

新写法应更多依赖：

- `SlotRef`
- `CspaceCtx`
- `SlotView` / `MdbView`

即：

- 对象本地关系先说清楚；
- 再由 `CspaceCtx` 提供跨 slot lookup / root / cnode 一致性。

### 7.3 mutator

以 `cte_insert`、`cte_move`、`cte_swap` 为例：

当前写法强调：

- old/new heap
- local transition set
- `spec_*_post(old_state, new_state, ...)`

新写法应分成两层：

1. object-level update proof  
   `SlotMut` / `CspaceOwner` 更新后恢复局部和 subsystem `wf()`

2. semantic projection proof  
   从更新后的 `view()/model()` 推出旧 `spec_*_post` 成立

这样可以把“更新正确”与“抽象语义解释”分开。

---

## 8. 编码阶段建议

### 阶段 1：建立新语言

先实现：

1. `CapView`
2. `MdbView`
3. `SlotView`
4. `CapRef`
5. `SlotRef`
6. `SlotMut`
7. 基本 `view()` / `wf()`

这个阶段不要动四个 mutator 的主要证明链。

### 阶段 2：拿 query 做样板

优先迁：

- `same_region_as`
- `same_object_as`
- `is_cap_revocable`

目标：

- 脱离显式 bridge 术语；
- 建立 cap-level object proof 风格。

### 阶段 3：拿 slot-derived query 做样板

再迁：

- `is_mdb_parent_of`
- `is_final_cap`
- `ensure_no_children`

目标：

- 建立 `SlotRef + CspaceCtx` 组合风格。

### 阶段 4：迁 lookup

迁：

- `resolve_address_bits`

目标：

- 把返回值解释纳入 `ResolveRetView`；
- 把 lookup 正确性纳入 `CspaceCtx`。

### 阶段 5：迁 mutator

最后迁：

- `cte_insert`
- `insert_new_cap`
- `cte_move`
- `cte_swap`

目标：

- 先 object-level update；
- 后 semantic projection；
- 最后旧 bridge/local-transition 语言退场。

---

## 9. 需要明确保留的资产

新架构不是推倒重来。

必须明确保留：

- `specs/*` 中已经提炼好的 CSpace 语义；
- `repr/*` 中已经收敛过的 raw-to-view 投影；
- 当前验证代码里已经验证通过的 case split 和 proof insight；
- `sel4_common` raw bitfield 作为 runtime truth 的地位。

真正要替换的是：

- 证明组织方式；
- public proof surface；
- bridge 的中心地位；
- 对外主语言从 `CSpaceState + bridge` 转向 `object + owner + wf`。

---

## 10. 最终目标图景

最终希望 `sel4_cspace` 看起来像这样：

1. runtime 仍然操作 raw `cap` / `cte_t`；
2. `repr/*` 提供稳定投影；
3. `verified/*` 提供 `view()` / `wf()` / owner / method；
4. `specs/*` 提供高层模型与全局语义对照；
5. `refinement_bridge.rs` 只剩薄 TCB；
6. `interface.rs` 只剩 facade。

也就是说，最终的主体验证语言从：

**“abstract state + explicit refinement bridge + direct post”**

变成：

**“verified object + owner/view/wf + subsystem model”**

这套方案既保留了当前 `sel4_cspace` 的语义积累，也真正吸收了 `vostd/ostd` 与 `atmo` 的有效部分。
