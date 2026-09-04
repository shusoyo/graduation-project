# CSpace Verus 替换实施计划（非删除主线，当前执行版）

状态：当前执行文档，2026-05-01

## 1. 文档定位

这份文档不是新的总计划，也不是论文草稿替代品，而是当前这一轮“对着文档直接写代码”的执行型工作单。

它与另外几份文档的分工是：

- `cspace-verification-plan.md`
  - 负责总目标、当前状态、TCB 台账、论文口径与长期边界。
- `cspace-session-log.md`
  - 负责记录已经发生过的阶段推进与口径变化。
- `cspace-thesis-draft.md`
  - 负责论文正文材料。
- 本文档
  - 只负责“如果目标是最终用 Verus 替换非删除主线的 `sel4_cspace`，那下一轮代码到底怎么做”。

因此，本文档默认服务于一个更强的技术目标：

`在暂不推进 delete / revoke / finalise 主线的前提下，把 sel4_cspace 的非删除主线尽量推进到“l4v 决定语义，Verus 决定实现体”的状态。`

## 2. 当前轮次的明确范围

### 2.1 当前要做的事

当前轮次只推进非删除主线，也就是：

- capability query：
  - `same_region_as`
  - `same_object_as`
  - `is_cap_revocable`
- slot-local query / derivation：
  - `is_mdb_parent_of`
  - `is_final_cap`
  - `ensure_no_children`
  - `derive_cap`
- lookup：
  - `resolve_address_bits`
- mutator family：
  - `cte_insert`
  - `insert_new_cap`
  - `cte_move`
  - `cte_swap`

### 2.2 当前明确不做的事

下面这些内容在本轮冻结，不作为当前执行目标：

- `finalise`
- `delete_all`
- `reduce_zombie`
- `revoke`
- `boundary_assumptions.rs` 中专门为 delete 主线保留的流程假设

这条冻结规则的含义不是“以后不做”，而是：

- 当前先把非删除主线打造成一个更干净的 Verus-native 基础；
- 删除主线等前 6 步稳定后再开；
- 本轮不为了 delete 路径扩张新的 bridge 语义层或临时 TCB。

### 2.3 本轮的停止条件

当下面这些条件同时成立时，本轮代码工作就可以暂停，不必继续往 delete 主线扩：

- capability query 三项不再维持“两套主实现”；
- `is_mdb_parent_of / is_final_cap / ensure_no_children / derive_cap`
  的主实现不再主要依赖 `external_body + refined wrapper`；
- `resolve_address_bits` 进入 Verus-native body 主路径；
- `cte_insert / insert_new_cap / cte_move / cte_swap`
  进入 Verus-native body 主路径；
- `refinement_bridge.rs` 不再承担核心 CSpace 语义恢复，只剩 observer / extractor / constructor witness；
- 非删除主线的 build object 与 verified object 基本对齐，或者只剩极薄、可解释的 runtime 适配层。

换句话说：

`只要前 6 步做完，就可以先停；第 7 步 delete 主线不属于这一轮的必做收尾条件。`

## 3. 当前出发点

### 3.1 当前已经具备的基础

截至 2026-05-01，当前代码已经具备：

- `bash tools/check-cspace-build-and-verify.sh` 通过；
- Verus 回归结果为 `249 verified, 0 errors`；
- `cte.rs` 中 raw `assume_specification[...]` 数量已经清零；
- mutator family 的 public/runtime-step 分层已从 raw assumption
  过渡到 contract-bearing staged glue；
- `cte_insert` 路径中的 `set_untyped_cap_as_full`
  现已进一步拆成：
  - 共享 runtime body `set_untyped_cap_as_full_runtime_body(...)`
  - verify 侧小粒度 contract helper `set_untyped_cap_as_full(...)`
  这使得 `cte_insert` 体内的 untyped effect 不再继续与整段 mutator trust shim 混成一块；
- mutator 写入侧现在又补出一组更基础的小 effect helper：
  - `write_slot_entry_runtime_body(...)` / `write_slot_entry(...)`
  - `set_slot_mdb_prev_runtime_body(...)` / `set_slot_mdb_prev(...)`
  - `set_slot_mdb_next_runtime_body(...)` / `set_slot_mdb_next(...)`
  它们把“整项写入 slot”和“只改一条 MDB link”从 `cte_insert` 大体内部继续分离出来，
  同时也开始为 `insert_new_cap / cte_move / cte_swap` 统一 runtime 写法；
