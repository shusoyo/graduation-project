# CSpace 新架构实施方案

## 1. 实施目标

本文档回答四个执行问题：

1. 具体要做哪些部分；
2. 每个部分怎么设计；
3. 每个部分怎么实现；
4. 实现顺序和阶段验收是什么。

本文档默认以下总目标已经成立：

**保留现有 CSpace 语义资产，逐步做出一个可替换旧 `sel4_cspace` 的 Verus-native CSpace core。**

---

## 2. 要做的部分

新架构需要同时建设 6 个部分：

1. raw 表示边界
2. repr/view 层
3. verified object 层
4. subsystem context / owner 层
5. query / lookup / mutator 新接口
6. runtime / compat 过渡层

这 6 个部分都要做，但顺序不同。

---

## 2.1 具体要证明什么，怎么证明

新架构下，证明任务分成 6 类，每一类的证明目标与证明方法都不同。

### A. raw-to-view 正确性

#### 要证明什么

- raw `cap` 能被稳定解释成 `CapView`
- raw `mdb_node` 能被稳定解释成 `MdbView`
- raw `cte_t` 能被稳定解释成 `SlotView`
- raw 返回值能被稳定解释成结果视图

#### 怎么证明

- 每个 raw 类型提供单点 view 投影函数
- 证明这些投影函数的字段解释与当前既有 spec 对齐
- 尽量把证明局限在 `repr/*`

#### 证明产物

- `cap_repr::cap_view(...)`
- `mdb_repr::mdb_view(...)`
- `slot_repr::slot_view(...)`
- `result_repr::*_view(...)`

#### 验收标准

- 旧证明和新证明都可复用这些投影
- public proof 不再直接散落 `trusted_view_*`

### B. 局部不变量

#### 要证明什么

- 单个 `cap` 的 raw 表示与 `CapView` 一致
- 单个 `cte_t` 的 raw 表示与 `SlotView` 一致
- `SlotMut` 做单步更新后，更新后的 raw 值与新 `SlotView` 一致

#### 怎么证明

- 给 `CapRef` 定义最小 `wf()`
- 给 `SlotRef` 定义最小 `wf()`
- 给 `SlotMut` 的每个更新原语写 postcondition，说明更新后的 `wf()` 保持

#### 证明方法

- 只做对象内部字段一致性证明
- 不在这一层引入跨 slot / cnode / root 的关系
- 一次只证明一个对象自己的表示正确

#### 验收标准

- `CapRef::wf()`
- `SlotRef::wf()`
- `SlotMut::{set_cap,set_prev,set_next,...}` 都有局部 post

### C. 全局不变量

#### 要证明什么

- slot 集合是闭合且一致的
- `mdb_prev` / `mdb_next` 关系相互匹配
- `cnode_lookup` 与 slot 内容一致
- root 集合合法
- subsystem 级上下文满足统一 `wf()`

#### 怎么证明

- 定义 `CspaceCtx::wf()`
- 定义 `CspaceOwner::wf()`
- 将全局不变量拆成若干组合式子不变量：
  - slot-view consistency
  - mdb-link consistency
  - cnode-lookup consistency
  - root consistency

#### 证明方法

- 先把每一项写成独立 spec predicate
- 再把 `CspaceCtx::wf()` 定义为这些 predicate 的合取
- mutator 时逐项恢复，而不是一口气恢复一个巨型黑盒不变量

#### 验收标准

- 全局不变量有分项名字
- `CspaceCtx::wf()` 不是一坨不可拆的黑盒

### D. query 正确性

#### 要证明什么

- `same_region_as`
- `same_object_as`
- `is_cap_revocable`

这些 query 在对象视图上与既有抽象语义一致。

#### 怎么证明

- 先把 query 逻辑写在 `CapView` 或 `CapRef` 上
- 再证明它们等价于现有 `spec_*`

#### 证明方法

