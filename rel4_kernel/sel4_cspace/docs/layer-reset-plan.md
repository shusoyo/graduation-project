# Layer Reset Plan

本文档记录 `sel4_cspace` 在新分层架构下的详细重启计划。

目标不是在旧的 `mdb_patch` 路线之上继续小修小补，而是明确重建下面的层次：

1. `cap`
2. `mdb_node`
3. `cte_t`
4. `mdb`
5. `cdt`
6. `cspace`

## Goals

这次 reset 的目标有四个：

1. 让 `cte_t` 不再只是 wrapper，而成为真正的 slot object。
2. 让 `mdb` 成为显式子系统，而不是散落在 `CSpaceManager` 中的 predicate 集合。
3. 让 `cdt` 成为显式子系统，而不是隐含在 derivation closeout 里的 ghost map。
4. 让 `CSpaceManager` 从“大一统 owner”收缩成组合层。

## Non-Goals

当前阶段不以这些为第一目标：

- 立即清零所有 trusted boundary
- 立即达到 public-wrapper fully verified
- 立即达到 whole-kernel / `l4v`-level claim
- 继续扩张旧 `mdb_patch` closeout vocabulary 作为主要路线

## Target Layer Design

### Layer 1: `cap`

责任：

- capability 自身语义
- `tag / object / rights / badge / cnode / zombie`
- derive/finalise 相关 cap-level reasoning

接口原则：

- 不默认承担 MDB 图或 CDT parent 语义
- 若 `mdb` 需要 capability 语义，应通过 capability projection 消费

迁移方向：

- 整理哪些 helper 是 cap 自身语义
- 整理哪些 helper 实际是给 `mdb` 或 `cdt` 消费的投影语义

### Layer 2: `mdb_node`

责任：

- `prev`
- `next`
- `revocable`
- `first_badged`

接口原则：

- 它是字段/表示层
- 不承担全局 graph invariant owner 职责

迁移方向：

- 明确 runtime field access 与 slot-level view 的对应关系
- 为 `cte_t` 的局部方法提供更清晰的数据边界

### Layer 3: `cte_t`

责任：

- 作为 slot object 承载 `cap + mdb_node`
- 局部读写与局部语义判断
- concrete layout 与 spec view 的桥接

归属关系：

- `cte_t` 属于 `cspace` 域
- 但不属于 `manager` 子层
- 它应作为 `cspace::cte` 存在，而不是 `cspace_manager` 的内部细节

目标形态：

- 不再只是 compatibility wrapper
- 具备 slot-local verified method

适合放进 `cte_t` 的内容：

- observer：`slot_ptr`、`slot_view`、`cap_view`、`prev/next` 读取
- slot-local mutation：`set_cap_only`、`set_prev`、`set_next`、`clear_entry`
- entry-level 约束：空槽/非空槽局部判断、slot-local bridge

`cte_t` 自己应该承担的验证：

- observer 的正确性
- slot-local mutation 的 postcondition
- entry-level 局部一致性
- concrete layout 到 slot view 的桥接正确性

不适合直接塞进 `cte_t` 的内容：

- whole-MDB patch proof
- whole-CDT tree proof
- multi-slot graph surgery 作为单槽自证

### Layer 4: `mdb`

责任：

- 对 `cte_t` 中 `mdb_node` 的图建模
- 基于 capability 投影的 MDB semantic edge 判断
- patch / frame / recovery / changed-slot obligation
- MDB 全局验证

核心 invariant 方向：

- link consistency
- no self link
- no two cycle
- incoming parent edge
- incoming badge edge
- incoming untyped edge

接口原则：

- 可以消费 capability projection
- 不负责完整 capability 语义
- 不负责 CDT parent/original 语义

建议模块形状：

- `cspace/mdb/mod.rs`
- `cspace/mdb/state.rs`
- `cspace/mdb/spec.rs`
- `cspace/mdb/proof.rs`
- `cspace/mdb/raw.rs`

