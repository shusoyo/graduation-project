# CSpace 验证时间线日志

状态：当前日志总表，2026-05-01

## 1. 说明

这份文档按时间顺序整理 `sel4_cspace` 验证推进中的关键节点。

整理原则是：

- 保留过程，不抹掉阶段性试探和口径变化；
- 历史细节继续保存在 `docs/verification/archive/`；
- 日常查看时优先看这份总日志，不必在多个阶段文档之间来回切换。

## 2. 时间线

### 2026-05-01：`insert_new_cap_runtime_step` / `cte_insert_runtime_step` / `cte_move_runtime_step` / `cte_swap_runtime_step` 推进到 verified glue

关键结论：

- `bash tools/check-cspace-build-and-verify.sh` 再次通过；
- Verus 回归结果更新为 `249 verified, 0 errors`；
- `insert_new_cap_runtime_step(...)` 已从顶层 `#[verifier::external_body]`
  改成真实的 verified glue；
- `cte_insert_runtime_step(...)` 也已从顶层 `#[verifier::external_body]`
  改成真实的 verified glue；
- `cte_move_runtime_step(...)` 现也已从顶层 `#[verifier::external_body]`
  改成真实的 verified glue；
- `cte_swap_runtime_step(...)` 现也已从顶层 `#[verifier::external_body]`
  改成真实的 verified glue；
- 这条顶层 step 现在显式按
  `write_slot -> rewrite_old_next -> link_parent`
  三段 staged runtime-step 组合；
- `cte_insert_runtime_step(...)` 顶层则显式按
  `set_untyped -> write_dest -> link_src -> rewrite_old_next`
  四段 staged runtime-step 组合；
- `cte_move_runtime_step(...)` 顶层则显式按
  `write_dest -> clear_src -> rewrite_old_prev -> rewrite_old_next`
  四段 staged runtime-step 组合；
- `cte_swap_runtime_step(...)` 顶层则显式按
  `write_swapped_slots -> rewrite_slot1_prev -> rewrite_slot1_next -> rewrite_slot2_prev -> rewrite_slot2_next`
  五段 staged runtime-step 组合；
- `refinement_bridge.rs` 又补出一层更通用的组合支撑：
  - `trusted_slot_mdb_next_addr(...)`
  - `trusted_slot_mdb_prev_addr(...)`
  - `trusted_slot_addr(...)`
  - `lemma_trusted_heap_matches_state_and_slot_ref_is_id_implies_raw_slot_view_matches_state(...)`
  - `lemma_trusted_cspace_local_heap_transition_at_weaken(...)`
- `specs/cspace_ops/swap.rs` 现已补出 `cte_swap` 的 staged abstract-state /
  runtime changed-set / frame lemma，并新增
  `lemma_cte_swap_runtime_changed_slots_eq_spec_changed(...)`
  这类集合对齐引理，供 `cte_swap_runtime_step(...)` 顶层 glue 直接复用；
- `cte_insert_write_dest_runtime_step(...)`
  与 `cte_insert_link_src_runtime_step(...)`
  的小步合同也已同步改成可顺序组合的 local-heap-transition 形状，
  因而 `cte_insert` 这条更长的 mutator 链现在也能沿同一条组合套路收顶层 step；
- 这使得 `insert_new_cap` 成为 mutator family 里第一个
  “public verified wrapper + internal verified runtime-step glue + residual micro-step external_body helper”
  的模板项。
  现在 `cte_insert`、`cte_move` 与 `cte_swap` 也已经进入同一模板。
- 当前 `cte_swap` 这条线上仍额外保留了一个 proof-side staging scaffold：
  - `lemma_cte_swap_post_implies_staged_final_state(...)`
  它不再属于 runtime raw assumption，
  但仍是下一步可继续收紧的 spec-side proof helper。

这一天的意义是：

- mutator 侧不再只是“public wrapper 已证”，而是开始把 wrapper 下方的最外层 runtime-step 也继续吃进 Verus 组合证明；
- `insert_new_cap` 先行收口验证了这条组合路线本身可行；
- `cte_insert` 随后完成顶层 glue 化，说明这条
  `staged abstract state -> post-heap-match -> transitive transition`
  的组合套路已经不只适用于最短 mutator，而是能开始覆盖更接近真实 CSpace 主路径的入口。

### 2026-04-15：建立基线与验证冻结点

关键结论：

- 冻结了早期验证分支与基线提交；
- 记录了 `xtask build` / `xtask run` 的通过结果；
- 明确后续验证改动只在验证分支上推进；
- 固定了当时的 Verus 工具链路线。

这一天的意义是：

- 验证工作不再直接漂浮在主开发线上；
- 后续所有证明改动都有一个清楚的工程对照点。

归并来源：

- `archive/cspace-baseline-regression-20260415.md`
- `archive/cspace-step11-dv-history-20260415.md`

### 2026-04-18：阶段 A 收口，可信边界与验证入口接通

关键结论：

- `boundary_assumptions` 模块已接入验证入口；
- 可信边界台账开始稳定成文；
- `specs` 入口从单文件占位发展为模块化结构；
- 验证工作从“只有边界合同”推进到“抽象模型也开始进入主线”。

这一天的意义是：

- CSpace 验证有了最小 TCB 口径；
- `specs` 开始成为真正的验证主入口，而不是零散占位代码。

归并来源：

- `archive/cspace-stageA-progress-report-20260418.md`
- `archive/cspace-session-log-20260418.md`

### 2026-04-22：第 4 步收口，小引理与可复用入口稳定

关键结论：

- `wf` 已收紧出更稳定的入口引理；
- `resolve_address_bits` 的前置条件也收紧出 bridge 前入口；
- 规格层不再要求每个调用点手拆大 conjunction。

这一天的意义是：

- 抽象模型从“能写 spec”推进到“能复用 spec”；
- 第 5 步 bridge 不再需要建立在松散前提之上。

归并来源：

- `archive/cspace-session-log-20260422.md`

### 2026-04-23：第 5 步 bridge 设计定型

关键结论：

- 第 5 步的职责被明确限定为“表示桥”，而不是新的长期语义层；
- 风格路线固定为：
  - `l4v` 负责语义来源；
  - 当前 `spec-first` relational model 继续保留；
  - bridge 粒度优先参考 `vostd`；
  - trusted surface 管理借鉴 `atmo`。

这一天的意义是：

- 后续 bridge 的扩张边界被卡住了；
- 第 6 步开始可以明确朝“把 proof 推回 `cte.rs` 本体”推进。

归并来源：

- `archive/cspace-stage5-bridge-design.md`

### 2026-04-24：第 6 步收口，第一轮工程闭环完成

关键结论：

- `cargo xtask verify` 记录结果为 `136 verified, 0 errors`；
- `resolve_address_bits`、`cte_insert`、`cte_move`、`cte_swap` 的 refined 入口已位于 `cte.rs`；
- 与它们配套的局部只读语义入口也已进入 `cte.rs`；
- bridge-local trusted surface 已压缩成“视图提取 + object-local util”为主的形式。

这一天的意义是：

- 第一轮“门禁 -> 抽象模型 -> 原语规格 -> 小引理 -> bridge -> trusted 收口”的工程闭环完成；
- 但也明确看见：工程闭环不等于论文口径上的 l4v 语义对齐完成。

归并来源：

- `archive/cspace-stage6-closeout-20260424.md`
- `archive/cspace-verification-steps.md`

### 2026-04-24：主线切换到论文范围与 l4v 对齐

关键结论：

- 范围被重新固定为：
  - 只证明 `CSpace` 本身；
  - 低层与非 `CSpace` 部分进入 TCB；
  - 对选定 `CSpace` 子集，语义尽量对齐 l4v；
  - 表示与证明工程保持 Verus-native。
- 后续主线不再是“继续扩第一轮 6 步”，而是：
  - 重构 capability 基础语义；
  - 提炼局部 invariant；
  - 把原语规格改写成 preservation-first；
  - 再压回 Rust 本体证明。

这一天的意义是：

- 文档口径从“工程收口”转为“论文主线”；
- 后续实现不会再停留在“证明当前工程化子模型内部自洽”。

归并来源：

- `archive/cspace-l4v-alignment-refactor-plan.md`
- `archive/cspace-thesis-local-proof-plan.md`

### 2026-04-24：文档目录精简

关键结论：

- 日常主文档缩减为两份：
  - `cspace-verification-plan.md`
  - `cspace-session-log.md`
- 旧的计划、阶段、TCB、风格与阶段收口文档移入 `archive/`；
- 保留历史细节，但不再让 `docs/verification` 根目录持续膨胀。

这一天的意义是：

- 后续查看与维护入口更清楚；
- 论文写作和工程推进都只需围绕一份主计划和一份总日志展开。

### 2026-04-24：P2 capability 语义重构完成

关键结论：

- `cargo xtask verify` 记录结果更新为 `137 verified, 0 errors`；
- `sameRegionAs` 已不再以 `region_id` 等价作为主语义，而是回到 capability 关系定义；
- `sameObjectAs`、`isCapRevocable`、`isMDBParentOf`/`mdb_parent_of` 已统一沉淀到抽象 helper；
- `deriveCap` 的 slot 绑定前提和 generic case split 已收紧；
- `ArchCap` 语义没有被随意补造，而是保留为显式 hook。

这一天的意义是：

- 第二轮重构里的 capability 语义层已经收口；
- 后续可以把主力切换到 `P3`，也就是局部 invariant 的拆分与对齐；
- 论文里关于“CSpace 证明内容与 l4v generic 语义口径一致”的表述基础更稳了。

### 2026-04-24：P3 第一轮 invariant split 落地

关键结论：

