# CSpace Verus 可信基（TCB）说明

## 总体原则

本轮在不改写生产实现的约束下，将以下部分视为可信边界（Trusted Base），在其合同前提上证明 CSpace 行为。

## TCB-1：自动生成位域代码

- 范围：`sel4_common` 中由生成工具产出的位域访问器与相关结构。
- 原因：生成代码体量大、位级操作密集，本轮不对其内部逐行证明。
- 约束：通过类型与接口合同限制其被调用方式。

## TCB-2：裸指针与底层转换工具

- 范围：指针地址到引用的转换、volatile 读取、底层内存视图映射。
- 原因：涉及体系结构与内存模型细节，不在本轮完备证明范围内。
- 约束：对调用方增加前置条件（地址有效、对齐、生命周期/别名约束）。

## TCB-3：外部 C 函数边界

- 范围：`finalise_cap`、`post_cap_deletion`、`preemption_point` 等 FFI 边界。
- 原因：外部实现不可在当前 Rust/Verus 单元内直接展开证明。
- 约束：以 contracts 形式明确语义假设并纳入台账；本轮不展开其内部实现证明。

## 阶段 A：可信边界合同映射（初稿）

本节给出“边界函数 -> 调用点 -> 约束合同”的最小可审计映射，作为后续 Verus 合同化的落地基线。

### 映射 1：位域与 VM 权限收敛

- 边界函数：
   - `sel4_common/src/arch/riscv64/vm_rights.rs` 中 `maskVMRights`
   - `sel4_common/src/arch/aarch64/vm_rights.rs` 中 `maskVMRights`
- CSpace 调用点：
   - `sel4_cspace/src/arch/riscv64/mod.rs` 中 `arch_mask_cap_rights`
   - `sel4_cspace/src/arch/aarch64/mod.rs` 中 `arch_mask_cap_rights`
- requires（最小）：
   - 输入 `vm_rights` 为合法枚举值。
   - `rights` 来自当前 syscall 路径的读写授权位。
- ensures（最小）：
   - 返回权限不会比输入权限更强（只会保持或收窄）。
   - 非 frame cap 路径不改动原 capability。

### 映射 2：指针转换与可变引用构造

- 边界函数：
   - `sel4_common/src/utils/mod.rs` 中 `convert_to_mut_type_ref`
   - `sel4_common/src/utils/mod.rs` 中 `convert_to_option_mut_type_ref`
- CSpace 调用点：
   - `sel4_cspace/src/cte.rs` 中 `set_empty`、`reduce_zombie`、`revoke`、`cte_insert` 等路径
   - `sel4_cspace/src/lib.rs` 中 `resolve_address_bits_test` 测试辅助路径
- requires（最小）：
   - 地址非空（`convert_to_mut_type_ref` 已包含非零断言）。
   - 地址对齐到目标类型，且生命周期内指向有效对象。
   - 调用方满足“无冲突可变别名”约束。
- ensures（最小）：
   - 返回引用指向输入地址对应对象。
   - `convert_to_option_mut_type_ref(0)` 返回 `None`；非 0 返回 `Some`。

### 映射 3：FFI CSpace 生命周期钩子

- 边界声明：
   - `sel4_cspace/src/deps.rs` 中 `finalise_cap`、`post_cap_deletion`、`preemption_point`
- Rust 侧实现位置：
   - `kernel/src/interfaces_impl/cspace.rs` 中同名导出函数
- CSpace 调用点：
   - `sel4_cspace/src/cte.rs` 中 `finalise`、`delete_one`、`set_empty`、`revoke`
- requires（最小）：
   - `finalise_cap` 输入 capability 与 `_final/_exposed` 标志与调用现场一致。
   - `post_cap_deletion` 的 `cleanupInfo` 必须来自对应 `finalise_cap` 输出。
   - `preemption_point` 在可中断循环中调用，调用方必须处理非 NONE 返回。
- ensures（最小）：
   - `finalise_cap` 返回 `remainder/cleanupInfo` 语义与删除流程一致。
   - `post_cap_deletion` 仅执行删除后副作用（例如 IRQ handler 清理）。
   - `preemption_point` 只返回 `EXCEPTION_NONE` 或可恢复的抢占异常码。

