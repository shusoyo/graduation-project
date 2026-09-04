# CSpace 第 5 步 Bridge 设计与风格约束

状态（2026-04-23）：

- 第 5 步已收口完成。
- `resolve_address_bits` 已有 read-only bridge 与首个 refinement 入口。
- `cte_insert` / `cte_move` / `cte_swap` 已补齐 heap-indexed local bridge vocabulary、source/dest/neighbor packaging lemma，以及从 local transition 提升到 post-heap match 的统一入口。

## 1. 目的

这份文档用于启动第 5 步：在 `sel4_cspace/src` 的 concrete 实现与 `sel4_cspace/specs` 的抽象模型之间建立
view / refinement bridge。

本步的目标不是直接证明具体函数，而是先把下面三类 concrete 载体稳定映射到抽象层：

- `cap`
- `cte_t`
- `resolveAddressBits_ret_t`

本步完成后，第 6 步不应只停在“旁证现有 wrapper 满足 spec”，而应逐个把 proof 从 bridge
推回 `sel4_cspace/src/cte.rs` 的函数本体；第 7 步再统一收口已证 / 未证 / TCB 清单。

## 2. 当前仓库已经形成的验证风格

截至 2026-04-22，`sel4_cspace` 当前不是“从具体代码直接往下压证明”的风格，而是已经稳定成以下路线：

### 2.1 spec-first，先抽象模型，后 bridge

主要证据：

- `sel4_cspace/specs/abstract_cspace.rs`
- `sel4_cspace/specs/cspace_ops/resolve.rs`

当前先建立：

- `CapSpec`
- `SlotEntrySpec`
- `CSpaceState`
- `wf`
- `slots_unchanged_except`
- `reachable_slot_from`

然后再给原语写整状态的抽象合同。

这意味着第 5 步不应该倒回来改成“直接围绕 bitfield getter 写函数级零散证明”的路线，而应该继续保持：

- 先有抽象语义
- 再有 concrete view
- 再有 refinement relation
- 最后进入函数级证明

### 2.2 relational contract 为主，不是 ownership/token 为主

当前规格层主要使用的是“旧状态 / 新状态 / 结果值”之间的关系，而不是重型资源所有权模型。

典型形状包括：

- `spec_cte_insert_pre` / `spec_cte_insert_post`
- `spec_cte_move_pre` / `spec_cte_move_post`
- `spec_cte_swap_pre` / `spec_cte_swap_post`
- `spec_resolve_address_bits_pre` / `spec_resolve_address_bits_post`

以及模型层 frame 条件：

- `slots_unchanged_except`
- `spec_cte_insert_changed_slots`
- `spec_cte_move_changed_slots`
- `spec_cte_swap_changed_slots`

因此第 5 步 bridge 也应优先服务于“把 concrete 结果解释成抽象状态变换”，而不是一开始就引入整套 tracked permission machinery。

### 2.3 packaging lemma 已经是当前证明入口风格

主要证据：

- `lemma_wf_implies_core_invariants`
- `lemma_wf_implies_valid_slot_entry`
- `lemma_resolve_pre_implies_base_invariants`
- `lemma_resolve_pre_implies_root_lookup_ready`

这些引理说明当前仓库已经形成一种明确习惯：

- 不在每个调用点手拆大 conjunction
- 先把“大前提”收紧成可复用入口
- bridge 和后续 refinement proof 都应复用这些入口

所以第 5 步也应继续产出“小而稳”的入口，而不是引入一个难以展开的超大 refinement predicate。

### 2.4 smoke-driven 收口已成为规格层基线

主要证据：

- `sel4_cspace/specs/lib.rs`
- `sel4_cspace/specs/cspace_ops/smoke.rs`

当前规格层通过 smoke check 做最小闭环，先确认：

- case taxonomy 完整
- pre/post 能联通
- 常用引理能在小例子里直接复用

第 5 步完成后，也应保持同样节奏：bridge 先有小闭环，再进入函数级证明。

### 2.5 当前已有的 `verify_bridge` 只是类型包装，不是 refinement bridge

主要证据：

- `sel4_common/src/verify_bridge.rs`

当前已有：

- `BridgeCap`
- `BridgeCapTag`
- `BridgeMdbNode`
- `BridgeException`

这层解决的是“验证代码依赖稳定包装类型”的问题，还没有解决：