- 抽象状态层现又补出一组中间状态/slot-entry helper：
  - `slot_entry_with_cap(...)`
  - `slot_entry_with_mdb_prev(...)`
  - `slot_entry_with_mdb_next(...)`
  - `slot_entry_written(...)`
  - `CSpaceState::with_slot_entry(...)`
  - `lemma_with_slot_entry_updates_only_target(...)`
  它们已经开始被 `set_untyped_cap_as_full` / `write_slot_entry` 小 contract，
  以及 `specs/cspace_ops/insert.rs` 中的 expected-entry / mid-state spec 复用；
- `slots_unchanged_except(...)` 现又补出两条可复用 frame proof 工具：
  - `lemma_slots_unchanged_except_weaken(...)`
  - `lemma_slots_unchanged_except_transitive(...)`
  同时 `specs/cspace_ops/insert.rs` 也补出了一套 `cte_insert` 分步状态 proof ladder，
  让 `set_untyped -> write_dest -> link_src -> rewrite_next` 这四步都能在 ghost 层被分开组织；
- `cte_insert` 的 concrete 后置合同现在又进一步补上了第三个 changed slot：
  - `spec_cte_insert_expected_old_next_entry(...)`
  - `lemma_cte_insert_post_implies_expected_old_next_entry(...)`
  - `lemma_cte_insert_local_heap_transition_post_implies_expected_old_next_view(...)`
  - `cte_insert_exec_contract(...)` 现已在 `old_next is Some` 时显式刻画该 slot 的 concrete view；
- `refinement_bridge.rs` 现又补出一组 local-heap-transition 组合工具：
  - `lemma_trusted_cspace_slots_unchanged_except_at_transitive(...)`
  - `lemma_trusted_cspace_cnode_lookups_unchanged_at_transitive(...)`
  - `lemma_trusted_cspace_local_heap_transition_at_transitive(...)`
  同时 `specs/cspace_ops/insert.rs` 也补出：
  - `lemma_cte_insert_pre_post_implies_old_next_not_src(...)`
  - `lemma_cte_insert_post_implies_state_matches_rewrite_next(...)`
  这让 `cte_insert` 的 staged ghost-state 不再只是“有几段中间状态名字”，
  而是已经具备可组合的 heap/frame proof 骨架；
- `cte.rs` 中也已补出一组面向后续去 trust 的小步 helper：
  - `cte_insert_set_untyped_runtime_step(...)`
  - `cte_insert_write_dest_runtime_step(...)`
  - `cte_insert_link_src_runtime_step(...)`
  - `cte_insert_rewrite_old_next_runtime_step(...)`
  随后这一条链也已进一步收口为真实的顶层 verified glue：
  - `cte_insert_runtime_step(...)` 已不再是顶层 `external_body`；
  - 顶层现在显式顺序组合
    `set_untyped -> write_dest -> link_src -> rewrite_old_next`
    四个 micro-step helper，并在 proof 中用 staged abstract state、
    transitive local-heap-transition lemma 和
    `trusted_slot_mdb_next_addr(...)`
    把中间态串到最终 `cte_insert_local_heap_transition_at(...)`；
  - 为了让这条顺序组合真正可用，
    `cte_insert_write_dest_runtime_step(...)`
    与 `cte_insert_link_src_runtime_step(...)`
    的小步合同也已改成可顺序组合的 local-heap-transition 形状；
- `insert_new_cap` 这条更短的 mutator 链当前也已经补出与 runtime 顺序对齐的 staged 骨架：
  - `spec_insert_new_cap_state_after_write_slot(...)`
  - `spec_insert_new_cap_state_after_rewrite_next(...)`
  - `spec_insert_new_cap_state_after_link_parent(...)`
  - `lemma_insert_new_cap_state_after_write_slot_step(...)`
  - `lemma_insert_new_cap_state_after_rewrite_next_frame(...)`
  - `lemma_insert_new_cap_state_after_link_parent_frame(...)`
  - `lemma_insert_new_cap_post_implies_state_matches_link_parent(...)`
  同时 `cte.rs` 中也补出：
  - `insert_new_cap_write_slot_runtime_step(...)`
  - `insert_new_cap_rewrite_old_next_runtime_step(...)`
  - `insert_new_cap_link_parent_runtime_step(...)`
  随后这一条链已经进一步收口为真实的顶层 verified glue：
  - `insert_new_cap_runtime_step(...)` 已不再是顶层 `external_body`；
  - 顶层现在显式顺序组合
    `write_slot -> rewrite_old_next -> link_parent`
    三个 micro-step helper，并在 proof 中用 staged abstract state、
    transitive / weaken local-heap-transition lemma 和
    `trusted_slot_mdb_next_addr(...)`
    把中间态串到最终 `insert_new_cap_local_heap_transition_at(...)`；
  - 因而 `insert_new_cap` 现已成为 mutator family 里第一条
    “public verified wrapper + internal verified runtime-step glue”
    的模板路径。
