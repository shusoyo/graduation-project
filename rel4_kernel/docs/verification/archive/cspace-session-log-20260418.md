# CSpace Verus 验证 Session 记录（2026-04-18）

## 1. 本 session 目的

- 检查 `rel4_kernel/docs/verification` 下现有文档是否与当前工作树一致。
- 记录本 session 中已经完成的验证相关工作，避免“代码状态已前进，但文档仍停在旧阶段”。
- 判断当前 CSpace Verus 验证推进到哪一步，并给出下一步落点。

## 2. 本 session 观察到的当前工作树变化

### 2.1 `specs` 入口已经从单文件占位，演进为模块化结构

- [sel4_cspace/specs/lib.rs](/workspace/rel4_kernel/sel4_cspace/specs/lib.rs) 已从“内联边界假设”改为“规格入口模块”：
  - `pub mod boundary_assumptions;`
  - `pub mod abstract_cspace;`
  - `pub mod cspace_ops;`
- `specs_smoke_check()` 当前会同时接通：
  - `boundary_assumptions::boundary_assumptions_smoke_check()`
  - `abstract_cspace::abstract_cspace_smoke_check()`
  - `cspace_ops::cspace_ops_smoke_check()`

这说明当前验证入口已经不只覆盖阶段 A 的边界合同，也已经把阶段 B 的抽象模型纳入了验证路径。

### 2.2 阶段 A 的边界合同已拆分为独立模块

- 新增文件：
  - [sel4_cspace/specs/boundary_assumptions.rs](/workspace/rel4_kernel/sel4_cspace/specs/boundary_assumptions.rs)
- 当前已落地并接通 smoke check 的可信边界锚点包括：
  - `assume_convert_to_mut_type_ref_contract`
  - `assume_convert_to_option_mut_type_ref_contract`
  - `assume_mask_vm_rights_non_escalation`
  - `assume_finalise_cap_pairing`
  - `assume_post_cap_deletion_origin`
  - `assume_preemption_point_progress`

结论：阶段 A 的“可信边界台账 + 合同锚点 + 验证入口接通”仍然成立，且组织方式已经从初版内联实现演进为独立模块。

### 2.3 阶段 B 的抽象模型已经开始落地

- 新增文件：
  - [sel4_cspace/specs/abstract_cspace.rs](/workspace/rel4_kernel/sel4_cspace/specs/abstract_cspace.rs)
- 当前已建模内容包括：
  - `SlotId`、`ObjectKind`、`CapKind`
  - `ObjectRef`、`Rights`
  - `CNodeCapDataSpec`、`UntypedCapDataSpec`
  - `CapSpec`、`SlotEntrySpec`、`CSpaceState`
  - `cnode_lookup`：从 CNode object 与 offset 到目标 slot 的抽象映射
- 当前已写出的核心谓词/关系包括：
  - 权限收敛与类型匹配：`rights_subseteq`、`object_kind_matches_cap_kind`、`rights_compatible_with_kind`、`valid_cap`
  - slot/mdb 关系：`slot_empty`、`same_object`、`same_region`、`mdb_links`、`immediate_derived`、`is_final_cap`
  - CNode lookup 关系：`cnode_slot_at`、`cnode_cap_slot_at`
  - CSpace 图模型：`cnode_targets`、`cspace_edge`、`reachable_slot_from`
  - 全局不变量：`valid_slots`、`mdb_prev_next_consistent`、`badge_derivation_wf`、`cnode_slots_wf`、`cnode_lookup_wf`、`cspace_roots_wf`、`cspace_graph_wf`、`wf`
  - 帧条件骨架：`slots_unchanged_except`
- `abstract_cspace_smoke_check()` 已构造一个最小状态并验证上述模型能被当前官方入口检查。

结论：阶段 B 并非“尚未开始”，而是已经进入“抽象模型草稿已落地并纳入验证入口”的状态。

### 2.4 本 session 还观察到的非文档工作树状态

- [sel4_cspace/src/capops.rs](/workspace/rel4_kernel/sel4_cspace/src/capops.rs) 当前在工作区中是删除状态。
- 当前删除没有影响 `sel4_cspace` 的 `cargo check` 与官方 verify 入口闭环，但该文件是否属于后续整理目标，尚未在本 session 内进一步归档说明。

### 2.5 阶段 C 的第一版原语规格骨架已经落地

- 新增文件：
  - [sel4_cspace/specs/cspace_ops.rs](/workspace/rel4_kernel/sel4_cspace/specs/cspace_ops.rs)
