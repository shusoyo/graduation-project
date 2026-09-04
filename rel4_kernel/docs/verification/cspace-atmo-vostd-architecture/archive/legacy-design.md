# CSpace 新架构设计方案

## 1. 文档目的

本文档描述 `sel4_cspace` 后续从当前验证架构迁移到更接近 `atmo` / `vostd` / `ostd` 风格的总体设计方案。

极简诉求：

**保留现有 CSpace 语义资产，逐步做出一个可替换旧 `sel4_cspace` 的 Verus-native CSpace core。**

本文档不追求立即给出完整实现代码，而是回答下面几个问题：

- 现有架构的中心问题是什么；
- 新架构的中心对象应该是什么；
- 在 `sel4_common` 由 generated bitfield 结构主导的前提下，应如何组织 verified layer；
- 各文件未来分别承担什么职责；
- 代码编写时应优先新增什么、避免什么。

---

## 2. 当前架构的基线判断

当前 `sel4_cspace` 的主轴可以概括为：

1. `specs/*` 定义抽象状态与操作规约；
2. `refinement_bridge.rs` 负责 raw 值到抽象语义的显式桥接；
3. `cte.rs` / `interface.rs` 证明实现满足外置的抽象 postcondition。

这套结构的优点是：

- 与 `l4v` 的抽象语义对齐较直接；
- 对“单个 concrete 操作对应哪个 abstract transition”表达清晰；
- 适合快速建立 CSpace 语义验证基线。

这套结构的局限是：

- 抽象层、精化层、实现层边界过于显式；
- concrete runtime struct 本身几乎不承载 verified object 语义；
- 很多 proof 需要先离开对象、进入 bridge/state，再回到实现；
- 后续若想组合成更大规模的 verified object system，会继续倾向于长出新的中间层。

---

## 3. 新架构的目标

新架构的目标不是删除 `spec`，而是改变 `spec` 与实现之间的组织方式。

目标风格：

- 保留 raw runtime struct 作为底层表示；
- 在 `sel4_cspace` 中建立 verified wrapper / owner / view / wf 层；
- 让对象自身携带主要规约与一致性约束；
- 让 mutator/query 的证明尽量围绕 verified object method 展开；
- 让 `refinement_bridge` 只剩纯 repr / witness / external specification；
- 让 `interface.rs` 退化为薄 facade 或 re-export。

一句话概括：

**从“外置抽象状态 + 显式 bridge + direct post”迁移到“verified object + owner/view/wf + method spec”。**

### 3.1 非目标

本方案当前明确不是：

- 继续优化 `refinement_bridge.rs` 并把它作为终局主轴；
- 单纯给旧 `cte.rs` / `interface.rs` 再加一层 proof shell；
- 机械照搬 `l4v` 的分层方式；
- 机械照搬 `atmo` 的整内核对象图；
- 立即删除 `specs/*` 并失去现有语义锚点。

### 3.2 终局目标

终局不是“证明旧实现”，而是：

- 形成一个 Verus-native 的 CSpace core；
- 让 query / mutator 的主体语义活在该 core 中；
- 让旧 runtime / compat / raw bitfield 世界退化为边界适配层。

---

## 4. 核心现实约束

### 4.1 `sel4_common` 不能被当作可自由重写的 verified ADT

`sel4_cspace` 的运行时实现依赖：

- `sel4_common::structures_gen::cap`
- `sel4_common::structures_gen::mdb_node`
- 其他 generated bitfield 类型

这些类型具有以下特征：

- ABI / layout 敏感；
- 以 generated getter/setter 为主；
- 不适合直接承载大规模验证语义；
- 不宜为了验证风格而大幅修改定义方式。

因此，新架构**不能**以“直接重写 raw struct 本体”为前提。

### 4.2 正确策略：保留 raw repr，外加 verified layer

新的证明中心不应是 raw bitfield struct 本身，而应是建立在其上的 verified layer。

应采用如下分层：

1. raw repr 层  
   运行时 bitfield struct，本体尽量不动。

2. repr/view 层  
   将 raw 值解释为更稳定的逻辑视图。