- `cte_move` 这条 mutator 链当前也已补出与 proof-friendly 顺序对齐的 staged 骨架：
  - `spec_cte_move_state_after_write_dest(...)`
  - `spec_cte_move_state_after_clear_src(...)`
  - `spec_cte_move_state_after_rewrite_prev(...)`
  - `spec_cte_move_state_after_rewrite_next(...)`
  - `lemma_cte_move_state_after_write_dest_step(...)`
  - `lemma_cte_move_state_after_clear_src_step(...)`
  - `lemma_cte_move_state_after_rewrite_prev_frame(...)`
  - `lemma_cte_move_state_after_rewrite_next_frame(...)`
  同时 `cte.rs` 中也已补出：
  - `cte_move_write_dest_runtime_step(...)`
  - `cte_move_clear_src_runtime_step(...)`
  - 复用 `cte_move_rewrite_old_prev_runtime_step(...)`
    / `cte_move_rewrite_old_next_runtime_step(...)`
    这组小步 link-rewrite helper
  随后这一条链也已进一步收口为真实的顶层 verified glue：
  - `cte_move_runtime_step(...)` 已不再是顶层 `external_body`；
  - 顶层现在显式顺序组合
    `write_dest -> clear_src -> rewrite_old_prev -> rewrite_old_next`
    四个 micro-step，并在 proof 中用 staged abstract state、
    transitive local-heap-transition lemma、
    `trusted_slot_mdb_prev_addr(...)`
    与 `trusted_slot_mdb_next_addr(...)`
    把中间态串到最终 `cte_move_local_heap_transition_at(...)`；
- `cte_swap` 当前也已进一步收口为真实的顶层 verified glue：
  - `spec_cte_swap_state_after_write_slots(...)`
  - `spec_cte_swap_state_after_slot1_prev(...)`
  - `spec_cte_swap_state_after_slot1_next(...)`
  - `spec_cte_swap_state_after_slot2_prev(...)`
  - `spec_cte_swap_state_after_slot2_next(...)`
  - `lemma_cte_swap_runtime_changed_slots_eq_spec_changed(...)`
  - `cte_swap_write_slots_runtime_step(...)`
  - `trusted_slot_addr(...)`
  - 以及对应的 runtime changed-set / frame lemma
  随后这一条链也已进一步收口为真实的顶层 verified glue：
  - `cte_swap_runtime_step(...)` 已不再是顶层 `external_body`；
  - 顶层现在显式顺序组合
    `write_swapped_slots -> rewrite_slot1_prev -> rewrite_slot1_next -> rewrite_slot2_prev -> rewrite_slot2_next`
    五个 micro-step，并在 proof 中用 staged abstract state、
    transitive local-heap-transition 与 changed-set 对齐引理
    串到最终 `cte_swap_local_heap_transition_at(...)`；
  - 当前 `cte_swap` 这条线上仍额外保留了一个 proof-side staging scaffold：
    `lemma_cte_swap_post_implies_staged_final_state(...)`；
    它不再属于 runtime raw assumption，
    但仍是下一步继续收紧的 spec-side helper。
- `interface.rs` 已形成稳定的 verify-facing facade；
- `refinement_bridge.rs` 已收回 crate 内部，不再作为公开 proof surface；
- capability query 三项已经进入“真实 `feature=verify` 接口”阶段。

### 3.2 当前离“最终 Verus 替换”还差什么

虽然当前基线已经完成，但它离“最终用 Verus 替换 `sel4_cspace` 非删除主线”还有四类本质差距：

1. 表示层差距：
   - 目前 `cap / cte_t / mdb_node / slot` 的 concrete-to-abstract 连接还主要集中在 `refinement_bridge.rs`。
2. 主实现差距：
   - 多数关键入口虽然已经有稳定合同，但主实现仍不是 Verus 直接验证的 body。
3. 路径分裂差距：
   - capability query 与若干入口仍同时保留 runtime path 与 verify path。
4. TCB 职责差距：
   - bridge 现在虽然已经收缩，但仍承担了一批高频视图提取、slot 观察和返回值投影工作。

所以当前下一轮的任务，不是再写一层新的 wrapper，而是把“已经证明了合同”继续推进到“实现体本身就是 Verus 证明对象”。

## 4. 参考路线

### 4.1 l4v：决定语义目标

上层语义继续以 `aux/l4v-master` 为基线，尤其是：

- `SEL4/Object/ObjectType.lhs`
- `SEL4/Object/CNode.lhs`
- `SEL4/Kernel/CSpace.lhs`

这里回答的是：

- 这些操作语义上应该证明什么；
- 抽象 case split 应该怎么组织；
- `query / derive / lookup / mutator` 的语义边界应该怎么切。

### 4.2 atmosphere：决定 mutator contract 的写法风格