- `cargo xtask verify` 记录结果更新为 `140 verified, 0 errors`；
- `wf` 已拆成 `mdb_state_wf` 与 `cspace_lookup_wf` 两组局部 invariant；
- `isFinalCapability` 已通过 `is_final_cap_wf_at(slot)` 使用局部前提；
- `deriveCap` 已通过 `derive_cap_wf_at(slot)` 使用 slot-local 前提；
- `ensureNoChildren` 已通过 `ensure_no_children_wf_at(slot)` 使用 MDB 局部前提；
- `resolveAddressBits` 已通过 `spec_resolve_address_bits_state_wf(state)` 使用 lookup 侧前提；
- bridge/refined proof 中已有调用点改用这些新 lemma，而不是统一回退到整包 `wf`。

这一天的意义是：

- `P3` 已不再停留在文档计划层，而是进入真实代码结构；
- 后续可以继续围绕 `isFinalCapability` 和 preservation-first 规格做第二轮收紧；
- 论文里可以更自然地区分“MDB 局部 invariant”和“lookup 局部 invariant”。

### 2026-04-24：P4 第一轮 preservation-first 改写落地

关键结论：

- `cargo xtask verify` 仍保持 `140 verified, 0 errors`；
- `cteInsert`、`insertNewCap`、`cteMove`、`cteSwap` 的 post 已拆成 `frame / invariant preservation / functional` 三层；
- `common.rs` 中已抽出通用 preservation helper；
- 新增了只读原语的 specs 入口文件，把 `isFinalCapability`、`ensureNoChildren`、`isMDBParentOf`、`longRunningDelete` 的合同命名统一起来；
- `cte.rs` 中对应 refined contract 已开始复用这些 specs 入口。

这一天的意义是：

- `P4` 已从“计划中的风格目标”进入可验证代码结构；
- 后续继续推进时，重点会从“把 post 拆开”转向“让更多 proof/bridge 直接按这三层结构组织”；
- 论文里对 primitive contract 层次的描述会更自然，也更接近 l4v 的 preservation 叙事。

### 2026-04-26：P5 第一轮 bridge 收缩落地

关键结论：

- `cargo xtask verify` 仍保持 `140 verified, 0 errors`；
- `resolve_address_bits` 的 projected-core 语义 helper 已从 `refinement_bridge.rs` 回流到 `specs/cspace_ops/resolve.rs`；
- bridge 保留下来的职责更集中到 concrete snapshot/view 提取、slot/lookup 对应关系、以及 local heap transition 骨架；
- `refinement_bridge.rs` 中的纯抽象控制流语义负担开始下降。

这一天的意义是：

- `P5` 不再只是文档目标，而是已经有了第一轮代码收缩；
- 后续 bridge 收缩可以沿着“继续把 bridge-neutral 语义 helper 挪回 specs”的方向稳定推进；
- 论文里对“bridge 只负责表示映射”的口径已经开始和代码结构对齐。

### 2026-04-26：P5 第二轮收缩与 P6 起步

关键结论：

- `cargo xtask verify` 记录结果更新为 `134 verified, 0 errors`；
- `resolve_address_bits` 的 cap-level expected/result/core refinement 证明已从 `refinement_bridge.rs` 回流到 `specs/cspace_ops/resolve.rs`；
- bridge 中原本按 `invalid_root / guard_mismatch / depth_mismatch / exact_success / early_stop / recursive_*` 分裂的一批 state-level 包装引理已大幅收缩，`resolve_address_bits` 在 bridge 里主要剩下：
  - one-step control-flow skeleton；
  - raw/state 连接；
  - `result_refines_state` 这一类通用包装；
- `resolve_address_bits` 的 trusted 假设已从“直接等于 expected core”收紧成“只满足 one-step skeleton”，最终合同改由 `exec_step + bridge` 推导；
- `cteInsert`、`insertNewCap`、`cteMove`、`cteSwap` 的 trusted 假设已从“直接满足完整 exec_contract”收紧成“满足 local heap transition”，具体 slot view 与最终合同改由 `exec_step + abstract post + bridge lemma` 推导；
- `ensureNoChildren` 的 verified 路径已切到 `via_is_mdb_parent_of`；
- `isFinalCapability` 的 verified 路径已切到 `mdb_prev/mdb_next + same_object_as` 的重建证明。

这一天的意义是：

- `P5` 已从“开始收缩”推进到“bridge 主要负责表示层骨架”；
- `P6` 不再只是调用 `assume_specification` 的薄封装，而是开始真实地把较弱的 trusted 假设拼装成最终 refinement 合同；
- 当前 remaining trusted surface 更集中，也更容易在论文里清楚说明“哪些属于 CSpace 内证明，哪些仍属于 TCB”。

### 2026-04-26：P6 查询侧继续收缩，并修正 `isMDBParentOf` 语义边界

关键结论：

- `cargo xtask verify` 当前通过，记录结果为 `135 verified, 0 errors`；
- `cte_t::is_mdb_parent_of` 的直接 `assume_specification` 已移除；
- `isMDBParentOf` 的 verified 路径已改为通过更小观察器重建：
  - parent slot 的 `mdb_revocable`
  - `same_region_as`
  - endpoint/notification 的 badge 相容性
  - child slot 的 `mdb_first_badged`
- `same_region_as` 现在作为更底层 capability observer 进入 trusted surface；
- `abstract_cspace.rs` 中 `mdb_parent_of` 的语义已修正为只表达 parent-of 判定本身，不再混入 `mdb_links / mdbNext` 邻接条件；
- `ensure_no_children_blocks(slot)` 继续在更上一层把 “`mdb_next` 存在” 与 “`mdb_parent_of(slot, next)` 成立” 组合起来，和 l4v/Haskell 的 `ensureNoChildren` 结构保持一致。

这一天的意义是：

- 这不只是又去掉了一个强假设，更重要的是把 `isMDBParentOf` 的抽象后置改回了更接近 l4v 的语义边界；
- `P6` 在 query 侧已经从“直接信任查询结果”进一步推进到“信任更小 observer，再在 Verus 里重建查询语义”；
- 论文里关于 `isMDBParentOf` 与 `ensureNoChildren` 关系的描述，现在可以更自然地沿用 l4v 的叙事：先定义 parent-of 判定，再在 `mdbNext` 上使用它。

### 2026-04-26：P6 继续收缩 `sameObjectAs`，并修正 IRQControl 语义

关键结论：

- `cargo xtask verify` 当前通过，记录结果为 `136 verified, 0 errors`；
- `same_object_as` 的 Rust 实现已修正为：
  - 左侧是 `UntypedCap` 时返回 `false`
  - 左侧是 `IRQControlCap` 时也统一返回 `false`
  - 这与 l4v/Haskell 的 `sameObjectAs` 语义保持一致；
- `cte.rs` 中新增了 `same_object_as_refined(...)`；
- `isFinalCapability` 在普通 non-arch 路径上，已不再直接依赖 `same_object_as` 的整体 trusted 假设，而是通过：
  - `same_region_as`
  - `UntypedCap` 左侧特例
  - `IRQControlCap` 左侧特例
  在 Verus 里重建 `sameObjectAs` 合同；
- 当前 `same_object_as` 的直接 trusted 假设只剩下 both-arch fallback，这与“arch 细节暂留 TCB，CSpace 子集先严格对齐 l4v”这一范围约束一致。

这一天的意义是：

- `P6` 不只是继续减少 trusted surface，也顺手修掉了一个会影响论文语义口径的实现偏差；
- `isFinalCapability` 现在对 `sameObjectAs` 的依赖边界更清楚了，可以更自然地解释为“基于 `sameRegionAs` 与少量 l4v 特例重建”；
- 当前 remaining trusted surface 正在逐步下沉到 capability observer / arch-specific 这一级，而不是停在更高层的 CSpace query 上。

### 2026-04-26：P6 收掉 `sameObjectAs` 整函数假设

关键结论：

- `cargo xtask verify` 当前通过，记录结果保持 `136 verified, 0 errors`；
- `same_object_as_refined(...)` 现在已经不再调用 `same_object_as` 本体；
- `sameObjectAs` 的合同在当前证明主线上改为通过：
  - `same_region_as`
  - 左侧 `UntypedCap` 特例
  - 左侧 `IRQControlCap` 特例
  - both-arch 在当前抽象模型下直接落到 `false`
  来重建；
- `assume_specification[same_object_as]` 已从 `cte.rs` 中移除；
- 目前 `isFinalCapability` 依赖的下一层 observer 已明确收敛到 `same_region_as`。

这一天的意义是：

- 这标志着 query 侧又少掉了一整层 capability helper 黑盒；
- `isFinalCapability` 现在不再通过“信任 `sameObjectAs`”成立，而是通过“信任更底层 `sameRegionAs`，并在 Verus 中恢复 l4v 特例”成立；
- 下一阶段如果继续收紧 trusted boundary，最自然的目标就变成 `same_region_as` 与它所需的更小 capability 观察器。

### 2026-04-26：P6 把 `sameObjectAs` 从 `sameRegionAs` 继续剥离

关键结论：

- `cargo xtask verify` 当前通过，记录结果更新为 `139 verified, 0 errors`；
- `same_object_as_refined(...)` 的 non-arch 路径已不再通过 `same_region_as` 重建，而是改为：
  - 小粒度 cap-kind observer
  - concrete view 到 abstract object 的 shape lemma
  - 按 endpoint / notification / reply / cnode / thread / irq-handler 分支恢复 `sameObjectAs`
