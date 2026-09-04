# reL4 CSpace Verus-Native Refactor TODO

本清单用于记录执行进度，不重复展开完整设计理由。

配套文档：

- 总纲：[/workspace/rel4_kernel/docs/verification/verus-native-refactor/README.md](/workspace/rel4_kernel/docs/verification/verus-native-refactor/README.md)
- 细化计划：[/workspace/rel4_kernel/docs/verification/verus-native-refactor/detailed-plan.md](/workspace/rel4_kernel/docs/verification/verus-native-refactor/detailed-plan.md)

---

## 0. 准备

- [x] 确认 `specs/*` 中 query / derive / resolve / mutator 的最终抽象 post 已完整可用。
- [x] 确认 `refinement_bridge.rs` 中当前 `trusted_*` 项按职责完成初步分类：view、bridge、make/check、observer、relation。
- [x] 确认查询类函数的当前 public 入口、proof 入口、旧 contract 入口分别位于哪些文件。
- [x] 为每个阶段准备单独提交，避免跨阶段大杂烩。

---

## 1. 查询函数

目标：

- public post 直接对齐抽象 spec；
- 删除查询类 `exec_contract`；
- `interface.rs` 不再为这些函数提供重复 proof shell。

### 1.1 `same_region_as`

- [x] 确认 public 入口只保留在 `capability/mod.rs`。
- [x] 确认 verify 模式下的最终 post 直接结束于 `spec_same_region_as_caps(...)`。
- [x] 保留 `bridge_cap` 与必要 shape lemma。
- [x] 删除与该函数相关的旧 contract 名称与重复包装。
- [x] 阶段级验证已统一尝试，当前阻塞见 `1.4`。

### 1.2 `same_object_as`

- [x] public post 直接结束于 `spec_same_object_as_caps(...)`。
- [x] 统一处理 null / untyped / irq-control / zombie / arch 直接为 false 的分支。
- [x] 保留 CNode / Thread / IRQHandler 等局部 shape lemma。
- [x] 删除对应的旧 contract 和对外 second wrapper。
- [x] 阶段级验证已统一尝试，当前阻塞见 `1.4`。

### 1.3 `is_cap_revocable`

- [x] public post 直接结束于 `spec_is_cap_revocable(...)`。
- [x] 逐分支对齐 endpoint / notification / irq_handler / untyped。
- [x] 删除 deterministic 类旧 lemma 与中间 contract。
- [x] 阶段级验证已统一尝试，当前阻塞见 `1.4`。

### 1.4 查询函数收尾

- [x] 从 `interface.rs` 中移除与上述三个函数相关的重复包装。
- [x] 检查 `cte.rs` 是否仍残留面向外部的查询类 `exec_contract`。
- [x] 将 verify 模式下 `cte.rs` 内部查询入口收为 `pub(crate)`，避免形成第二套 public proof surface。
- [x] 更新文档中的阶段状态。

注：

- 已尝试运行 `env TARGET=riscv64gc-unknown-none-elf PLATFORM=spike MARCOS='' cargo check -p sel4_cspace --features verify --offline`。
- 当前 `sel4_cspace` crate 的 verify `cargo check` 已通过；第 1 阶段查询函数已不再构成集中验证阻塞。

---

## 2. 派生查询函数

目标：

- 直接结束于最终抽象语义；
- 不再通过 `exec_contract` 作为主要终点；
- 尽量不依赖 `interface.rs` 的二次包装层。

### 2.1 `is_mdb_parent_of`

- [x] public post 改为 `ret == state.mdb_parent_of(parent, child)` 或等价最终谓词。
- [x] raw parent/child 到抽象 slot 的对应证明内聚到 `cte.rs`。
- [x] `same_region_as` 作为子证明只通过最终语义调用。
- [x] badge compatibility 与抽象谓词对齐。
- [x] 删除 `is_mdb_parent_of_exec_contract` 的 public-facing 用途。

### 2.2 `is_final_cap`

