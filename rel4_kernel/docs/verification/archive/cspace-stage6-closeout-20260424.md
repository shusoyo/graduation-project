# CSpace Stage 6 收口报告（2026-04-24）

## 1. 验证结论

- 本地回归命令：
  - `cargo xtask verify`
- 当前结果：
  - `136 verified, 0 errors`
- 说明：
  - 仍有少量 Rust `unexpected cfg(test)` 与 trigger 提示，但不阻塞当前 Verus 验证闭环。

## 2. 第 6 步完成判定

本轮 6 步路线中的第 6 步“收口最小 trusted surface 与最终台账”现已完成，依据如下：

- 四个核心目标原语的 refinement 入口已位于 `sel4_cspace/src/cte.rs`：
  - `resolve_address_bits_refined(...)`
  - `cte_insert_refined(...)`
  - `cte_move_refined(...)`
  - `cte_swap_refined(...)`
- 与它们配套的只读/局部语义入口也已在 `cte.rs` 收口：
  - `is_final_cap_refined(...)`
  - `is_long_running_delete_refined(...)`
  - `ensure_no_children_refined(...)`
  - `ensure_no_children_via_is_mdb_parent_of_refined(...)`
  - `derive_cap_refined(...)`
  - `derive_cap_via_ensure_no_children_refined(...)`
  - `is_mdb_parent_of_refined(...)`
  - `is_long_running_delete_via_is_final_cap_refined(...)`
- `refinement_bridge.rs` 当前保留的 trusted 词汇已压缩到“抽象视图桥 + object-local util”两类，不再承担长期语义归宿。
- 最小 TCB 与剩余 trusted util 已能逐项列举，不再依赖模糊的“大 bridge 黑盒”描述。

## 3. 当前已证 exec 入口

当前已经进入 `cte.rs` 本体侧、并有稳定 exec 合同的入口包括：

- 目标原语：
  - `resolve_address_bits`
  - `cte_insert`
  - `cte_move`
  - `cte_swap`
- 配套只读/派生语义：
  - `is_final_cap`
  - `is_long_running_delete`
  - `ensure_no_children`
  - `derive_cap`
  - `is_mdb_parent_of`

说明：

- 这些主函数当前仍处于“Rust exec body + `external_body` / `verus_spec(...)` 过渡合同”的阶段。
- 这不再阻塞本轮 6 步计划的完成；它属于下一轮“signature-first / 原生 Verus exec 化”的继续收缩目标。

## 4. 当前仍保留的最小 trusted surface

### A. 运行时与 FFI 边界

- `maskVMRights`
- `convert_to_mut_type_ref`
- `convert_to_option_mut_type_ref`
- `finalise_cap`
- `post_cap_deletion`
- `preemption_point`

### B. concrete -> abstract 视图桥

- `trusted_extract_cap`
- `trusted_extract_cte`
- `trusted_extract_resolve_address_bits_ret`
- `trusted_concrete_slot_view_at`
- `trusted_concrete_cnode_lookup_slot_at`

### C. object-local 小型 util

- 异常/返回值构造：
  - `trusted_make_exception_none`
  - `trusted_make_exception_syscall_error`
  - `trusted_check_exception_is_none`
  - `trusted_check_exception_is_syscall_error`
  - `trusted_make_null_cap`
  - `trusted_clone_cap`
  - `trusted_make_derive_cap_ret`
- capability 标签观察器：
  - `trusted_cap_is_zombie`
  - `trusted_cap_is_untyped`
  - `trusted_cap_is_reply`
  - `trusted_cap_is_irq_control`
- slot 局部观察器：
  - `trusted_slot_cap_is_null`
  - `trusted_slot_cap_is_thread`
  - `trusted_slot_cap_is_zombie`
  - `trusted_slot_cap_is_cnode`
  - `trusted_has_mdb_next`
  - `trusted_follow_mdb_next`

当前保留这些 util 的原则是：

- 只允许承接 bitfield / raw pointer / enum 构造等 concrete 细节。
- 不允许重新长成“整函数语义黑盒”。
- 新增 util 必须直接服务于把 proof 继续压回 `cte.rs` 本体，而不是扩张 bridge 层。

## 5. 本轮收口后的语义对齐

- `ensure_no_children` 现按 l4v 的 `isMDBParentOf` 语义收紧，不再把“是否阻塞删除”混同为泛化 derivation 关系。
- `is_long_running_delete` 通过 `is_final_cap` 与 slot-kind 观察器收口为 object-local 组合 proof。
- `derive_cap` 的 untyped 分支已显式走 `ensure_no_children` 入口，而不是留在外层大 wrapper 黑盒中。
- 异常构造器现在带有互斥合同，避免 proof 依赖隐式“枚举值互斥”猜测。

## 6. 超出本轮 6 步的后续工作

如果继续推进到“用 Verus 风格主函数逐步替换现有 Rust 合同壳”的目标，下一轮建议是：

1. 继续减少 `#[verus_spec(...)]` 的覆盖面，把主函数合同往签名式 `requires/ensures` 迁移。
2. 把 `trusted_extract_*` 与 heap-view 黑盒继续拆成更细的字段级 bridge lemma。
3. 把 `finalise` / `revoke` / `delete` 路径纳入下一批本体 proof，而不是只停在当前 cspace 原语闭环。

结论：

- 当前 6 步计划已闭环完成。
- 下一阶段不再叫“补完第 6 步”，而是进入新一轮的“去 wrapper / 去 attribute / 去 trusted util”工作。