- 先局部展开 `CapView`
- 再按 capability kind 分支证明
- 最后做“new object query == old abstract query”的对照 lemma

#### 验收标准

- 至少 1 个 cap-level query 完全以 `CapRef` 为主语
- 旧 `spec_same_*` 仍可用于结果对照

### E. derived query / lookup 正确性

#### 要证明什么

- `is_mdb_parent_of`
- `is_final_cap`
- `ensure_no_children`
- `is_long_running_delete`
- `resolve_address_bits`

这些操作不仅依赖单个对象，还依赖 slot 间关系和 lookup 环境。

#### 怎么证明

- 对象内部部分通过 `SlotRef` / `SlotView` / `MdbView`
- 跨对象部分通过 `CspaceCtx`
- 最后对照当前 `CSpaceState` 语义

#### 证明方法

- 把证明拆成“局部关系 + 全局前提 + 结果对照”三段

例如：

- `is_mdb_parent_of`
  - 先证明 badge / revocable / same-region 条件
  - 再证明它等价于旧 `mdb_parent_of`

- `is_final_cap`
  - 先证明前驱/后继观察逻辑
  - 再证明它等价于旧 `state.is_final_cap(slot)`

- `resolve_address_bits`
  - 先证明 lookup 过程在 `CspaceCtx` 下合法
  - 再证明 `ResolveRetView` 等价于旧 expected core

#### 验收标准

- 至少 1 个 derived query 脱离旧 `heap observer + state id` 主叙事
- `resolve_address_bits` 的结果主要通过 `ResolveRetView` 解释

### F. mutator 正确性

#### 要证明什么

- `cte_insert`
- `insert_new_cap`
- `cte_move`
- `cte_swap`

每个 mutator 需要同时证明三件事：

1. raw 更新做对了
2. 局部/全局不变量保持了
3. 高层抽象语义与旧 `spec_*_post` 一致

#### 怎么证明

每个 mutator 分成两层证明：

##### 第一层：object-level update proof

证明：

- 哪些 slot 被改了
- 每个改动后的 `SlotMut` 仍满足局部 `wf()`
- `CspaceOwner` 的全局 `wf()` 被恢复

##### 第二层：semantic projection proof

证明：

- 更新后的对象视图组合起来
- 等价于旧 `spec_cte_insert_post` / `spec_cte_move_post` / ...

#### 证明方法

- 先把 mutator 内部更新原子化为 slot 级小更新
- 每个小更新证明局部 post
- 再组合成 subsystem-level `wf()`
- 最后做 old/new 语义对照

#### 验收标准

- mutator proof 明确分成两层
- 不再只有 `local_heap_transition_at` 一种主叙事

### G. 替换正确性

#### 要证明什么

最终新 core 的 query / lookup / mutator 语义与当前 `specs/*` 中已有语义一致，因此它能替换旧 `sel4_cspace` 的核心语义角色。

#### 怎么证明

- 每迁完一个操作族，就补一组对应 lemma：
  - new query == old query spec
  - new lookup == old lookup spec
  - new mutator post == old mutator post

#### 验收标准

- 新世界不是“另起一套语义”，而是语义上可投影回现有 `specs/*`

### H. CSpace 语义本体

#### 要证明什么

新架构不是只证明“对象包装层工作正常”，还必须证明 CSpace 作为一个系统满足当前抽象语义里已经固定下来的四类要求：

1. capability 语义正确
2. MDB 关系语义正确
3. CNode / lookup 语义正确
4. mutator 状态转移语义正确

#### 怎么证明

必须把每一类证明都显式对照回当前 `specs/*` 中的既有语义定义，而不是重新发明一套平行语义。

#### 来源是什么

直接来源如下：

- 抽象状态与全局不变量：
  [abstract_cspace.rs](/workspace/rel4_kernel/sel4_cspace/specs/abstract_cspace.rs)

- query / derived query 语义：
  [queries.rs](/workspace/rel4_kernel/sel4_cspace/specs/cspace_ops/queries.rs)