- `cte.rs` 中与 `sameObjectAs` 相关的证明结构已从“大块 `compute_only`”改成“小引理 + 分支回拼”；
- `isFinalCapability` 因为走 `same_object_as_refined(...)`，现在也不再依赖 `same_region_as`；
- 当前 query 侧 remaining trusted surface 进一步收缩为：
  - `isMDBParentOf` 路径上的 `same_region_as`
  - concrete view 提取
  - 少量 object-local observer

这一天的意义是：

- `sameObjectAs` 这条线现在已经基本落到“bridge 只给表示、证明自己恢复语义”的结构上；
- 这比上一轮“通过 `sameRegionAs` 加特例恢复 `sameObjectAs`”更接近我们希望的 Verus-native 风格；
- 后续如果继续推进 `P6`，最自然的目标就是继续下沉 `same_region_as`，或者把它明确收口成一个更小、论文里更容易解释的 observer。

### 2026-04-26：P6 开始收掉 `cteInsert` 接口里的过渡 ghost

关键结论：

- `cargo xtask verify` 当前通过，记录结果更新为 `160 verified, 0 errors`；
- `cte.rs` 中新增了 `cte_insert_refined_auto_revocable(...)`；
- 这个新 wrapper 不再要求外部手工传入 `new_cap_is_revocable`，而是改为：
  - 从 `src_slot` 提取 raw cap ref；
  - 调用已经验证过的真实接口 `is_cap_revocable(...)`；
  - 再把结果回接到 `spec_cte_insert_expected_revocable(...)`；
- 为了支撑这一步，bridge 新增了一个很小的连接引理：
  - `lemma_cte_insert_call_pre_at_implies_raw_slot_views_match_state(...)`
  - 它只负责把 `cte_insert_call_pre_at` 下的 raw `src/dest` slot 视图与抽象 `old_state` 对齐。
- 随后又进一步做了一步接口收口：
  - 旧的带 `new_cap_is_revocable` ghost 的版本退成内部 helper；
  - 不带这个 ghost 的版本已扶正为 `cte_insert_refined(...)` 主入口。

这一天的意义是：

- 这不是单纯新增一个 helper，而是开始把“证明接口里为了过渡而保留的 ghost 参数”往真实 capability query 上折叠；
- 它很符合当前希望形成的最终风格：
  - bridge 给最小表示连接；
  - capability query 用真实 Verus 接口恢复语义；
  - `cte` proof 入口尽量不暴露本来可以内部推导出来的 ghost；
- 接下来如果继续沿这个方向推进，最自然的动作是：
  - 逐步把旧的 `cte_insert_refined(...)` 退居内部实现；
  - 或对其他仍带明显过渡痕迹的 proof entry 做同样的 ghost 收缩。

### 2026-04-26：P6 把 `is_cap_revocable` 也切到真实 Verus 接口

关键结论：

- `cargo xtask verify` 当前通过，记录结果更新为 `159 verified, 0 errors`；
- `capability::is_cap_revocable(...)` 在 `feature=verify` 下已切换为真实的 Verus 入口，不再只是普通 Rust 函数；
- `cte.rs` 中新增了：
  - `is_cap_revocable_exec_contract(...)`
  - `is_cap_revocable_refined(...)`
- 当前三个最核心、只依赖 `&cap` 的 capability-level query：
  - `same_region_as(...)`
  - `same_object_as(...)`
  - `is_cap_revocable(...)`
  都已经开始收敛到“真实 rs 接口本身带 Verus 契约”的形态；
- 为了支撑 `is_cap_revocable` 的 badge 语义恢复，bridge 的 `cap_snapshot_wf(...)` 也显式补上了：
  - `EndpointCap` / `NotificationCap` 的 `badge_present` 结构约束；
  - 这样 badge 相关证明就不再依赖“外部提取器实现细节默认正确但合同里没写出来”的隐含前提。

这一天的意义是：

- capability query 这一层，已经不再只有旁边的 refined wrapper 可以证明，而是开始把“真实对外函数”本身逐步变成 Verus-native 入口；
- 这比单纯增加几个 proof helper 更接近最终目标：将现有 Rust 接口逐步替换成带 `requires/ensures` 的验证实现；
- 下一步最自然的工作点也更清楚了：
  - 继续把 `cteInsert` 一类接口中仅为过渡保留的 ghost 参数收掉；
  - 或继续下沉 `trusted_extract_*` / object-local observer；
  - `trusted_range_top_u128_if_small` 仍然可以单独作为 arithmetic trusted-boundary 收紧小任务处理。

### 2026-04-26：P6 开始把真实 query 接口切到 Verus 入口

关键结论：

- `cargo xtask verify` 当前通过，记录结果更新为 `157 verified, 0 errors`；
- `capability::same_region_as(...)` 与 `capability::same_object_as(...)` 在 `feature=verify` 下已不再只是普通 Rust 函数，而是切换成带 `ensures` 的真实 Verus 入口；
- `cte.rs` 里的 refined proof 仍保留原有 bridge-level helper，但 query 主线已经可以通过真实接口合同来复用它们；
- 为了让上层 `cte` query proof 能逐步接上这条真实接口链路，bridge 新增了一个更小的 object-local observer：
  - `trusted_cap_ref_from_slot(...)`
  - 它只负责 `slot -> raw cap ref` 这一件事，不再一次性暴露整个 `cte` 快照；
- 这次也确认了一个边界事实：
  - 由于 `cte_t` 目前仍是 opaque external type，上层 proof 还不能直接写 `raw_slot.capability`；
  - 因此，后续如果继续把 `isFinalCapability` / `isMDBParentOf` 完全改接到真实 query 接口，仍然需要继续下沉 concrete field observer，而不是简单替换调用点。

这一天的意义是：

- 这是从“证明旁边的 refined wrapper”向“证明现有 rs 接口本身”迈出的第一步；
- 它很符合当前论文口径：语义仍由 l4v 选择，工程形态则逐步收敛到 Verus-native 的签名式入口；
- 同时也更明确了 P6 后续的自然方向：
  - 继续收缩 `trusted_extract_*` 这类较大的 concrete-view 提取；
  - 或继续把 `slot -> raw field ref` 这类 object-local observer 细化到最小；
  - `trusted_range_top_u128_if_small` 则保留为一个单独的 arithmetic trusted-boundary 收紧小任务。

### 2026-04-26：P6 收掉 `sameRegionAs` 整函数假设

关键结论：

- `cargo xtask verify` 当前通过，记录结果回到 `140 verified, 0 errors`；
- `cte.rs` 中新增了 `same_region_as_refined(...)`；
- `isMDBParentOf` 的 verified 路径已经不再依赖 `same_region_as` 整函数，而是通过：
  - cap-kind observer
  - concrete view object/cnode shape lemma
  - 更小的 range-top arithmetic helper
  - badge / `mdb_first_badged`
  来恢复抽象 `sameRegionAs` 与 `mdb_parent_of` 语义；
- `assume_specification[same_region_as]` 已从 `cte.rs` 中删除；
- 当前 query 侧 remaining trusted surface 进一步收缩为：
  - concrete view 提取
  - `trusted_range_top_u128_if_small`
  - 少量 object-local observer

这一天的意义是：

- query 侧已经不再停留在 `sameObjectAs` / `sameRegionAs` 这一层黑盒上；
- `isFinalCapability` 与 `isMDBParentOf` 现在都更接近“用小观察器恢复 l4v 语义”的结构；
- 后续如果继续推进 `P6`，最自然的目标就变成继续收紧 concrete view 提取与这类更小 arithmetic/object-local observer，或者开始把它们明确整理成最终 TCB 台账。

### 2026-04-26：P6 继续缩小 untyped containment 的 trusted 粒度

关键结论：

- `cargo xtask verify` 当前通过，记录结果更新为 `141 verified, 0 errors`；
- `trusted_untyped_contains_cap` 已从代码中删除；
- `same_region_as_refined(...)` 的 untyped 分支已改为：
  - 在抽象层收紧 `valid_cap`，显式要求 untyped/cnode size bits 落在 machine-word 范围内；
  - 在 `cte.rs` 中直接重建 `spec_untyped_cap_contains_cap(...)` 的主体逻辑；
  - 只通过一个更小的 `trusted_range_top_u128_if_small(...)` 提供 machine arithmetic 到抽象 `pow2/range-top` 的连接；
- 当前 trusted surface 比上一轮更细粒度，也更接近“bridge 负责表示、proof 负责语义恢复”的目标形态。

这一天的意义是：

- 我们没有把 untyped containment 整块留在 TCB，而是把它继续压成了一个更小、更好解释的 arithmetic helper；
- 这让论文里对 trusted boundary 的表述更自然：不再是“信任一个 CSpace 关系判定”，而是“信任一个很小的机器数值连接点”；
- 后续如果继续推进，就可以把火力更集中地放在 concrete view 提取和剩余 object-local observer 上。

### 2026-04-26：P6 继续下沉到 bridge-level query helper

关键结论：

- `cargo xtask verify` 当前通过，记录结果更新为 `155 verified, 0 errors`；
- `trusted_cap_is_*`、`trusted_slot_cap_is_*` 与 `trusted_slot_cap_clone` 已从主证明链移除，并已从 bridge 代码中删除；
- `trusted_has_mdb_prev/next` 也已从主证明链与 bridge 代码中删除，`mdbPrev/mdbNext` 是否存在现在直接由 `bridge_cte` 快照给出；
- `trusted_follow_mdb_prev/next` 也已被更小的 `trusted_slot_ref_from_addr(...)` 取代，trusted pointer primitive 不再携带 `mdb` 业务语义；
- `sameRegionAs` / `sameObjectAs` 已拆成 bridge-level helper：
  - 对外仍保留 `raw cap -> refined contract` 入口；
  - 对内则直接在 `CapBridge` 上恢复抽象语义；