3. owner/wf 层  
   表达对某些 raw 对象的持有关系、更新权限与一致性约束。

4. method/spec 层  
   在 verified object method 上直接写前后条件。

---

## 5. 新架构中的建议分层

### 5.1 `repr` 层

该层负责：

- raw `cap` 到 `CapView`；
- raw `mdb_node` 到 `MdbView`；
- raw `cte_t` 到 `SlotView`；
- raw 返回值到结果视图。

该层不负责：

- 表达完整 CSpace 语义；
- 证明跨对象更新后的全局不变量；
- 暴露大规模 public proof surface。

建议文件：

- `sel4_cspace/src/repr/cap_repr.rs`
- `sel4_cspace/src/repr/mdb_repr.rs`
- `sel4_cspace/src/repr/slot_repr.rs`
- `sel4_cspace/src/repr/result_repr.rs`

### 5.2 `view/model` 层

该层负责定义比 raw repr 更稳定的逻辑结构，例如：

- `CapView`
- `MdbView`
- `SlotView`
- `ResolveRetView`

这些类型的职责是：

- 成为 method spec 与 object `view()` 的返回对象；
- 成为对象级 `wf()` 的语义参照；
- 在不泄露 bitfield 细节的前提下表达抽象行为。

### 5.3 `owner/wf` 层

该层是新架构的核心。

建议逐步引入类似下面的 verified wrapper：

- `VerifiedCapRef`
- `VerifiedSlotRef`
- `VerifiedSlotMut`
- `CspaceCtx`
- `CspaceOwner`

这些 wrapper 的职责：

- 指向或拥有某个 raw 对象；
- 保证对象与其 `view()` / `model()` 一致；
- 为 query/mutator 提供局部可重用的前提；
- 承载更新后的 `wf()` 恢复证明。

### 5.4 method/spec 层

新架构里，query/mutator 不应主要围绕外置的 `CSpaceState` 中转证明，而应逐步转成：

- object method precondition；
- object method postcondition；
- owner/resource 更新后局部与全局 `wf()` 保持；
- 必要时再从对象级语义推出更高层抽象语义。

---

## 6. 未来文件职责建议

### 6.1 保留但降级的文件

#### `sel4_cspace/src/refinement_bridge.rs`

未来职责：

- `external_type_specification`
- 小型 constructor witness
- 必要的 raw observer

未来不应承担：

- CSpace 操作的主要语义入口
- 大量按操作命名的 proof helper
- 外部 public proof surface

#### `sel4_cspace/src/interface.rs`

未来职责：

- 薄 facade
- 必要 re-export

未来不应承担：

- 第二套 contract 语言
- 操作级核心证明组织

### 6.2 需要新增的文件/目录

建议新增：

- `sel4_cspace/src/verified/mod.rs`
- `sel4_cspace/src/verified/cap.rs`
- `sel4_cspace/src/verified/mdb.rs`
- `sel4_cspace/src/verified/slot.rs`
- `sel4_cspace/src/verified/cspace.rs`

这些模块用于承载：

- verified wrapper
- owner/wf/view
- object-level query/mutator method

必要时再新增：

- `sel4_cspace/src/owner/mod.rs`
- `sel4_cspace/src/owner/cspace_owner.rs`

### 6.3 `specs/*` 的新定位

`specs/*` 不应立即删除，但其角色要调整。

未来角色：

- 作为抽象语义锚点；
- 为 `view/model` 与 object method 提供高层语义解释；
- 在迁移早期继续作为 old/new 体系对照基准。

未来不宜继续演化成：

- 所有 public proof 的唯一语言中心；
- 依赖大规模 bridge 才能使用的外置总控状态机。

### 6.4 项目组织演化方向

长期看，项目组织应逐步接近：

1. `verified core`
   以 Verus 为主体，承载对象、规约、证明和主要算法。

2. `compat/runtime glue`
   负责对接 `sel4_common` raw struct、旧接口和运行时副作用。

在当前仓库中，即使短期内不真的拆成两个 Cargo crate，也应按这两个世界来组织模块边界。

---

## 7. 建议的代码编写顺序

