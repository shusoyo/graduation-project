# `sel4_cspace` Project Verification Map

本文档记录 layer reset 之后的项目地图：

- 当前承认什么
- 当前不再沿用什么
- 未来按什么顺序推进

## Reset Statement

项目从本日开始进入新的架构阶段。

旧文档没有被删除，而是整体归档到：

- `sel4_cspace/docs/archive/2026-05-13-layer-reset/`

归档的意思不是“那些工作都没价值”，而是：

- 不再让旧的 `mdb_patch` 叙事主导新架构
- 不再把旧 checklist 当作当前优先级地图
- 需要历史事实时再回 archive 查

## New Architecture Baseline

当前项目采用下面的层次设计：

1. `cap`
2. `mdb_node`
3. `cte_t`
4. `mdb`
5. `cdt`
6. `cspace`

它们各自负责：

- `cap`：capability 自身语义
- `mdb_node`：MDB 链字段表示
- `cte_t`：`cspace::cte` 中的 slot object，承载 `cap + mdb_node`
- `mdb`：`cspace::mdb` 中的 MDB 图关系与全局验证
- `cdt`：`cspace::cdt` 中的 capability derivation tree 与 `is_original`
- `cspace`：总域，内部再分 `cte / mdb / cdt / manager / resolve`

## What Remains Valid From Before

下面这些高层判断仍然有效：

- `resolve` 仍是当前最完整的一条 verified line
- `insert / move / swap / delete` 的 runtime 语义仍应继续对照 `reference_0ca248f`
- `l4v` 继续校准 semantic strength
- `atmo` 继续校准 Verus organization
- manager-level verified core 仍然是当前默认 claim 边界

## What Is No Longer The Main Story

下面这些不再作为当前阶段的主叙事：

- 以 `mdb_patch` 为中心的 replacement narrative
- “先清空所有旧 closeout wrapper” 作为唯一主线
- 把 `cte.rs` 长期视为 pure compatibility wrapper
- 把 `cdt` 继续隐含在大一统 manager ghost state 里不单独命名

这些内容仍然有历史价值，但不再主导新阶段的设计判断。

## Current State By Layer

### `cap`

当前状态：可用，但尚未按新层次重新文档化。

说明：capability 语义已经存在，但仍与 `cte`/`mdb`/delete dependency cone 有较强耦合。

### `mdb_node`

当前状态：存在 concrete representation，但尚未被提升成明确的数据层边界。

说明：它已经在 runtime 中稳定存在，但 proof 仍多半通过 `CSpaceManager` 间接消费它。

### `cte_t`

当前状态：已经迁入 `cspace::cte` 并成为 slot object。

说明：`src/cte.rs` 现在只保留 crate 内 free-function 兼容入口；真实类型、slot-local spec、raw bridge 和 slot method 都在 `cspace::cte` 下。multi-slot operation 不再由 `cspace::cte` 拥有，crate-level/free-function API 统一经 `cspace::kernel -> CSpaceManager` 接入。`cte_t` 负责 slot-local observer、derive/children/final 判断和必要 dispatch，不承担 whole-MDB / whole-CDT recovery，也不再拥有 cap-derivation 之外的跨层语义 owner。

### `mdb`

当前状态：已经成为独立命名的 proof-bearing 子系统。

说明：`cspace::mdb` 当前只保留 `raw` 和 `table`。旧的 `MdbState` / `MdbPayload` / `MdbRankWitness` 以及 `mdb::{state,spec,proof}` 兼容路线已经删除。MDB 新主线是 `MdbTable` owner：tracked entries 是唯一 concrete truth，`order/live_slots` 是 owner 内 ghost summary，insert 线通过 `MdbTable::insert_node_after` 和 `MdbTable::insert_between_rel` 表达 post-state。move/swap/delete 后续必须补自己的 `MdbTable` owner primitive，不再回退到 manager 投影。

### `cdt`

当前状态：已经成为独立命名层。

说明：`cspace::cdt::{state,spec,proof}` 现在只拥有显式 `dom`、`parent_of` / `is_original` ghost state、独立 `CdtDepthWitness`、纯结构性 derivation wf，以及 insert/move/swap/delete 的 owner post-state transition。`should_be_parent_of` 和其它 cap-derived parent 语义已经收回 `CSpaceManager`。`depth` 和 MDB `rank` 一样是证明 witness，不进入主状态语义。

### `cspace`

当前状态：已经收编成 `cspace` 域内的组合层。

说明：`cspace::manager` 拥有 `MdbTable`、`CdtState`、zombie set 和总 `wf`，并且仍是所有 cap 语义、cross-layer glue、以及总 invariant 汇总的 owner。`cspace::kernel` 提供长期存在的 `CSpaceKernel { manager }`，让 kernel-facing free functions 能在 API 不变的前提下优先走 manager-backed execution。旧 `cspace_manager` 文件树不再作为当前实现存在。

## Priority Order

当前推荐优先级：

1. 以 `insert` 作为第一条 owner-based post-state proof 试点。
2. 在 proof 中把 concrete slot writes 归约到 `cte` local post、`mdb` graph primitive post-state、`cdt` owner post-state。
3. 为 kernel integration 补 boot/init 级 manager population bridge，把真实 CSpace slot domain、tracked slot perms、CDT/original ghost state 装入长期 `CSpaceKernel`。
4. 将 `unchanged` 仅用于 patch 外 frame，不再恢复 backup 风格的 unchanged 主路线。
5. 对 `move / swap / delete` 复用同一证明骨架。
6. 新架构下的 mutation proof 稳定后，再重新做 trusted-boundary shrink 和 paper-level claim 评估。

## Chosen Proof Strategy

当前冻结的证明策略是：

- 主骨架采用 owner-based post-state proof。
- `cte / mdb / cdt / manager` 各自拥有自己的 post-state / preservation 责任。
- `unchanged` 不再作为 mutation proof 主骨架，只保留成薄 frame 工具。
- `rank` 只保留给 `mdb` 的 acyclic / no-cycle 一类结构性证明。
- `mdb` 默认只对 graph primitive transition 负责，不直接以 `insert / move / swap / delete` 作为 owner 命名。

这意味着后续 operation proof 的默认形状是：

1. 先对 concrete exec 写出一个最终后状态；
2. 再把该后状态分别归约给 `cte` / `mdb` / `cdt` owner；
3. 最后由 `manager` 收回总 `wf`。

## Claim Boundary

当前更安全的表述仍然是：

- manager-level verified core under a layered architecture reset
- resolve remains the strongest verified line
- cte/mdb/cdt/cspace decomposition is the active architecture direction

当前不安全的表述仍然是：

- fully verified CSpace
- whole-kernel invariant preservation for all CSpace operations
- public wrapper level fully aligned and verified

## Proof Direction

当前 mutation proof 的默认方向冻结为：

- owner-based post-state proof
- `unchanged` 退成薄 frame 工具
- `rank` 仅作为 `mdb` acyclic 证明的候选 witness

这意味着：

- 不再延续 backup 风格的 `unchanged` 主路线。
- 不把 `mdb` 直接做成顶层 operation owner；`mdb` 只拥有 graph primitive。
- `manager` 继续是 orchestration / combiner，而不是 semantic owner 回流中心。

## Working Rule

从现在起，新的设计文档、代码组织讨论、proof 迁移计划，都默认以新层次为中心。

如果某个旧 helper、旧 proof route、旧 residual 仍然重要，应当用“它在新层次里属于哪一层、是否保留”为口径重新判断，而不是延续旧文档里的优先级语言。