- `isFinalCapability` 与 `isMDBParentOf` 已改为直接复用 bridge-level helper，不再为了调用 query 再 clone raw cap；
- bridge 的 `cap_snapshot_wf` 已显式纳入 `supported cap tag` 约束，把“当前 CSpace 子集只讨论这些构造子”从隐含前提改成了明示边界。

这一天的意义是：

- query 侧 trusted surface 又下沉了一层，从“按 raw cap 分类的 observer”进一步压到了“表示提取 + 支持 tag 边界 + 小 arithmetic helper”；
- `mdb` 相关 trusted primitive 也从“顺着链跟随”继续压到了“由 concrete address 取 raw ref”这一更小粒度；
- `bridge 负责表示，proof 负责语义恢复` 这条风格线更稳定了；
- 后续如果继续推进 `P6`，优先目标就更清晰了：
  - 收紧 `trusted_range_top_u128_if_small`；
  - 评估 `trusted_has_mdb_* / trusted_follow_mdb_*` 是否还能再细化；
  - 或者开始整理最终 TCB 台账。

### 2026-04-26：P6 收尾完成，准备转入 P7

关键结论：

- `cargo xtask verify --package sel4_cspace --jobs 1 --max-errors 50` 当前通过，记录结果为 `160 verified, 0 errors`；
- `capability::sameRegionAs`、`sameObjectAs`、`isCapRevocable` 现在都已经在 `feature=verify` 下切到真实 Verus 入口；
- `cteInsert` 已切到不需要外部显式传入 `new_cap_is_revocable` ghost 的主 wrapper；
- `cte.rs` 中只用于内部证明分解的 `via_*_refined` helper 已收回为私有函数，不再和稳定的 refined 主入口一起对外暴露；
- `P6` 当前保留下来的 remaining trusted surface 已稳定收敛为四类：
  - concrete view 提取：`trusted_view_*` / `trusted_extract_*`
  - heap/state 对应观察器：`trusted_concrete_slot_view_at`、`trusted_concrete_cnode_lookup_slot_at`
  - pointer/object-local primitive：`trusted_slot_ref_is_id`、`trusted_slot_ref_from_addr`、`trusted_cap_ref_from_slot`
  - 小粒度 arithmetic / return helper：`trusted_range_top_u128_if_small` 与若干 `trusted_make_*`

这一天的意义是：

- `P6` 可以正式结束了，因为 capability/query 主线已经从“信任整函数”稳定切到“弱 observer + Verus 语义恢复”；
- 当前剩余 trusted boundary 已经小到足以直接进入论文口径的 TCB 台账，而不必再继续把 `P6` 拉长；
- 接下来的主线应切换为 `P7`：整理已证入口、未证范围、l4v 对应关系与最终 TCB 清单。

### 2026-04-26：P7 启动，第一轮论文台账落地

关键结论：

- 主计划文档中已经补入 `P7` 第一轮总表，明确区分了两类“已证入口”：
  - 真实 `feature=verify` Verus 接口
  - 对 opaque exec body 建立 refinement 合同的 `*_refined` 入口
- 当前 capability query 三项：
  - `sameRegionAs`
  - `sameObjectAs`
  - `isCapRevocable`
  已被固定为“真实接口已验证”的口径；
- `cteInsert`、`insertNewCap`、`cteMove`、`cteSwap`、`resolveAddressBits`、`deriveCap`、`isMDBParentOf`、`ensureNoChildren`、`isFinalCapability`、`slotCapLongRunningDelete`
  已被固定为“refined wrapper 已验证”的口径；
- l4v 对应关系已经整理到：
  - Haskell 定义来源
  - 代表性的 correspondence / refinement proof 文件
- 当前未覆盖范围与 remaining TCB 也已被整理成可以直接放进论文的分类表，而不再只是过程性笔记。

这一天的意义是：

- 从这一刻开始，后续讨论不必再反复口头解释“我们到底证明了什么、没证明什么、哪些还在 TCB”；
- `P7` 不再只是最后一个待办标题，而是已经有了第一轮可引用的论文材料基线；
- 如果后面继续技术推进，新增证明工作也可以直接往这套台账里追加，而不是重新发明一套进度口径。

### 2026-04-26：P7 继续压缩到论文正文口径

关键结论：

- 主计划文档中新增了 `P7.6`，把前一轮工程台账继续压成：
  - 摘要版主张
  - 正文章节可直接复用的段落模板
  - 图表标题建议
  - 建议避免的过度主张表述
- 这样一来，当前文档不再只回答“工程上做了什么”，也开始直接回答“论文里应该怎么准确地写出来”；
- 同时把几个最容易写过头的点固定了下来，例如：
  - 不能声称完成了整个 kernel 的完整验证
  - 不能把 refined wrapper 和真实对外接口混写成同一层完成度
  - 不能把当前局部 invariant 直接写成整系统 `invs`

这一天的意义是：

- `P7` 的产出已经从“技术台账”进一步推进到“写作台账”；
- 后续即使先暂停技术证明，也已经有了一套比较稳的论文表述基线；
- 如果后面继续推进证明，新增结果也可以直接挂接到这套写作口径下，而不会再次出现工程语言和论文语言脱节的问题。

### 2026-04-26：P7 补出摘要、引言与贡献草稿

关键结论：

- 主计划文档中新增了 `P7.7`，直接补入了：
  - 中文摘要草稿
  - 中文引言开头草稿
  - 贡献列表草稿
  - 更短的摘要版贡献点
- 这样一来，当前 `P7` 产出已经不只是“如何描述当前验证状态”，还开始提供“论文正文可以直接改写和裁剪的文字素材”；
- 同时也把一些容易失真的写法继续压回到了统一口径下：
  - 强调局部验证而非整系统验证
  - 强调显式 trusted boundary
  - 强调“真实 Verus 接口”和“refined wrapper 完成度”之间的区别

这一天的意义是：

- `P7` 已经从工程总结进一步推进到论文初稿准备阶段；
- 即使暂时不继续做新的证明工作，当前文档也已经可以支持你开始搭正文结构；
- 后续如果继续写作，下一步最自然的动作就是把这些素材进一步裁成摘要定稿、引言定稿和贡献列表定稿。

### 2026-04-26：P7 继续压成候选定稿版

关键结论：

- 主计划文档中新增了 `P7.8`，在 `P7.7` 草稿素材之上进一步给出了：
  - 候选摘要定稿
  - 候选引言收束段
  - 候选贡献定稿
  - 答辩时可直接使用的一句话概括
- 这一轮的重点不再是“先把素材列出来”，而是主动做第一轮压缩和措辞统一，把更像最终成文的版本直接写出来；
- 这样后面无论是写摘要、写引言还是准备答辩介绍，都已经有可以直接删改的起点，而不是还要从长草稿重新组织一遍。

这一天的意义是：

- `P7` 已经进入论文定稿准备的前一阶段；
- 工程文档和论文语言之间的距离进一步缩短了；
- 如果接下来继续推进，最自然的动作就不是“再解释当前进度”，而是直接做摘要、引言和贡献列表的最终定稿选择。

### 2026-04-26：P7 形成推荐终稿版

关键结论：

- 主计划文档中新增了 `P7.9`，不再只是列多套候选，而是明确给出了当前推荐直接采用的一版：
  - 推荐摘要终稿
  - 推荐引言主线
  - 推荐贡献终稿
  - 推荐题目风格
  - 推荐答辩开场 30 秒版本
- 这一轮的重点是把“候选定稿”继续推进成“建议你优先使用的终稿版本”，减少后续从多版草稿中反复筛选的成本；
- 同时也把几个容易在论文和答辩中失真的地方继续卡死了：
  - 强调是局部验证，不是整系统验证
  - 强调 trusted boundary 是显式前提，不是隐含省略
  - 强调 capability query 三项与 refined wrapper 原语在完成度上的区别

这一天的意义是：

- `P7` 已经基本进入可以直接服务论文定稿和答辩准备的阶段；
- 从工程验证文档到论文写作文本之间，已经不再隔着明显的“再翻译一遍”的步骤；
- 如果接下来继续推进，最自然的工作将变成按学校格式做最后润色，或者转回技术主线继续做 trusted boundary 收紧工作。

### 2026-04-27：P7 产出独立论文草稿文件

关键结论：

- 在 `docs/verification/` 下新增了独立写作文件：
  - `cspace-thesis-draft.md`
- 这份文件不再记录工程过程，而是专门承载：
  - 题目候选
  - 摘要终稿草稿
  - 引言终稿草稿
  - 贡献终稿草稿
  - 局限性与边界表述
  - 答辩口径与图表建议
- 同时主计划文档也同步调整了入口说明：
  - `cspace-verification-plan.md` 继续负责技术口径、已证范围、未证范围与 TCB
  - `cspace-thesis-draft.md` 负责面向论文正文的独立文字草稿

这一天的意义是：

- `P7` 已经不只是“推荐终稿素材”，而是有了一份可以单独拿来修改和排版的论文草稿文件；
- 后续写作时，不必继续从计划文档中抽取段落，可以直接围绕独立草稿推进；
- 技术台账和写作文本现在正式分流，后续维护成本会更低。

### 2026-04-27：P7 继续补齐全文写作骨架

关键结论：

- 独立论文草稿 `cspace-thesis-draft.md` 继续补入了更适合全文落稿的支撑部分：
  - 英文摘要草稿
  - 研究问题
  - 术语统一建议
  - 章节安排建议
- 这一步的重点不再只是“给出几段推荐表述”，而是开始为后续整篇论文写作提供骨架，减少前后口径漂移；
- 尤其是 `研究问题` 和 `术语统一` 两部分，能帮助后面在引言、方法、总结中保持同一套说法，而不会在不同章节里把“局部验证”“TCB”“真实接口”“refined wrapper”混在一起。