- [sel4_cspace/specs/lib.rs](/workspace/rel4_kernel/sel4_cspace/specs/lib.rs) 已新增：
  - `pub mod cspace_ops;`
  - `cspace_ops::cspace_ops_smoke_check()`
- 当前已落地的 `cte_insert` 规格骨架包括：
  - `spec_cte_insert_derivable`
  - `spec_set_untyped_cap_as_full_applies`
  - `spec_set_untyped_cap_as_full_effect`
  - `spec_cte_insert_changed_slots`
  - `spec_cte_insert_pre`
  - `spec_cte_insert_mdb_shape`
  - `spec_cte_insert_post`
  - `spec_cte_insert`
- 当前已落地的 `cte_move` 规格骨架包括：
  - `spec_cte_move_cap_compatible`
  - `spec_cte_move_changed_slots`
  - `spec_cte_move_pre`
  - `spec_cte_move_mdb_shape`
  - `spec_cte_move_post`
  - `spec_cte_move`
- 当前已落地的 `cte_swap` 规格骨架包括：
  - `spec_swap_slot_ref`
  - `spec_cte_swap_root_compatible`
  - `spec_cte_swap_changed_slots`
  - `spec_cte_swap_pre`
  - `spec_cte_swap_mdb_shape`
  - `spec_cte_swap_post`
  - `spec_cte_swap`
- 当前已落地的 `resolve_address_bits` 规格骨架包括：
  - `ResolveAddressBitsStatusSpec`
  - `ResolveAddressBitsFaultSpec`
  - `ResolveAddressBitsResultSpec`
  - `spec_pow2`
  - `spec_extract_bits`
  - `spec_cnode_level_bits`
  - `spec_resolve_guard_value`
  - `spec_resolve_guard_matches`
  - `spec_resolve_address_bits_next_slot`
  - `spec_cnode_cap_lookup_total`
  - `spec_cspace_lookup_total`
  - `spec_resolve_address_bits_success`
  - `spec_resolve_address_bits_pre`
  - `spec_resolve_address_bits_fault`
  - `spec_resolve_address_bits_post`
  - `spec_resolve_address_bits`
- smoke check 也已拆分为：
  - `cte_insert_smoke_check()`
  - `cte_move_smoke_check()`
  - `cte_swap_smoke_check()`
  - `resolve_address_bits_smoke_check()`

这一步的设计取向是：

- 抽象词汇仍复用 [abstract_cspace.rs](/workspace/rel4_kernel/sel4_cspace/specs/abstract_cspace.rs)；
- 原语规格单独放在 `cspace_ops.rs`，让“模型层”和“行为规格层”分开；
- `cte_insert` 的关键语义点尽量向 l4v 对齐：
  - 目标槽必须为空；
  - 新节点插到 `src` 之后；
  - 旧 `src.mdb_next` 的反向指针要改到 `dest`；
  - `setUntypedCapAsFull` 对源槽的影响先用抽象合同表达，再留待后续收紧为精确算术语义。
- `cte_move` 的关键语义点也已开始向 l4v 对齐：
  - 目标槽必须为空；
  - 目标槽继承源槽原有的 `mdb` 位置；
  - 源槽清空为 `NullCap + null MDB`；
  - `mdbPrev/mdbNext` 相邻节点要改指向新的目标槽。
- `cte_swap` 的关键语义点也已开始向 l4v 对齐：
  - 两个 `(cap, mdb-position)` 对一起交换，而不是只交换 `cap`；
  - 如果两个被交换节点本来就是 MDB siblings，需要把指向 partner slot 的指针一并重写；
  - 当前用 `spec_swap_slot_ref` 抽象这种 slot-id 重命名；
  - 对 roots 位置先加了保守兼容条件 `spec_cte_swap_root_compatible`，避免把非 `CNodeCap` 交换进 root slot。
- `resolve_address_bits` 的关键语义点也已开始向 l4v / Rust 实现对齐：
  - 用 `cnode_lookup` 抽象 `locateSlotCap` 的 offset 查找；
  - 用 `spec_pow2` / `spec_extract_bits` 表达 guard 与 radix 的位段抽取；
  - 用 `spec_resolve_guard_value` 明确当前 unresolved prefix 上实际抽到的 guard 值；
  - 用 `spec_resolve_guard_matches` 表达当前 unresolved prefix 上的 guard 一致性；
  - 用 `spec_resolve_address_bits_next_slot` 表达单步 lookup；
  - fault taxonomy 已收紧为 `InvalidRoot / GuardMismatch / DepthMismatch`；
  - 额外引入 `spec_cnode_cap_lookup_total` / `spec_cspace_lookup_total`，把当前有限 `cnode_lookup` map 与 l4v 的 total `locateSlotCap` 语义接起来。