### 假设化优先级（阶段 A 落地顺序）

1. 先给 `preemption_point` 写返回值约束假设并登记到台账。
2. 再给 `finalise_cap/post_cap_deletion` 写配对假设（输入输出关联）并登记台账。
3. 最后给指针转换与 `maskVMRights` 写“只收窄权限 + 地址有效性”假设并登记台账。

## 阶段 A.1：sel4_common 边界假设清单（模板 + 首版）

本节将“可信边界假设映射”固化为可维护台账，目标是让每个边界都具备可追踪的
假设/规格锚点、风险等级与去 TCB 条件。

### 命名约定（参考 vostd）

- `assume_` 前缀：用于可信边界假设，表示当前阶段接受但未在本轮内部展开证明的语义。
- `axiom_` 前缀：仅用于数学或模型公理，默认不用于 FFI/运行时行为边界。
- `lemma_` 前缀：用于可证明的辅助引理（`proof fn`）。
- `spec_` 前缀：用于目标原语的行为规格骨架，后续由引理与实现证明逐步收敛。

### 字段模板

- ID：稳定编号，后续评审与变更记录统一引用。
- 边界符号：被视为 TCB 的函数或外部边界。
- CSpace 调用点：当前 crate 内的调用位置。
- specs 假设/规格锚点：`sel4_cspace/specs/lib.rs` 内对应谓词名。
- 状态：`draft`（仅识别）/`modeled`（已建模）/`wired`（已接入调用校验）/`discharged`（已移出 TCB）。
- 最小 requires：调用方必须满足的最低前置条件。
- 最小 ensures：可依赖的最低后置语义。
- 风险等级：`H`（高）/`M`（中）/`L`（低）。
- 去 TCB 条件：将该边界从可信假设收缩为可证明对象的触发条件。

### 首版条目（2026-04-18）