`/workspace/aux/atmosphere-main` 当前最值得借鉴的是：

- `*_unchanged`
- `*_unchanged_except`
- 在真实更新 body 后逐条重建 invariant 的组织方式

对应参考：

- `/workspace/aux/atmosphere-main/kernel/verified/process_manager/spec_util.rs`
- `/workspace/aux/atmosphere-main/kernel/verified/process_manager/impl_new_thread.rs`

这条参考路线最适合用在：

- `cte_insert`
- `insert_new_cap`
- `cte_move`
- `cte_swap`

也就是“先定义 frame / unchanged-except，再证明具体更新后的局部保持性”。

### 4.3 vostd：决定表示层与 owner/view/model 的写法风格

`/workspace/aux/vostd-main` 当前最值得借鉴的是：

- `repr / owner / view / model` 的分层；
- 把最难直接验证的表示转换压缩到很小的边界里；
- 让主语义尽量基于 owner/model proof 展开，而不是一直靠 bridge 复原。

对应参考：

- `/workspace/aux/vostd-main/ostd/specs/mm/frame/linked_list/linked_list_specs.rs`
- `/workspace/aux/vostd-main/ostd/specs/mm/frame/linked_list/linked_list_owners.rs`

这条参考路线尤其适合用在：

- `mdb_node` 的前后驱关系；
- slot 到 `cte_t` 的观察；
- `cap / cte / ret` 的只读视图抽取；
- 后续如果要继续压缩 trusted extractor，这会是最自然的风格来源。

## 5. 总体工程原则

后续写代码时，统一遵循下面这些原则。

### 5.1 一层只做一类事

- `specs/*`
  - 只定义抽象语义、抽象状态、局部 invariant、primitive spec。
- `src/repr/*` 或等价的表示层模块
  - 只处理 concrete representation、slot observer、pointer/address 连接、return view。
- `cte.rs` / `capability/mod.rs`
  - 只放真实实现体和紧贴实现体的 proof。
- `interface.rs`
  - 只放对外稳定接口，不继续暴露 proof backend 内脏。

### 5.2 bridge 只准做 observer，不准做 semantic oracle

任何新 trusted helper，如果它表达的是：

- “某个 concrete 值长什么样”
- “某个 slot/address 对应到哪个对象”
- “某个返回值观察到的状态是什么”

那它可以留在表示层。

但如果它表达的是：

- `same_region_as`
- `derive_cap`
- `resolve_address_bits`
- `cte_insert`

这类高层语义本身，就不应该继续放进 trusted 层。

### 5.3 同一个入口，最终应尽量只有一个主实现

最终目标不是：

- 一套 runtime Rust body；
- 一套 verify wrapper；
- 一套 bridge 解释层；

而是尽量收敛成：

- 一个 Verus-native 主实现体；
- 如果必须保留 runtime 适配，则只允许保留很薄、可解释的一层。

### 5.4 新增 TCB 只能收缩，不能扩张

如果一个 helper 只是临时推进用的，但未来没有清楚的收紧方向，那就不应该新增。

优先级是：

1. 先尝试把语义写回 `specs/*` + Verus body；
2. 实在不行，再把最小 observer / extractor 留进表示层；
3. 不要把“为了先过验证”的高层语义判断重新塞回 trusted 模块。

## 6. 建议的新模块落点

这一节不是要求一口气把目录全建完，而是给当前轮次一个清楚的落点规划，避免继续把所有东西都堆回 `refinement_bridge.rs`。

### 6.1 最小新增模块建议

建议优先考虑下面这些模块：

- `sel4_cspace/src/repr/mod.rs`
- `sel4_cspace/src/repr/cap_repr.rs`
- `sel4_cspace/src/repr/cte_repr.rs`
- `sel4_cspace/src/repr/mdb_repr.rs`
- `sel4_cspace/src/repr/slot_repr.rs`
- `sel4_cspace/src/repr/resolve_ret_repr.rs`
- `sel4_cspace/src/memory_axioms.rs`

其中：

- `repr/*`
  - 负责视图、observer、owner/model 连接；
- `memory_axioms.rs`
  - 负责暂时无法在当前内存模型下完全消掉的地址算术、裸指针身份和小粒度构造假设。

### 6.2 当前 trusted helper 的建议迁移方向