### Layer 5: `cdt`

责任：

- capability derivation tree
- `cdt_parent`
- `is_original`
- `spec_should_be_parent_of(...)`
- derivation tree 层 `wf`

接口原则：

- 与 `mdb` 并列
- 不混进 `mdb` proof vocabulary
- 与 `cte_t` / `cap` 一起决定 parent-child derivation 语义

建议模块形状：

- `cspace/cdt/mod.rs`
- `cspace/cdt/state.rs`
- `cspace/cdt/spec.rs`
- `cspace/cdt/proof.rs`

### Layer 6: `cspace`

责任：

- 组合 `cte / mdb / cdt / resolve / zombie`
- 承载 higher-level operation contract
- 负责最终总 `wf` 组合

接口原则：

- 不再重复承担所有底层 graph / derivation 细节
- 更像系统协调者，而不是所有证明的唯一 owner

建议模块形状：

- `cspace/mod.rs`
- `cspace/manager.rs`
- `cspace/cte/mod.rs`
- `cspace/cte/types.rs`
- `cspace/cte/spec.rs`
- `cspace/cte/slot.rs`
- `cspace/mdb/*`
- `cspace/cdt/*`
- `cspace/resolve/*`

## Pre-Proof Freeze

这一节是正式推进 `insert / move / swap / delete` 证明前的冻结点。后续 proof 可以补强 contract，但默认不再搬这些 owner 边界。

### Proof Skeleton Freeze

本轮 mutation proof 采用下面的主骨架：

- 主骨架采用 owner-based post-state proof。
- 每个 operation 只证明一个最终后状态，不再围绕一串中间态逐步堆 frame lemma。
- `unchanged` 降级为薄 frame 工具，只表达 patch 外 concrete slot data 不变，不再承担主要语义保持工作。
- `rank` 只服务 `mdb` 的 acyclic / no-cycle 一类结构性目标，不承担 badge / parent / untyped 语义。
- `manager` 负责把 concrete exec 归约到 owner post-state；`mdb` / `cdt` 各自负责自己的保持性证明。

### Resolved Boundary Questions

这几条边界在证明开始前视为已定：

- `mdb` 是图 owner，不是顶层 operation owner；它不直接以 `insert / move / swap / delete` 命名自己的核心变换。
- `mdb` 只拥有图状态、图 primitive transition、图 invariant，以及由 capability 投影得到的 MDB edge semantics。
- 顶层 operation 由 `cspace::manager` 负责编排，再把结果归约到 `mdb` / `cdt` primitive post-state。
- `cte_t` 只证明 slot-local method 和 slot view bridge，不直接证明 whole-MDB / whole-CDT recovery。
- `cdt` 与 `mdb` 并列；`parent/original` 语义不再混入 `mdb` 层。

### Owner Boundary

当前 owner 边界冻结如下：

- `cspace::cte` 只管 slot 局部语义：slot pointer、offset slot、slot view、单槽 derive/children/final 判断的本地 contract。
- `cspace::mdb` 当前只保留 `MdbTable` owner 路线：tracked entries、`order/live_slots` summary、graph primitive relation、以及 owner-local structural wf。
- `cspace::cdt` 只管纯 derivation tree：`dom`、`parent_of`、`is_original`、depth witness、以及纯结构性 wf。
- `cspace::manager` 拥有 `MdbTable`、`CdtState`、所有 cap 语义、所有 cross-layer glue，以及总 invariant 汇总；`immediate_derived`、`incoming_*`、`mdb_parent_*`、`cdt_parent_*` 这类入口直接归 manager 所有，不再伪装成下层 owner forwarding。
- `resolve / cnode_lookup` 本轮不单独拆 owner，冻结在 `cspace::manager` 的 resolve/cnode lookup 责任里；等四个 mutation operation 的分层 proof 稳住后再决定是否提成 `cspace::resolve` owner。

### Base Spec Type Home

