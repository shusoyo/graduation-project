# reL4 CSpace Verus-Native 细化重构计划

## 1. 文档目的

本计划服务于如下目标：

- 继续借鉴 `l4v` 对 CSpace 的语义划分、性质命名和证明目标；
- 不再模仿 `l4v` 的证明组织方式；
- 将当前 `sel4_cspace` 的验证风格从“bridge-heavy / contract-heavy”收缩为“Verus-native / spec-direct / local-proof”；
- 给出可以直接执行的文件级、函数级重构步骤；
- 对每个核心函数说明最终应证明什么、依赖什么、证明应如何组织，但不提供具体代码。

本计划不追求一步到位消灭所有 trusted boundary。目标是：

- 保留必要的 concrete-to-abstract 表示桥接；
- 删除多余的中转 contract、重复包装层和对外 proof 壳；
- 让“核心语义 postcondition”尽可能直接写在实现证明入口上。

---

## 2. 总体判断

当前代码的“证明内容”基本正确，但“证明组织方式”离目标还有明显差距。

### 2.1 当前风格的主要问题

- `interface.rs` 目前承担了过多 proof-surface 职责，不只是薄 API 层，而是额外引入了 `*_at_pre`、`*_at`、`verify_*`、返回值解释器、状态观察器包装。
- `cte.rs` 里很多函数仍然优先证明 `*_exec_contract`，而不是直接证明对应的抽象性质。
- `refinement_bridge.rs` 目前既做表示投影，又承担了较多辅助解释和 trusted witness 职责，边界偏厚。
- 查询函数、派生查询函数、mutator 的最终结构不统一：有些已经接近 Verus-native，有些仍在依赖中间壳。

### 2.2 目标风格

目标架构应当是：

1. `specs/*`
   只表达抽象语义与状态转移，不表达运行时实现细节。
2. `refinement_bridge.rs`
   只保留表示投影、observer、少量 trusted constructor witness。
3. `cte.rs` / `capability/mod.rs`
   作为主要证明入口，直接给出最终 postcondition。
4. `interface.rs`
   如果保留，只保留真正需要的对外稳定入口；不再作为第二套 contract 语言。

一句话概括：

**保留“桥”，删除“壳”；保留“抽象语义”，删除“重复语义中介”。**

---

## 3. 重构后的目标分层

## 3.1 `specs/*` 的职责

保留并继续扩展以下内容：

- `abstract_cspace.rs`：抽象状态、cap/slot 观察、基础谓词；
- `cspace_ops/queries.rs`：查询类 post；
- `cspace_ops/derive.rs`：`derive_cap` 前后条件；
- `cspace_ops/resolve.rs`：`resolve_address_bits` 抽象返回值语义；
- `cspace_ops/insert.rs`、`move.rs`、`swap.rs`：mutator 的状态转移与 expected-entry lemma。

禁止继续向 `specs/*` 中加入如下内容：

- 直接描述 raw `cap` / raw `cte_t` 地址读取过程；
- 用于“解释某个 Rust 返回结构体字段”的临时壳；
- 本质上只是 bridge 的别名。

## 3.2 `refinement_bridge.rs` 的职责

这一层必须保留，但必须瘦身到“表示边界”。

允许保留：

- `cap`、`cte_t`、返回结构体的 view/projection；
- heap/slot observer；
- `external_type_specification`；
- 少量 small constructor witness，例如“构造一个状态为 NONE 的返回值”。

逐步迁出的内容：

- 对核心 CSpace 语义的解释；
- 仅仅为了服务 `interface.rs` 的同义包装；
- 可以在 `cte.rs` 本地证明的辅助壳。

最终要求：

- `refinement_bridge.rs` 中不出现“这个操作的语义是什么”；
- 只出现“这个 raw 值在抽象上看起来是什么”。

### 3.2.1 TCB 命名规范

本项目后续不应继续泛用 `trusted_*` 作为统一前缀。

原因不是“`trusted` 一定错误”，而是它把几类完全不同的验证边界混在了一起：

- 逻辑公理；
- 临时假设；
- 纯表示投影；
- 外部函数的 witness constructor；
- observer / extractor。

在 Verus / `vostd` 风格下，这几类对象应该严格区分命名。

#### 一、`axiom_*`

只用于以下对象：

- `pub axiom fn ...`
- `broadcast axiom fn ...`
- 明确作为逻辑公理引入、没有执行体、其正确性不由当前 crate 内证明的事实。