| 当前 helper | 建议未来落点 | 长期职责 |
| --- | --- | --- |
| `trusted_view_cap` / `trusted_extract_cap` | `repr/cap_repr.rs` | capability 的只读表示投影 |
| `trusted_view_cte` / `trusted_extract_cte` | `repr/cte_repr.rs` | `cte_t` 的只读表示投影 |
| `trusted_view_resolve_address_bits_ret` / `trusted_extract_resolve_address_bits_ret` | `repr/resolve_ret_repr.rs` | lookup 返回值投影 |
| `trusted_slot_ref_from_addr` / `trusted_slot_ref_is_id` | `repr/slot_repr.rs` 或 `memory_axioms.rs` | slot identity / address 连接 |
| `trusted_cap_ref_from_slot` | `repr/slot_repr.rs` | slot 到 capability 的局部 observer |
| `trusted_concrete_slot_view_at` | `repr/cte_repr.rs` 或 `repr/slot_repr.rs` | 某 slot 的 concrete 观察 |
| `trusted_concrete_cnode_lookup_slot_at` | `repr/slot_repr.rs` | lookup 使用的 slot 观察 |
| `trusted_make_*` / `trusted_check_*` | `repr/resolve_ret_repr.rs` 或更小 constructor 模块 | 结果构造和状态观察 |
| `trusted_range_top_u128_if_small` | `memory_axioms.rs` | 小粒度地址范围算术 |

完成 Step 2 后，原则上 `refinement_bridge.rs` 应逐步退化成：

- 过渡期 re-export；
- 少量兼容 wrapper；
- 或直接被拆空后删除。

## 7. 七步实施计划

下面这七步是工作包，不是七次提交，也不是七个自然日。每一步都给出目标、代码落点、交付物与完成标准。

### Step 1：重画模块边界，先把“语义层 / 表示层 / 公共接口层”分开

#### 目标

先把当前代码里的职责边界重新画清楚，避免后面一边写 Verus body，一边继续把 observer、语义 helper 和 facade 混在一起。

#### 为什么这一步必须先做

如果不先分层，后面的每一步都会继续发生两件坏事：

- `cte.rs` 越写越像“运行时实现 + proof backend + bridge 接头间”的混合体；
- `refinement_bridge.rs` 越收越难收，因为所有过渡 helper 都会自然地被塞回去。

#### 主要文件

- `sel4_cspace/src/lib.rs`
- `sel4_cspace/src/interface.rs`
- `sel4_cspace/src/cte.rs`
- `sel4_cspace/src/capability/mod.rs`
- `sel4_cspace/src/refinement_bridge.rs`

#### 建议动作

1. 为表示层预留明确模块落点，例如 `src/repr/*`。
2. 明确 `interface.rs` 的唯一职责是稳定对外验证接口和最终公共入口。
3. 把仅用于表示投影的 helper 从“语义桥”命名改成“repr / observer”口径。
4. 让 `cte.rs` 更聚焦于：
   - 非删除主线原语 body；
   - 紧贴 body 的 proof；
   - 少量局部 helper。
5. 不再让 `refinement_bridge.rs` 承担新的长期职责。

#### 交付物

- 模块职责在代码结构上可见，而不只写在注释里；
- 至少建出最小的表示层模块骨架，哪怕一开始只是 re-export 或占位；
- `interface.rs` 不再继续吸收内部 proof backend 命名。

#### 完成标准

- 后续新增 helper 时，可以明确回答它属于：
  - `specs`
  - `repr`
  - `body/proof`
  - `interface`
  四者中的哪一层；
- `refinement_bridge.rs` 不再是默认落点。

#### 暂时不要做

- 不要在这一步重写 delete 主线；
- 不要在这一步大改抽象语义；
- 不要为了“看起来整洁”先移动一堆无关代码。

### Step 2：建立 Verus-native 的 `repr / owner / view / model` 基础层

#### 目标

为 `cap / cte_t / mdb_node / slot / resolve ret` 建立一套最小但可扩展的 Verus-native 表示层，让后续语义证明尽量基于这些本地视图展开，而不是持续依赖 bridge 复原。

#### 为什么这是替换工作的真正地基

只要表示层仍是：

- 一个大 bridge 文件；
- 若干 trusted extractor；
- 若干 ad-hoc slot observer；

那么后面无论 `cte_insert` 还是 `resolve_address_bits`，都很难真正变成“实现体已证”，因为语义解释权还不在 Verus body 手里。

#### 主要文件

- 新增：`sel4_cspace/src/repr/*`
- 新增：`sel4_cspace/src/memory_axioms.rs`
- 过渡修改：`sel4_cspace/src/refinement_bridge.rs`
- 可能需要同步：`sel4_cspace/src/lib.rs`

#### 建议动作

1. 先为 capability 建最小只读 view：
   - tag
   - object identity 相关字段
   - badge / guard / radix / ptr 等被 query 和 lookup 真正使用的字段
2. 为 `cte_t` 建 slot-local view：
   - capability view
   - `mdb_prev`
   - `mdb_next`
   - `mdb_revocable`
   - `mdb_first_badged`
