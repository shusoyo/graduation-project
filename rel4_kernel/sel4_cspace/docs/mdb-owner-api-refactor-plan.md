# MDB Owner API Refactor Plan

## Goal

把 `mdb` 层补成真正的 owner 层：

- `mdb` 只负责 `cteMDBNode` / link / `MdbTable` summary
- `manager` 不再直接做 MDB link patch
- `cap` 写入留在 `manager` / `cte-cap` 侧
- `cdt` 更新继续留在 `manager`
- 不为了兼容旧 helper 保留错误分层

## Frozen Boundary

`MdbTable` 允许做的事：

- 持有并转移 `Tracked<Map<SlotPtr, PointsTo<cte_t>>>`
- 读取 slot 的 MDB view
- 修改 `cteMDBNode`
- 维护 `order` / `live_slots`
- 提供 MDB 结构 post-state relation

`MdbTable` 不允许做的事：

- 写 capability payload
- 推导 CDT parent / original
- 承担 manager cross-layer invariant

## Required Operations

### Ownership

- `[done]` `take_entry_perm`
- `[done]` `put_entry_perm`

### MDB Link Primitives

- `[done]` `insert_node_after`
- `[done]` `move_node`
- `[done]` `swap_nodes`
- `[done]` `remove_node`

### Post-State Vocabulary

- `[done]` `insert_between_rel`
- `[done]` `move_slot_rel`
- `[done]` `swap_slots_rel`
- `[done]` `remove_slot_rel`

## Manager Migration

- `[done]` `cte_insert` 改成：
  - manager 写 cap
  - manager 处理 untyped-full
  - manager 调 `mdb.insert_node_after`
  - manager 更新 `cdt`

- `[done]` `insert_new_cap` 改成：
  - manager 写 cap
  - manager 调 `mdb.insert_node_after`
  - manager 更新 `cdt`

- `[done]` `cte_move` 改成：
  - manager 改 cap
  - manager 调 `mdb.move_node`
  - manager 更新 `cdt/zombie`

- `[done]` `cte_swap` 改成：
  - manager 改 cap
  - manager 调 `mdb.swap_nodes`
  - manager 更新 `cdt/zombie`

- `[done]` `set_empty/delete` 改成：
  - manager 清 cap
  - manager 调 `mdb.remove_node`
  - manager 更新 `cdt/zombie`

## Cleanup

- `[done]` manager 里的 `set_slot_mdb_*_tracked` 已删除
- `[done]` manager 的 `take_slot_perm/put_slot_perm` 已删除，上层直接使用 `mdb.take_entry_perm/put_entry_perm`
- `[done]` `mdb` 层不承担 cap/CDT 语义；只保留 cap payload frame

## Current Status

当前 `MdbTable` owner API 已落到代码中：

1. 冻结 `MdbTable` 的操作集合
2. 把 insert/move/swap/delete 需要的 MDB primitive 放进 `mdb`
3. 把 manager 上层调用切过去
4. 最后再收编旧 helper 和 wrapper