- `derive_cap` 语义：
  [derive.rs](/workspace/rel4_kernel/sel4_cspace/specs/cspace_ops/derive.rs)

- lookup / `resolve_address_bits` 语义：
  [resolve.rs](/workspace/rel4_kernel/sel4_cspace/specs/cspace_ops/resolve.rs)

- mutator 通用语义：
  [common.rs](/workspace/rel4_kernel/sel4_cspace/specs/cspace_ops/common.rs)

- `cte_insert` / `insert_new_cap`：
  [insert.rs](/workspace/rel4_kernel/sel4_cspace/specs/cspace_ops/insert.rs)

- `cte_move`：
  [move.rs](/workspace/rel4_kernel/sel4_cspace/specs/cspace_ops/move.rs)

- `cte_swap`：
  [swap.rs](/workspace/rel4_kernel/sel4_cspace/specs/cspace_ops/swap.rs)

---

## 2.2 CSpace 本体要证明什么，怎么证，来源是什么

这一节只讨论 **CSpace 语义本体**，不讨论一般性的架构口号。

### A. 抽象状态合法性

#### 要证明什么

任何能被新 core 公开暴露的 CSpace 状态，都必须满足当前 `CSpaceState::wf()` 展开的全局语义：

1. `mdb_state_wf()`
2. `cspace_lookup_wf()`
3. `cspace_roots_wf()`

继续展开，就是至少要维持：

- `valid_slots()`
- `mdb_prev_next_consistent()`
- `badge_derivation_wf()`
- `cnode_slots_wf()`
- `cnode_lookup_wf()`
- `cspace_graph_wf()`

#### 怎么证

新架构里应将这些条件拆成两层：

1. 局部对象层  
   `CapRef::wf()`、`SlotRef::wf()`、`SlotMut` 的更新 post

2. 全局上下文层  
   `CspaceCtx::wf()` / `CspaceOwner::wf()`

每个 mutator 的证明都要明确回答：

- 哪些局部对象的 `wf()` 被改变了；
- 哪些全局关系需要被恢复；
- 恢复后如何推出新的 `CspaceCtx::wf()`；
- 再由此投影回旧 `CSpaceState::wf()`。

#### 来源

- `CSpaceState::wf()` 及其分项：
  [abstract_cspace.rs](/workspace/rel4_kernel/sel4_cspace/specs/abstract_cspace.rs)

### B. capability 语义

#### 要证明什么

至少要保留当前抽象 capability 语义：

- `spec_same_region_as_caps`
- `spec_same_object_as_caps`
- `spec_is_cap_revocable`
- `valid_cap`
- `rights_subseteq`

这意味着新 core 里的 capability view / query 必须与今天的 capability 语义保持一致。

#### 怎么证

建议分两段证明：

1. `CapView` 与 `CapSpec` 的语义对齐  
   证明 `CapView` 的字段解释与当前 capability 抽象定义相同。

2. `CapRef` 上的 query 与旧 spec 对齐  
   对每个 query 证明：
   - `CapRef::same_region_as(...) == spec_same_region_as_caps(...)`
   - `CapRef::same_object_as(...) == spec_same_object_as_caps(...)`
   - `CapRef::is_cap_revocable(...) == spec_is_cap_revocable(...)`

#### 来源

- capability 抽象语义：
  [abstract_cspace.rs](/workspace/rel4_kernel/sel4_cspace/specs/abstract_cspace.rs)

### C. MDB 关系语义

#### 要证明什么

至少要保留当前关于派生树和邻接关系的抽象定义：

- `mdb_links`
- `immediate_derived`
- `mdb_parent_of`
- `mdb_parent_badge_compatible`
- `is_final_cap`
- `ensure_no_children_blocks`
- `slot_cap_long_running_delete`

#### 怎么证

要把旧状态机式定义拆成：

1. slot 局部语义  
   通过 `SlotView` / `MdbView` 读出：
   - `prev`
   - `next`
   - `revocable`
   - `first_badged`