| ID | 边界符号 | CSpace 调用点 | specs 假设/规格锚点 | 状态 | 最小 requires | 最小 ensures | 风险 | 去 TCB 条件 |
|---|---|---|---|---|---|---|---|---|
| SC-BC-01 | `maskVMRights`（riscv64/aarch64） | `sel4_cspace/src/arch/*/mod.rs::arch_mask_cap_rights` | `boundary_assumptions::assume_mask_vm_rights_non_escalation` | modeled | 输入 capability 权限与 syscall 请求权限合法 | 输出权限不放大，仅保持或收窄 | M | 以位域语义引理替代黑盒假设，覆盖 read/write 收敛证明 |
| SC-BC-02 | `convert_to_mut_type_ref` | `sel4_cspace/src/cte.rs`（`set_empty/reduce_zombie/revoke/cte_insert` 等） | `boundary_assumptions::assume_convert_to_mut_type_ref_contract` | modeled | 地址非 0、对齐正确、对象有效、无冲突可变别名 | 返回可变引用绑定到输入地址对象 | H | 将关键路径替换为可验证抽象内存模型或局部安全包装层 |
| SC-BC-03 | `convert_to_option_mut_type_ref` | `sel4_cspace/src/cte.rs` 与测试辅助路径 | `boundary_assumptions::assume_convert_to_option_mut_type_ref_contract` | modeled | 非 0 地址满足 SC-BC-02 要求；0 地址允许空指针语义 | 0 地址返回 None，非 0 返回 Some | M | 对 Option 分支建立端到端不变量，减少 raw pointer 暴露 |
| SC-BC-04 | `finalise_cap`（FFI） | `sel4_cspace/src/cte.rs::finalise/delete_one/revoke` | `boundary_assumptions::assume_finalise_cap_pairing` | modeled | capability 输入与 `_final/_exposed` 标志与调用现场一致 | `remainder/cleanupInfo` 配对且满足删除流程语义 | H | 为导出实现建立可验证行为规范并完成调用一致性证明 |
| SC-BC-05 | `post_cap_deletion`（FFI） | `sel4_cspace/src/cte.rs::set_empty/delete_one/revoke` | `boundary_assumptions::assume_post_cap_deletion_origin` | modeled | `cleanupInfo` 必须来自对应 `finalise_cap` 输出 | 仅执行删除后副作用，不引入额外 capability 状态变化 | M | 建立 `finalise_cap -> post_cap_deletion` 链路保持性质 |
| SC-BC-06 | `preemption_point`（FFI） | `sel4_cspace/src/cte.rs::revoke/finalise` 循环路径 | `boundary_assumptions::assume_preemption_point_progress` | modeled | 调用点处于可抢占循环，调用方处理非 NONE 返回 | 仅返回继续执行或可恢复抢占路径 | H | 在循环不变量中消解返回分支并完成异常传播证明 |
| SC-BC-07 | `trusted_extract_cap`（Stage 5 bridge） | `sel4_cspace/src/refinement_bridge.rs::bridge_cap` | `refinement_bridge::trusted_view_cap` | modeled | 输入 `cap` 为 live raw capability，getter 读取与当前 bitfield 布局一致 | 返回 snapshot 满足 `cap_snapshot_wf`，且 `view_cap(snapshot)` 与原始 concrete cap 的抽象语义一致 | H | 用更细粒度的 bitfield 语义引理替代 extractor 黑盒确保 |
| SC-BC-08 | `trusted_extract_cte`（Stage 5 bridge） | `sel4_cspace/src/refinement_bridge.rs::bridge_cte` | `refinement_bridge::trusted_view_cte` | modeled | 输入 `cte_t` 为 live slot，内部 capability/MDB getter 与当前布局一致 | 返回 snapshot 满足 `cte_snapshot_wf`，且 `view_cte(snapshot)` 与原始 concrete slot 的抽象语义一致 | H | 将 slot 局部布局读取拆成可验证 view lemma，缩小 trusted surface |
| SC-BC-09 | `trusted_extract_resolve_address_bits_ret`（Stage 5 bridge） | `sel4_cspace/src/refinement_bridge.rs::bridge_resolve_address_bits_ret` | `refinement_bridge::trusted_view_resolve_address_bits_ret` | modeled | 输入返回值来自 concrete `resolve_address_bits` 调用现场，状态码/slot/bits 字段读取与当前布局一致 | 返回 snapshot 满足 `resolve_address_bits_ret_snapshot_wf`，且 core view 与 concrete 返回值一致 | M | 在函数级 refinement 中将返回值 core relation 进一步替换为可证明的字段级 bridge |
| SC-BC-10 | `trusted_concrete_slot_view` / `trusted_concrete_slot_view_at`（Stage 5 heap bridge） | `sel4_cspace/src/refinement_bridge.rs::resolve_address_bits_bridge_pre` 与 `cte_*_bridge_pre_at` / `cte_*_local_heap_transition_at` | `refinement_bridge::trusted_cspace_slot_views_match_state` / `refinement_bridge::trusted_cspace_slot_views_match_state_at` / `refinement_bridge::trusted_cspace_local_heap_transition_at` | modeled | 输入 slot id 指向 live concrete `cte_t`，且该 slot 在当前 concrete heap（read-only 或 heap-indexed before/after）中有稳定局部视图 | 返回的 `SlotEntrySpec` 与抽象 `state.slot_entry(slot)` 一致；对 mutating ops，unchanged-slot frame 通过 `_at` 版本表达 | H | 用地址到局部 slot view 的可证明 bridge 替代这一地址级黑盒视图 |
| SC-BC-11 | `trusted_concrete_cnode_lookup_slot` / `trusted_concrete_cnode_lookup_slot_at`（Stage 5 heap bridge） | `sel4_cspace/src/refinement_bridge.rs::resolve_address_bits_bridge_pre` 与 `cte_*_local_heap_transition_at` | `refinement_bridge::trusted_cspace_cnode_lookups_match_state` / `refinement_bridge::trusted_cspace_cnode_lookups_match_state_at` / `refinement_bridge::trusted_cspace_cnode_lookups_unchanged_at` | modeled | 输入 CNode object / offset 对应 concrete CNode 中的有效 slot lookup | 返回 slot id 与抽象 `state.cnode_lookup[obj][offset]` 一致；对 mutating ops，lookup frame 通过 `_at` 版本表达 | H | 用可证明的 CNode pointer arithmetic / slot addressing 语义引理替代这一 lookup 黑盒 |
| SC-BC-12 | `trusted_call_resolve_address_bits`（Stage 6 入口） | `sel4_cspace/src/refinement_bridge.rs::resolve_address_bits_refined` | `refinement_bridge::resolve_address_bits_one_step_refines_state` | modeled | 调用前满足 `resolve_address_bits_bridge_pre`（现已包含 `valid_cap(trusted_view_cap(raw_root))`） | concrete 返回值的 core view 满足一步 branch / recursive relation；整体 refinement 由 proof-side lemma 提升 | H | 把 trusted 调用包装继续下推为更局部的 raw getter / slot-step relation，最终消解调用级黑盒 |