- concrete `cap` 如何映射到 `CapSpec`
- concrete `cte_t` 如何映射到 `SlotEntrySpec`
- concrete `resolveAddressBits_ret_t` 如何映射到 `ResolveAddressBitsResultSpec`
- concrete 局部堆状态如何映射到 `CSpaceState`

因此第 5 步应当在此基础上继续向上搭建，而不是把这层误判为 bridge 已经完成。

## 3. 参考策略：哪些跟，哪些不跟

### 3.1 `aux/l4v`：跟语义与 case split，不跟 Isabelle proof script

主要参考：

- `/workspace/aux/l4v-master/spec/haskell/src/SEL4/Kernel/CSpace.lhs`
- `/workspace/aux/l4v-master/spec/haskell/src/SEL4/Object/CNode.lhs`
- `/workspace/aux/l4v-master/spec/abstract/Intro_Doc.thy`

第 5 步最该借的是它的语义分解方式。

对 `resolveAddressBits`，`l4v` 的顺序很明确：

1. 先算 `levelBits = radixBits + guardBits`
2. 尽早定位下一跳 slot
3. 先判 guard mismatch
4. 再判 depth mismatch
5. 再区分 exact success / recursive descent / early stop on non-CNode

对 `cteInsert` / `cteMove` / `cteSwap`，应借的是：

- 对 MDB 更新顺序的语义理解
- sibling 场景下更新次序的敏感性
- source / destination / neighbor 这几类局部影响面的划分

不应借的是 Isabelle 证明脚本风格本身。原因很直接：

- 当前仓库已经是 Verus 风格，不是 theorem-script 风格
- 直接移植 `corres` / `ccorres` 证明组织会把第 5 步做得过重
- 第 5 步最需要的是稳定 bridge vocabulary，不是复制 l4v 证明外形

一句话概括：`l4v` 是 semantic source of truth，不是 proof-engineering 模板。

### 3.2 `aux/vostd`：跟 Verus-native 的合同粒度与局部桥接

主要参考：

- `/workspace/aux/vostd-main/proofs/sample_primitives/src/lib.rs`
- `/workspace/aux/vostd-main/proofs/sample_ops/src/lib.rs`

这里更值得借的是 Verus 的“轻量、局部、合同贴近代码”的写法：

- 小函数配小合同
- `requires` / `ensures` 紧贴执行代码
- 必要时用 `assume_specification[...]` 固定可信边界
- 不把所有语义都塞进一个总谓词

对第 5 步的直接启发是：

- ghost view 函数要小
- 每个 bridge 入口要能独立复用
- concrete getter 的语义应通过局部 ensures 固定
- 不要上来定义一个覆盖整片 CSpace 堆的巨型 view predicate，然后所有证明都靠展开它

一句话概括：实现与证明工程的默认风格，优先靠近 `vostd`。

### 3.3 `aux/atmo`：跟模块边界 discipline，不跟第一阶段就重型 tracked 化

主要参考：

- `/workspace/aux/atmosphere-main/kernel/verified/bridge.rs`
- `/workspace/aux/atmosphere-main/kernel/src/bridge.rs`
- `/workspace/aux/atmosphere-main/kernel/verified/pagetable/pagetable_impl_base.rs`

这里最值得借的是两点：

- 把 trusted bridge surface 单独命名、单独隔离
- verified 逻辑与 concrete 实现通过清楚的桥接边界对接

但当前 `sel4_cspace` 第 5 步不应直接照搬它的重型部分：

- `tracked` permission
- 复杂 borrow/remove/insert token 流
- 大量 object-local ownership reasoning

原因是当前 CSpace 证明仍处于：

- 先解释 concrete 数据形状
- 先接通 abstract spec
- 先做第一个 refinement 闭环

这个阶段如果直接跳到 `atmo` 那种强资源化写法，会让 bridge 先于需求膨胀。

一句话概括：借 `atmo` 的边界组织，不借它第一阶段的全部证明负担。

## 4. 第 5 步的推荐总风格

综合三类参考和当前仓库现状，第 5 步最合适的路线是：

- 语义来源跟 `l4v`
- 抽象层继续沿用当前仓库的 relational spec 风格
- bridge 函数和局部合同的粒度跟 `vostd`
- 模块边界和 trusted surface 的管理借 `atmo`

也就是：

`l4v semantics + current spec-first relational model + vostd-sized contracts + atmo-style boundary discipline`

这条路线的优点是：