- [x] public post 改为 `ret == state.is_final_cap(slot)`。
- [x] 前驱、后继、有无邻居三类情况分别验证。
- [x] `same_object_as` 只通过最终语义调用。
- [x] 删除 `is_final_cap_exec_contract` 的 public-facing 用途。

### 2.3 `ensure_no_children`

- [x] public post 直接表达异常返回与抽象阻塞条件的等价。
- [x] next 指针存在与否的分支证明收束到 `state.ensure_no_children_blocks(slot)`。
- [x] 删除 `ensure_no_children_exec_contract` 的主要用途。

### 2.4 `is_long_running_delete`

- [x] public post 改为 `ret == state.slot_cap_long_running_delete(slot)`。
- [x] `is_final_cap` 作为子证明只通过最终语义调用。
- [x] 删除 `is_long_running_delete_exec_contract` 的 public-facing 用途。

### 2.5 `derive_cap`

- [x] public post 直接表达 expected cap / syscall error / none 三项最终语义。
- [x] untyped 分支通过 `ensure_no_children` 的最终语义接到 `spec_derive_cap_expected_cap`。
- [x] 删除 `derive_cap_exec_contract` 的 public-facing 用途。
- [x] 审查返回值解释 helper 是否可以部分从 `interface.rs` 下沉或删除。

### 2.6 派生查询收尾

- [x] 删除与上述函数对应的 `*_at_pre` / `*_at` 重复包装。
- [x] 检查 `interface.rs` 是否还只是转述同义 post。
- [x] 删除 `cte.rs` 内部仅作语义转述的 `*_exec_step` 中转层，`*_refined` 直接结束到最终语义证明。
- [x] 将 verify 模式下 `derive/is_final/ensure_no_children/is_long_running_delete` 的 `cte_t` 方法收为 `pub(crate)`。
- [x] 已尝试一轮集中验证；当前 `sel4_cspace` crate 的 verify `cargo check` 已通过。

---

## 3. Lookup

### 3.1 `resolve_address_bits`

- [x] 保留返回值 bridge，但核心 post 只保留“返回 core 等于 abstract expected core”。
- [x] `status / slot / bits_remaining` accessor 不再保留 public 同义层，统一直接通过 `ret.view()` 读取。
- [x] 删除不必要的 `interface.rs` 返回值展开包装与 `resolve_address_bits_at` 同义中转层；public pre 命名同步收敛为 `resolve_address_bits_pre`。
- [x] 删除 `cte.rs` 内部仅作转发的 `resolve_address_bits_exec_step`，`resolve_address_bits_refined` 直接结束到 expected core。
- [x] 已尝试运行 `env TARGET=riscv64gc-unknown-none-elf PLATFORM=spike MARCOS='' cargo check -p sel4_cspace --features verify --offline`；当前 `sel4_cspace` crate 的 verify `cargo check` 已通过，lookup 壳层整理未引入新的验证阻塞。

---

## 4. Mutator

目标：

- public verification entry 每个操作最多一层；
- observer 只证明局部 heap 变化；
- public post 直接面向 `spec_*_post` 和 expected-entry post。

### 4.1 `cte_insert`

- [x] 合并 `cte_insert_at` / `cte_insert` 的重复结构，verify 公共层只保留 `cte_insert`。
- [x] public post 保留：heap matches state、`spec_cte_insert_post`、必要 expected entry。
- [x] local heap observer 保留在证明内部，不再暴露为旧 contract 语言。
- [x] 跑验证并记录依赖变动。

### 4.2 `insert_new_cap`

- [x] 合并 `insert_new_cap_at` / `insert_new_cap` 的重复结构。
- [x] public post 直接对齐 `spec_insert_new_cap_post`。
- [x] 仅保留必要的 expected parent/slot entry post。
- [x] 跑验证并记录依赖变动。

### 4.3 `cte_move`

- [x] 合并 `cte_move_at` / `cte_move` 的重复结构。
- [x] public post 直接对齐 `spec_cte_move_post`。
- [x] expected src empty / expected dest filled 分开验收。
- [x] 跑验证并记录依赖变动。

