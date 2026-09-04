# 03 Target Architecture

## 1. 总体结构

目标结构已经固定成 5 层：

1. raw runtime truth
2. abstract model
3. local verified objects
4. subsystem context
5. thin operation / compatibility shell

对应到当前 crate，就是：

1. raw runtime truth
   `sel4_common::structures_gen::{cap, mdb_node}` 和 `cte_t`
2. abstract model
   [src/specs/abstract_cspace.rs](/workspace/rel4_kernel/sel4_cspace/src/specs/abstract_cspace.rs)
3. local verified objects
   [src/verified/cap.rs](/workspace/rel4_kernel/sel4_cspace/src/verified/cap.rs),
   [mdb.rs](/workspace/rel4_kernel/sel4_cspace/src/verified/mdb.rs),
   [slot.rs](/workspace/rel4_kernel/sel4_cspace/src/verified/slot.rs)
4. subsystem context
   [src/verified/cspace.rs](/workspace/rel4_kernel/sel4_cspace/src/verified/cspace.rs)
5. thin operation / compatibility shell
   `src/verified/{derive,resolve,insert}.rs` 和 runtime glue

## 2. 每层职责

### 2.1 raw runtime truth

这一层只负责：

- ABI/layout 真相
- bitfield 读写边界
- 实际运行时数据

它不负责主证明叙事。

### 2.2 abstract model

[abstract_cspace.rs](/workspace/rel4_kernel/sel4_cspace/src/specs/abstract_cspace.rs) 的长期职责很明确：

- 定义 `CSpaceState`
- 定义 capability / slot / MDB 的抽象关系
- 定义 subsystem 级不变量

它是数学模型层，不是 mutator theorem farm。

### 2.3 local verified objects

对象层长期只保留三类对象：

- `CapRef`
- `MdbRef`
- `SlotRef`

它们负责：

- 持有 raw 引用
- 收 ghost view
- 暴露 `view()`
- 暴露局部 `wf()`
- 暴露 query 和局部 post

对象层不负责：

- 全局不变量恢复
- 整个 subsystem 的 state rewrite

### 2.4 subsystem context

`CspaceCtx` 是整个架构的中心。

它负责：

- 持有全局 ghost model
- 定义 subsystem 级 `wf()`
- 定义 `has_slot` / lookup / roots 等全局观察
- 构造 patch
- 证明 patch 恢复全局不变量

这层已经在 [verified/cspace.rs](/workspace/rel4_kernel/sel4_cspace/src/verified/cspace.rs) 中体现为：

- `PatchTouchedSlots`
- `CspacePatchSpec`
- `patch_post_ctx`
- `patch_preserves_*`

### 2.5 thin operation shell

`verified/{derive,resolve,insert}.rs` 的长期目标不是继续变厚，而是继续变薄。

它们只应承担：

- case split
- 调对象层 helper
- 调 `CspaceCtx` 的恢复接口
- 形成顶层 post

如果某个操作最终能完全收进 `SlotRef` 或 `CspaceCtx`，对应文件甚至可以继续缩小或消失。

## 3. patch-centered mutator 语义

这是新架构和旧架构最关键的分水岭。

### 3.1 旧做法

旧做法习惯把 mutator 写成：

- 直接给出 `expected_state`
- 再写很多 map update lemma
- 再写很多 `preserves_*`

这样会自然长出厚 spec。

### 3.2 新做法

新做法先定义 patch：

- 改哪些 slot
- 每个 touched slot 的 post view 是什么
- `post_model` 是什么
- 哪些 context field 不变

当前已经对应到：

- `PatchTouchedSlots`
- `CspacePatchSpec`

然后再让：

- `SlotRef` 负责 touched slot 的局部 post
- `CspaceCtx` 负责 patch 后的全局恢复

## 4. 目录与模块的长期分工

### 4.1 `src/specs/abstract_cspace.rs`

长期保留，作为数学模型层。

### 4.2 `src/specs/cspace_ops/*`

长期目标是极薄。

它们最多保留：

- precondition
- 效果摘要
- 少量结果 spec/helper

它们不再承担主 proof body。

### 4.3 `src/repr/*`

长期降级成：

- `CapView`
- `MdbView`
- `SlotView`
- result view
- raw-to-view helper

这里不再长出新的语义中心。

### 4.4 `src/verified/*`

这是长期主战场。

- `cap/mdb/slot` 负责 local object semantics
- `cspace` 负责 subsystem semantics
- `derive/resolve/insert` 负责 thin shell

## 5. 与 atmo 的对应关系

更准确的类比不是：

- `sel4_cspace` = atmo kernel

而是：

- `sel4_cspace` = atmo 里的一个 verified subsystem module

所以从写法上应该学：

- 小对象如何内嵌 ghost
- subsystem 如何拆 `wf()`
- mutator 如何先做局部更新，再做全局恢复

而不是执着于“atmo 是否也有一个叫 spec 的文件夹”。

## 6. 反模式清单

后续看到下面这些迹象，就说明架构又开始偏回旧路了：

- 单个 `specs/cspace_ops/*.rs` 文件重新长成几百上千行 proof body
- `verified/cspace.rs` 只剩 delegate，不再承担实质恢复证明
- `repr/*` 重新变成比对象层还厚的中间世界
- `refinement_bridge.rs` 再次承接主证明
- 新操作继续先写全局 map rewrite，再去找对象层对接

这些都应视为需要回滚方向的信号。