- 与已经完成的第 2、3、4 步连续
- 不会为了 bridge 把证明工程突然改型
- 第 6 步逐函数 refinement 时能复用同一套命名和入口
- 第 7 步做 TCB 清单时也更容易把 trusted bridge 单独列出

## 5. 第 5 步建议分层

第 5 步建议明确分成五层，不要把它们混在一起写。

### 5.1 第 0 层：concrete 原始载体

主要来自：

- `sel4_common::structures_gen::cap`
- `sel4_common::structures_gen::mdb_node`
- `sel4_cspace/src/cte.rs` 中的 `cte_t`
- `sel4_cspace/src/structures.rs` 中的 `resolveAddressBits_ret_t`

这层保持生产实现形状，不为了验证改写业务语义。

### 5.2 第 1 层：稳定包装层

主要来自：

- `sel4_common/src/verify_bridge.rs`

这层继续只做稳定包装，不承担完整抽象语义。

换句话说：

- `BridgeCap` 仍然是 wrapper
- 它不是 `CapSpec`
- `BridgeMdbNode` 仍然是 wrapper
- 它不是 `SlotEntrySpec`

### 5.3 第 2 层：ghost view 层

这是第 5 步真正要补的新层。

它应负责：

- 把 concrete `cap` 解释成 `CapSpec`
- 把 concrete `cte_t` 解释成 `SlotEntrySpec`
- 把 concrete `resolveAddressBits_ret_t` 解释成 `ResolveAddressBitsResultSpec`

建议这层 future layout 放在 `sel4_cspace/src` 侧，而不是塞回 `sel4_common`。

原因：

- 这是 `sel4_cspace` 专属语义，不是通用 common 语义
- 当前第 5 步是“给 `src` 建 bridge”，不是继续扩 common wrapper
- 后续函数 refinement 也会直接依赖这层

### 5.4 第 3 层：refinement relation 层

这层负责把局部 view 提升到“具体实现满足哪条抽象合同”的关系式。

例如未来应出现但不必一开始就做大的关系：

- concrete cap 与 `CapSpec` 一致
- concrete slot 局部布局与 `SlotEntrySpec` 一致
- concrete `resolve_address_bits` 返回值与 `spec_resolve_address_bits_post` 一致

这里的关键约束是：

- 先做局部 relation
- 后做组合 relation
- 不要第一天就定义一个覆盖整片内存和所有 reachable slot 的单一全局 refinement predicate

当前已落地的一组关系词汇正沿着这个方向收紧：

- `refines_cap`
- `refines_cte`
- `refines_resolve_address_bits_ret`
- `trusted_cspace_slot_views_match_state`
- `trusted_cspace_cnode_lookups_match_state`
- `trusted_cspace_heap_matches_state`

其中后 3 个不是“再造一个整 heap 大黑盒”，而是把原先 `resolve_address_bits` 入口里那类
“concrete CSpace matches abstract state”的责任拆成两块：

- concrete slot entry 与 `SlotEntrySpec` 对齐
- concrete CNode offset lookup 与 `cnode_lookup` 对齐

这正是第 5 步后半程应继续坚持的方向：按可复用语义面拆，而不是按调用点堆一个新总谓词。

### 5.5 第 4 层：函数级 refinement proof

这层才是第 6 步。

推荐首个闭环仍然是：

1. `resolve_address_bits`
2. `cte_insert`
3. `cte_move`
4. `cte_swap`

原因仍然与当前文档一致：`resolve_address_bits` 的抽象合同和 bridge 前入口最完整，最适合作为第一条 refinement 闭环。

## 6. 命名与组织规则

为了让第 5、6、7 步风格一致，建议现在就固定下面的命名和组织规则。

### 6.1 纯抽象语义继续留在 `specs/`

规则：

- `spec_...` 只用于纯抽象语义或纯抽象合同
- `lemma_...` 只用于纯证明入口或包装引理
- `specs/` 不直接承载 concrete bitfield 解码

这保证第 2、3、4 步已经沉淀下来的语义层不会被 bridge 污染。

### 6.2 concrete 到 abstract 的投影统一叫 `view_...`

建议未来统一使用：

- `view_cap(...) -> CapSpec`
- `view_cte(...) -> SlotEntrySpec`
- `view_resolve_address_bits_ret(...) -> ResolveAddressBitsResultSpec`

如果需要区分可信边界上的原始 getter 包装，也可用：