### 4.4 `cte_swap`

- [x] 合并 `cte_swap_at` / `cte_swap` 的重复结构。
- [x] public post 直接对齐 `spec_cte_swap_post`。
- [x] expected slot1/slot2 entry post 保留为必要补充。
- [x] 跑验证并记录依赖变动。

### 4.5 Mutator 收尾

- [x] `interface.rs` 中四组 mutator 的 `_at` 同义 public wrapper 已删除。
- [x] 检查是否还存在“仅为了 public wrapper 存在”的 local transition lemma 使用点。
- [x] 已尝试一轮集中验证；当前 `sel4_cspace` crate 的 verify `cargo check` 已通过，后续重点转为 warning 清理与 bridge/interface 继续瘦身。

---

## 5. `interface.rs` 清理

目标：

- 从 proof-surface 语言层收缩成薄公共层或 re-export 层。

### 5.1 删除重复 pre wrapper

- [x] 查询 / 派生查询范围内无需存在的 `*_at_pre` 重复包装已删除；mutator / lookup 相关 pre wrapper 也已从 `interface.rs` 删除并内联回各 public wrapper 的 `requires`。
- [x] 将确需复用的前置条件回收到 `cte.rs` 本地 helper。

### 5.2 删除重复返回值解释层

- [x] `verify_view_cap` 已内联，直接使用 `crate::repr::cap_repr::cap_view(...)`。
- [x] `verify_heap_matches_state_at` 已内联，直接使用 `trusted_cspace_heap_matches_state_at(...)`。
- [x] `derive_cap_ret_*` 与 `exception_status_*` 的查询 / 派生查询同义解释层已删除。
- [x] `resolve_address_bits_ret_*` 同义解释层已删除，lookup 公共 post 只保留 `ret.view() == expected_core`。
- [x] 能直接使用 bridge/repr 的地方直接使用，不保留同义命名层。

### 5.3 收尾

- [x] 判断 `interface.rs` 最终是薄 wrapper 还是纯 re-export。
- [x] mutator public entry 目前每个操作只剩一层；其余是否继续瘦身留待后续阶段判断。

注：

- 当前结论：`interface.rs` 保持为“薄 wrapper + 少量稳定 proof surface”，而不是纯 re-export。

---

## 6. `refinement_bridge.rs` 收口

目标：

- 只保留表示边界；
- 不再承担核心语义模块职责；
- 完成 TCB 命名消毒。

### 6.1 保留并整理 view / bridge / make / check

- [x] 统一 `view_*` / `*_view` 命名。
- [x] 保留 `bridge_*` 命名给 snapshot/wrapper。
- [x] 统一 `make_*` / `mk_*` 命名给 constructor witness。
- [x] 统一 `check_*` / `*_is_*` 命名给状态判断 helper。

### 6.2 删除或迁出不该留在 bridge 的内容

- [x] 删除仅为 `interface.rs` 服务的同义壳。
- [x] 将可在 `cte.rs` 本地消化的辅助证明移出 bridge。
- [x] 确认 bridge 中不再承载核心 CSpace 操作语义。

注：

- 已将四个只被 `interface.rs` 使用的 local-transition-to-expected-view lemma 从 `refinement_bridge.rs` 迁回 `interface.rs`：
  `lemma_cte_insert_local_heap_transition_post_implies_expected_src_dest_views`
  `lemma_insert_new_cap_local_heap_transition_post_implies_expected_parent_slot_views`
  `lemma_cte_move_local_heap_transition_post_implies_expected_src_dest_views`
  `lemma_cte_swap_local_heap_transition_post_implies_expected_slot_views`
- 已将两个只被 `cte.rs` 使用的 raw-slot-view 对齐 helper 从 `refinement_bridge.rs` 迁回 `cte.rs`：
  `lemma_cte_insert_call_pre_at_implies_raw_slot_views_match_state`
  `lemma_insert_new_cap_call_pre_at_implies_raw_slot_views_match_state`