适用语义：

- “这是一个作为逻辑公理引入的事实”；
- “这不是投影函数，也不是运行时构造器，而是 proof system 的外加公理”。

不适用对象：

- `#[verifier::external_body] fn ...`
- 纯 `view` 投影；
- 返回值构造器；
- observer。

结论：

- 不能把现在所有 `trusted_*` 都无脑改成 `axiom_*`。

#### 二、`assume_*`

只用于以下对象：

- 具有明显“暂时假设 / 未完成建模 / 待消除占位”性质的内容；
- 对应 `assume(...)`、`assume_specification[...]`、或 boundary assumption 风格的临时桥接。

适用语义：

- “这里不是系统公理，而是当前阶段先假定成立，后续应当消除或下沉”。

不适用对象：

- 稳定长期保留的 view/projection；
- 返回值构造器；
- 已经有明确 `ensures` 且作为 TCB witness 固化的 helper。

结论：

- 不能把长期保留的 `trusted_view_cap` 之类改成 `assume_view_cap`。
- `assume_*` 会给读者明确暗示：这是技术债，而不是稳定边界。

#### 三、`view_*` / `*_view`

用于纯表示投影：

- raw `cap` 到 `CapSpec`
- raw `cte_t` 到 `SlotEntrySpec`
- raw 返回结构体到抽象结果视图

适用语义：

- “这是一个抽象观察视图”；
- “它的职责是 projection，不是公理，不是临时假设”。

推荐用法：

- `cap_view`
- `cte_view`
- `derive_cap_ret_view`
- `resolve_address_bits_ret_view`

如果当前函数是 bridge 层的唯一入口，也可以接受：

- `view_cap`
- `view_cte`

但要避免同一项目中同时混用 `trusted_view_cap`、`cap_view`、`view_cap` 三套同义名字。

#### 四、`bridge_*`

用于从 raw/external 值提取 ghost snapshot 或 bridge wrapper 的对象。

适用语义：

- “这是桥接对象，不是最终抽象语义”；
- “它提供的是快照或 wrapper，而不是逻辑公理”。

推荐用法：

- `bridge_cap`
- `bridge_cte`
- `bridge_resolve_ret`

这类命名在本项目中已经比较自然，应继续保留。

#### 五、`make_*` / `mk_*`

用于小型 witness constructor。

适用对象：

- 构造一个“抽象上对应 null cap”的 raw cap；
- 构造一个“抽象上对应 success/fault core”的返回桥对象；
- 构造一个 NONE / SYSCALL_ERROR 状态值。

推荐用法：

- `make_null_cap`
- `make_exception_none`
- `make_exception_syscall_error`
- `make_resolve_address_bits_fault_ret`

如果其正确性来自 `external_body` + `ensures`，它仍然不是 `axiom_*`，也不该叫 `assume_*`。

#### 六、`check_*` / `is_*`

用于从 raw/external 值中提取布尔观察结果的 helper。

适用对象：

- 判断异常返回是否为 NONE；
- 判断 tag 是否属于某个 kind；
- 判断某个返回结构体是否处于某状态。

推荐用法：

- `check_exception_is_none`
- `exception_is_none`
- `derive_cap_ret_is_syscall_error`

### 3.2.2 对当前 `trusted_*` 的具体重命名策略

当前项目中的 `trusted_*` 应逐步拆解成下面几类，而不是整体替换成 `axiom_*` 或 `assume_*`。

#### 应改成 `*_view` / `view_*`

适用对象：

- `trusted_view_cap`
- `trusted_view_cte`
- `trusted_view_derive_cap_ret_capability`

目标语义：

- 这些是“抽象视图”，不是“相信它对”。

#### 应改成 `make_*`

适用对象：

- `trusted_make_null_cap`
- `trusted_make_exception_none`
- `trusted_make_exception_syscall_error`
- `trusted_make_derive_cap_ret`
- `trusted_make_resolve_address_bits_fault_ret`
- `trusted_make_resolve_address_bits_success_ret`

目标语义：

- 这些是 witness constructor。

#### 应改成 `check_*` / `*_is_*`

适用对象：

- `trusted_check_exception_is_none`
- `trusted_exception_is_none`
- `trusted_exception_is_syscall_error`
- `trusted_derive_cap_ret_is_none`
- `trusted_derive_cap_ret_is_syscall_error`

目标语义：

- 这些是状态判定或抽象判定函数。