### 7.1 阶段 A：先长新骨架，不删旧逻辑

第一阶段只新增，不拆旧。

先做：

1. 在 `repr/*` 中稳定 raw-to-view 投影；
2. 定义 `CapView` / `MdbView` / `SlotView`；
3. 新增 `VerifiedCapRef` / `VerifiedSlotRef` / `VerifiedSlotMut`；
4. 为这些 wrapper 定义 `view()` / `wf()`。

这一阶段的目标是：

- 建立新的证明语言；
- 不影响现有行为；
- 不让新实验直接撞上全部 mutator 复杂度。

### 7.2 阶段 B：先迁 query

优先迁移：

- `same_region_as`
- `same_object_as`
- `is_cap_revocable`
- `is_mdb_parent_of`
- `is_final_cap`
- `ensure_no_children`

这些函数适合率先转成：

- 输入为 verified ref / wrapper；
- post 直接对 object `view()` 或 owner 语义表述；
- 尽量不依赖大范围 old/new `CSpaceState` 中转。

### 7.3 阶段 C：迁移 `resolve_address_bits`

该函数适合充当中间样板，因为它兼有：

- 查询语义；
- 结果值解释；
- 局部 lookup 逻辑。

目标是把返回值证明重心从 bridge/result 中转，转到 `ResolveRetView` 与 verified context 上。

### 7.4 阶段 D：迁移 mutator

最后迁移：

- `cte_insert`
- `insert_new_cap`
- `cte_move`
- `cte_swap`

迁移重点：

- 不再以 `local_heap_transition_at + spec_*_post` 为唯一主语言；
- 转为围绕 slot/cspace owner 的更新与 `wf()` 恢复展开；
- 最后再推出与旧抽象语义的一致性。

---

## 8. 编码约束

### 8.1 不要做的事情

- 不要直接修改 `sel4_common` generated bitfield 结构以迎合证明风格；
- 不要在新架构里再长出第二个“大一统 bridge 模块”；
- 不要一开始就全面引入超重的 tracked permission graph；
- 不要在没有稳定 `view()/wf()` 之前就重写全部 mutator。

### 8.2 应优先做的事情

- 统一 raw-to-view 命名；
- 尽量把 bitfield getter/setter 隔离进 `repr` 或 verified wrapper method；
- 让每个新增 wrapper 都先有最小 `wf()`；
- 优先让 query 成为新架构的试验场；
- 让旧 `specs/*` 暂时充当对照系，而不是第一时间删除。

---

## 9. 第一批建议落地项

建议优先实现下面这些最小构件：

1. `CapView`
2. `MdbView`
3. `SlotView`
4. `VerifiedCapRef`
5. `VerifiedSlotRef`
6. `VerifiedSlotMut`
7. `view()` / `wf()` for the above
8. 一个基于新 wrapper 的 query 样板

不建议第一步就做：

- `CspaceOwner` 的完整 tracked resource 图；
- 四个 mutator 的全量迁移；
- 删除旧 `refinement_bridge.rs`。

### 9.1 第一批成功判据

第一批实现完成后，应至少满足：

- `repr/*` 能稳定提供 `cap` / `mdb_node` / `cte_t` 的逻辑视图；
- 至少一个 `CapRef`-level query 不再需要经过旧式 public bridge shell；
- 至少一个 `SlotRef`-level query 能以 `view()/wf()` 语言完成证明；
- 旧 `specs/*` 仍可作为对照系验证新结果语义不偏移。

---

## 10. 预期结果

迁移完成后，希望 `sel4_cspace` 的主体验证结构变成：

- raw bitfield/runtime struct 仍服务运行时；
- `repr/*` 负责解释 raw 值；
- `verified/*` 负责 owner/view/wf/method；
- `specs/*` 作为高层抽象锚点与对照系；
- `interface.rs` 成为薄 facade；
- `refinement_bridge.rs` 成为小型 TCB 支持层。

这将使得 `sel4_cspace` 的风格从

**“spec + refinement bridge + direct post”**

迁移到更接近

**“verified object + owner/view/wf + method-level spec”**

的架构。