3. 为 `mdb_node` 明确一个局部 view / model 层，而不是总在 `cte_t` observer 里顺手拆。
4. 为 slot/address 连接单独做一个 observer 层：
   - `SlotId <-> addr`
   - `addr -> &cte_t`
   - `slot -> current cap`
5. 为 `resolve_address_bits` 返回值单独做 ret view，不再把 lookup 结果观察全塞进 bridge。
6. 把地址算术、identity 和不得不信任的小 lemma 压进 `memory_axioms.rs`，不要继续散落。

#### 交付物

- 最小 `repr/*` 模块存在并可被 `cte.rs` / `capability/mod.rs` 使用；
- 旧 trusted helper 至少开始分批迁出 bridge；
- slot / cap / cte / ret 这些 observer 的落点不再含糊。

#### 完成标准

- 后续写 `same_region_as`、`cte_insert`、`resolve_address_bits` 时，不需要再新增“带语义色彩的 bridge helper”；
- 每一个 remaining trusted helper 都能明确回答：
  - 它观察的 concrete 对象是什么；
  - 它不负责哪些高层语义；
  - 未来还能否继续收缩。

#### 暂时不要做

- 不要一上来追求所有 extractor 全部消失；
- 不要把 owner/model 设计搞成整套大框架后再落第一个 proof；
- 不要为了仿照 `vostd` 而机械复制完全相同的抽象层次。

### Step 3：把 capability query 三项真正收成“一套主实现”

#### 目标

把下面三项从“verify path 已经很强，但 runtime / verify 仍分离”推进到“同一个 Verus-native 主实现承担语义责任”：

- `same_region_as`
- `same_object_as`
- `is_cap_revocable`

#### 为什么这一步不能只满足于当前状态

这三项现在已经最接近 fully replaced，但它们仍然没有完全实现：

- 同一入口；
- 同一实现语义；
- 同一条最终对外模块口径。

如果这里不收合，后面更复杂的 `derive_cap` 和 `is_final_cap` 也会继续踩在分裂路径上。

#### 主要文件

- `sel4_cspace/src/capability/mod.rs`
- `sel4_cspace/src/interface.rs`
- `sel4_cspace/src/repr/cap_repr.rs`
- `sel4_cspace/specs/abstract_cspace.rs`
- `sel4_cspace/specs/cspace_ops/queries.rs`

#### 建议动作

1. 把这三项内部都改成统一依赖 capability view，而不是一部分走 runtime helper，一部分走 verify wrapper。
2. 明确保留还是删除 `*_runtime`：
   - 如果保留，它们只能是薄适配层；
   - 不应继续承载另一套主语义。
3. 把 `feature=verify` 下已经存在的强语义入口，继续往“最终对外接口”方向收。
4. 不为这三项新增新的 trusted semantic helper。

#### 交付物

- 三项 capability query 的主逻辑基于同一套 view/helper；
- runtime 与 verify path 之间只剩构建适配差异，而不是语义差异；
- 文档和代码都能明确说“这三项已经基本完成主实现收合”。

#### 完成标准

- 这三项不再需要靠“旁边另有 refined 版本”才能说自己被验证；
- 外部描述时可以自然说：
  - 主接口就是已验证对象；
  - 如果还留 runtime 兼容层，它不是另一套语义主体。

#### 暂时不要做

- 不在这一步扩 ArchCap 全覆盖；
- 不在这一步为 capability query 引入新的 bridge 前置条件层。

### Step 4：替换 slot-local query / derivation 链

#### 目标

把下面这一簇从“external + refinement 结构”推进到“Verus-native body 主导”：

- `is_mdb_parent_of`
- `is_final_cap`
- `ensure_no_children`
- `derive_cap`

补充说明：

- `is_long_running_delete` 虽然技术上也在这一簇附近，但它属于 delete gate；
- 当前轮次不主动扩它；
- 只要求它在重构中不被破坏，不把它列为本轮推进对象。

#### 为什么这一步重要

这一簇是 capability query、MDB 局部关系、slot 观察和派生语义的汇合点。

只要这一步没有完成，当前 `sel4_cspace` 仍然更像：

- capability query 已经比较像 Verus-native；
- 但真正串起局部语义主线的中层函数，仍主要由 refinement wrapper 托住。

#### 主要文件

- `sel4_cspace/src/cte.rs`
- `sel4_cspace/src/interface.rs`
- `sel4_cspace/src/repr/cte_repr.rs`
- `sel4_cspace/src/repr/mdb_repr.rs`
- `sel4_cspace/specs/cspace_ops/queries.rs`
- `sel4_cspace/specs/cspace_ops/derive.rs`
- `sel4_cspace/specs/abstract_cspace.rs`

#### 建议动作