#### 可以保留 `bridge_*`

适用对象：

- `bridge_cap`
- `bridge_cte`
- 各类 bridge wrapper / snapshot type

#### 只有真正的逻辑公理才使用 `axiom_*`

例如未来若出现如下情况，才使用 `axiom_*`：

- 某个硬件事实无法在当前层证明，只能以 `axiom fn` 注入；
- 某个全局不变量暂时作为逻辑公理引入。

#### 只有明确的阶段性假设才使用 `assume_*`

例如未来若出现如下情况，才使用 `assume_*`：

- 暂时无法证明、计划后续消除的占位规范；
- 明确标记为“过渡期未完成”的 boundary assumption。

### 3.2.3 最终建议

对你这套代码，我建议的总体命名原则是：

- 不采用“全量 `trusted_* -> assume_*`”；
- 也不采用“全量 `trusted_* -> axiom_*`”；
- 而是做一次**按语义分类的命名消毒**。

一句话版本：

- 视图叫 `view`
- 桥对象叫 `bridge`
- 构造器叫 `make`
- 公理叫 `axiom`
- 临时假设叫 `assume`

这比单纯把 `trusted` 换成另一个统一前缀，更符合 Verus-native 和 `vostd` 风格。

## 3.3 `cte.rs` 的职责

`cte.rs` 应成为以下两类事情的主战场：

- 运行时逻辑本体；
- 从 bridge 视图到抽象 spec 的局部精化证明。

`cte.rs` 中保留的证明应是：

- 以局部 lemma 为单位的“字段级对齐”；
- 以函数为单位的“ret / new state 满足抽象 post”；
- 对 query/derive/mutator 的最终证明入口。

`cte.rs` 中逐步删除的东西：

- 面向外部调用者暴露的 `*_exec_contract`；
- 只作为 public wrapper 存在的二次包装函数；
- 与 `interface.rs` 重复表达的 postcondition。

## 3.4 `interface.rs` 的职责

重构后，`interface.rs` 只允许承担二选一中的一种：

1. 真正的公共稳定验证入口；
2. 纯 re-export 层。

不允许继续承担：

- 第二套 contract DSL；
- 对每个函数复制一遍 `*_at_pre` 与 `*_at`；
- 将 `cte.rs` 的局部 proof backend 再包装成一层 public proof language。

最终建议：

- 查询类函数优先直接由 `capability/mod.rs` 和 `cte.rs` 暴露；
- mutator 若确实需要对外稳定验证入口，可以保留一个薄 wrapper；
- 但 wrapper 的 `ensures` 直接写最终 spec post，而不是引用旧 contract 名字。

---

## 4. 文件级目标状态

## 4.1 `sel4_cspace/src/capability/mod.rs`

目标：

- 成为查询类 API 的唯一对外入口；
- 非验证模式走 runtime body；
- 验证模式直接连接 `*_refined`；
- 对外只保留一个函数名，不再出现 runtime/verified 双出口污染。

最终适用函数：

- `same_region_as`
- `same_object_as`
- `is_cap_revocable`

每个函数的 postcondition 都应直接陈述：

- 返回值等于对应的抽象 spec；
- 不再引用 `*_exec_contract`。

## 4.2 `sel4_cspace/src/cte.rs`

目标：

- 保留 runtime body；
- 保留 query/derive/mutator 的主证明；
- 删除可删的 contract 壳；
- 将辅助证明内聚到具体函数附近。

最终应保留的“局部证明入口”：

- `same_region_as_refined`
- `same_object_as_refined`
- `is_cap_revocable_refined`
- `is_mdb_parent_of_refined`
- `is_final_cap_refined`
- `ensure_no_children_refined`
- `is_long_running_delete_refined`
- `derive_cap_refined`
- `resolve_address_bits_refined`

## 4.3 `sel4_cspace/src/interface.rs`

目标：

- 大幅变薄；
- 删除绝大多数 `*_at_pre` / `*_at` 重复结构；
- 如果某个 mutator 仍需对外验证包装，则只保留“一个 public wrapper + 最终 spec post”。

优先删除对象：

- `verify_view_cap`
- `verify_heap_matches_state_at`
- `derive_cap_ret_*`
- `resolve_address_bits_ret_*`
- `is_final_cap_at_pre` / `ensure_no_children_at_pre` / `derive_cap_at_pre` 等成套 pre wrapper
- `cte_insert_at` 与 `cte_insert` 这种成对重复包装