这一天的意义是：

- `P7` 已经从“有独立草稿”进一步推进到“独立草稿具备全文展开能力”；
- 后续如果直接写正文，已经不需要再先回头补方法主线和术语表；
- 如果接下来继续推进，最自然的动作就是按学校格式把这份草稿拆成真正的摘要、绪论和方法章节初稿。

### 2026-04-27：P7 收口完成

关键结论：

- `cargo xtask verify --package sel4_cspace --jobs 1 --max-errors 50` 再次通过，记录结果保持 `160 verified, 0 errors`；
- 主计划文档已把 `P7` 状态正式更新为“已完成”；
- `cspace-verification-plan.md`、`cspace-session-log.md`、`cspace-thesis-draft.md` 三份主文档的职责边界已经固定：
  - 技术口径与台账
  - 过程时间线
  - 论文正文草稿
- 独立论文草稿已补到可直接复用“摘要 / 引言 / 贡献 / 边界 / 术语 / 章节安排 / 结论与后续工作”的程度。

这一天的意义是：

- `P7` 不再只是“正在收尾”，而是已经形成了可直接支撑论文写作和答辩表述的一轮完整交付；
- 第一轮工程闭环和第二轮 `P2` 到 `P7` 的论文口径收口，到这里都已有稳定落点；
- 从这里往后，主线就不再是“补齐 `P7`”，而是二选一：
  - 继续按学校模板打磨论文正文
  - 或转回技术主线，继续做 trusted boundary 收紧与 opaque exec body 替换

### 2026-04-29：补出“当前代码总结”和“最终展示代码要求”

关键结论：

- 主计划文档新增了面向工程收口的两部分：
  - 当前代码总结
  - 毕设最终展示代码要求
- 这两部分不再只描述“论文怎么写”，而是明确回答：
  - 当前 `CSpace` 代码已经证明到哪一层
  - 当前更像 `refined wrapper` 路线还是 `atmo` 式 verified runtime 路线
  - 最终展示代码至少应该覆盖哪些接口
  - 哪些是加强目标，哪些不应被误设为必须完成
- 当前推荐的最低展示子集已经固定为：
  - `same_region_as`
  - `same_object_as`
  - `is_cap_revocable`
  - `derive_cap`
  - `ensure_no_children`

这一天的意义是：

- 从这一步开始，“最后到底交什么代码、展示什么代码”不再停留在口头判断，而是已经形成文档化标准；
- 论文口径、验证台账和最终代码收口目标之间，已经建立了明确对应关系；
- 后续如果继续做技术推进，可以直接对照这份要求判断某项工作属于“必须完成”“加强完成”还是“理想完成”。

### 2026-04-29：把后续实现顺序固定为可执行路线

关键结论：

- 主计划文档进一步补入了“推荐实现顺序”；
- 这个顺序不再只说阶段编号，而是直接对应到当前工程中的函数族和推进顺序；
- 固定下来的主线是：
  - 先冻结最小展示子集；
  - 再做 `ensure_no_children -> derive_cap -> is_final_cap`；
  - 再补 `resolve_address_bits`；
  - 再以 `cte_insert` 为模板推进其余 mutating primitive；
  - 最后才进入 delete/revoke/finalise 与 trusted boundary 收口/构建对齐。

这一天的意义是：

- 后续工程推进从“看到哪里改哪里”转成“按固定顺序逐段推进”；
- 论文展示子集、实际代码收口目标和下一轮技术任务之间，已经建立了明确的先后关系；
- 后面如果继续推进代码，可以直接判断某项工作是在加强最小展示主线，还是属于更后置的扩展任务。

### 2026-04-29：开始执行前 7 步，先完成第一轮 proof surface shrink

关键结论：

- 已开始按“前 7 步”推进代码，而不是只停留在计划层；
- `sel4_cspace/src/cte.rs` 中仅供 proof backend 使用的 refined 入口已做一轮可见性收缩：
  - `cte_insert_refined`
  - `is_final_cap_refined`
  - `ensure_no_children_refined`
  - `derive_cap_refined`
  - `insert_new_cap_refined`
  - `cte_move_refined`
  - `cte_swap_refined`
  - `resolve_address_bits_refined`
  - 以及同类的内部 query/delete 辅助入口
  现在都不再作为对外 `pub` proof 表面暴露；
- capability query 三项在 `cte.rs` 中对应的 backend 已进一步收紧为 `pub(crate)`，稳定 verify-facing 接口继续只放在 `sel4_cspace/src/capability/mod.rs`；
- 这轮收口后再次运行
  - `cargo xtask verify --package sel4_cspace --jobs 1 --max-errors 50`
  并通过，结果为 `160 verified, 0 errors`。

这一天的意义是：

- “前 7 步”已经从计划进入代码执行；
- 第一轮完成的不是大规模 body 重写，而是更符合目标方向的 proof surface shrink；
- 后续如果继续推进前 7 步，重点就不再是新增更多公开 refined 入口，而是继续把 runtime-adjacent interface、opaque body 与 trusted surface 收紧。

### 2026-04-29：把前 7 步收口到 `interface.rs` 的稳定 verify-facing 层

关键结论：

- 在上一轮 proof surface shrink 的基础上，又完成了一轮更关键的接口收口；
- `sel4_cspace/src/interface.rs` 不再只是 runtime re-export，还新增了一层稳定的 verify-facing facade：
  - `is_final_cap_at`
  - `ensure_no_children_at`
  - `derive_cap_at`
  - `resolve_address_bits_at`
  - `cte_insert_at`
  - `insert_new_cap_at`
  - `cte_move_at`
  - `cte_swap_at`
- 与此同时，`sel4_cspace/src/cte.rs` 中对应的 `*_refined` proof backend 已进一步收回为 `pub(crate)` 或模块私有；
- 重新运行
  - `cargo xtask verify --package sel4_cspace --jobs 1 --max-errors 50`
  后通过，结果更新为 `168 verified, 0 errors`。

这一天的意义是：

- 前 7 步不再只是“内部 proof backend 已经写好”，而是已经拥有了对外稳定的 verify-facing 接口层；
- `cte.rs` 和 `interface.rs` 的职责分工进一步清楚：
  - `cte.rs` 承担 concrete exec body 与内部 proof backend
  - `interface.rs` 承担公共导出入口与 verify-facing facade
- 因此前 7 步在当前轮次上已经可以视为完成，后续主线应转向 delete/revoke/finalise 与 trusted boundary 收紧，或继续挑单点做 opaque body 深化替换。

### 2026-04-29：把第 8、9 步也收口到当前轮次完成态

关键结论：

- delete/revoke/finalise 主线这次没有继续停在“还没动”，而是完成了一轮可引用的工程拆解；
- `sel4_cspace/src/interface.rs` 新增了 delete-gate verify-facing 入口：
  - `is_mdb_parent_of_at`
  - `is_long_running_delete_at`
- `sel4_cspace/specs/boundary_assumptions.rs` 新增了 delete 主线的显式边界台账：
  - `assume_reduce_zombie_local_progress`
  - `assume_delete_all_local_flow`
  - `assume_revoke_loop_flow`
- `sel4_cspace/src/refinement_bridge.rs` 中仅供 crate 内部使用的一批 trusted constructor / extractor / bridge helper 已进一步收回为 `pub(crate)`；
- 新增联合检查脚本：
  - `tools/check-cspace-build-and-verify.sh`
  它会先跑带环境的 `cargo check`，再跑 `cargo xtask verify --package sel4_cspace --jobs 1 --max-errors 50`；
- 这条联合检查已在 2026-04-29 跑通，Verus 结果更新为 `170 verified, 0 errors`。

这一天的意义是：

- 第 8、9 步在当前轮次上已经不再是“待做事项”，而是已经有了清楚的工程落点；
- delete 主线现在已经被拆成：
  - 已证局部 gate
  - 显式 boundary assumption
  - 尚未 full proof 的 runtime body
- build 对齐也不再只是一句建议，而是已经有了固定的联合检查入口；
- 因此，当前 1 到 9 步都已经完成了本轮目标，下一轮应转向更深入的 trusted boundary 收紧和 full local proof，而不是继续补阶段框架。

### 2026-04-30：确认历史 `assume_specification` 的真实约束，并固定推荐公共口径

关键结论：

- `sel4_cspace/src/cte.rs` 中非删除主线相关的历史 `assume_specification[...]`
  依然需要保留为与对应 runtime 函数同级可见；
  这是 Verus 对 `assume_specification` 的可见性要求，不是我们可以随意再往下收的自由度。
- 但项目口径已经固定：
  - 外部验证代码的推荐入口是 `sel4_cspace/src/interface.rs` 中的
    `*_at_pre / *_at` verify-facing facade；
  - `cte.rs` 中这批 `assume_specification` 属于遗留支撑合同，而不是推荐公共 proof surface。
- 在此基础上，`sel4_cspace/src/interface.rs` 又新增了两条显式区分路径的入口：
  - `is_final_cap_runtime_at`
  - `ensure_no_children_runtime_at`
  - `is_long_running_delete_runtime_at`
  - `derive_cap_runtime_at`
  - `is_mdb_parent_of_runtime_at`
  - `resolve_address_bits_runtime_at`
  - `cte_insert_runtime_at`
  - `insert_new_cap_runtime_at`
  - `cte_move_runtime_at`
  - `cte_swap_runtime_at`
  它们统一表示“当前 raw runtime body 的假设路径”，与原有 refined proof 路径并列存在。