2. slot 间关系  
   通过 `CspaceCtx` 保证：
   - 邻居 slot 存在
   - `prev/next` 关系与 lookup 上下文一致

3. 与旧 spec 的对照  
   对每个 derived query 证明结果等价于：
   - `state.mdb_parent_of(...)`
   - `state.is_final_cap(...)`
   - `state.ensure_no_children_blocks(...)`
   - `state.slot_cap_long_running_delete(...)`

#### 来源

- `mdb_*` 与 derived query 抽象定义：
  [abstract_cspace.rs](/workspace/rel4_kernel/sel4_cspace/specs/abstract_cspace.rs)
- query 契约入口：
  [queries.rs](/workspace/rel4_kernel/sel4_cspace/specs/cspace_ops/queries.rs)

### D. `derive_cap` 语义

#### 要证明什么

必须保留当前 `derive_cap` 的三件语义：

1. 哪些 capability 会被导出成 `null`
2. untyped 情况是否触发 `ensure_no_children`
3. 是否返回 syscall error

对应当前抽象语义：

- `spec_derive_cap_pre`
- `spec_derive_cap_expected_cap`
- `spec_derive_cap_returns_syscall_error`

#### 怎么证

建议新架构把 `derive_cap` 分成：

1. cap-level case split  
   按 `CapView.kind` 分支讨论。

2. slot/context dependency  
   untyped 分支依赖 `SlotRef` + `CspaceCtx` 证明 `ensure_no_children` 语义。

3. 返回值语义投影  
   `DeriveCapRetView` 与旧 `spec_derive_cap_*` 对齐。

#### 来源

- `derive_cap` 抽象契约：
  [derive.rs](/workspace/rel4_kernel/sel4_cspace/specs/cspace_ops/derive.rs)

### E. lookup / `resolve_address_bits` 语义

#### 要证明什么

必须保留当前 lookup 语义：

1. root 必须是合法 CNode cap
2. guard 检查正确
3. depth / radix / bits_remaining 解释正确
4. 返回的 slot 和剩余 bits 与抽象结果一致

对应当前抽象语义：

- `spec_resolve_address_bits_state_wf`
- `resolve_address_bits_expected_core_from_cap`
- `ResolveAddressBitsRetCoreSpec`

#### 怎么证

建议拆成四段：

1. `CapView` 级别证明  
   root cap 的 cnode 数据被正确解释。

2. `CspaceCtx` 级别证明  
   lookup map / root / reachable relation 提供查找环境。

3. 执行路径证明  
   对 guard mismatch、depth mismatch、success 逐分支证明。

4. 返回值对照  
   `ResolveRetView == old expected core`

#### 来源

- lookup 抽象语义：
  [resolve.rs](/workspace/rel4_kernel/sel4_cspace/specs/cspace_ops/resolve.rs)
- lookup 依赖的 graph / roots / cnode map 语义：
  [abstract_cspace.rs](/workspace/rel4_kernel/sel4_cspace/specs/abstract_cspace.rs)

### F. mutator 语义

#### 要证明什么

四个 mutator 都至少要保留三类语义：

1. frame semantics  
   哪些 slot 变，哪些 slot 不变

2. invariant preservation  
   改完以后 `wf()` 还成立

3. functional semantics  
   目标 slot / 邻居 slot / root 兼容性 / capability 内容变化符合预期

这是当前 mutator spec 的共同结构。

#### 怎么证

必须统一采用两层证明：

##### 第一层：对象级更新证明

证明：

- 具体哪些 `SlotMut` 被更新
- 每一步更新后局部 `wf()` 保持
- 更新结束后 `CspaceOwner::wf()` 恢复

##### 第二层：语义投影证明

证明：

- 更新后的 `SlotView` / `CspaceCtx` 组合起来
- 满足旧的：
  - `spec_cte_insert_post`
  - `spec_insert_new_cap_post`
  - `spec_cte_move_post`
  - `spec_cte_swap_post`