- `bridge_cap_tag`
- `bridge_mdb_prev`
- `bridge_mdb_next`

但最终面向 refinement 的 ghost 投影，名字应统一收敛到 `view_...`。

### 6.3 concrete 与 spec 的一致性关系统一叫 `refines_...` 或 `matches_...`

建议统一使用下列风格之一，不混用太多别名：

- `refines_cap`
- `refines_cte`
- `refines_resolve_address_bits_ret`

如果某些地方更像点对点结构相等，也可以用 `matches_...`，但整个 CSpace 路线最好只选一套主命名。

当前更推荐 `refines_...`，因为第 6 步最终要证明的是 concrete function refine 抽象合同，而不是简单字段相等。

### 6.4 bitfield 细节只在 bridge 层拆，不在函数证明里到处拆

这是第 5 步最重要的风格约束之一。

目标是让第 6 步的函数证明读起来像：

- 取 concrete 输入
- 通过 `view_...` 转成抽象输入
- 调用已存在的 `spec_...` / `lemma_...`
- 证明 concrete 返回满足 `refines_...`

而不是在每个函数证明里反复手工展开：

- `get_tag()`
- `get_capCNodeGuard()`
- `get_mdbPrev()`
- `get_mdbNext()`

这些解析工作应尽量沉到 bridge 层。

### 6.5 trusted 边界要窄且显式

如果某些 getter / pointer conversion / raw slot walk 在第一轮必须暂信任，应当：

- 在 bridge 边界集中出现
- 有最小输入前提和输出语义
- 可以在第 7 步被单独列为 TCB

不要把 trusted 假设散落到每条函数 proof 里。

截至本轮，`resolve_address_bits` 入口已经开始按这个规则落地：

- root cap 的 bitfield 解码留在 `trusted_extract_cap`
- slot 局部视图留在 `trusted_concrete_slot_view`
- CNode offset lookup 留在 `trusted_concrete_cnode_lookup_slot`
- 调用级黑盒已从“直接承诺 abstract refinement”收紧为“只承诺 deterministic core result”
- 对应命名为：
  - `resolve_address_bits_expected_core_from_cap`
  - `resolve_address_bits_expected_core`
  - `trusted_call_resolve_address_bits`

再往下一轮收紧后，调用级边界已经继续下推成“一步 branch / recursive relation”：

- `resolve_address_bits_one_step_refines_state`
- `lemma_resolve_address_bits_one_step_refines_state_implies_core_refines_state`

也就是说：

- `expected_core` 现在更像 proof-side 的 deterministic helper
- 调用侧 trusted wrapper 只需要承诺 concrete 实现走对了当前这一步
- 从一步关系提升到整体 refinement 的责任已移回可证明层

这说明第 5 步当前不是“缺少 bridge”，而是已经进入“持续压缩剩余 trusted wrapper 责任”的阶段。

## 7. 第 5 步的最小里程碑

第 5 步不建议一上来追求“大而全”，建议按下面顺序完成。

### 7.1 里程碑 M1：`cap` 的 view 固定

要求：

- 能稳定区分 tag / object / rights / badge / cnode data / untyped data
- 对应到 `CapSpec`
- 先只覆盖第 6 步马上会用到的 tag 集合

这一步的目的，是把 `resolve_address_bits` 和 `cte_*` 都会重复用到的 cap 解释统一起来。

### 7.2 里程碑 M2：`cte_t` 的局部 view 固定

要求：

- cap view
- mdb prev / next
- revocable / first_badged

对 `SlotEntrySpec` 的局部解释固定下来后，第 6 步中的 insert / move / swap 才能稳定表达“改了哪些 slot，没改哪些 slot”。

### 7.3 里程碑 M3：`resolveAddressBits_ret_t` 的结果 view 固定

要求：

- `status`
- `slot`
- `bitsRemaining`

这一步完成后，可以先证明 `resolve_address_bits` 的结果级 refinement，而不必马上解决整个 CSpace 堆的全局桥接。

### 7.4 里程碑 M4：先打通 `resolve_address_bits`

原因：

- 当前抽象合同最完整
- `l4v` 对应语义最清楚
- 当前已经有 `lemma_resolve_pre_implies_root_lookup_ready`
- 它更像 lookup/refinement proof，而不像 `cte_*` 那样立刻需要更强的局部状态更新关系

### 7.5 里程碑 M5：再接 `cte_insert` / `cte_move` / `cte_swap`（已完成）