保留条件：

- 只有当外部证明客户端真的需要稳定 surface，且该 surface 不能直接依赖 `cte.rs` 时才保留。

## 4.4 `sel4_cspace/src/refinement_bridge.rs`

目标：

- 保持为 TCB 边界；
- 但只保留 projection 与 observer；
- 不继续成为“第二个语义模块”。

---

## 5. 总体迁移顺序

建议按四个阶段推进。

## 阶段 A：清理查询类函数

范围：

- `same_region_as`
- `same_object_as`
- `is_cap_revocable`

目标：

- 先把最容易 Verus-native 化的部分做干净；
- 建立统一模板；
- 降低后续派生函数和 mutator 的复杂度。

完成标志：

- `capability/mod.rs` 中查询 API 只有单一对外入口；
- `cte.rs` 中不再需要对应的 `*_exec_contract`；
- `interface.rs` 不再为这三个函数提供额外 proof shell。

## 阶段 B：清理派生查询函数

范围：

- `is_mdb_parent_of`
- `is_final_cap`
- `ensure_no_children`
- `is_long_running_delete`
- `derive_cap`

目标：

- 用“直接 spec post + 局部桥接证明”取代“先证明子函数 contract，再包一层上层 contract”的风格。

完成标志：

- 这些函数的最终 `ensures` 直接指向抽象谓词或抽象返回值语义；
- `interface.rs` 的 `*_at_pre` 与 `*_at` 成对包装大幅减少。

## 阶段 C：清理 lookup

范围：

- `resolve_address_bits`

目标：

- 保留返回值 bridge；
- 但让 proof entry 直接陈述最终返回 core 与 `spec` 的一致性。

## 阶段 D：清理 mutator

范围：

- `cte_insert`
- `insert_new_cap`
- `cte_move`
- `cte_swap`

目标：

- 保留必要的 heap transition observer；
- 删除 public wrapper 中重复的 state-post 展开；
- 统一为“一个 public verification entry，直接指向 `spec_*_post` 与 expected-entry post”。

---

## 6. 函数级重构说明

下面按函数给出最终形态、保留内容、删除内容、证明方法、验收标准。

## 6.1 `same_region_as`

### 最终形态

- 对外入口放在 `capability/mod.rs`；
- 验证模式直接保证“返回值等于 `spec_same_region_as_caps(view(cap1), view(cap2))`”；
- `cte.rs` 中保留局部证明入口；
- `interface.rs` 不再提供这个函数的单独验证壳。

### 保留内容

- `bridge_cap`；
- cap tag 到 `CapKind` 的对齐 lemma；
- object/range/cnode 字段对齐 lemma；
- `same_region_as_from_bridges` 这一类从快照到 spec 的局部推导。

### 删除内容

- 与 `same_region_as` 对应的 `exec_contract`；
- 所有仅用于“证明 contract 是确定的”之类的冗余 lemma；
- 对外 second wrapper。

### 如何证明

证明应分三层：

1. 先证明桥接后的 `CapBridge.view()` 与 `trusted_view_cap(raw_cap)` 一致；
2. 再证明基于 snapshot 计算出来的布尔值，与 `spec_same_region_as_caps(cap1.view(), cap2.view())` 一致；
3. 最后把两者拼接，得到 raw 输入层面的最终 post。

核心技巧：

- 每个 tag 分支只证明本分支；
- 把 `object_present`、`badge_present`、`cnode_present` 之类字段存在性证明做成局部 shape lemma；
- 不要再构造一个“contract 函数”去代替最终命题。

### 验收标准

- public postcondition 直接是 `ret == spec_same_region_as_caps(...)`；
- 外部调用者无需知道 `exec_contract` 这个名字。

## 6.2 `same_object_as`

### 最终形态

与 `same_region_as` 相同。

### 保留内容

- `bridge_cap`；
- kind/tag 对齐 lemma；
- same-object 的字段级对齐 lemma。

### 删除内容

- 对应 `exec_contract`；
- 对外包装壳。

### 如何证明

- 证明路线与 `same_region_as` 相同；
- 特别注意 `UntypedCap`、`IRQControlCap`、`ZombieCap`、`ArchCap` 的“直接为 false”分支；
- CNode 分支要显式证明 object 一致且 radix 一致。

### 验收标准

- `cte.rs` 中局部证明结束于抽象 post；
- `capability/mod.rs` 直接暴露最终语义。

## 6.3 `is_cap_revocable`