- 已从 `refinement_bridge.rs` 删除一批无调用点的本地脚手架 proof helper：
  `lemma_cte_insert_bridge_pre_at_implies_src_dest_refine`
  `lemma_cte_insert_bridge_pre_at_implies_old_next_refine`
  `lemma_cte_move_bridge_pre_at_implies_src_dest_refine`
  `lemma_cte_move_bridge_pre_at_implies_neighbors_refine`
  `lemma_cte_swap_bridge_pre_at_implies_core_slots_refine`
  `lemma_cte_swap_bridge_pre_at_implies_neighbors_refine`
  `lemma_insert_new_cap_local_heap_transition_post_implies_expected_old_next_view`
  `lemma_cte_insert_local_heap_transition_post_implies_expected_old_next_view`
- 已删除四个只为 `*_call_pre_at` 再包一层的 bridge pre shell，直接把最终前置条件内联进：
  `cte_insert_call_pre_at`
  `insert_new_cap_call_pre_at`
  `cte_move_call_pre_at`
  `cte_swap_call_pre_at`
- 已将 `resolve_address_bits` 仍在使用的 state-only observer 词汇收缩为基于 `trusted_runtime_heap()` 的薄包装层，不再维护与 `_at` observer 平行的第二套独立语义：
  `trusted_concrete_slot_view`
  `trusted_concrete_cnode_lookup_slot`
  `trusted_cspace_slot_views_match_state`
  `trusted_cspace_cnode_lookups_match_state`
  `trusted_cspace_heap_matches_state`
  以及三条对应 state-only lemma 现已直接委托到 `_at` 版本。
- 在此基础上，又将两个只被 `resolve_address_bits` concrete proof 使用的 state-only lemma 下沉到 `cte.rs`：
  `lemma_trusted_cspace_slot_views_match_state_implies_slot_refines`
  `lemma_trusted_cspace_cnode_lookups_match_state_implies_cap_lookup_entry`
  同时删除 bridge 中无调用点的
  `lemma_trusted_cspace_cnode_lookups_match_state_implies_lookup_entry`。
- 已将一个只被 `interface.rs` 四个 mutator expected-view helper 使用的 local-transition 拼接 lemma 迁回 `interface.rs`：
  `lemma_local_heap_transition_at_and_slot_entry_eq_implies_concrete_slot_eq`
- 已将一批只在 `refinement_bridge.rs` 内部互相调用、外部无直接依赖的 local-observer / transition proof helper 收为 bridge 私有项，不再作为对外 proof surface 暴露：
  `lemma_trusted_cspace_selected_slot_views_match_state_at_implies_slot_refines`
  `lemma_trusted_cspace_cnode_lookups_match_state_at_implies_lookup_entry`
  `lemma_trusted_cspace_slots_unchanged_except_at_implies_slot_view_eq`
  `lemma_trusted_cspace_cnode_lookups_unchanged_at_implies_lookup_entry_eq`
  `lemma_trusted_cspace_local_heap_transition_at_implies_untouched_slot_refines_old_state`
  `lemma_trusted_cspace_local_heap_transition_at_implies_post_slot_views_match_state_at`
  `lemma_trusted_cspace_local_heap_transition_at_implies_post_cnode_lookups_match_state_at`
  `lemma_trusted_cspace_slots_unchanged_except_at_transitive`
  `lemma_trusted_cspace_cnode_lookups_unchanged_at_transitive`
- `resolve_address_bits` 一侧又进一步收缩了一批仅在 bridge 内部用于结果对齐的 helper，可见性已收为 bridge 私有：
  `lemma_resolve_address_bits_expected_core_refines_state`
  `lemma_resolve_address_bits_result_refines_state`
  `lemma_resolve_address_bits_success_result_refines_state`
  `lemma_resolve_address_bits_fault_result_refines_state`
- 已将 `resolve_address_bits` 的操作语义 helper 从 bridge 迁回 `cte.rs`：
  `resolve_address_bits_pre`
  `resolve_address_bits_expected_core`
  `resolve_address_bits_core_refines_state`
  `resolve_address_bits_one_step_refines_state`
  以及对应的 `lemma_resolve_address_bits_*_state*` 证明链。