#### 具体四类 mutator 的语义来源

##### `cte_insert`

要保留：

- `dest` 必须空
- 新 entry 插到 `src` 后面
- old next 的 `mdb_prev` 需要重写
- source cap 可能发生 `setUntypedCapAsFull`
- `revocable` 由 `spec_is_cap_revocable` 决定

来源：

- [insert.rs](/workspace/rel4_kernel/sel4_cspace/specs/cspace_ops/insert.rs)
- [common.rs](/workspace/rel4_kernel/sel4_cspace/specs/cspace_ops/common.rs)

##### `insert_new_cap`

要保留：

- slot 必须空
- parent 之后插入新 slot
- new slot 的 entry 结构与 derivable/revocable 规则一致
- old next 的 `mdb_prev` 如有则重写

来源：

- [insert.rs](/workspace/rel4_kernel/sel4_cspace/specs/cspace_ops/insert.rs)

##### `cte_move`

要保留：

- source 被清空
- dest 获得 source 的 cap 与 mdb 位置
- source 的 prev/next 邻居重连到 dest
- capability compatibility 条件保持

来源：

- [move.rs](/workspace/rel4_kernel/sel4_cspace/specs/cspace_ops/move.rs)

##### `cte_swap`

要保留：

- 两个 slot 的 cap 与 mdb 位置交换
- 若邻居是外部 slot，则对应 prev/next 引用要重写
- roots 与 CNode 兼容性保持

来源：

- [swap.rs](/workspace/rel4_kernel/sel4_cspace/specs/cspace_ops/swap.rs)

### G. 替换正确性语义

#### 要证明什么

新 core 不只是“有一套自洽对象语义”，而是必须满足：

- 新 query 结果 = 旧 query spec
- 新 lookup 结果 = 旧 lookup spec
- 新 mutator post = 旧 mutator post

只有这样它才具备“可替换旧 `sel4_cspace`”的资格。

#### 怎么证

每完成一类操作，就必须补一组对照 theorem：

- `new_cap_query == old_cap_query_spec`
- `new_slot_query == old_slot_query_spec`
- `new_lookup == old_lookup_spec`
- `new_mutator_projection == old_mutator_post`

#### 来源

- 当前 `specs/*` 全部语义文件


---

## 3. 第一部分：raw 表示边界

### 3.1 目标

把 raw bitfield 世界固定成“边界输入”，不再让它直接成为主要证明语言。

### 3.2 涉及对象

- `sel4_common::structures_gen::cap`
- `sel4_common::structures_gen::mdb_node`
- `sel4_cspace::cte::cte_t`
- 结果类型：
  - `deriveCap_ret`
  - `resolveAddressBits_ret_t`
  - `exception_t`

### 3.3 设计原则

- 不重写 raw struct；
- 不在此层承载主要语义；
- 所有 raw getter/setter 的解释尽量集中收口。

### 3.4 实现位置

- 继续保留 `sel4_common/src/verify_bridge.rs`
- 继续保留并强化 `sel4_cspace/src/repr/*`
- 逐步压缩 `sel4_cspace/src/refinement_bridge.rs`

### 3.5 实现要求

- 每个 raw 类型都要有明确的单点投影函数；
- 每个返回结构都要有单点 view 解释；
- 尽量不在 `cte.rs` 中直接散落大量 raw 字段意义解释。

### 3.6 完成标志

- `cap`、`mdb_node`、`cte_t` 的逻辑读法主要走 `repr/*`
- 新写的 query/mutator proof 不再直接依赖大块 raw 字段拼装逻辑

---

## 4. 第二部分：repr/view 层

### 4.1 目标

构造稳定逻辑视图，取代今天很多“临时拼装 `CapSpec` / `SlotEntrySpec`”的做法。

### 4.2 建议新增或稳定化的类型