### 最终形态

- 对外入口继续放在 `capability/mod.rs`；
- 后置条件直接陈述 `ret == spec_is_cap_revocable(...)`。

### 保留内容

- `bridge_cap`；
- tag 到 kind 的等价 lemma；
- badge 字段存在性与取值对应关系。

### 删除内容

- 与 deterministic 相关的旧 lemma；
- 查询壳式 contract。

### 如何证明

- endpoint、notification、irq_handler、untyped 四个主要分支逐一对齐；
- 其他分支统一证明为 false；
- 重点不是复述 runtime 分支，而是建立每个分支与 `spec_is_cap_revocable` 的等价。

### 验收标准

- public API 不再依赖任何中间 contract 名字；
- 证明脚手架只留 tag/kind 对齐和字段形状 lemma。

## 6.4 `is_mdb_parent_of`

### 最终形态

- 直接保证“返回值等于 `state.mdb_parent_of(parent, child)` 或等价的 `spec_is_mdb_parent_of_post`”；
- 若保留 `spec_is_mdb_parent_of_post`，它应只是最终命题的名字，而不是额外翻译层。

### 保留内容

- `is_mdb_parent_of_call_pre_at` 或等价的 raw-to-state 对应前置；
- parent/child 两个 `cte` 的 view 对齐；
- `same_region_as` 作为子证明调用；
- badge compatibility 的抽象谓词。

### 删除内容

- `is_mdb_parent_of_exec_contract` 作为外部 contract 名字；
- `interface.rs` 中与之成套重复的 `_at_pre/_at` 包装，除非外部客户端确实依赖。

### 如何证明

分四步：

1. 从 precondition 推出 `raw_parent/raw_child` 分别对应 `state.slot_entry(parent/child)`；
2. 调用 `same_region_as` 的最终语义结果；
3. 单独证明 badge 兼容条件与抽象谓词一致；
4. 合成 `parent_revocable && same_region && badge_compatible` 与 `state.mdb_parent_of(parent, child)` 的等价。

### 验收标准

- `is_mdb_parent_of` 的最终 post 是抽象父子关系，不再是中间 contract 名称。

## 6.5 `is_final_cap`

### 最终形态

- 直接保证 `ret == state.is_final_cap(slot)`。

### 保留内容

- slot raw view 与 `state.slot_entry(slot)` 的对齐 lemma；
- 对前后邻居 slot 的 raw-to-state 对齐；
- `same_object_as` 的最终语义调用。

### 删除内容

- `is_final_cap_exec_contract` 作为 public-facing 目标；
- `interface.rs` 中对它的重复包装。

### 如何证明

分三种情况：

- 无前驱；
- 无后继；
- 同时有前驱和后继。

证明策略：

- 前驱存在时，通过 `same_object_as(prev, slot)` 证明 `prev_same` 与抽象关系一致；
- 后继存在时，通过 `same_object_as(slot, next)` 证明 `next_same` 与抽象关系一致；
- 最后证明 `!prev_same && !next_same` 与 `state.is_final_cap(slot)` 一致。

### 验收标准

- 外部调用只看得到“final cap”的最终语义。

## 6.6 `ensure_no_children`

### 最终形态

- 直接保证：
  - 返回状态是否 `NONE` 等于“不会阻塞”；
  - 返回状态是否 `SYSCALL_ERROR` 等于“会阻塞”。

### 保留内容

- slot view 对齐；
- next 指针到抽象 `mdb_next` 的对齐；
- `is_mdb_parent_of` 的最终语义调用；
- 异常返回值的 bridge witness。

### 删除内容

- `ensure_no_children_exec_contract` 作为主要目标；
- 只为公共包装服务的 `_at_pre/_at`。

### 如何证明

证明应围绕“是否存在 next 且是否为 parent”展开：

1. 若无 next，直接证明抽象上 `ensure_no_children_blocks(slot)` 为 false；
2. 若有 next，先建立 raw child 与抽象 child 的对应；
3. 调用 `is_mdb_parent_of`；
4. 将 runtime 返回值映射到抽象布尔条件。

### 验收标准

- postcondition 直接写异常语义与抽象阻塞条件的等价；
- 不再先证明 contract，再从 contract 推导最终语义。

## 6.7 `is_long_running_delete`

### 最终形态

- 直接保证 `ret == state.slot_cap_long_running_delete(slot)`。

### 保留内容