1. 先把 `mdb_parent_of` 依赖的局部观察收回到 `cte/mdb` view。
2. 再把 `is_final_cap` 明确成：
   - 邻接关系观察；
   - `same_object_as` 语义；
   - 局部 case split；
   而不是外部大 wrapper。
3. 把 `ensure_no_children` 继续收紧成：
   - `mdb_next` 局部观察；
   - `mdb_parent_of(slot, next)` 判断；
   - 返回 status 与抽象条件的直接对应。
4. 让 `derive_cap` 直接在 body 中满足：
   - 成功返回什么 cap；
   - 失败返回什么 status；
   - 哪些 case 仍是当前受限 arch hook。

#### 交付物

- 这一簇的主证明不再以 `*_refined` 为主要 public story；
- `derive_cap` 与 `ensure_no_children` 形成一条本地可解释的 capability derivation 主线；
- `is_final_cap` 和 `is_mdb_parent_of` 成为局部判断组件，而不是外围证明黑盒。

#### 完成标准

- 这四项的语义都能直接在实现体附近解释清楚；
- 对外描述时，不需要先解释一个 proof-only backend，再解释真正接口。

#### 暂时不要做

- 不要借机把 delete / finalise 全链路带进来；
- 不要为了 `is_long_running_delete` 再扩一轮 delete-gate proof surface。

### Step 5：替换 `resolve_address_bits`

#### 目标

让 `resolve_address_bits` 从“lookup 语义已证明，但实现体仍未真正被 Verus 接管”推进到“Verus-native body 直接承担 lookup 语义”。

#### 为什么它值得单列

`resolve_address_bits` 是非删除主线里控制流最复杂的只读原语，也是 bridge 目前仍较明显参与结果投影的地方。

如果这一步做成了，基本就证明：

- 当前项目已经不仅能做 query；
- 也能做非平凡的 lookup 主线替换。

#### 主要文件

- `sel4_cspace/src/cte.rs`
- `sel4_cspace/src/interface.rs`
- `sel4_cspace/src/repr/slot_repr.rs`
- `sel4_cspace/src/repr/resolve_ret_repr.rs`
- `sel4_cspace/specs/cspace_ops/resolve.rs`

#### 建议动作

1. 继续保留 l4v 风格的 one-step / recursive 语义分解，但把主控制流搬进 Verus body。
2. 给 lookup 结果一个更本地的 ret view，不再主要依赖 trusted return projection。
3. 把当前 bridge 里的 result/core 对齐关系尽量下沉到：
   - spec helper；
   - 实现体后的直接 proof；
   而不是继续堆桥接包装。
4. 如果需要中间结构，可以引入小的 lookup cursor / frame ghost 模型，但不要搞成大框架。

#### 交付物

- `resolve_address_bits` 的 body 成为主要证明对象；
- 结果观察口径尽量由本地 ret view 负责；
- bridge 不再主导 lookup 结果解释。

#### 完成标准

- 外部再说 `resolve_address_bits` 时，不是“opaque body 满足 spec”；
- 而是“Verus body 直接实现并满足 lookup spec”。

#### 暂时不要做

- 不要顺手扩到更大范围的 cnode/invs 整体证明；
- 不要为了追求一步到位把所有 lookup 相关 trusted helper 同时归零。

### Step 6：替换 mutator family

#### 目标

把下面四个更新原语从“contract-bearing `external_body`”推进到“实现体本身由 Verus 证明”：

- `cte_insert`
- `insert_new_cap`
- `cte_move`
- `cte_swap`

#### 为什么这是非删除主线里最关键的一步

只要 mutator family 还没真正换成 verified body，当前项目就仍然主要是：

- 规格很强；
- 合同很清楚；
- 但核心更新仍未由 Verus 接管。

真正要说“最终用 Verus 替换 `sel4_cspace` 非删除主线”，这一组必须做完。

#### 主要文件

- `sel4_cspace/src/cte.rs`
- `sel4_cspace/src/repr/cte_repr.rs`
- `sel4_cspace/src/repr/mdb_repr.rs`
- `sel4_cspace/src/repr/slot_repr.rs`
- `sel4_cspace/src/memory_axioms.rs`
- `sel4_cspace/specs/cspace_ops/common.rs`
- `sel4_cspace/specs/cspace_ops/insert.rs`
- `sel4_cspace/specs/cspace_ops/move.rs`
- `sel4_cspace/specs/cspace_ops/swap.rs`

#### 建议动作

1. 先把 `cte_insert` 当模板项。
2. 为 mutator family 建更统一的 frame 写法，参考 `atmo` 风格：
   - `slots_unchanged_except`
   - `mdb_fields_unchanged_except`
   - `caps_unchanged_except`
   - 或等价的局部 unchanged predicate