基础 spec 类型在操作证明开始前不再移动，暂时继续以 `cspace::manager::spec_proof` 为 home：

- `CapSpec`
- `SlotEntrySpec`
- `Rights`
- `ObjectRef`
- `CapKind`

这些类型以后可以外提到更中性的 spec prelude，但不在当前证明前置清场中做。原因是它们被所有 proof cone 消费，提前搬家会让每个 operation proof 跟着抖。

### Cap-Level Semantic Split

cap-level 语义按消费方冻结为三类：

- cap 基础语义：cap kind、object/region、rights 字段、badge 值、untyped range、arch cap 投影。当前仍以既有 capability/trusted/spec_proof helper 为基础。
- MDB owner 语义：`rights_subseteq`、`same_region_as_caps`、`same_object_as_caps`、`untyped_cap_contains_cap`、`badge_chain_allows`、`mdb_parent_of_caps`、entry-level `parent_of_entries`。
- CDT owner 语义：`should_be_parent_of(parent_cap, parent_original, child_cap, child_original)`。

同一个语义词只能有一个 owner。旧 `spec_proof.rs` 里保留同名 public 入口时，只作为兼容薄委托存在。

### `cte_t` Local Contract Surface

`cte_t` 本地方法在进入 manager proof 前必须有稳定 contract：

- `get_ptr`：返回 `cte_slot_ptr(self)`。
- `get_offset_slot`：要求 `cte_offset_slot_call_pre(base, index)`，返回地址等于 `base + sizeof(cte_t) * index`。
- `derive_cap`：返回状态只允许 none/syscall-error；非 arch cap 的返回 cap 由 `cte_derive_cap_expected_cap(slot_view, cap_view)` 决定；untyped children-block case 返回 syscall error 和 null cap。
- `ensure_no_children`：返回值由 `cte_ensure_no_children_blocks(slot_view)` 决定，blocking case 是 syscall error，non-blocking case 是 none。
- `is_final_cap`：返回 `cte_is_final_cap_at(cte_slot_ptr(self))`，该 predicate 只看当前 slot 的 prev/next 邻居和 same-object 关系。

这些 contract 仍允许 runtime body 通过 manager wrapper 过渡，但 proof 消费的是 `cspace::cte::spec` 的 slot-local 语义，而不是旧 manager 大中心名字。

### Operation Effect Surface

下面是 mutation operation 的分层 effect surface。它不是最终 proof，但后续 proof 必须按这个面来拆 obligation。