- `is_final_cap` 的最终语义调用；
- slot cap tag 的桥接；
- null/thread/zombie/cnode 等删除类别判断。

### 删除内容

- `is_long_running_delete_exec_contract` 的 public 使用。

### 如何证明

按结构证明：

1. 先调用 `is_final_cap`；
2. 证明 raw tag 是否为 null / thread / zombie / cnode；
3. 将 runtime 判定条件与抽象谓词 `slot_cap_long_running_delete` 对齐。

### 验收标准

- 对外只有最终布尔语义。

## 6.8 `derive_cap`

### 最终形态

- 直接保证：
  - capability 结果等于 `spec_derive_cap_expected_cap(...)`；
  - syscall error 状态等于 `spec_derive_cap_returns_syscall_error(...)`；
  - none 状态等于其否定。

### 保留内容

- 返回结构体的 bridge；
- `trusted_make_derive_cap_ret` 这类 small witness；
- `ensure_no_children` 的最终语义调用；
- capability clone/null 的 bridge witness。

### 删除内容

- `derive_cap_exec_contract` 作为 public-facing 语义层；
- `interface.rs` 中对返回值状态的额外同义函数，除非确有外部客户端依赖。

### 如何证明

按照 capability tag 分支证明：

- zombie；
- untyped；
- reply / irq_control；
- default clone。

每个分支都要同时完成三件事：

1. 结果 cap 的抽象视图正确；
2. error/nil 状态正确；
3. 与 `spec_derive_cap_post` 或其展开目标一致。

特别注意：

- untyped 分支不要再以 `derive_cap_exec_contract` 为主要终点；
- 应直接把 `ensure_no_children` 的语义结果接到 `spec_derive_cap_expected_cap` 上。

### 验收标准

- public postcondition 直接表达返回结果与抽象导出语义的对应。

## 6.9 `resolve_address_bits`

### 最终形态

- 保留返回值 bridge；
- 直接保证返回 core 等于 `resolve_address_bits_expected_core(...)`。

### 保留内容

- `ResolveAddressBitsRetBridge`；
- success/fault constructor witness；
- root cap 的 bridge；
- resolve one-step/refinement lemma。

### 删除内容

- `interface.rs` 中针对 status/slot/bits_remaining 的重复展开包装，如果外部客户端不直接依赖可删。

### 如何证明

建议分成两个层次：

1. `cte.rs` 中证明返回 bridge 的 `view()` 等于抽象 expected core；
2. 如果外部需要拆 status/slot/bits_remaining，再由极薄 wrapper 做投影，不再复制整套逻辑。

### 验收标准

- 核心语义只有一个：返回 core 与 abstract core 一致。

## 6.10 `cte_insert`

### 最终形态

- 一个 public verification entry；
- 后置条件直接写：
  - `new_heap` 匹配 `new_state`；
  - `spec_cte_insert_post(old_state, new_state, ...)` 成立；
  - 如有必要，再写 `src/dest` 的 expected entry。

### 保留内容

- `cte_insert_call_pre_at`；
- `cte_insert_local_heap_transition_at`；
- `spec_cte_insert_post`；
- expected-entry lemma；
- local-heap-transition 推 expected-view 的 lemma。

### 删除内容

- `cte_insert_at` 与 `cte_insert` 的双层重复，如果它们 post 完全同构，应合并；
- `interface.rs` 里只做同义转发的壳。

### 如何证明

mutator 统一采用同一套路：

1. 调用 runtime body；
2. 用 local heap transition observer 建立 `old_heap -> new_heap` 的局部变化事实；
3. 从 observer 推出 `new_heap` 对应 `new_state`；
4. 调用 `spec_cte_insert_post`；
5. 用 expected-entry lemma 证明 `src/dest` 两个关键 slot 的抽象内容。

关键原则：

- observer 只证明“哪些位置变了、变后长什么样”；
- 抽象 post 只在 `specs` 中定义；
- public post 不再通过 `exec_contract` 转一遍。

### 验收标准

- mutator 公共入口只有一层；
- 读者只需看最终 spec post，不需要知道旧 contract 名字。

## 6.11 `insert_new_cap`

### 最终形态

与 `cte_insert` 同风格。

### 保留内容

- `insert_new_cap_call_pre_at`；
- `insert_new_cap_local_heap_transition_at`；
- `spec_insert_new_cap_post`；
- expected parent/slot entry lemma。

### 删除内容

- `insert_new_cap_at` 与 `insert_new_cap` 的重复层。

