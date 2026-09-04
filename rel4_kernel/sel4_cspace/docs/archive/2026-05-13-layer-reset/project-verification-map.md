# `sel4_cspace` Project Verification Map

本文档总结 `sel4_cspace` 当前各条主线的成熟度、优先级和论文表述边界。

## Methodology Baseline

当前项目默认采用的路线是：

- Verus 设计与证明组织优先参考 `atmo`
- `l4v` 主要用来校准不变量、`requires/ensures` 强度、和语义一致性
- 当前阶段不把显式 `l4v`-style refinement tower 作为 blocker

因此，下面的“完成度”判断，默认是按这条路线来评估的，而不是按“是否已经复刻出 `l4v` 的外形”来评估。

## Current State

### `mdb_patch` proof-library refactor

当前状态：

- `mdb_patch` 作为 proof-architecture 方向成立
- narrative-refactor scope 已完成
- replacement refactor 尚未完成

判断依据：

- `spec_util/mdb_patch.rs` 已经提供共享 patch vocabulary：`patch_frame(...)`、`patch_structural_frame(...)`、`patch_non_mdb_frame(...)`、`patch_derivation_obligations(...)`、`slot_semantic_edge_ok(...)`、`changed_slots_local_structural_ok(...)`、`changed_slots_semantic_edge_ok(...)`、`changed_slots_zombie_sound_ok(...)`
- `move`、`insert`、`swap`、`delete/set_empty` 都已经至少部分接入 shared `mdb_patch` closeout：operation exact/local post -> patch obligations -> `lemma_patch_recovers_wf_from_obligations(...)` -> `wf()`
- 这说明 proof organization 的方向已经从 “每个 operation 自己打仗” 转向 “local patch + shared recovery”
- 但当前 diff 仍然是明显净增长，说明 shared route 还没有替代足够多旧的 operation-specific proof；当前工作树在 `sel4_cspace/src/cspace_manager` 上仍是 `4588 insertions, 411 deletions`
- 最大增长点集中在 `impl_swap.rs`、`spec_util/insert.rs`、`spec_util/delete.rs`、`spec_util/move.rs`，这表明当前状态仍是 “new route + old scaffolding partly coexisting”

保留问题：

- 当前真正的问题不是 `mdb_patch` 方向本身，而是 replacement 没做完：新 closeout route 已接入，但旧 component-wise closeout wrapper、compatibility wrapper、以及部分 runtime-to-semantic bridge proof 还没有系统删除、迁移或合并
- `swap` 的 runtime/semantic bridge 和 changed-slot semantic-edge proof 现在大块堆在 `impl_swap.rs` / `spec_util/swap/*`；这对 proof strength 有帮助，但它们必须被纳入 “canonical route + deletion audit” 才能算重构完成
- `insert` 的 old-next 相关 proof 虽然已经退出 broad external body，但当前阶段应把它当作显式 residual precondition，而不是继续在 replacement 阶段扩张 caller-side admissibility proof
- `delete/set_empty` 已经接入 shared route，但四条 caller-admissibility residual 和 `reduce_zombie(true)` owner-shape residual 仍然存在；这些属于 trusted-boundary shrink，不属于当前 replacement 完成条件

下一步口径：

- 先完成 replacement refactor，而不是继续扩展 residual-TCB shrink
- replacement 的完成标准不是 “shared helper 存在”，而是 “每个 operation 最终只保留一条 canonical closeout route，旧重复路线被删除或显式降级”
- 当前推荐顺序是：先做 deletion-driven audit，再以 `move` 为最小替换模板，之后做 `insert` replacement、`swap` replacement/impl cleanup，最后只清理已经接入 shared route 的 `set_empty`
- 如果某一段 proof 不能在不损失 proof strength 的前提下明显缩短，应直接记录该事实，而不是继续堆新的 abstraction

### `resolve`

当前状态：

- 基本完成

判断依据：

- 公开主线已经收成 cap-centric canonical exec
- exec 是单一路径 loop
- proof 直接挂在 loop 本体
- 抽象语义与 concrete result 已建立 refinement
- trusted boundary 主要限于 primitive read bridge

论文表述：

- 可以作为当前最完整的一条 verified line

### `insert`

当前状态：

- mostly done

判断依据：

- manager-level `wf` 收回稳定
- exec 与 `reference_0ca248f` 在 helper inline 后仍较接近
- proof 结构总体可维护

保留问题：

- public wrapper 当前刻意不作为 verified claim；`cte.rs` 只保留 runtime compatibility API 和必要表示桥
- 还不是 whole-kernel 层级的强度

### `move`

当前状态：

- mostly done

判断依据：

