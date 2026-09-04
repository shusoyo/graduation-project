# CSpace Atmo-Style Re-architecture TODO

本文件只记录当前有效的执行单，不再保留旧路线图的历史叙事。

使用规则：

- `[x]` 已完成
- `[ ]` 未完成
- `进行中` 表示已经开工但还没到验收线

## 0. 当前判断

- `sel4_cspace` 的目标形态已经明确为一个 atmo 风格 verified subsystem。
- `abstract_cspace.rs` 是模型层，不是主证明中心。
- `verified/cspace.rs` 是 subsystem 核心。
- `verified/{cap,mdb,slot}.rs` 是局部对象层。
- `verified/{derive,resolve,insert}.rs` 应继续变薄。
- `specs/cspace_ops/*` 是过渡层，应继续压缩。

## 1. 已完成的结构收敛

- [x] 建立 crate 内模型层：
  [src/specs/abstract_cspace.rs](/workspace/rel4_kernel/sel4_cspace/src/specs/abstract_cspace.rs)
- [x] 建立 raw-backed 对象层：
  [src/verified/cap.rs](/workspace/rel4_kernel/sel4_cspace/src/verified/cap.rs),
  [mdb.rs](/workspace/rel4_kernel/sel4_cspace/src/verified/mdb.rs),
  [slot.rs](/workspace/rel4_kernel/sel4_cspace/src/verified/slot.rs)
- [x] 建立 subsystem context：
  [src/verified/cspace.rs](/workspace/rel4_kernel/sel4_cspace/src/verified/cspace.rs)
- [x] 在 `verified/cspace.rs` 中落地 `PatchTouchedSlots`
- [x] 在 `verified/cspace.rs` 中落地 `CspacePatchSpec`
- [x] 让 `cte_insert` 走 patch-centered 语义
- [x] 让 `insert_new_cap` 走 patch-centered 语义
- [x] 在 `verified/slot.rs` 中补齐 patch touched slot 的局部 post helper
- [x] 让 `verified/insert.rs` 收敛成 thin shell 风格
- [x] `cargo xtask verify --package sel4_cspace` 已通过

## 2. 当前主线任务

### 2.1 subsystem 中心化

- [ ] 继续把全局恢复证明收口到 `verified/cspace.rs`
- [ ] 明确哪些 `patch_preserves_*` 还能继续合并成共享组合子
- [ ] 让 `CspaceCtx` 更清楚地区分：
  - patch construction
  - patch frame facts
  - invariant restoration

### 2.2 对象层继续变厚，操作层继续变薄

- [ ] 继续把 insert 家族的 touched-slot 局部 post 压回 `verified/slot.rs`
- [ ] 检查 `CapRef` / `MdbRef` / `SlotRef` 上还有哪些 query 仍停留在上层壳文件
- [ ] 避免在 `verified/{derive,resolve,insert}.rs` 新增重复 delegate

### 2.3 推广 patch 架构

- [ ] 把 `cte_move` 改成 patch-centered mutator
- [ ] 把 `cte_swap` 改成 patch-centered mutator
- [ ] 抽出 insert/move/swap 共享的 rewiring pattern

### 2.4 压缩 `specs/cspace_ops/*`

- [ ] 继续压缩 [src/specs/cspace_ops/insert.rs](/workspace/rel4_kernel/sel4_cspace/src/specs/cspace_ops/insert.rs)
- [ ] 检查 `derive.rs` / `resolve.rs` 中是否还有本应迁到对象层或 subsystem 层的逻辑
- [ ] 严格禁止再把新的全局恢复主证明放回 `specs/cspace_ops/*`

## 3. 下一阶段任务

### 3.1 compat / bridge 清理

- [ ] 盘点 [src/refinement_bridge.rs](/workspace/rel4_kernel/sel4_cspace/src/refinement_bridge.rs) 剩余职责
- [ ] 盘点 `interface.rs` 和 `compatibility.rs` 中仍然依赖旧叙事的入口
- [ ] 把它们压回 observer / façade / glue

### 3.2 `repr/*` 最小化

- [ ] 再检查 `repr/*` 是否还有多余中间层味道
- [ ] 保持它只承担 `view/result/helper`

### 3.3 文档与代码同步

- [x] 把文档主线改成 “atmo-style verified subsystem”
- [ ] 后续每做完一个阶段，就同步更新本 TODO

## 4. 阶段验收标准

### 阶段 A：结构方向正确

- [x] 代码和文档都明确 `verified/cspace.rs` 是 subsystem 中心
- [x] 代码和文档都明确 `specs/cspace_ops/*` 不是长期主证明中心

### 阶段 B：insert 家族 fully patch-centered

- [ ] insert 家族的主叙事完全变成 patch + object post + ctx recovery
- [ ] `verified/insert.rs` 继续瘦身
- [ ] `src/specs/cspace_ops/insert.rs` 继续缩小

### 阶段 C：mutator 统一

- [ ] `cte_move` 与 `cte_swap` 跟上同一套 patch 骨架
- [ ] insert/move/swap 不再各自维护平行大证明

### 阶段 D：边界收口

- [ ] `refinement_bridge.rs` 退化为边界层
- [ ] `interface.rs` 退化为薄入口
- [ ] `repr/*` 稳定停在 helper 层

## 5. 当前最值得做的三件事

1. 把 `move/swap` 拉进 patch-centered 骨架。
2. 继续把全局恢复组合子沉到 `verified/cspace.rs`。
3. 持续压缩 `specs/cspace_ops/*`，特别是 `insert.rs`。