| Operation | `cte` local fields | `mdb` graph/effects | `cdt` parent/original | `resolve / cnode_lookup` |
| --- | --- | --- | --- | --- |
| `insert` / `cte_insert` | `dest` 写入 `new_cap`，`dest` 从 empty 变 non-empty；`src` cap 可能被 untyped-full 标记更新 | `dest.prev = src`，`dest.next = old(src.next)`，`src.next = dest`，old next 的 `prev` 改为 `dest`；`revocable/first_badged` 来自 `is_cap_revocable(new_cap, src_cap)` | `cdt_parent(dest) = Some(src)`；`is_original(dest) = true`；其他 slot 保持 | 不直接改变 lookup table；只要求 `src/dest/old_next` 都在 slot domain，若 dest 属于某 root 的 lookup 集合，映射关系由既有 cnode bookkeeping 保持 |
| `insert_new_cap` | `slot` 写入 `capability`，从 empty 变 non-empty | `slot.prev = parent`，`slot.next = old(parent.next)`，`parent.next = slot`，old next 的 `prev` 改为 `slot`；`slot.revocable = true`，`slot.first_badged = true` | `cdt_parent(slot) = Some(parent)`；`is_original(slot) = true`；其他 slot 保持 | 不直接改变 lookup table；仍依赖调用者提供已注册/可寻址 slot |
| `move` / `cte_move` | `dest` 接收 `new_cap` 和 `src` 原 MDB 字段；`src` 被清成 empty/null | old prev 的 `next` 改为 `dest`，old next 的 `prev` 改为 `dest`；`dest` 占据原 `src` 的 MDB 位置；`src` 的 MDB 字段清空 | `cdt_parent(src) = None`；`cdt_parent(dest) = old(cdt_parent(src))`；所有以 `src` 为 parent 的 child 改指向 `dest`；`is_original(dest) = old(is_original(src))`，`is_original(src) = false` | lookup table 不直接改；若外部把 `src/dest` 当 lookup target，证明需显式说明 caller-level alias/registration 条件 |
| `swap` / `cte_swap` | `slot1` 接收 `cap2` 和 `slot2` 原 MDB 字段；`slot2` 接收 `cap1` 和 `slot1` 原 MDB 字段 | 非相邻邻居重接到对方 slot；相邻/self-neighbor case 通过 exact swap 规则修正 prev/next，保持 MDB 链一致 | `cdt_parent(slot1) = old(cdt_parent(slot2))`，`cdt_parent(slot2) = old(cdt_parent(slot1))`；`is_original` 对两个 slot 同步交换；其他 slot 保持 | lookup table 不直接改；若 lookup 映射到这两个 slot，证明语义按 slot identity 保持、cap 内容交换 |
| `delete` / `set_empty` | 目标 slot 清成 empty/null；delete_one/delete_all 先 finalise 再清 slot | 若目标非 null，old prev 的 `next` 改为 old next，old next 的 `prev` 改为 old prev；old next 的 `first_badged` 吸收 deleted slot 的 `first_badged`；目标 MDB 字段清空 | `cdt_parent(slot) = None`；所有以该 slot 为 parent 的 child 断 parent；`is_original(slot) = false`；其他 original 标记保持 | lookup table 不直接改；删除注册 root 或 lookup-reachable slot 的更强约束留给 manager/caller 层处理 |

后续每个 operation proof 都应先证明本层 effect，再由 `mdb` / `cdt` owner 给出对应 recovery lemma，最后由 `manager` 汇总总 `wf`。

### MDB Primitive Transition Rule

`mdb` 层后续默认以 graph primitive 为中心组织 proof，例如：

- unlink / detach
- insert-between / splice
- rewire-prev-next
- swap-neighborhood

默认避免把 `cte_insert`、`cte_move`、`cte_swap`、`cte_delete` 直接做成 `mdb` owner API。顶层 operation 可以调用多个 primitive，但 primitive 本身不携带顶层业务命名。

### Example: `cte_insert` Layer Split

以 `cte_insert(src, dest)` 为例，顶层语义上它是一个 CSpace mutation operation；但按分层设计拆开后，各层责任应当是：

- `cap`：负责 `new_cap` 与 `src_cap` 相关的 capability 语义，例如 revocable / badge / region/object 投影所需条件。
- `cte`：负责 `src`、`dest` 以及可能涉及的 `old_next` 的 slot entry 读写结果，也就是 concrete slot update 与 slot view post-state。
- `mdb`：不直接证明“insert”这个顶层操作，而只消费 `src`、`dest`、邻边关系形成的 graph patch，并证明该 patch 满足 parent / badge / untyped 这条 MDB edge 语义，同时保持图 invariant。
- `cdt`：负责 `dest` 挂到 `src` 下面、`is_original(dest)` 置位这类 derivation tree patch 的合法性。
- `manager`：负责把 concrete exec、`cte` 后状态、`mdb` graph patch、`cdt` patch 串起来，最终收回总 `wf`。

这个例子的关键口径是：`cte_insert` 可以在顶层作为 operation 名存在，但 `mdb` owner 不应直接以 `insert` 为自己的核心 API，而应把它看成某种 insert-between / splice primitive 的一次实例化。

### Proof Backbone Freeze

本轮 mutation proof 的主骨架冻结为：

- owner-based post-state proof
- `unchanged` 降级为薄 frame 工具，不再作为主证明叙事
- `rank` 只服务 `mdb` 的 acyclic / topological-order 责任