### 台账维护规则

1. 新增可信边界时，必须先新增一条台账记录，再允许进入实现分支。
2. 任何 `assume` 或 `external_body` 进入验证主路径时，必须映射到一个台账 ID。
3. 每次回归至少检查一次：台账总数是否增加、`H` 级条目是否减少、是否新增无锚点假设。
4. 当条目标记为 `discharged`，需在变更说明里给出“替代证明位置 + 回归命令结果”。

## TCB 风险与缓解

1. 风险：合同过弱导致证明价值降低。
   - 缓解：优先收紧 contracts，避免“真但无用”的宽松假设。
2. 风险：边界语义漂移。
   - 缓解：将 contracts 与接口实现变更联动审查，纳入 CI 检查项。
3. 风险：验证结果被误解为“全栈内存安全证明”。
   - 缓解：在报告中显式标注 TCB 与非覆盖范围。

## 阶段 6 收口补充（2026-04-24）

- 当前回归结果：
  - `cargo xtask verify`
  - `136 verified, 0 errors`
- 当前 bridge-local trusted surface 已收口为三组：

### 1. 视图提取与 heap bridge

- `trusted_extract_cap`
- `trusted_extract_cte`
- `trusted_extract_resolve_address_bits_ret`
- `trusted_concrete_slot_view_at`
- `trusted_concrete_cnode_lookup_slot_at`

这组边界仍是当前 TCB 的主体，因为它们负责把 raw bitfield / pointer 形状接到抽象 `CSpaceState`。

### 2. object-local 构造器与观察器

- 异常/返回值构造：
  - `trusted_make_exception_none`
  - `trusted_make_exception_syscall_error`
  - `trusted_check_exception_is_none`
  - `trusted_check_exception_is_syscall_error`
  - `trusted_make_null_cap`
  - `trusted_clone_cap`
  - `trusted_make_derive_cap_ret`
- capability/slot 局部观察器：
  - `trusted_cap_is_zombie`
  - `trusted_cap_is_untyped`
  - `trusted_cap_is_reply`
  - `trusted_cap_is_irq_control`
  - `trusted_slot_cap_is_null`
  - `trusted_slot_cap_is_thread`
  - `trusted_slot_cap_is_zombie`
  - `trusted_slot_cap_is_cnode`
  - `trusted_has_mdb_next`
  - `trusted_follow_mdb_next`

这组 util 的保留原则是：

- 只承接 object-local concrete 判别或构造。
- 不允许承接 whole-function 级语义。
- 新增时必须能映射回“帮助把 proof 推回 `cte.rs` 本体”的明确目的。

### 3. 当前已收口但尚未 discharged 的原因

- `trusted_extract_*` / `trusted_concrete_*_at` 仍依赖 generated bitfield 与地址级 concrete 视图，不适合在本轮直接完全拆掉。
- object-local util 仍用于避免在 proof 中直接展开 enum/bitfield 构造细节。
- FFI 与 raw pointer 边界仍不在当前 CSpace 原语闭环的内部证明范围内。

### 4. 下一轮去 TCB 优先级

1. 优先拆 `trusted_extract_*` 与 `trusted_concrete_*_at`。
2. 再继续减少 `verus_spec(...)` / `external_body` 对主函数的包裹。
3. 最后才考虑更大范围地进入 `finalise` / `revoke` / FFI 删除链路。