## 3. 本 session 实测结果

### 3.1 交叉 `cargo check`

- 命令：

```bash
RUSTUP_TOOLCHAIN=1.94.0-x86_64-unknown-linux-gnu \
RUSTC_BOOTSTRAP=1 \
PLATFORM=spike \
MARCOS="KERNEL_STACK_BITS=12 FASTPATH=true HAVE_FPU=true RISCV_EXT_D=true" \
CARGO_BUILD_TARGET=riscv64gc-unknown-none-elf \
cargo check -p sel4_cspace
```

- 结果：通过。

### 3.2 官方验证入口

- 命令：`./tools/verify-cspace-official.sh`
- 结果：通过。
- 当前输出：`12 verified, 0 errors`

与先前“`2 verified, 0 errors`”相比，当前结果增加的直接原因是：

- `boundary_assumptions_smoke_check()` 仍在；
- 新增的 `abstract_cspace_smoke_check()` 也已经接入验证入口。
- 新增的 `cspace_ops_smoke_check()` 也已经接入验证入口。
- `cspace_ops` 内部的 `cte_insert_smoke_check()` 与 `cte_move_smoke_check()` 也已分别被验证。
- 本 session 新增的 `cte_swap_smoke_check()` 也已接入并通过。
- 本 session 新增的 `resolve_address_bits_smoke_check()` 也已接入并通过。
- 本 session 继续收紧后的 `resolve_address_bits_smoke_check()` 现在还覆盖了：
  - 递归 success；
  - 提前停在非 `CNodeCap` 的 success；
  - `InvalidRoot`；
  - `GuardMismatch`；
  - `DepthMismatch`。

### 3.3 当前已知 warning / 噪声

- Rust 侧仍存在 `sel4_common/src/structures.rs` 的 `repr(C)` enum 判别值 warning。
- Verus 当前会对 [abstract_cspace.rs](/workspace/rel4_kernel/sel4_cspace/specs/abstract_cspace.rs) 中大量 `Option::is_Some()` / `is_None()` 用法给出 deprecation warning。
- Verus 还会为部分量词自动打印 trigger 选择提示。

这些问题当前没有阻断验证通过，但属于后续值得收口的技术债。

### 3.4 本 session 的 review 与修复

在继续推进 Stage C 之前，本 session 又做了一轮针对当前抽象模型的 review，发现并修复了两个会影响后续证明方向的规格问题：

1. `cnode_slots` / `cnode_lookup` / 图可达性的语义裂缝
- 问题位置：
  - [sel4_cspace/specs/abstract_cspace.rs](/workspace/rel4_kernel/sel4_cspace/specs/abstract_cspace.rs)
- 原问题是：
  - `resolve_address_bits` 通过 `cnode_lookup` 建模 offset lookup；
  - 但 `cspace_edge` / `reachable_slot_from` 仍经由 `cnode_slots` 建模图边；
  - 因而同一个抽象状态里，lookup 可到达的 slot 与 graph 可到达的 slot 可以分叉。
- 本 session 的修复是：
  - 将 `cnode_targets` 改为从 `cnode_lookup` 的像集生成；
  - 保留 `cnode_slots` 作为 CNode 成员集合台账，但不再让 graph reachability 直接依赖它；
  - `cnode_lookup_wf` 继续要求 `cnode_slots` / `cnode_lookup` 至少在 object domain 上对齐，且 lookup 指向的 slot 必须属于对应 CNode。

2. 三个 slot 更新原语缺少 `cnode_lookup` 的 frame condition
- 问题位置：
  - [sel4_cspace/specs/cspace_ops.rs](/workspace/rel4_kernel/sel4_cspace/specs/cspace_ops.rs)
- 原问题是：
  - `cte_insert` / `cte_move` / `cte_swap` 的 postcondition 固定了 `slots`、`roots`、`cnode_slots`；
  - 但没有固定 `cnode_lookup`；
  - 这会让规格层错误地允许“slot 内容不变，但 offset lookup 语义漂移”的后状态。
- 本 session 的修复是：
  - 为 `spec_cte_insert_post`、`spec_cte_move_post`、`spec_cte_swap_post` 补上
    `new_state.cnode_lookup =~= old_state.cnode_lookup`。

修复后，本 session 已重新执行：