对应约束：

- 每个 operation 默认只证明一个最终后状态，不再按微步中间态堆叠 proof。
- `mdb` 不直接以 `insert/move/delete/swap` 作为 owner API，而只证明 graph primitive 的保持性。
- `cdt` 同理只证明自己的 parent/original 更新与 tree invariant。
- `manager` 负责把顶层 operation 归约成 `cte` 写入、`mdb` primitive patch、`cdt` patch，再组合收回总 `wf`。

### Questions Settled

本轮讨论中已经冻结的关键判断如下：

- `cte_t` 不是纯 wrapper；它负责 slot-local contract，但不负责 whole-MDB / whole-CDT 自证。
- `mdb` 是图 owner，不应直接绑顶层 operation 语义；它只拥有图、图 primitive、图 invariant。
- `cdt` 与 `mdb` 并列，不再继续隐在 manager ghost 包里。
- backup 风格的 `unchanged` 主路线不再采用；保留的 `unchanged` 只服务 patch 外 frame。
- 若后续需要更强的 no-cycle 证明，优先在 `mdb` 层引入独立 `rank` witness，而不是把整个 operation proof 重新改回 manager 中心。

## Execution Plan

### Phase 0: Documentation Reset

目标：

- 归档旧文档
- 建立新的 baseline 文档
- 统一团队口径

完成标准：

- 新 `review-standard.md` 已生效
- 新 `project-verification-map.md` 已生效
- 本文档成为当前架构路线主文档

### Phase 1: Name The Layers In Code

当前状态：已完成。

目标：

- 在代码树中显式引入 `mdb` / `cdt` 目录或模块
- 不要求一步迁完 proof，但先把 ownership 名字立住

已落地动作：

1. 建立 `cspace/` 目录，并引入 `cte/`、`mdb/`、`cdt/`、`resolve/`、`manager/`。
2. 让 `cte` 成为 `cspace` 下面的 primitive object 层；顶层 `cte.rs` 退成 legacy compatibility surface。
3. 把 `mdb` / `cdt` 的主要 predicate owner 入口从 manager 老中心迁到对应子模块。
4. 保留 crate-level API 口径，通过薄 wrapper 接到新内部结构。

完成标准：

- 代码树中已经能看出 `cte / mdb / cdt / manager` 是分层关系
- 相关 predicate 不再都埋在单个大文件里

### Phase 2: Promote `cte_t` Into A Slot Object

当前状态：proof 前准备已完成，后续 proof 可继续补强 contract。

目标：

- 让 `cte_t` 承载真实 slot-local method
- 结束“new manager then forward”作为默认形态

已落地动作：

1. `cte_t` 类型和 slot-local impl 已迁入 `cspace::cte::{types,slot,spec,raw}`。
2. `get_ptr`、`get_offset_slot`、`derive_cap`、`ensure_no_children`、`is_final_cap` 等 slot-local 方法归入 `cspace::cte::slot`。
3. multi-slot operation 不再归入 `cspace::cte`；crate-level/free-function compatibility 统一经 `cspace::kernel` 进入长期 `CSpaceManager`。
4. `cspace::cte::spec` 不再依赖 manager 老中心名字；MDB owner 语义由 `MdbTable` 提供。

建议优先迁移：

- `get_ptr`
- `get_offset_slot`
- slot-local view/observer helper
- 局部 entry write helper

边界说明：

- `cte_insert` / `cte_move` / `cte_swap` / `delete_all` / `revoke` 仍是 multi-slot operation，不作为 `cte_t` 的本地证明主体。
- 它们的 crate-level/free-function API 只作为 compatibility surface，实际执行入口统一是 `cspace::kernel -> CSpaceManager`。

完成标准：

- `cspace::cte` 的主要叙事不再是 thin wrapper
- `cte_t` 已成为清晰的 slot abstraction boundary
- 外部 compatibility API 和 slot object API 已经分离