- `CapView`
- `MdbView`
- `SlotView`
- `ResolveRetView`
- `DeriveCapRetView`

### 4.3 设计要求

#### `CapView`

需要表达：

- `kind`
- `object`
- `region`
- `rights`
- `badge`
- `cnode data`
- `untyped data`

它在语义上接近今天的 `CapSpec`，但定位是“对象视图”，不是“系统外部 spec 的一片切片”。

#### `MdbView`

需要表达：

- `prev`
- `next`
- `revocable`
- `first_badged`

#### `SlotView`

需要表达：

- `cap: CapView`
- `mdb: MdbView`

#### 结果视图

需要把：

- `deriveCap_ret`
- `resolveAddressBits_ret_t`
- `exception_t`

都转成稳定逻辑视图，避免桥接名称散落在 public surface。

### 4.4 实现位置

- `sel4_cspace/src/repr/cap_repr.rs`
- `sel4_cspace/src/repr/mdb_repr.rs`
- `sel4_cspace/src/repr/slot_repr.rs`
- `sel4_cspace/src/repr/result_repr.rs`
- 如有必要，拆出 `view_types.rs`

### 4.5 实现方式

先不要重写所有旧 `spec`。

第一阶段可以：

- 直接让 `CapView` 与当前 `CapSpec` 同构；
- 直接让 `SlotView` 与当前 `SlotEntrySpec` 同构；
- 先通过“命名与模块边界改变”完成角色切换；
- 后续再逐步让它们从旧 spec 类型中解耦。

### 4.6 完成标志

- 每种 raw 值都有稳定逻辑视图；
- public proof 不再频繁直接提 `trusted_view_*`
- `view` 类型能成为下一层 `wf()` 的语义参照

---

## 5. 第三部分：verified object 层

### 5.1 目标

让对象第一次“带着规约活着”。

### 5.2 建议新增模块

- `sel4_cspace/src/verified/mod.rs`
- `sel4_cspace/src/verified/cap.rs`
- `sel4_cspace/src/verified/mdb.rs`
- `sel4_cspace/src/verified/slot.rs`

### 5.3 第一批对象

#### `CapRef`

职责：

- 持有一个 raw `cap` 的只读引用；
- 提供 `view()`；
- 提供最小 `wf()`；
- 承载 cap-level query。

建议接口：

- `view(self) -> CapView`
- `wf(self) -> bool`
- `same_region_as(self, other)`
- `same_object_as(self, other)`
- `is_cap_revocable(self, src)`

#### `SlotRef`

职责：

- 持有一个 raw `cte_t` 的只读引用；
- 提供 `view()`；
- 提供 slot 级只读语义；
- 支撑 derived query。

建议接口：

- `view(self) -> SlotView`
- `wf(self) -> bool`
- `cap(self) -> CapRef`
- `mdb(self) -> MdbView`

#### `SlotMut`

职责：

- 持有一个 raw `cte_t` 的可写引用；
- 在更新前后证明对象保持局部 `wf()`；
- 承担 slot 级小变更。

建议接口：

- `view(self) -> SlotView`
- `wf(self) -> bool`
- `set_cap(...)`
- `set_prev(...)`
- `set_next(...)`
- `set_revocable(...)`
- `set_first_badged(...)`

### 5.4 `wf()` 的设计

第一批 `wf()` 不要过重。

建议：

- `CapRef::wf()`  
  只表达 raw `cap` 与 `CapView` 一致

- `SlotRef::wf()`  
  只表达 raw `cte_t` 的组成部分与 `SlotView` 一致

不要一开始就把整个 CSpace 全局一致性塞进单个 slot 的 `wf()`。

### 5.5 实现顺序

1. `CapRef`
2. `SlotRef`
3. `SlotMut`
4. `MdbRef` 或仅 `MdbView`

### 5.6 完成标志

- 至少一个 cap-level query 直接在 `CapRef` 上完成
- 至少一个 slot-level query 直接在 `SlotRef` 上完成