- 交叉 `cargo check -p sel4_cspace`：通过。
- `./tools/verify-cspace-official.sh`：通过。
- 官方验证入口当前输出仍为：`12 verified, 0 errors`。

## 4. 当前阶段判断

截至 2026-04-18 本 session，建议将当前状态表述为：

1. 阶段 A：已完成并收口。
2. 阶段 B：已开始，且抽象模型草稿已经进入 `specs` 主入口并通过官方验证入口检查。
3. 阶段 C：已开始，且 `cte_insert`、`cte_move`、`cte_swap`、`resolve_address_bits` 的第一版规格骨架都已落地。
4. 阶段 D/E：尚未开始。

因此，当前最准确的口径不是“还停在阶段 A”，而是：

- 已完成阶段 A；
- 已进入阶段 B 的模型落地期；
- 已进入阶段 C 的起步期，但还没有进入函数级证明。

## 5. 当前缺口

当前阶段 B 虽然已经启动，但仍有明显缺口：

1. `cte_insert`、`cte_move`、`cte_swap`、`resolve_address_bits` 虽然已经有第一版骨架，但当前 smoke check 主要验证“核心构件可联通”，还没有把 `wf` 整体自动推出成可复用引理。
2. `resolve_address_bits` 当前虽然已经补到了递归 success、early-stop success 与三类 l4v 风格 fault，但这些还主要体现在 smoke check 中，尚未收束成后续函数级证明可直接复用的引理包。
3. `spec_cnode_cap_lookup_total` / `spec_cspace_lookup_total` 当前还是 Stage C 的桥接前提，尚未被进一步下沉进更稳定的模型层不变量或由 `wf` 直接推出。
4. `spec_set_untyped_cap_as_full_effect` 当前仍是结构化合同，还没有收紧到与 l4v `maskedAsFull` 完全一致的精确 free-index 算术语义。
5. `spec_cte_move_cap_compatible` 与 `spec_cte_swap_root_compatible` 当前都还是阶段性兼容合同，还需要继续收紧到更稳定的“前后 slot 关系保持”引理族。
6. `abstract_cspace.rs` 使用的部分 Verus API 语法已经出现 deprecation warning，后续最好在继续扩大规格前先做一次语法收口。

## 6. 下一步建议

建议下一步按以下顺序推进：

1. 先把四个 Stage C 骨架补成更稳的可复用入口：
   - 为 `spec_cte_insert_pre/post` 补辅助引理，尽量把 `wf` 相关大前提拆小。
   - 为 `spec_cte_move_pre/post` 补辅助引理，明确哪些邻接关系必须保持。
   - 为 `spec_cte_swap_mdb_shape` 补“adjacent siblings / pointer renaming”相关引理，减少后续 sibling 情况下的证明摩擦。
   - 为 `spec_resolve_address_bits_success/fault/post` 补逐层递归与结果打包的辅助引理，减少后续把 smoke check 升格为总包断言时的自动化摩擦。
   - 收紧 `spec_set_untyped_cap_as_full_effect` 的后效描述。
2. 收紧 `resolve_address_bits` 的 lookup totality 桥接：
   - 评估是否把 `spec_cnode_cap_lookup_total` / `spec_cspace_lookup_total` 下沉为更稳定的不变量；
   - 或者至少为它们补足后续证明可直接复用的入口引理。
3. 继续冻结阶段 B 的最小抽象词汇表：
   - 明确哪些谓词是后续目标原语必须依赖的核心入口，例如 `wf`、`immediate_derived`、`is_final_cap`、`reachable_slot_from`、`slots_unchanged_except`。
4. 在规格骨架继续增长前，先收口 `abstract_cspace.rs` 中的 deprecation warning 与 trigger 噪声，避免后面规格增长后诊断成本继续上升。

## 7. 本 session 对文档的处理结果

本 session 已完成以下文档动作：

1. 复核 `docs/verification` 下现有阶段 A 文档与当前工作树的一致性。
2. 保留 [cspace-stageA-progress-report-20260418.md](/workspace/rel4_kernel/docs/verification/cspace-stageA-progress-report-20260418.md) 作为“阶段 A 收口”的历史报告，不把它误改成阶段 B 报告。
3. 新增本文件，作为当前 session 的状态记录与阶段判断依据。
4. 记录 `cspace_ops.rs` 中四个目标原语的第一版规格骨架已经全部进入主验证入口。
5. 同步更新计划/步骤文档，使其不再把当前状态误写成“默认停在阶段 A 完成点”或“阶段 C 尚未开始”。