### Phase 3: Split Out `mdb`

当前状态：旧 `MdbState` 路线已经删除，MDB 新主线是 `MdbTable` owner。

目标：

- 把 MDB 图建模和相关全局验证从大 `CSpaceManager` 中提炼出来

已落地动作：

1. 删除 `cspace::mdb::{state,spec,proof}` 旧兼容路线。
2. 删除 `CSpaceManager::mdb_state()` 投影。
3. 引入 `MdbTable` owner：tracked entries 是唯一 concrete truth，`order/live_slots` 是 ghost summary。
4. insert 线使用 `MdbTable::insert_node_after` 和 `MdbTable::insert_between_rel`。

关键判断：

- 这一步不是只拆文件
- 也不是立刻重写所有 operation
- 而是先把“谁负责 MDB 复杂度”明确下来

完成标准：

- `mdb` 已经成为清晰的 proof-bearing submodule
- operation proof 不再把 MDB 细节无边界地摊开

### Phase 4: Split Out `cdt`

当前状态：proof 前准备已完成，保持性 lemma 仍待正式证明。

目标：

- 把 `cdt_parent` / `is_original` / derivation 语义提成独立层

已落地动作：

1. 提取 `cdt_parent_dom_wf` / `is_original_dom_wf` 到 `cspace::cdt::proof`。
2. 提取 `cdt_parent_slots_wf` / `cdt_parent_semantics_wf` / `derivation_wf` 到 `cspace::cdt::proof`。
3. 引入 `CdtState` 的显式 `dom`、独立 `CdtDepthWitness`，以及 owner post-state：`state_after_insert`、`state_after_move`、`state_after_swap`、`state_after_delete`。
4. manager operation 已改为通过 `old_mgr.cdt@.state_after_*` 更新 derivation ghost state。

完成标准：

- `cdt` 已经成为命名清楚的层
- derivation proof 不再散落在各 operation closeout 的尾部

### Phase 5: Rebuild `cspace` As Composition

当前状态：proof 前准备已完成，后续 operation proof 应在这个 composition shape 上推进。

目标：

- 让 `CSpaceManager` 回归组合层

已落地动作：

1. `CSpaceManager::wf` 已按 `slot_perms`、`mdb` structural/semantic、`cdt` derivation、non-mdb frame 组合。
2. manager 现在拥有 `MdbTable` / `CdtState`，以及所有 cap/cross-layer 语义入口；旧 `mdb_state()` 投影已经删除。
3. 旧 `cspace_manager` / `trusted` 源码目录已从当前 source tree 清出。

完成标准：

- `CSpaceManager` 更像 orchestrator
- 下层 contract 清晰，顶层组合简化

### Phase 6: Re-audit Operations Under The New Architecture

目标：

- 在新层次下重新看 `resolve / insert / move / swap / delete`

推荐顺序：

1. `resolve`
2. `insert`
3. `move`
4. `swap`
5. `delete / revoke`

理由：

- `resolve` 最独立，可先校准 `cspace` 外围边界
- `insert / move` 是 `cte + mdb + cdt` 协作的最小练手
- `swap` 次复杂
- `delete / revoke` 最适合放到后面重新结算 trusted boundary

## Archive Policy

旧文档整体保留在：

- `sel4_cspace/docs/archive/2026-05-13-layer-reset/`

使用规则：

- 需要历史 residual、旧 closeout 路线、旧 audit 细节时再回看
- 不再把 archive 里的优先级和命名直接带回当前主线
- 如果某个 archive 内容仍有现实价值，应在新文档中重写并重新定层，而不是直接继续引用旧口径

## Success Criteria

这次 reset 完成，不以“所有 proof 都过”作为唯一标准，而以这几件事为准：