- manager-level proof 已经成形
- exec 形状和 old implementation 接近
- 证明复杂度可接受，明显好于早期重 trace 风格

保留问题：

- 强前提主要存在于 manager 内部 API
- public wrapper 没把 proof domain 显式化

### `swap`

当前状态：

- mostly done

判断依据：

- proof 已完成结构化重构
- `core / post / wf` 分层清晰
- exec 与 old implementation 仍能对上
- 验证已经通过

保留问题：

- 仍有高杠杆 semantic bridge
- public wrapper 仍弱于 manager-level 内部证明接口

### `delete contract`

当前状态：

- mostly done, with named residual semantic bridges

判断依据：

- `finalise_slot_contract`、`set_empty`、`delete_all`、`revoke` 相关 contract vocabulary 已经成形
- delete 依赖锥第一批 helper 已基本收成 “contracted helper / manager bridge”
- `impl_delete.rs` 已不再拥有 `external_body` semantic helper；delete/revoke 的 reusable contract packaging 主要集中到 `spec_util/delete.rs`
- `reduce_zombie` 已不再是 whole-function black box，而是 “verified outer dispatch + non-immediate verified swap path + immediate owner-shape bridge”

保留问题：

- `set_empty -> wf()` 的 hard closeout 已拆成 explicit admissibility route；主 `lemma_set_empty_exact_post_preserves_wf(...)` 现在显式消费 `set_empty_wf_recovery_pre(...)`，不再在最终 `wf` closeout 里隐式投影 admissibility；非空路径已经通过 `set_empty_patch_slots(...)` 接入 `mdb_patch` shared closeout，先证明 `patch_frame(...)` / `patch_non_mdb_frame(...)` / `changed_slots_*_ok(...)` / `patch_derivation_obligations(...)`，再调用 `lemma_patch_recovers_wf_from_obligations(...)` 拼回 `wf()`；`impl_delete.rs::set_empty` 本体也已经改成只在 caller 显式提供 `set_empty_wf_recovery_pre(...)` 时承诺 `wf`，不再内部无条件消费 residual bridge；`spec_util/delete.rs` 已不再保留 `set_empty` external body，剩余非空清空事实被移动到 `trusted/common.rs` caller-admissibility boundary；`mdb_no_two_cycle_wf()` 已纳入 manager local structural `wf`，non-root-shaped slot root exclusion 已从 `root_caps_wf` 证明，removed-slot `first_badged=false` 已从 `incoming_edge_flags_wf` 证明，head-delete patched semantic edge 已从 edge-flag admissibility 证明，空槽路径已通过 `lemma_set_empty_empty_slot_preserves_wf(...)` 退出该 residual，非空清空路径仍有 root-shaped CNode slot root exclusion、head next-node edge-flag、non-head patched semantic-edge、no-two-cycle patch admissibility 四类 projection bridge
- `reduce_zombie(true)` 仍保留一条 owner-slot trichotomy trusted caller-admissibility bridge；generic `delete_all_contract(end_slot,false)` 不再被隐藏在 specialized contract bridge 里，success witness / owner-shape projectors 已在该 owner-shape post 之上证明，且 `spec_util/delete.rs` 已不再直接承载这条 external body
- `finalise_slot` cap-only write 的 easy/frame 部分已证明；affected incoming edge / affected CDT edge / target-admissibility witness / root-slot exclusion 都已从 delete-spec semantic bridge 改成 proved composition，剩余事实归入 `finalise_cap` / caller-admissibility dependency projectors；no-two-cycle availability 已由 manager `wf` 提供，caller 侧 projectors 已验证
- `deps::{preemption_point, post_cap_deletion}` 的正式语义边界停在 manager bridge，而不是 raw extern

### `delete core`

当前状态：

- mostly done, not Paper-Max complete

说明：

- `finalise_slot` 已经是 loop-direct verified body，不再是 whole-function external
- `delete_one` / `delete_all` / `revoke` 已能建立在当前 contract closure 上，并且已有 single-point verification evidence
- 当前 frontier 主要停在 trusted caller-admissibility bridges：`set_empty` admissibility 和 `reduce_zombie(true)` zombie-end contract projection。`set_empty` helper 本体已经不再隐藏该 residual，`spec_util/delete.rs` 也不再直接承载这些 external body；`finalise_slot` cap-write target/affected-edge/root 侧已经退出 delete-specific semantic bridge，改由 verified composition 消费 `finalise_cap` / caller-admissibility dependency projectors。

### `trusted boundary shrink`

当前状态：

- 方向明确
- 执行未完成

说明：

- `trusted-boundary-plan.md` 已明确哪些是长期可接受 TCB
- 但 `capability/*`、`arch/*`、`cte.rs` 等语义层 trusted surface 还未系统收缩