- 同一轮里，也把 `cte.rs` 顶部那批 runtime body 的职责重新写清楚了：
  - 这些函数仍是 runtime-adjacent 实现体；
  - 但在验证口径上，不再建议把它们直接当作最终公共证明接口。
- 重新运行
  - `bash tools/check-cspace-build-and-verify.sh`
  并通过，Verus 结果更新为 `180 verified, 0 errors`。

这一天的意义是：

- 这不是“已经把所有 runtime body 直接换成 fully verified Verus body”；
- 但它把当前最容易混淆的一层先拍板了：
  - `interface.rs` 是推荐公共验证口径；
  - `cte.rs` 里的历史 `assume_specification` 仍然存在，但只是遗留支撑合同；
  - `refinement_bridge.rs` 继续是内部 bridge-TCB；
- 更重要的是，这层区分现在不再只体现在文档里，而是直接落进了代码接口：
  - refined 已证路径
  - runtime 假设路径
  已经可以在 query、lookup 和四个 non-delete mutator 上并列引用和对比；
- capability query 三项现在也补上了对应的 raw runtime 别名：
  - `same_region_as_runtime`
  - `same_object_as_runtime`
  - `is_cap_revocable_runtime`
  因而 capability 基础语义这一层也不再只剩“文档上说有两条路径”，而是代码接口里已可显式区分；
- 因而后续非删除主线真正剩下的重点，已经从“先区分哪层该公开”转成了：
  - `is_mdb_parent_of / is_final_cap / ensure_no_children / derive_cap`
    的 Verus-native body 替换；
  - `resolve_address_bits`
  - `cte_insert / insert_new_cap / cte_move / cte_swap`
    的实现体验证化。

### 2026-04-30：mutator family 完成去 assumption 化，当前进度推进到 `9 / 10`

关键结论：

- `sel4_cspace/src/cte.rs` 中四个 non-delete mutator 的 raw `assume_specification[...]`
  已全部移除：
  - `cte_insert`
  - `insert_new_cap`
  - `cte_move`
  - `cte_swap`
- 这四个入口在 `feature=verify` 下，现已统一改成显式的 contract-bearing
  `#[verifier::external_body]` 入口；
  普通构建路径则继续保留 `#[cfg(not(feature = "verify"))]` 的 runtime body。
- 因此，`sel4_cspace/src/cte.rs` 当前剩余 raw assumption 数量已经降到 `0`；
  但这不等于四个 mutator 已经变成 fully verified Verus body。
- 重新运行
  - `bash tools/check-cspace-build-and-verify.sh`
  并通过，Verus 结果保持为 `211 verified, 0 errors`。
- `docs/verification/cspace-verification-plan.md` 也已同步更新：
  - 当前量化进度改为 `9 / 10`
  - 当前下一步改为第 `10` 步
  - 当前口径明确区分：
    - raw assumption 已清零
    - mutator 仍是 contract-bearing `external_body`
    - bridge / TCB 收紧仍然是最后一项主线

这一天这一步的意义是：

- 第 `9` 步“mutator family 去 assumption 化”现在可以视为完成；
- 非删除主线当前不再剩下 raw `assume_specification` 这一级公开合同；
- 但最后一段差距也更清楚了：
  - 不是“还有 assumption 没收完”；
  - 而是“当前 external_body + bridge precondition 的过渡层，还没有继续压到更直接的 Verus body / 更薄的 TCB”。

### 2026-04-30：bridge 模块内收，当前 `10` 步主线基线收口到 `10 / 10`

关键结论：

- `sel4_cspace/src/lib.rs` 中的 `refinement_bridge` 已从 crate root 的公开模块收回为
  `pub(crate) mod`；
  它现在是明确的 crate 内部 bridge-TCB，而不是机械性公开 proof surface。
- 为了不让外部 verify-facing 接口丢失类型名，
  `sel4_cspace/src/interface.rs` 现已直接承接：
  - `ConcreteHeapId`
  - `ResolveAddressBitsAtRet`
  也就是说，外部验证代码如果需要这些名字，应从 `interface.rs` 取，而不是再绕回
  `refinement_bridge.rs`。
- 重新运行
  - `bash tools/check-cspace-build-and-verify.sh`
  并通过，Verus 结果保持为 `211 verified, 0 errors`。
- `docs/verification/cspace-verification-plan.md` 已同步更新为：
  - 当前量化进度 `10 / 10`
  - 当前这条“非删除主线 verify-native 化”基线主线已完成
  - 后续工作转入可选加强项，而不是继续把这条 `10` 步清单保留成未完成状态

这一天这一步的意义是：

- 第 `10` 步“bridge / TCB 最终收口与 build/verify 口径对齐”在当前基线口径下可以视为完成；
- `interface.rs` 现在不只是推荐公共入口，而且已经把外部还需要的 verify-facing 类型名一起接住；
- `refinement_bridge.rs` 则进一步回到“内部 bridge-TCB”这一更准确的位置；
- 但这仍然不是“remaining TCB 已全部消失”：
  后续如果继续推进，目标应是进一步收紧 bridge helper，或继续把 `external_body` 压到更直接的 Verus body。

### 2026-04-30：非删除主线的 7 步 Verus 替换实施文档重建

关键结论：

- `docs/verification/cspace-improvement-plan.md` 已重建为一份执行型文档；
- 当前文档不再把重点放在“论文如何表述已经完成的基线”，而是直接回答：
  - 如果目标是最终用 Verus 替换 `sel4_cspace` 的非删除主线，
  - 那下一轮代码到底按什么顺序推进；
- 新文档把当前工作拆成 7 个工作包：
  - Step 1：模块边界重画
  - Step 2：`repr / owner / view / model` 基础层
  - Step 3：capability query 主实现收合
  - Step 4：slot-local query / derive 主线替换
  - Step 5：`resolve_address_bits`
  - Step 6：mutator family
  - Step 7：delete 主线
- 同时明确冻结：
  - `finalise`
  - `delete_all`
  - `reduce_zombie`
  - `revoke`
  - 以及 delete 主线对应的边界假设扩张；
- 当前执行轮只要求做完 Step 1 到 Step 6；
  Step 7 继续保留在总路线里，但不作为这一轮代码停止前的必做项。

这一天这一步的意义是：

- 当前项目的下一轮技术主线被重新从“论文收尾优先”切回了“实现体替换优先”；
- 非删除主线后续不再按“继续补 wrapper / helper”理解，而是按：
  - 表示层地基
  - Verus body 替换
  - bridge 退化成 observer
  这条路线推进；
- 这也意味着后续代码验收的关键标准不再是“还能不能再去掉一个 assumption 名字”，
  而是“Verus 是否正在更直接接管真实实现体”。

### 2026-04-30：Step 1 / Step 2 起步，`repr` 与 `memory_axioms` 骨架落地

关键结论：

- `sel4_cspace/src/repr/` 已新增一组最小表示层模块骨架：
  - `cap_repr.rs`
  - `cte_repr.rs`
  - `mdb_repr.rs`
  - `slot_repr.rs`
  - `resolve_ret_repr.rs`
  - `result_repr.rs`
- `sel4_cspace/src/memory_axioms.rs` 已新增，并先接住了
  `trusted_range_top_u128_if_small(...)` 这一类小粒度地址算术 helper；
- `sel4_cspace/src/lib.rs` 已把 `repr` 与 `memory_axioms` 作为新的 crate 内 verify 模块显式接入；
- `sel4_cspace/src/interface.rs` 已开始从新的 `repr::*` 落点取 capability view、result observer 和 lookup return type，
  而不是继续直接依赖 `refinement_bridge` 的原始命名；
- 重新运行
  - `bash tools/check-cspace-build-and-verify.sh`
  通过，结果保持为 `211 verified, 0 errors`。

这一天这一步的意义是：

- 当前不只是“写出了一份 7 步计划”，而是已经真的开始执行其中的 Step 1 和 Step 2；
- `refinement_bridge.rs` 虽然还没有被大规模拆空，但它已经不再是唯一默认落点；
- 这为下一步真正开始替换 `cte_insert` 等 mutator body 提前准备好了模块边界。

### 2026-04-30：Step 6A 起步，mutator 共享实现体与 `repr` vocabulary 上卷到 contract

关键结论：

- `sel4_cspace/src/cte.rs` 中四个 non-delete mutator 现已先收成“一份共享 Rust 更新体 + 两侧入口复用”：
  - `cte_insert_runtime_body`
  - `insert_new_cap_runtime_body`
  - `cte_move_runtime_body`
  - `cte_swap_runtime_body`
- 非 `verify` 构建和 `feature=verify` 下的 contract-bearing `external_body` 入口，
  现在都统一调用这组共享实现体，而不再各自维护一份 mutator 更新细节；
- `sel4_cspace/src/repr/` 也从单纯 re-export 骨架，进一步长出了本地 vocabulary：
  - `cap_view`
  - `cte_view`
  - `concrete_slot_view_at`
  - `exception_is_none`
  - `exception_is_syscall_error`
  - `derive_cap_ret_cap_view`
- `interface.rs` 与 `cte.rs` 里一批核心 exec contract 已开始改用这些 `repr` 命名，
  而不是继续直接把 `trusted_*` 名字当作最上层表述；
- 重新运行
  - `bash tools/check-cspace-build-and-verify.sh`
  通过，结果保持为 `211 verified, 0 errors`。

这一天这一步的意义是：

- mutator family 虽然还没有完成“去 `external_body` 化”，
  但已经先完成了“实现体不再双份维护”的收口；
- 这一步会直接降低下一轮把 `cte_insert` 往 verified body 推时的改动面；
- 同时，`repr` 层也已经开始从“内部准备层”进入真实 contract 口径，
  为继续把 `refinement_bridge` 降成 observer-only 过渡层打下基础。