3. 把 slot 写入、mdb 更新、cap 复制这些动作拆成更小的本地步骤。
4. 如果当前内存模型下某个写操作确实无法直接证，就把最小必要假设压进 `memory_axioms.rs`，而不是继续让整个 mutator 保持 `external_body`。
5. 在 `cte_insert` 成形后，再平推：
   - `insert_new_cap`
   - `cte_move`
   - `cte_swap`

#### 交付物

- 四个 mutator 都拥有 Verus body 主路径；
- `external_body` 如果还存在，只能留在极小表示写入边界；
- mutator family 共享一致的 frame / transition / post 组织方式。

#### 完成标准

- 当前非删除主线的更新原语不再主要依赖 `external_body + bridge post`；
- `cte_insert` 成为模板；
- 另外三个入口沿同一工程风格推进，而不是各写各的特殊逻辑。

#### 当前建议子顺序

1. `cte_insert`
2. `insert_new_cap`
3. `cte_move`
4. `cte_swap`

#### 暂时不要做

- 不要为了“先全部过”重新把大语义塞回 bridge；
- 不要在 mutator 还没稳定前，把 delete 主线接进来。

### Step 7：delete / revoke / finalise 主线

#### 当前状态

这一整步在本轮明确冻结。

#### 为什么仍然保留在计划里

因为这一步确实存在，而且它是完整 Verus 替换的最后大块，但它不应该继续污染当前执行轮的优先级。

#### 当前对它的要求

本轮只要求：

- 不破坏现有 delete gate；
- 不新增 delete 方向的临时语义黑盒；
- 在前 6 步收口后，再决定是否开启。

#### 解冻条件

只有在下面这些条件基本满足后，才建议重新打开 Step 7：

- Step 1 到 Step 6 已完成；
- `refinement_bridge.rs` 不再承担核心非删除语义恢复；
- mutator family 已有清楚的 Verus-native 模板；
- `derive_cap / ensure_no_children / resolve_address_bits`
  已经证明这条非删除主线能够独立支撑实现体替换。

## 8. 步骤编号与实际编码顺序

逻辑依赖顺序仍然是：

1. Step 1
2. Step 2
3. Step 3
4. Step 4
5. Step 5
6. Step 6
7. Step 7

但当前轮次的实际编码顺序，建议更偏下面这条：

1. 先完成 Step 1 的最小分层重画。
2. 再完成 Step 2 的最小表示层骨架。
3. 进入 Step 6，但先只打 `cte_insert` 这个模板点。
4. 用 `cte_insert` 固定 mutator family 的工程风格后，平推 `insert_new_cap / cte_move / cte_swap`。
5. 再回头做 Step 5，把 `resolve_address_bits` 也放进同一套表示层/本地 proof 风格。
6. 然后做 Step 4，把 `is_mdb_parent_of / is_final_cap / ensure_no_children / derive_cap` 收到新的本地主线里。
7. 最后做 Step 3 的清扫收口，把 capability query 三项和最终接口口径彻底合一。

这样排的原因是：

- Step 2 是所有后续工作的表示层地基；
- mutator family 对表示层要求最强，最适合作为真正的驱动测试；
- `cte_insert` 先做成后，后面的 `move/swap/new_cap` 不会继续各写各的风格；
- query 三项虽然相对容易，但更适合作为“最后统一主接口口径”的收尾项，而不是最早的驱动项。

## 9. 当前轮次的推荐验收标准

如果当前轮次最后能达到下面这些结果，就可以认为“非删除主线的 Verus 替换工作已经做到了一个可以停笔的版本”：

1. `repr / owner / view / model` 最小地基已经建起来，bridge 不再是默认落点。
2. capability query 三项的主实现已经收合。
3. `derive_cap` 与 `ensure_no_children` 已形成直接可解释的 Verus-native 局部主线。
4. `resolve_address_bits` 已进入 Verus body 主路径。
5. 四个 mutator 已进入 Verus body 主路径，或只剩极薄的表示写入边界。
6. remaining TCB 已经主要集中在：
   - concrete 表示观察；
   - 小粒度地址/指针公理；
   - 必要的 constructor witness；
   而不再集中在核心业务语义。

如果做到这一步，本轮就可以停，不需要因为 delete 主线还没做完而继续加码。

## 10. 当前轮次最重要的判断

后续代码推进时，最重要的判断不是“还能不能再去掉一个 helper 名字”，而是下面这句：

`这个改动，究竟是在让 Verus 更直接接管 sel4_cspace 的真实实现体，还是只是在旁边又多搭了一层证明外壳？`

如果答案是后者，就说明这一步不该继续按当前方向写。

当前这份 7 步计划的目的，就是把后续工程稳定压到前者。