### 如何证明

- 路线与 `cte_insert` 相同；
- 重点在于 parent slot 与新 slot 两个位置的变化；
- 证明上优先依赖 abstract post + expected-entry lemma，而不是在 wrapper 中手写太多中间陈述。

### 验收标准

- 对外 post 直接指向 `spec_insert_new_cap_post`。

## 6.12 `cte_move`

### 最终形态

- 一个 public verification entry；
- 直接保证 `spec_cte_move_post` 与 expected `src/dest` entry。

### 保留内容

- `cte_move_call_pre_at`；
- `cte_move_local_heap_transition_at`；
- `spec_cte_move_post`；
- expected-entry 与 expected-view lemma。

### 删除内容

- `cte_move_at` 与 `cte_move` 的重复壳。

### 如何证明

- 与 `cte_insert` 同套路；
- 特别注意 `src` 变为空、`dest` 接收新 cap 这两个效果要分开证明。

### 验收标准

- public post 直接面向抽象 move 语义。

## 6.13 `cte_swap`

### 最终形态

- 一个 public verification entry；
- 直接保证 `spec_cte_swap_post` 与两个 slot 的 expected entry。

### 保留内容

- `cte_swap_call_pre_at`；
- `cte_swap_local_heap_transition_at`；
- `spec_cte_swap_post`；
- expected slot lemma。

### 删除内容

- `cte_swap_at` 与 `cte_swap` 的重复壳。

### 如何证明

- 与其他 mutator 同套路；
- 重点证明两个 slot 的内容交换、相邻指针修正和 abstract state 中对应 entry 的置换。

### 验收标准

- 对外 post 直接是抽象 swap 语义。

---

## 7. `interface.rs` 的具体处理策略

`interface.rs` 需要分三轮清理。

## 第一轮：删除查询类重复包装

删除对象：

- `is_final_cap_at_pre`
- `ensure_no_children_at_pre`
- `is_long_running_delete_at_pre`
- `derive_cap_at_pre`
- `is_mdb_parent_of_at_pre`

替代方式：

- 让这些前置条件回到 `cte.rs` 对应函数的 `requires` 中；
- 如果确实需要复用，则保留为 `cte.rs` 内部的私有 spec helper，而不是 public interface item。

## 第二轮：删除返回值解释器同义层

审查对象：

- `verify_view_cap`
- `verify_heap_matches_state_at`
- `exception_status_is_none`
- `exception_status_is_syscall_error`
- `derive_cap_ret_*`
- `resolve_address_bits_ret_*`

策略：

- 能直接使用 `repr::*` / bridge view 的地方直接使用；
- 如果只是 public 名称重命名，应删除；
- 只有在确实能显著简化外部 proof surface 时才保留。

## 第三轮：压缩 mutator wrapper

对 `cte_insert`、`insert_new_cap`、`cte_move`、`cte_swap`：

- 删除 `*_at` / 非 `*_at` 双函数结构；
- 若需要公共包装，只保留一个；
- postcondition 只保留：
  - heap matches state；
  - 对应 `spec_*_post`；
  - 必要的 expected-entry post。

---

## 8. `refinement_bridge.rs` 的具体处理策略

此文件不追求“删除”，而追求“瘦身并收口”。

## 8.0 命名规范