- 已将按操作命名、只服务 `cte.rs` / `interface.rs` 的 wrapper spec 从 bridge 迁回 `cte.rs`：
  `is_final_cap_call_pre_at`
  `derive_cap_call_pre_at`
  `is_mdb_parent_of_call_pre_at`
  `cte_insert_call_pre_at`
  `cte_insert_local_heap_transition_at`
  `insert_new_cap_call_pre_at`
  `insert_new_cap_local_heap_transition_at`
  `cte_move_call_pre_at`
  `cte_move_local_heap_transition_at`
  `cte_swap_call_pre_at`
  `cte_swap_local_heap_transition_at`
- 已将 single-heap state observer wrapper 一并迁回 `cte.rs`，bridge 仅保留 `_at` observer 族与 generic local transition vocabulary：
  `trusted_runtime_heap`
  `trusted_concrete_slot_view`
  `trusted_concrete_cnode_lookup_slot`
  `trusted_cspace_slot_views_match_state`
  `trusted_cspace_cnode_lookups_match_state`
  `trusted_cspace_heap_matches_state`
- 当前 bridge 中保留的 helper 已主要是 concrete observer / local heap transition 通用边界，而非按具体 CSpace 操作组织的语义层。

### 6.3 命名消毒

- [x] 不采用“全量 `trusted_* -> axiom_*`”。
- [x] 不采用“全量 `trusted_* -> assume_*`”。
- [x] 仅把真正逻辑公理命名为 `axiom_*`。
- [x] 仅把阶段性假设命名为 `assume_*`。
- [x] 完成当前 `trusted_*` 的按职责拆分重命名。

---

## 7. 验证与回归

- [x] 每完成一个函数组就跑一次局部验证。
- [x] 每完成一个阶段就跑一次集中验证。
- [x] 记录因 wrapper 删除而需要同步调整的调用点。
- [x] 记录哪些 helper 变成了私有本地 lemma。
- [x] 记录哪些 bridge helper 被保留为长期 TCB 边界。

注：

- 最近一次集中验证命令：`env TARGET=riscv64gc-unknown-none-elf PLATFORM=spike MARCOS='' cargo check -p sel4_cspace --features verify --offline`。
- 当前状态：通过；在 resolve 语义 helper、操作级 pre/local-transition wrapper、state-only observer wrapper 回收到 `cte.rs`，以及 bridge 内部 / resolve-only proof helper 私有化之后，`sel4_cspace` verify `cargo check` 仍然无 error、无 warning。
- 当前已私有化到 `interface.rs` 的 helper：
  `lemma_cte_insert_local_heap_transition_post_implies_expected_src_dest_views`
  `lemma_insert_new_cap_local_heap_transition_post_implies_expected_parent_slot_views`
  `lemma_cte_move_local_heap_transition_post_implies_expected_src_dest_views`
  `lemma_cte_swap_local_heap_transition_post_implies_expected_slot_views`
  `lemma_local_heap_transition_at_and_slot_entry_eq_implies_concrete_slot_eq`
- 当前已私有化到 `cte.rs` 的 helper：
  `lemma_cte_insert_call_pre_at_implies_raw_slot_views_match_state`
  `lemma_insert_new_cap_call_pre_at_implies_raw_slot_views_match_state`
  `lemma_trusted_cspace_slot_views_match_state_implies_slot_refines`
  `lemma_trusted_cspace_cnode_lookups_match_state_implies_cap_lookup_entry`

---

## 8. 收尾验收

- [x] public postcondition 一眼能看出最终抽象语义。
- [x] `*_exec_contract` 不再作为主要 public 语义层存在。
- [x] `interface.rs` 明显变薄。
- [x] `refinement_bridge.rs` 只承担表示边界职责。
- [x] TCB 命名不再依赖统一的 `trusted_*` 大口袋前缀。
- [x] 查询、派生查询、lookup、mutator 四类函数的证明风格统一。