### 2026-04-30：mutator public 入口脱离直接 `external_body`，验证数推进到 `215`

关键结论：

- `sel4_cspace/src/cte.rs` 中四个 non-delete mutator 的 public verify 入口：
  - `cte_insert`
  - `insert_new_cap`
  - `cte_move`
  - `cte_swap`
  现已不再直接标记为 `#[verifier::external_body]`；
- 当前结构改为：
  - public mutator 入口是普通 verified wrapper；
  - crate 内部保留更小的 `*_runtime_step(...)` 作为 `external_body` 过渡层；
  - 两侧共同复用同一份共享 Rust 更新体；
- 这意味着当前 remaining trust 已从“公共 mutator 名字本身”进一步收缩到“内部 runtime-step helper”；
- 重新运行
  - `bash tools/check-cspace-build-and-verify.sh`
  通过，Verus 结果更新为 `215 verified, 0 errors`。

这一天这一步的意义是：

- mutator family 现在已经不只是“public assumption 清零”；
- 它还进一步完成了“public `external_body` -> internal `external_body`”这一步收口；
- 这让下一轮真正把 `cte_insert` 继续往 fully verified body 推时，
  需要替换的对象已经从公共接口进一步缩小到内部 runtime-step helper。

### 2026-04-30：`interface.rs` mutator 改走 public verified wrapper，旧 refined backend 删除，验证数回到 `210`

关键结论：

- `sel4_cspace/src/interface.rs` 中四个 mutator verify-facing 入口：
  - `cte_insert_at`
  - `insert_new_cap_at`
  - `cte_move_at`
  - `cte_swap_at`
  现已不再直接调用 `cte.rs` 里的旧 `*_refined` backend；
- 当前结构改为：
  - `interface.rs` 直接调用 `cte.rs` 中的 public verified mutator wrapper；
  - 接口层自己基于 local heap transition 合同和 bridge lemma 完成最终 `spec_*` post 推导；
- 因此，`cte.rs` 里原先只服务这四个入口的旧 proof backend：
  - `cte_insert_refined_with_revocable`
  - `cte_insert_refined`
  - `insert_new_cap_refined`
  - `cte_move_refined`
  - `cte_swap_refined`
  已经可以直接删除；
- 重新运行
  - `bash tools/check-cspace-build-and-verify.sh`
  通过，Verus 结果为 `210 verified, 0 errors`。

这一天这一步的意义是：

- public verify-facing surface 现在真正对齐到了 public verified mutator wrapper，
  而不是继续穿透到旧 backend 名字；
- `cte.rs` 里这条非删除 mutator 线的 proof layering 更接近最终想要的形态：
  - public runtime/verify 入口
  - internal runtime-step trust shim
  - shared Rust body
- 验证数从 `215` 回到 `210` 不是回归，而是因为上面 5 个旧 proof 函数已被删除，
  当前统计结果更准确地反映了“仍然存在并被验证的对象”。

### 2026-04-30：mutator public wrapper 直接发布 final exec contract，旧 `*_exec_step` 删除，验证数回到 `206`

关键结论：

- `sel4_cspace/src/cte.rs` 中四个 non-delete mutator 的 public verified wrapper：
  - `cte_insert`
  - `insert_new_cap`
  - `cte_move`
  - `cte_swap`
  现在不再只承诺 “local heap transition”；
- 当前结构改为：
  - public mutator wrapper 直接给出最终 `*_exec_contract`；
  - `interface.rs` 对应的 `*_at` 包装只需消费这些 public contract，
    不再重复内联整段 bridge/post 推导；
  - 因此，原先只服务旧分层的内部函数：
    - `cte_insert_exec_step`
    - `insert_new_cap_exec_step`
    - `cte_move_exec_step`
    - `cte_swap_exec_step`
    已可以直接删除；
- 重新运行
  - `bash tools/check-cspace-build-and-verify.sh`
  通过，Verus 结果为 `206 verified, 0 errors`。

这一天这一步的意义是：

- mutator family 的公共证明入口已经从
  “public wrapper 只给 local transition，最终合同靠下一层再拼”
  推进到
  “public wrapper 自己就是 final exec contract 的发布点”；
- `interface.rs` 现在更接近稳定 facade 的角色，
  而不是继续承担 mutator 主证明骨架；
- 验证数从 `210` 回到 `206` 不是回归，而是因为又删除了 4 个
  已被新公共 contract 路线取代的旧内部 proof 函数。

### 2026-04-30：`cte_insert` 内部先拆出 `set_untyped_cap_as_full` 小边界，验证数保持 `206`

关键结论：

- `sel4_cspace/src/cte.rs` 中原先直接混在 `cte_insert` runtime body 里的
  `set_untyped_cap_as_full`
  现已拆成两层：
  - 共享 runtime body `set_untyped_cap_as_full_runtime_body(...)`
  - verify 侧小粒度 contract helper `set_untyped_cap_as_full(...)`
- 这个 helper 当前仍是 `#[verifier::external_body]`，
  但它现在只发布一个明确而局部的抽象 effect：
  - `src_slot` 的 `cap` 视图更新为
    `spec_set_untyped_cap_as_full_result(...)`
  - `mdb_prev / mdb_next / mdb_revocable / mdb_first_badged` 保持不变；
- 重新运行
  - `bash tools/check-cspace-build-and-verify.sh`
  通过，Verus 结果保持为 `206 verified, 0 errors`。

这一天这一步的意义是：

- 虽然 `cte_insert_runtime_step` 这层 internal trust shim 还没有继续拆掉，
  但 `cte_insert` 体内最像“业务语义 effect”的 untyped 更新已经先从大黑盒里分离出来；
- 这样下一轮继续拆 `cte_insert_runtime_step` 时，
  我们面对的主要剩余对象就更集中在 slot/link 写入，而不是还要把 untyped effect 混在一起解释；
- 这一步是把“整个 mutator opaque”推进到“mutator 内部已有第一颗被单独命名和约束的小边界”。

### 2026-04-30：继续拆 mutator 写入 effect，补出 slot-entry / mdb-link 小 helper，验证数保持 `206`

关键结论：

- `sel4_cspace/src/cte.rs` 里又补出一组更基础的小写入 helper：
  - 共享 runtime body：
    - `write_slot_entry_runtime_body(...)`
    - `set_slot_mdb_prev_runtime_body(...)`
    - `set_slot_mdb_next_runtime_body(...)`
  - verify 侧小粒度 contract helper：
    - `write_slot_entry(...)`
    - `set_slot_mdb_prev(...)`
    - `set_slot_mdb_next(...)`
- `cte_insert_runtime_body(...)` 现在不再直接散写：
  - `dest_slot.capability`
  - `dest_slot.cteMDBNode`
  - `src_slot.cteMDBNode.set_mdbNext(...)`
  而是统一走上面这组命名好的小 effect；
- 同一套 runtime helper 也已经开始被复用到：
  - `insert_new_cap_runtime_body(...)`
  - `cte_move_runtime_body(...)`
  - `cte_swap_runtime_body(...)`
  这让 mutator family 的 concrete 写法开始朝一个统一模板收；
- 重新运行
  - `bash tools/check-cspace-build-and-verify.sh`
  通过，Verus 结果保持为 `206 verified, 0 errors`。

这一天这一步的意义是：

- `cte_insert` 这条主线里，除了前一轮已经独立出来的
  `set_untyped_cap_as_full`
  之外，slot 整项写入和 MDB link 重写也都有了各自单独命名的 effect 边界；
- 当前 `cte_insert_runtime_step` 这层 internal trust shim 还没有完全拆掉，
  但它下面承接的 concrete effect 已经不再是一整坨匿名写内存操作；
- 下一轮如果继续做 `cte_insert_runtime_step` 的 verified glue 化，
  可以直接围绕这些小 helper 和 bridge/frame 条件来收，
  而不是重新从裸赋值和裸指针更新开始解释。

### 2026-04-30：补出抽象 slot-entry / state-update helper，并把 `cte_insert` spec 切到中间状态骨架，验证数更新为 `207`

关键结论：

- `sel4_cspace/specs/abstract_cspace.rs` 中新增了一组抽象层 helper：
  - `slot_entry_with_cap(...)`
  - `slot_entry_with_mdb_prev(...)`
  - `slot_entry_with_mdb_next(...)`
  - `slot_entry_written(...)`
  - `CSpaceState::with_slot_entry(...)`
  - `lemma_with_slot_entry_updates_only_target(...)`
- `sel4_cspace/src/cte.rs` 中前一轮补出的几个小写入 contract
  现在已经改成直接复用这些抽象 helper，
  不再继续在 `ensures` 里手工展开整段 `SlotEntrySpec { ... }`；
- `sel4_cspace/specs/cspace_ops/insert.rs` 中：
  - `spec_cte_insert_expected_src_entry(...)`
  - `spec_cte_insert_expected_dest_entry(...)`
  - `spec_insert_new_cap_expected_parent_entry(...)`
  也已经改成复用同一组抽象 helper；
- 同时又补出一组 `cte_insert` 的中间状态 spec：
  - `spec_cte_insert_state_after_set_untyped(...)`
  - `spec_cte_insert_state_after_write_dest(...)`
  - `spec_cte_insert_state_after_link_src(...)`
  - `spec_cte_insert_state_after_rewrite_next(...)`
  它们专门为后续把 `cte_insert_runtime_step` 继续拆成 verified glue 做准备；
- 重新运行
  - `bash tools/check-cspace-build-and-verify.sh`
  通过，Verus 结果更新为 `207 verified, 0 errors`。

这一天这一步的意义是：