---

## 6. 第四部分：subsystem context / owner 层

### 6.1 目标

承接 `atmo` 式 subsystem invariant 思路，但只做到 CSpace 子系统粒度。

### 6.2 建议新增模块

- `sel4_cspace/src/verified/cspace.rs`
- 如需要，再新增 `sel4_cspace/src/owner/*`

### 6.3 第一批对象

#### `CspaceCtx`

职责：

- 描述一段 proof 中默认可依赖的 CSpace 一致性环境；
- 为 query / lookup 提供跨 slot / cnode / root 的语义前提。

建议包含的语义：

- slot domain
- root set
- cnode lookup consistency
- slot view consistency

#### `CspaceOwner`

职责：

- 表达一组 slot/cnode/root 结构的可更新所有权；
- 为 mutator 提供更新权与 post-update 恢复点。

### 6.4 设计原则

- 第一阶段不强求复杂 tracked permission graph；
- 可以先用 ghost context + spec relation 承接；
- 等 query / lookup / 小 mutator 稳定后，再引入更强 owner 结构。

### 6.5 `wf()` 设计

建议拆成组合式：

- slot-level `wf`
- lookup-level `wf`
- root-level `wf`
- subsystem-level `wf`

最终：

- `CspaceCtx::wf()`
- `CspaceOwner::wf()`

由上述局部不变量组成。

### 6.6 完成标志

- `resolve_address_bits` 不再依赖旧式 heap observer 主语言
- 至少一个 derived query 依赖 `CspaceCtx` 而不是旧 `CSpaceState + heap id`

---

## 7. 第五部分：新接口与新证明组织

### 7.1 Query

优先迁移：

- `same_region_as`
- `same_object_as`
- `is_cap_revocable`

#### 设计方式

- 直接写在 `CapRef` 或 `CapView` 上；
- 证明风格为“对象读视图 + 局部 `wf()`”；
- 不再以旧 `interface.rs` wrapper 作为主要 public 入口。

#### 实现方式

- 先保留旧 public API
- 在内部把实现切到新对象方法
- 最后再决定 public surface 是否直接导出对象方法

### 7.2 Derived Query

优先迁移：

- `is_mdb_parent_of`
- `is_final_cap`
- `ensure_no_children`
- `is_long_running_delete`

#### 设计方式

- 主要依赖 `SlotRef` + `CspaceCtx`
- 通过 `SlotView` / `MdbView` 组织局部关系
- 最后再投影到旧抽象语义

### 7.3 Lookup

优先迁移：

- `resolve_address_bits`

#### 设计方式

- 返回值统一表达为 `ResolveRetView`
- lookup 正确性写成 `CspaceCtx` 下的 object-level 语义
- old/new state 证明退居解释层

### 7.4 Mutator

最后迁移：

- `cte_insert`
- `insert_new_cap`
- `cte_move`
- `cte_swap`

#### 设计方式

分两层证明：

1. object-level update  
   `SlotMut` / `CspaceOwner` 更新后保持 `wf()`

2. semantic projection  
   从更新后的 view/model 推出旧 `spec_*_post`

#### 实现方式

不要一开始全重写。

建议次序：

1. 先抽出 slot 级写操作
2. 再构造一个最小 mutator 样板
3. 再迁 `cte_insert`
4. 再迁其余 mutator

### 7.5 完成标志

- 至少一个 mutator 不再以 `local_heap_transition_at` 为主叙事
- object-level update 与 semantic projection 两层已经分开

---

## 8. 第六部分：runtime / compat 过渡层

### 8.1 目标

明确新 core 与旧世界的边界。

### 8.2 现有文件的处理

#### `cte.rs`

短期：

- 保留 runtime body
- 保留当前成熟实现
- 逐步把 proof 逻辑迁出

长期：

- 只剩 runtime glue 与少量紧贴实现的局部证明

#### `interface.rs`

短期：