## Priority Order

当前推荐优先级：

1. 先做 replacement refactor 的 `Phase 0`：deletion-driven route audit。先列 `lemma/function | file | current callers | role | decision | reason`，再改代码。
2. 以 `move` 作为最小替换模板，完成第一条真正的单线 canonical closeout route。
3. 在不新增 old-next residual proof 的前提下完成 `insert` replacement：shared route 保留，兼容性 wrapper 和重复 closeout 尽量删除或合并。
4. 完成 `swap` replacement 和 `impl_swap.rs` cleanup：runtime impl 保持 runtime-shaped，可复用 proof 留在 `spec_util/swap/*`，旧 parallel route 删除或降级。
5. 只清理已经接入 shared route 的 `set_empty` closeout，不进入四条 caller-admissibility residual 和 `reduce_zombie(true)` owner-shape residual。
6. replacement 稳定后，再回到 trusted-boundary shrink：`insert` old-next caller-side admissibility、`set_empty` 四条 residual、`reduce_zombie(true)` residual。

不建议现在把主要精力放在：

- 继续证明 residual-TCB shrink，但让 replacement 永远停在中间态
- 只为了目录美观先拆 `delete.rs`，但不删除旧 closeout 路线
- 对已收住的 `resolve` 做非 blocker 级别重写
- 把 `l4v` proof script 逐字 port 到 Verus；这里仍然是 semantic and contract calibration from `l4v`，Verus organization from `atmo`

## Practical Distinctions

项目里讨论“完成没有”时，默认分三层：

### Manager-Level

意思是：

- `CSpaceManager` 内部主操作有清晰 contract
- runtime 主体与 spec/refinement 对齐
- 最终能收回 `wf`

### Public Wrapper Level

意思是：

- `cte.rs` 或 `kernel_api.rs` 对外接口也具有与 manager-level 对齐的前提/后置
- 不再只是 runtime compatibility shell

当前项目尚未声称达到这一层。当前 `cte.rs` public wrappers 只保持原 API 与单一路径转发；manager-level `requires/ensures` 不会在 kernel 侧没有 proof state 的情况下批量抬到 public API 上。

### Whole-Kernel Or l4v-Level

意思是：

- 不只是 CSpace 局部 `wf`
- 还要接到更强的全局 invariant 或 refinement story

当前项目主要完成到第一层，少量触及第二层，尚未声称第三层。

## Paper-Safe Claims

当前更安全的论文表述是：

- conditional verification of the core CSpace operation layer
- verified core for resolve, insert, move, and swap
- delete path under contract-first decomposition
- explicit trusted boundary analysis

当前不安全的论文表述是：

- fully verified CSpace
- l4v-equivalent end-to-end proof
- whole-kernel invariant preservation for all CSpace operations

## What To Finish Before Stronger Paper Claims

如果想把论文说法从“已经有较强 verified core”提升到“CSpace 核心操作层基本完成条件化验证”，建议分两步完成：

第一步先完成 replacement refactor：

1. 做 deletion-driven route audit，明确每个 operation 的 canonical closeout route 以及要删除的旧 wrapper。
2. 完成 `move`、`insert`、`swap`、`set_empty` 的 single-route replacement，让 shared `mdb_patch` closeout 成为唯一最终路线，而不是与旧 closeout 并存。
3. 对当前 full-package verification evidence 做一次可复现的 checkpoint，并清理生成物。
4. 在最终报告里明确记录 replacement 后的行数结果；如果代码量没有下降，也要直接写明。

第二步再增强 trusted-boundary claim：

1. 证明或结构化消除 `insert` old-next explicit admissibility precondition 的 caller-side 来源：`insert_new_cap` old-next、`cte_insert` old-next
2. 证明或结构化消除 `set_empty` 剩下的四条精确 admissibility residual bridge：root-shaped CNode-slot root exclusion、head next-node edge-flag clearance、non-head revocable patched semantic edge、no-two-cycle patch admissibility
3. 强化 `delete_all_contract(end_slot,false)` 的 zombie-end success witness，让它直接推出 non-null owner-slot trichotomy，从而去掉 `reduce_zombie(true)` 剩下的一条 semantic residual bridge
4. 强化 `finalise_cap_contract(...)` 或 caller admissibility，最终证明当前 dependency-level target/affected-edge/root-exclusion projectors

如果还想进一步强化 trusted boundary 叙事，再继续做：

1. `same_region_as / same_object_as` 的 refinement 化
2. `finalise_cap / preemption_point / post_cap_deletion` 的强 contract
3. 更清晰的 public wrapper 对齐