`refinement_bridge.rs` 的命名规范已在 [3.2.1 TCB 命名规范](#321-tcb-命名规范)
中统一定义。

在执行第 8 节时，应直接按该规范落地，不再额外引入第二套命名规则。

## 8.1 必须保留

- `trusted_view_cap`
- `trusted_view_cte`
- `bridge_cap`
- `bridge_cte`
- slot ref / cap ref observer
- small constructor witness
- external type specifications

## 8.2 应迁出或删除

- 可以直接在 `cte.rs` 本地消化的“结果解释壳”；
- 为 `interface.rs` 服务的别名层；
- 对核心语义的重复说明。

## 8.3 验收标准

- 新读者打开这个文件时，应只看到“表示边界”；
- 看不到“某个 CSpace 操作到底是什么语义”的核心逻辑。

---

## 9. 证明策略模板

为避免后续函数继续长成旧风格，所有新证明都应遵循以下模板。

## 9.1 查询函数模板

适用：

- `same_region_as`
- `same_object_as`
- `is_cap_revocable`

模板：

1. bridge raw input 到 ghost snapshot；
2. 用 snapshot 计算 runtime 结果；
3. 分 tag 证明该结果与抽象 spec 等价；
4. 收束到 raw input 的最终 post。

禁止：

- 先定义一个 contract，再证明 contract，再从 contract 推最终结论。

## 9.2 派生查询函数模板

适用：

- `is_mdb_parent_of`
- `is_final_cap`
- `ensure_no_children`
- `is_long_running_delete`
- `derive_cap`

模板：

1. 用 precondition 建立 raw slot 与 `CSpaceState` 中 slot 的对应；
2. 调用更基础的已完成 query proof；
3. 将中间布尔量或返回结构体字段对齐到抽象谓词；
4. 直接结束于最终抽象 post。

禁止：

- 通过 `interface.rs` 再包装一次；
- 在 proof 中混用多个同义层的“view”名称。

## 9.3 Mutator 模板

适用：

- `cte_insert`
- `insert_new_cap`
- `cte_move`
- `cte_swap`

模板：

1. 运行 runtime body；
2. 通过 local heap observer 证明哪些位置发生改变；
3. 建立 `new_heap` 与 `new_state` 的匹配；
4. 证明对应 `spec_*_post`；
5. 通过 expected-entry lemma 得到关键 slot 的最终形状。

禁止：

- 在 public wrapper 中手写大量重复展开；
- 用 contract 名字代替真正 post。

---

## 10. 具体执行顺序

建议按照以下提交粒度推进。

## 提交 1：查询类函数定型

涉及：

- `capability/mod.rs`
- `cte.rs`
- `interface.rs`

动作：

- 清理 `same_region_as`
- 清理 `same_object_as`
- 清理 `is_cap_revocable`
- 删除对应重复 wrapper 与 contract

验收：

- 三个函数都直接指向抽象 spec。

## 提交 2：删除 `interface.rs` 中的查询 proof surface

动作：

- 删除查询类 `*_at_pre` / `*_at`；
- 调整依赖它们的调用点。

验收：

- `interface.rs` 不再作为查询 proof 的核心入口。

## 提交 3：派生查询函数定型

涉及：

- `is_mdb_parent_of`
- `is_final_cap`
- `ensure_no_children`
- `is_long_running_delete`
- `derive_cap`

动作：

- 把每个函数改成直接结束于抽象 post；
- 将旧 `exec_contract` 从 public-facing 位置移除；
- 保留必要的局部 lemma。

验收：

- 调用图变成“基础 query -> 派生 query”，而不是“query -> contract -> interface wrapper -> 派生 query”。

## 提交 4：`resolve_address_bits` 定型

动作：

- 收缩返回值 wrapper；
- 统一到“ret core equals expected core”。

## 提交 5：Mutator 包装收缩

动作：

- 合并 `*_at` / 非 `*_at` 双层；
- 将 public post 统一到 `spec_*_post` + expected-entry；
- 删除旧 contract 壳。

## 提交 6：`refinement_bridge.rs` 收口

动作：

- 删除多余别名与旧说明；
- 将边界层说明改写为“表示投影专用”。

---

## 11. 风险与应对

## 11.1 风险：一次删太多 wrapper 导致调用面崩塌

应对：

- 先改查询类，再改派生查询，再改 mutator；
- 每阶段先让依赖图收敛，再继续下一阶段。

## 11.2 风险：bridge 删除过度，导致 raw-to-state 对齐事实丢失

应对：

- 不删除 observer；
- 只删除“语义中介壳”；
- 任何涉及 raw 地址、slot 指针、返回结构体字段解释的事实，都优先保留在 bridge/repr 一侧。

## 11.3 风险：mutator 直接写最终 post 后证明负担增大

应对：

- 保留 local heap transition lemma；
- 用 expected-entry lemma 分摊证明；
- 不在一个函数里同时展开所有状态字段，始终依赖 `spec_*_post` 的局部结论。

---

## 12. 完成后的理想状态

完成重构后，代码应呈现以下特征：

- 查询函数的 public post 一眼就是抽象 spec；
- 派生查询函数直接陈述最终语义，而不是陈述“满足某个 contract”；
- mutator 的 public post 一眼就是抽象状态转移；
- `interface.rs` 变成薄层或近似消失；
- `refinement_bridge.rs` 明确是表示边界，而不是语义中转站；
- `l4v` 只决定“证明目标”，不再决定“证明代码的形状”。

这才是本项目真正应追求的“借鉴 l4v 目标、采用 Verus-native 风格”。
