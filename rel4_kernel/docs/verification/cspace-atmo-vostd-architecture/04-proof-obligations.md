# 04 Proof Obligations

## 1. 总体证明分工

在新架构里，证明义务按 4 层拆开：

1. raw-to-view
2. object-local
3. patch semantics
4. subsystem recovery

这 4 层必须分别有主语，不能再混进一个大操作 spec 文件里。

## 2. raw-to-view 义务

### 2.1 要证明什么

需要证明 raw 表示与逻辑 view 一致：

- raw `cap` 对应 `CapView`
- raw `mdb_node` 对应 `MdbView`
- raw `cte_t` 对应 `SlotView`

### 2.2 应该放在哪

主要放在：

- `repr/*`
- `verified/{cap,mdb,slot}.rs` 的 `wf()`

### 2.3 目标效果

对象层以后不该再频繁手动展开 raw bitfield 语义，而应直接依赖：

- `view()`
- `wf()`

## 3. object-local 义务

### 3.1 `CapRef`

`CapRef` 需要承担：

- capability 局部合法性
- `same_region_as`
- `same_object_as`
- `is_revocable_against`

### 3.2 `MdbRef`

`MdbRef` 需要承担：

- prev/next/revocable/first_badged 的局部视图一致性

### 3.3 `SlotRef`

`SlotRef` 需要承担：

- `entry_spec()`
- `is_mdb_parent_of`
- `is_final_cap`
- `ensure_no_children`
- `derive_cap` 所需的局部判断
- patch touched slot 的 post view helper

## 4. patch 语义义务

### 4.1 为什么 patch 是必要层

只要 mutator 会改多个 slot，就不能只靠单个对象 post 就结束。

但也不应该直接跳到“整个 `CSpaceState` 如何逐格改写”。

中间最合适的层就是 patch：

- touched slots 是谁
- touched slots 的 post 是什么
- untouched slots 保持不变
- context fields 是否保持不变

### 4.2 当前 patch 层的主语

当前这个层的主语已经确定是：

- `PatchTouchedSlots`
- `CspacePatchSpec`

它们属于 [verified/cspace.rs](/workspace/rel4_kernel/sel4_cspace/src/verified/cspace.rs)。

### 4.3 patch 层需要证明什么

至少需要有：

- `patch_post_ctx`
- `patch_preserves_other_slots`
- `patch_preserves_has_slot`
- `patch_preserves_context_fields`
- old-next rewiring 正确性

## 5. subsystem recovery 义务

这一层是 `CspaceCtx` 的核心职责。

### 5.1 要恢复哪些东西

需要恢复：

- `valid_slots`
- `mdb_prev_next_consistent`
- `cnode_lookup_wf`
- `cnode_slots_wf`
- `cspace_roots_wf`
- 总 `wf()`

### 5.2 为什么必须集中在 `CspaceCtx`

因为这些性质本质上都不是单个对象能完成的。

如果把它们继续留在 `specs/cspace_ops/*`，就会自然形成：

- theorem farm
- delegate farm
- 一个操作文件复写一遍 subsystem 逻辑

所以正确做法是：

- 对象层只提供局部 post
- `CspaceCtx` 组合出恢复结论

## 6. query / derive / resolve 的义务

### 6.1 query

query 的目标是：

- 尽量成为对象或 `CspaceCtx` 的直接方法
- 不再依赖 bridge-style helper chain

### 6.2 derive

`derive_cap` 的目标是：

- 由 `SlotRef` 提供局部语义判断
- 由 `CspaceCtx` 提供全局上下文
- 结果落到 result view

### 6.3 resolve

`resolve_address_bits` 的目标是：

- 由 `CspaceCtx` 成为主语
- 用 `CapRef` 提供 root-cap 局部语义
- 用 result view 表达 lookup 结果

## 7. mutator 的义务

当前 mutator 至少包括：

- `cte_insert`
- `insert_new_cap`
- `cte_move`
- `cte_swap`

它们应统一遵循下面的证明节奏：

1. precondition 成立
2. 构造 patch
3. 对 touched slot 给出局部 post
4. 对 untouched slot 给出 frame
5. `CspaceCtx` 恢复 subsystem invariant
6. 得到 post-context

## 8. 什么不再算“好的证明结构”

后续如果一个操作证明主要靠下面这些东西成立，就说明结构还没改到位：

- 大段 `Map::insert(...).insert(...).insert(...)`
- 一个操作文件里同时写局部 post 和全局恢复主证明
- `verified/*` 只是具名转发
- `specs/cspace_ops/*` 承担主要复杂度

这类结构即使能过验证，也不应再视为目标结构。