1. 层次边界已经稳定，团队不再反复争论 `cte/mdb/cdt/cspace` 各自负责什么。
2. `cte_t` 已经不是纯 wrapper。
3. `mdb` 和 `cdt` 已经从大 `CSpaceManager` 中获得独立命名与独立 proof 叙事。
4. 顶层 `cspace` 的 `wf` 组合方式更清晰。
5. 后续 operation proof 的复杂度开始沿分层边界下降，而不是继续集中堆在单个 closeout 中。

## Proof-Entry State

正式写 mutation proof 前的准备工作当前冻结为：

1. 内部 canonical tree 是 `cspace::{cte,mdb,cdt,kernel,manager,resolve}`。
2. 顶层 `cte.rs` 只保留 crate-internal legacy compatibility surface；`interface.rs` 继续导出原 crate-level API。
3. `cspace::cte` 不再拥有 multi-slot operation body；`cte_t` 只保留 slot-local method，以及必要的 public method dispatch。
4. `cspace::mdb` 拥有 graph projection、MDB semantic edge vocabulary、primitive post-state、rank witness shell。
5. `cspace::cdt` 拥有 parent/original ghost state 和 operation post-state。
6. `cspace::kernel` 提供长期存在的 `CSpaceKernel { manager }` state，给 kernel-facing free functions 一个 atmo-style 接入点。
7. `cspace::manager` 是 orchestration + invariant combiner，后续 proof 应把 concrete mutation 归约到 `cte` write effect、`mdb` primitive patch、`cdt` post-state。

### Kernel Integration Shape

当前 kernel 接入口采用 atmo-style 长期 state，而不是每次从 raw pointer 临时重建 manager：

- `cspace::kernel::CSpaceKernel` 包含一个长期存在的 `CSpaceManager`。
- `init_cspace_kernel(...)` / `init_empty_cspace_kernel()` 负责安装全局 CSpace state。
- 原有 free functions 保持 API 不变；它们通过全局 `CSpaceKernel.manager` 调用 manager operation。
- `CSpaceKernel` 初始化是 CSpace operation 的前置条件；当前实现不再提供备用旧实现分支。
- 后续真正接 kernel 时，应在 boot/init 阶段把真实 CSpace slot domain、tracked slot perms、CDT/original ghost state 装进 `CSpaceManager`，而不是在每次 operation 调用时重新承认完整上下文。

### Duplicate Ops Policy

当前代码中同名 operation 允许出现在多个入口，但含义必须固定：

- `src/cte.rs`、`kernel_api`、`interface` 只能是 compatibility wrapper，不拥有真实语义；它们初始化后应转发到 `cspace::kernel` 的长期 manager state。
- `cspace::manager::impl_*` 是 proof 主线和 verified-core runtime surface，拥有 tracked slot mutation、owner post-state 更新和总 `wf` contract。
- `cspace::kernel` 是 kernel-facing dispatch 层；它不重新证明 operation，只负责把原 free function API 接到长期 `CSpaceKernel.manager`。
- `get_volatile_value` 这种 arch-specific helper 可以按 `riscv64` / `aarch64` 分 cfg 出现；但不能再通过第二套 CTE operation body 形成重复实现。

后续写 proof 时只消费 `cspace::manager` 的 operation body。`cspace::cte` 不得继续长出第二个 operation proof 中心；如果要减少 wrapper 层，应通过 `cspace::kernel` / public API bridge 收薄 compatibility surface，而不是让 manager 调回 pointer-style body。

## Immediate Next Steps

下一轮不再建骨架，而是开始正式 proof：

1. 先选 `insert` 作为第一条 owner-based post-state proof。
2. 打开或加强 `MdbTable::insert_node_after` 的内部证明，缩小当前 trusted owner primitive。
3. 为 move/swap/delete 补 `MdbTable` owner primitive；不再恢复 `MdbState` 投影。
4. 在 `cdt::proof` 中证明 `state_after_insert` 保持 derivation wf。
5. 只在 patch 外 frame 需要时使用 `unchanged` helper，不把它重新升级成主证明路线。