- `cte_insert` 的 ghost-side 组织现在已经不只是“有几个小写入 helper”，
  还开始有一套明确的“每做一步后抽象状态长什么样”的中间状态骨架；
- 这会直接减少下一轮把 runtime-step 分解成多步证明时的样板工作，
  因为 slot-entry 更新和 frame 推理终于有了统一表达方式；
- 验证数从 `206` 到 `207` 不是语义扩张，而是因为这轮确实新增了一条抽象状态更新 proof helper。

### 2026-04-30：补出 `slots_unchanged_except` 的 weaken/transitive 工具，并把 `cte_insert` 中间状态变成完整 proof ladder，验证数更新为 `214`

关键结论：

- `sel4_cspace/specs/abstract_cspace.rs` 中新增了两条通用 frame proof 工具：
  - `lemma_slots_unchanged_except_weaken(...)`
  - `lemma_slots_unchanged_except_transitive(...)`
- `sel4_cspace/specs/cspace_ops/insert.rs` 中基于这些工具补出了一组真正可复用的 `cte_insert` 分步引理：
  - `lemma_cte_insert_pre_implies_old_next_not_dest(...)`
  - `lemma_cte_insert_state_after_set_untyped_step(...)`
  - `lemma_cte_insert_state_after_write_dest_step(...)`
  - `lemma_cte_insert_state_after_link_src_step(...)`
  - `lemma_cte_insert_state_after_rewrite_next_frame(...)`
- 这意味着当前 `cte_insert` 在 ghost 层已经不只是“有若干中间状态 spec 名字”，
  而是开始拥有一条从
  `set_untyped`
  到 `write_dest`
  再到 `link_src`
  最后到 `rewrite_next`
  的完整 proof ladder；
- 重新运行
  - `bash tools/check-cspace-build-and-verify.sh`
  通过，Verus 结果更新为 `214 verified, 0 errors`。

这一天这一步的意义是：

- 下一轮继续收 `cte_insert_runtime_step` 时，
  我们已经不需要临时手搓“前一步 frame 怎么传到后一步”的证明骨架；
- `slots_unchanged_except` 现在终于有了可以跨多步状态更新复用的组合工具，
  后面不管是 `cte_insert` 继续去 trust，还是平推 `insert_new_cap / cte_move / cte_swap`，
  都能直接吃到这套基础设施；
- 验证数从 `207` 到 `214` 对应的是这轮新增的一组通用 proof helper 与 step lemma，
  不是语义边界扩张。

### 2026-04-30：把 `cte_insert` 的 `old_next` 也收进 concrete contract，验证数更新为 `216`

关键结论：

- `sel4_cspace/specs/cspace_ops/insert.rs` 中新增：
  - `spec_cte_insert_expected_old_next_entry(...)`
  - `lemma_cte_insert_post_implies_expected_old_next_entry(...)`
- `sel4_cspace/src/refinement_bridge.rs` 中新增：
  - `lemma_cte_insert_local_heap_transition_post_implies_expected_old_next_view(...)`
- `sel4_cspace/src/cte.rs` 中的 `cte_insert_exec_contract(...)`
  现在不再只刻画：
  - `src`
  - `dest`
  这两个核心 slot 的 concrete post view；
  它在 `old_state.slot_entry(src).mdb_next is Some` 时，
  也会显式刻画第三个 changed slot `old_next` 的 concrete post view；
- `cte_insert(...)` 的 public verified wrapper 现已把这条新后置一并证明出来；
- 重新运行
  - `bash tools/check-cspace-build-and-verify.sh`
  通过，Verus 结果更新为 `216 verified, 0 errors`。

这一天这一步的意义是：

- 现在 `cte_insert` 的 verified contract 已经更接近真实局部更新语义，
  不再只盯住“插入前后两个主 slot”；
- 这会直接帮助下一轮继续收 `cte_insert_runtime_step`，
  因为 runtime body 里真正发生变化的第三个 slot 终于已经被拉进正式合同面；
- 验证数从 `214` 到 `216` 对应的是这轮新增的 expected-entry / bridge-view 引理，
  不是证明范围外扩。

## 3. 当前总结

到目前为止，可以用一句话总结当前状态：

`第一轮工程验证闭环已经完成；第二轮重构中的 P2、P3、P4、P5、P6、P7 也已经完成当前范围的首轮收口。当前基线已经同时具备通过的 Verus 回归结果、稳定的技术台账、可直接展开的论文草稿，以及一份明确的最终展示代码要求；下一步主线是继续按学校模板打磨正文，或转回技术主线推进更细粒度的 trusted boundary 收紧与 exec 替换工作。`

## 4. 归档说明

所有被合并的旧文档均保留在：

- `docs/verification/archive/`

如果后续需要追溯某一次阶段性判断、旧编号体系或某份详细台账，可以直接回到归档文件查看。

### 2026-04-30：补出 `cte_insert` staged transition composition 骨架，验证数更新为 `223`

这一步完成了三块直接服务于 `cte_insert_runtime_step` 后续去 trust 的基础设施。

- `sel4_cspace/src/refinement_bridge.rs` 中新增：
  - `lemma_trusted_cspace_slots_unchanged_except_at_transitive(...)`
  - `lemma_trusted_cspace_cnode_lookups_unchanged_at_transitive(...)`
  - `lemma_trusted_cspace_local_heap_transition_at_transitive(...)`
  它们把 local heap transition 的 slot-frame / lookup-frame / selected-slot-view 三部分组合能力补齐了。
- `sel4_cspace/specs/cspace_ops/insert.rs` 中新增：
  - `lemma_cte_insert_pre_post_implies_old_next_not_src(...)`
  - `lemma_cte_insert_post_implies_state_matches_rewrite_next(...)`
  这让 `spec_cte_insert_post(...)` 不再只是“后置条件成立”，
  而是可以继续收敛到 staged `rewrite_next` 终态本身。
- `sel4_cspace/src/cte.rs` 中补出一组小步 runtime helper：
  - `cte_insert_set_untyped_runtime_step(...)`
  - `cte_insert_write_dest_runtime_step(...)`
  - `cte_insert_link_src_runtime_step(...)`
  - `cte_insert_rewrite_old_next_runtime_step(...)`
  它们已经把 `cte_insert` runtime body 内部真正想拆开的四段 effect 先独立命名出来。

同时，这一步也明确暴露出一个工程现实：

- 目前直接把顶层 `cte_insert_runtime_step(...)` 完整改成 verified glue，
  还需要进一步解决“step-call 级旧值作用域”和 raw ref / heap observer 之间的连接方式；
- 因此当前这轮先保留顶层 `cte_insert_runtime_step(...)` 为稳定的 `external_body`，
  但把后续真正要用到的 staged proof / transition-composition / uniqueness 骨架先全部铺好。

重新运行：

- `bash tools/check-cspace-build-and-verify.sh`

通过，Verus 结果更新为 `223 verified, 0 errors`。

这一步的意义是：

- `cte_insert` 继续去 trust 时，已经不需要再先补 ghost-side 基础设施；
- 下一轮可以直接围绕 staged helper 和 composition lemma 去收最外层 runtime-step；
- 虽然顶层 trust shim 这轮还没完全拿掉，但“怎么拆、拆完靠什么合成回来”已经不再模糊。

### 2026-04-30：把同一套 staged 思路平推到 `insert_new_cap`，验证数更新为 `229`

这一步把上一轮先在 `cte_insert` 上铺开的 ghost-side 方法，继续推到更短的 `insert_new_cap` 主线上。

- `sel4_cspace/specs/cspace_ops/insert.rs` 中新增：
  - `spec_insert_new_cap_state_after_write_slot(...)`
  - `spec_insert_new_cap_state_after_rewrite_next(...)`
  - `spec_insert_new_cap_state_after_link_parent(...)`
  - `lemma_insert_new_cap_pre_implies_old_next_not_slot(...)`
  - `lemma_insert_new_cap_state_after_write_slot_step(...)`
  - `lemma_insert_new_cap_state_after_rewrite_next_frame(...)`
  - `lemma_insert_new_cap_state_after_link_parent_frame(...)`
  - `lemma_insert_new_cap_pre_post_implies_old_next_not_parent(...)`
  - `lemma_insert_new_cap_post_implies_state_matches_link_parent(...)`
- 这意味着 `insert_new_cap` 现在也不再只有“最终 post 条件”；
  它已经具备了与真实 runtime 顺序对齐的三段中间状态，以及一条从 post 收敛回最终 staged 终态的唯一化证明。
- `sel4_cspace/src/cte.rs` 中又补出三段与 runtime 顺序一一对应的小步 helper：
  - `insert_new_cap_write_slot_runtime_step(...)`
  - `insert_new_cap_rewrite_old_next_runtime_step(...)`
  - `insert_new_cap_link_parent_runtime_step(...)`
  当前顶层 `insert_new_cap_runtime_step(...)` 仍先保持为稳定的 `external_body`，
  但下一轮如果直接收这层 trust shim，已经不需要再先做“小步 effect 命名”和“ghost 中间状态对齐”。

重新运行：

- `bash tools/check-cspace-build-and-verify.sh`

通过，Verus 结果更新为 `229 verified, 0 errors`。

这一步的意义是：

- `insert_new_cap` 现在已经成为比 `cte_insert` 更短、也更适合先试做 verified glue 的模板项；
- 后面无论我们先收 `insert_new_cap_runtime_step(...)`，还是拿它反哺 `cte_insert_runtime_step(...)`，
  都已经有一套对齐 runtime 顺序的 staged vocabulary 可以直接复用；
- mutator family 的“先补最终 post，再补中间状态，再补小步 helper，最后收顶层 trust shim”这条路线，
  现在不再只停留在 `cte_insert` 一个点上。