这部分现在已经落地为一套和 `resolve_address_bits` 同风格、但面向“有状态变化原语”的 local bridge vocabulary：

- heap-indexed concrete view：
  - `trusted_concrete_slot_view_at`
  - `trusted_concrete_cnode_lookup_slot_at`
- heap-indexed state match：
  - `trusted_cspace_slot_views_match_state_at`
  - `trusted_cspace_cnode_lookups_match_state_at`
  - `trusted_cspace_heap_matches_state_at`
- local frame / transition：
  - `trusted_cspace_slots_unchanged_except_at`
  - `trusted_cspace_cnode_lookups_unchanged_at`
  - `trusted_cspace_local_heap_transition_at`
- 三个目标原语的统一入口：
  - `cte_insert_bridge_pre_at` / `cte_insert_local_heap_transition_at`
  - `cte_move_bridge_pre_at` / `cte_move_local_heap_transition_at`
  - `cte_swap_bridge_pre_at` / `cte_swap_local_heap_transition_at`
- 三类可直接复用的 packaging lemma：
  - pre -> spec pre；
  - pre -> source / destination / neighbor concrete view refine；
  - local transition + abstract post -> post heap matches state。

这里的关键点不是重新引入一个新的整 heap post-state 黑盒，而是把 bridge 收紧成：

- old heap 全局匹配 old state；
- new heap 只要求 changed slots 与 new state 对齐；
- untouched slots 和 `cnode_lookup` 通过 concrete frame + abstract frame 回收。

这样第 6 步证明 `cte_insert` / `cte_move` / `cte_swap` 时，就能统一从同一套 local-transition 入口起跑。

## 8. 明确延后，不在第 5 步做的事

下面这些方向不是“不做”，而是明确不在当前第 5 步启动阶段做：

- 不直接引入 full tracked permission / ownership token 体系
- 不直接建立整内核状态的大一统 refinement predicate
- 不直接证明 `finalise_cap` / `post_cap_deletion` / `preemption_point` 的内部语义
- 不直接把 `l4v` 的 Isabelle proof script 结构搬到 Verus
- 不把 bridge 先扩成覆盖所有 capability tag 的一次性大全

第 5 步的关键不是“先把体系做满”，而是“先把 bridge 做成下一步函数证明真正会复用的公共入口”。

## 9. 对第 6、7 步的统一风格要求

为了让后续阶段不漂移，建议现在就固定三条约束。

### 9.1 第 6 步每个函数都走同一证明节奏

统一节奏应为：

1. concrete 输入通过 `view_...` 进入抽象层
2. 用已有 `lemma_...` 打开前提
3. 证明 concrete 结果满足 `refines_...`
4. 只在 bridge 层解释 raw getter 和 bitfield

### 9.2 第 6 步每个函数都先做最小闭环，再扩覆盖面

例如：

- 先 `resolve_address_bits`
- 先覆盖最关键成功 / fault case
- 再扩更多 tag / 更多边界情形

这和第 4 步通过 smoke 收紧入口的做法保持一致。

### 9.3 第 7 步报告按三类收口

最终报告建议固定三类：

- 已证明的 refinement
- 暂信任的 bridge / boundary / FFI
- 明确延后的能力类型、路径或全局不变量

这样第 5 步留下的 trusted bridge surface 可以自然进入最终 TCB 清单，而不会在项目末期重新归档。

## 10. 当前结论

对当前仓库来说，“更好”的选择不是在第 5 步立刻重写成另一种验证范式，而是保持连续性：

- 抽象语义继续对齐 `l4v`
- Verus 写法继续保持当前仓库已形成的 spec-first / relational / packaging-lemma 风格
- bridge 的粒度参考 `vostd`
- trusted surface 的组织参考 `atmo`

如果按这个方向推进，那么第 5 步产出的 bridge 不只是为 `resolve_address_bits` 服务，也会自然成为
`cte_insert` / `cte_move` / `cte_swap` 在第 6 步中的统一入口。

截至 2026-04-23，这个目标已经达到关闭条件：

- snapshot bridge 已固定；
- `resolve_address_bits` 的 read-only bridge 与 branch/refinement 入口已固定；
- `cte_insert` / `cte_move` / `cte_swap` 的 heap-indexed local bridge 与 post-heap lift 已固定；
- 第 5 步后续不再需要继续扩 bridge vocabulary，下一步正式进入第 6 步逐函数 refinement。