- 保留稳定入口
- 内部逐步调用新 verified object API

长期：

- 变成 facade / re-export

#### `refinement_bridge.rs`

短期：

- 保留当前可用桥接
- 不再继续往里加新语义中心 helper

长期：

- 只剩 external spec / observer / witness

### 8.3 项目组织方向

长期应朝两个世界演化：

1. verified core
2. runtime / compat glue

即使短期内不拆成两个 Cargo crate，也应按这个边界做模块组织。

---

## 9. 具体实现顺序

### 阶段 0：文档与边界冻结

先冻结：

- 当前语义锚点：`specs/*`
- 当前 raw 边界：`sel4_common` bitfield types
- 当前不得继续扩张的模块：`refinement_bridge.rs`

### 阶段 1：稳定 `repr/*`

完成：

- `CapView`
- `MdbView`
- `SlotView`
- 结果视图统一

验收：

- 新旧 query proof 都能复用这些 view

### 阶段 2：建立 `verified/cap.rs`

完成：

- `CapRef`
- `CapRef::view`
- `CapRef::wf`
- cap-level query 样板

验收：

- `same_region_as`
- `same_object_as`
- `is_cap_revocable`

至少 1-2 个切到新风格。

### 阶段 3：建立 `verified/slot.rs`

完成：

- `SlotRef`
- `SlotMut`
- `SlotView`
- slot-local `wf`

验收：

- `is_mdb_parent_of`
- `is_final_cap`

至少一个切到新风格。

### 阶段 4：建立 `verified/cspace.rs`

完成：

- `CspaceCtx`
- `CspaceOwner`
- lookup / root / slot relation 的组合式 `wf`

验收：

- `ensure_no_children`
- `resolve_address_bits`

至少一个切到 `CspaceCtx` 风格。

### 阶段 5：迁 mutator

完成：

- `cte_insert` object-level update proof
- semantic projection proof

验收：

- `cte_insert` 脱离旧 local transition 主轴

再按同样模式迁：

- `insert_new_cap`
- `cte_move`
- `cte_swap`

### 阶段 6：收口旧层

完成：

- `interface.rs` 变薄
- `refinement_bridge.rs` 降级
- 旧 public proof shell 删除或转内部使用

验收：

- public proof surface 主要由 verified object API 构成

---

## 10. 每阶段都要回答的验证问题

每做一个阶段，都要检查：

1. 这个新对象的 `view()` 是否稳定？
2. 这个对象的 `wf()` 是否过重？
3. 是否在无意中又造了一个新 bridge？
4. 是否还能把结果投影回当前 `specs/*` 语义？
5. 是否真的减少了 public proof surface 的中间层？

---

## 11. 当前最优先的第一批编码任务

如果现在就开始写代码，优先级应该是：

1. 统一 `repr/*` 的 view 类型命名
2. 在 `verified/` 下创建骨架模块
3. 实现 `CapRef` 与 `CapView`
4. 迁 `same_region_as`
5. 迁 `same_object_as`
6. 实现 `SlotRef` 与 `SlotView`
7. 迁 `is_mdb_parent_of`

不要先做：

- 完整 `CspaceOwner` tracked graph
- 四个 mutator 一起重写
- 删除旧 `specs/*`
- 删除旧 `refinement_bridge.rs`

---

## 12. 终局判据

当下面这些成立时，可以认为新架构基本成型：

- query 主要写在 `CapRef` / `SlotRef` 上
- lookup 主要写在 `CspaceCtx` 上
- mutator 主要写在 `SlotMut` / `CspaceOwner` 上
- `specs/*` 退为模型与对照层
- `refinement_bridge.rs` 退为小型 TCB
- `interface.rs` 退为 facade
- 核心 CSpace 逻辑已经主要活在 Verus-native object world 中

这时，`sel4_cspace` 就不再只是“旧实现外贴证明”，而开始真正向“可替换旧实现的 Verus-native CSpace core”演化。
