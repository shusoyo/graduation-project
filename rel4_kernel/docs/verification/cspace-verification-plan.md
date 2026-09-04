# CSpace Verus 验证总计划

状态：当前主文档，2026-05-01

## 1. 文档约定

`docs/verification` 目录现在只保留三份日常主文档：

- `cspace-verification-plan.md`
  - 总目标、当前状态、TCB、风格约束、后续阶段计划
- `cspace-session-log.md`
  - 按时间顺序整理的阶段记录与里程碑
- `cspace-thesis-draft.md`
  - 面向论文写作的独立文字草稿

其余历史计划、阶段报告、桥接设计、收口报告均移入 `docs/verification/archive/`。

## 2. 项目定位

本项目当前不追求复刻整个 seL4/l4v 的全系统证明，而是把目标收敛为：

- 只证明 `sel4_cspace` 这一子系统本身的语义与局部不变量；
- 底层指针、位域、FFI、非 `CSpace` 子系统先作为 TCB；
- 对已选定的 `CSpace` 子集，语义尽量对齐 `aux/l4v`；
- 表示方式、模块分层与证明工程采用 Verus-native 设计。

论文口径固定为：

`在把底层实现细节与非 CSpace 子系统视为可信边界的前提下，对 Rust 重写中的 CSpace 子集建立与 l4v 一致的局部语义、局部不变量与 refinement 证明。`

## 3. 当前状态

### 3.1 当前验证闭环

- `bash tools/check-cspace-build-and-verify.sh` 当前通过；
- 最近一次联合回归结果（2026-05-01）为 `249 verified, 0 errors`；
- 当前 `sel4_cspace/src/cte.rs` 剩余 raw assumption 为 `0` 个；
- `cte_insert` / `insert_new_cap` / `cte_move` / `cte_swap`
  已不再由 public mutator 入口本身直接承担 `#[verifier::external_body]`；
  当前 public 入口改为 verified wrapper，
  内部再调用 crate 内的 runtime-step `external_body` helper，
  因而 public contract 已不再继续通过 raw `assume_specification[...]` 发布；
- 其中 `insert_new_cap_runtime_step(...)` 已进一步从顶层 `external_body`
  改成真实的 verified glue：
  - 顶层顺序现在显式按
    `write_slot -> rewrite_old_next -> link_parent`
    三段 staged runtime-step 组合；
  - `refinement_bridge.rs` 新增了
    `trusted_slot_mdb_next_addr(...)`
    与更通用的 local-heap-transition weaken/raw-slot-view helper，
    用来把中间 ghost state、post-heap-match 和最终 transition 串起来；
  - 当前 `insert_new_cap` 已成为 mutator family 里第一个
    “public verified wrapper + internal verified runtime-step glue + residual micro-step external_body helper”
    的模板项；
- `cte_insert_runtime_step(...)` 现也已进一步从顶层 `external_body`
  改成真实的 verified glue：
  - 顶层顺序现在显式按
    `set_untyped -> write_dest -> link_src -> rewrite_old_next`
    四段 staged runtime-step 组合；
  - `cte_insert_write_dest_runtime_step(...)`
    与 `cte_insert_link_src_runtime_step(...)`
    的小步合同也已改成可顺序组合的 local-heap-transition 形状；
  - 当前 `cte_insert` 也已进入
    “public verified wrapper + internal verified runtime-step glue + residual micro-step external_body helper”
    的模板形态；
- `cte_move_runtime_step(...)` 现也已进一步从顶层 `external_body`
  改成真实的 verified glue：
  - 顶层顺序现在显式按
    `write_dest -> clear_src -> rewrite_old_prev -> rewrite_old_next`
    四段 staged runtime-step 组合；
  - `specs/cspace_ops/move.rs` 现已补出对应的 staged abstract state、
    runtime changed-set 与 frame proof ladder；
  - `refinement_bridge.rs` 新增了
    `trusted_slot_mdb_prev_addr(...)`
    来承接旧 `mdb_prev` 地址观察；
  - 当前 `cte_move` 也已进入
    “public verified wrapper + internal verified runtime-step glue + residual micro-step external_body helper”
    的模板形态；
- `cte_swap_runtime_step(...)` 现也已进一步从顶层 `external_body`
  改成真实的 verified glue：
  - 顶层顺序现在显式按
    `write_swapped_slots -> rewrite_slot1_prev -> rewrite_slot1_next -> rewrite_slot2_prev -> rewrite_slot2_next`
    五段 staged runtime-step 组合；
  - `specs/cspace_ops/swap.rs` 现已补出对应的 staged abstract state、
    runtime changed-set、frame proof ladder 与
    `lemma_cte_swap_runtime_changed_slots_eq_spec_changed(...)`；
  - `refinement_bridge.rs` 新增了
    `trusted_slot_addr(...)`
    来承接 raw slot ref 到 slot 地址/`SlotId` 的连接；
  - 当前 `cte_swap` 这条线上仍额外保留了一个 proof-side staging scaffold
    `lemma_cte_swap_post_implies_staged_final_state(...)`，
    它不再属于 runtime raw assumption，
    但仍是下一步可以继续收紧的 spec-side helper；
- mutator family 的 public verified wrapper
  现已直接发布最终 `*_exec_contract`，
  `interface.rs` 这层也已改为直接消费这些 public contract，
  中间只服务旧分层的 `*_exec_step` 已删除；
- `sel4_cspace/src/interface.rs` 现已接住 verify-facing 的
  `ConcreteHeapId` / `ResolveAddressBitsAtRet` 类型名；
- `sel4_cspace/src/refinement_bridge.rs` 当前已收回为 crate-private 模块，
  不再作为 crate root 上的机械性公开 proof surface；
- `sel4_cspace/specs` 已具备抽象模型、原语规格与部分可复用引理；
- `sel4_cspace/src/refinement_bridge.rs` 已有 concrete 到 abstract 的桥接层；
- `sel4_cspace/src/cte.rs` 已有一批 refined proof 入口。

#### 当前量化进度看板（2026-04-30）

- 当前收口主线统一按“非删除主线 verify-native 化”这 `10` 步统计。
- 当前完成度：`10 / 10` 已完成。
- 当前下一步：当前这条 `10` 步主线已完成；后续转入可选加强项。
- 下文保留的 `168 / 170 / 180` 等数字主要作为历史里程碑；看当前进度时，以本看板和本节顶部数字为准。

1. verify-facing facade 统一命名，并显式区分 verified path / runtime path
   - 状态：已完成
2. capability query 三项去 assumption 化
   - 范围：`same_region_as` / `same_object_as` / `is_cap_revocable`
   - 状态：已完成
3. `is_final_cap` 改成 verify-native 方法
   - 状态：已完成
4. `is_long_running_delete` 改成 verify-native 方法
   - 状态：已完成
5. `is_mdb_parent_of` 改成 verify-native 方法
   - 状态：已完成
6. `ensure_no_children` 改成 verify-native 方法
   - 状态：已完成
7. `derive_cap` 改成 verify-native 方法
   - 状态：已完成
8. `resolve_address_bits` 去 assumption 化
   - 状态：已完成
9. mutator family 去 assumption 化
  - 范围：`cte_insert` / `insert_new_cap` / `cte_move` / `cte_swap`
   - 状态：已完成
10. bridge / TCB 最终收口与 build/verify 口径对齐
   - 状态：已完成

### 3.2 对阶段的统一理解

当前仓库其实已经完成了第一轮“工程闭环”：

1. 固定门禁与最小 TCB
2. 建立抽象模型与 `wf`
3. 写原语规格
4. 收紧可复用小引理
5. 建立 bridge 与 refined entry
6. 收口最小 trusted surface

但这还不等于“论文口径下已经与 l4v 对齐”。因此，后续主线改成下面这句话：

- 第一轮工程闭环已完成；
- 下一轮工作转向“面向论文口径的 l4v 语义对齐与证明重组”。

### 3.3 当前最准确的进度判断

- 工程进度：
  - 第一轮 6 步闭环已完成
- 论文/语义进度：
  - 范围与主线已经固定
  - `P2` capability 基础语义重构已完成
- `P3` 已完成第一轮 invariant split
- `P4` 已完成第一轮 preservation-first 改写
- `P5` 已完成第二轮 bridge 收缩
- `P6` 已完成
- `P7` 已完成
- 当前下一步：
  - 以 `cspace-thesis-draft.md` 为主版本，按学校模板继续压缩摘要、引言、贡献与结论
  - `trusted_range_top_u128_if_small`、concrete view 提取器继续作为独立的 TCB 收紧小任务推进
  - 如果转回技术主线，则从可选加强项继续：
    例如进一步收紧 bridge / TCB，
    或把 mutator family 从当前“public verified wrapper + internal runtime-step `external_body`”
    继续推进到更直接的 Verus body

### 3.4 按轮次理解整个项目

为了避免把“阶段编号”“当前轮次完成”“最终理想形态”混在一起，后续统一用下面四轮来理解整个 `CSpace` 验证项目。

#### 第 1 轮：工程闭环轮

目标：

- 固定最小证明范围与 trusted boundary；
- 建立抽象模型、primitive spec、bridge 与 refined proof entry；
- 让 Verus 验证链路稳定跑通。

完成标志：

- `sel4_cspace/specs/*`、`refinement_bridge.rs`、`cte.rs` 之间形成基础闭环；
- 第一轮 6 步工程闭环完成；
- 能稳定得到通过的验证结果。

当前状态：

- 已完成。

#### 第 2 轮：论文口径与 TCB 冻结轮

目标：

- 收口 verify-facing 接口层；
- 系统整理已证范围、未证范围与 TCB 台账；
- 固定论文与答辩口径；
- 将 TCB 的分类、取舍原则与代码落点同步冻结。

完成标志：

- `interface.rs` 成为稳定 verify-facing facade；
- `boundary_assumptions`、`refinement_bridge`、`specs/*` 的职责边界清楚；
- 论文主张可以稳定区分“真实 Verus 接口已证”“refined wrapper 已证”“remaining TCB”；
- TCB 设计原则进入“只允许收缩、不允许扩张”的冻结状态。

当前状态：

- 已完成。

#### 第 3 轮：最终展示与提交轮

目标：

- 以最终展示子集为中心，把最重要的几个入口继续做硬；
- 让代码、验证、文档、论文和答辩口径完全对齐；
- 尽量缩小“运行时实现”和“验证入口”之间的落差。

当前优先子集：

- `same_region_as`
- `same_object_as`
- `is_cap_revocable`
- `derive_cap`
- `ensure_no_children`

加强目标：

- `is_final_cap`
- `resolve_address_bits`

当前状态：

- 已完成当前基线收口；
- 最终展示子集已经形成一套稳定的 verify-facing 入口、结果观察口径与论文对应说法；
- 这是毕设最低可交付目标所在的一轮，后续不再作为“待完成主线”保留。

#### 第 4 轮：可选加强轮

目标：

- 继续把部分 `opaque body + refined wrapper` 推向更直接的 Verus verified body；
- 进一步收紧 `trusted_range_top_u128_if_small`、concrete extractor、slot-level observer；
- 向 `delete/revoke/finalise` 主线扩展更完整的 local proof。

当前状态：

- 作为“可选加强”仍然不是毕设最低完成要求；
- 但当前已经完成一轮接口层收紧包：
  - `interface.rs` 现已为主要 verify-facing 入口补齐 `*_at_pre` 包装；
  - 外部验证代码可以优先依赖 `interface::*_at_pre / *_at`，而不是直接把 `refinement_bridge::*_call_pre_at` 当成首选公共入口；
- 更进一步的 language-level privatization 还需要连同 `cte.rs` 中这批 public contract-bearing mutator 入口与 bridge precondition 的连接方式一起重构，因此不再作为本轮必做项继续扩张。

#### 当前所在位置

就当前文档口径而言，应统一理解为：

- 第 1 轮已完成；
- 第 2 轮已完成；
- 第 3 轮当前基线已完成；
- 第 4 轮是有余力时再做的加强项。

因此，文档里的“9 步完成”应理解为：

- 它表达的是前三轮当前基线都已经备齐；
- 不是说整个项目已经达到“最终理想形态全部完成”。

### 3.5 面向完全替换目标的非删除主线剩余流程

如果目标从“毕设展示子集”提升为：

`让 sel4_cspace 在非删除主线范围内，尽量成为可由 Verus 直接替换的实现模块，`

那么当前代码还不能算完成。下面这份清单只统计“先不做删除主线”前提下，仍然没有完成的流程。

这里明确排除的内容：

- `finalise`
- `delete_all`
- `reduce_zombie`
- `revoke`
- 以及它们在 `boundary_assumptions.rs` 中对应的流程假设

因为这条删除主线是当前工作量最大、控制流最复杂的一块，暂时不纳入这一轮“完全替换”推进范围。

#### 3.5.1 capability query 三项还没有实现路径合一

涉及对象：

- `same_region_as`
- `same_object_as`
- `is_cap_revocable`

当前状态：

- 这三项已经拥有稳定的 Verus 合同与 refined 证明入口；
- 同时也已经补上了显式的 raw runtime 别名入口：
  - `same_region_as_runtime`
  - `same_object_as_runtime`
  - `is_cap_revocable_runtime`
- 但运行时路径与验证路径仍然是分离的：
  - 普通构建走 `capability/mod.rs` 里的 Rust body；
  - `verify` 构建走 refined wrapper。

这意味着：

- 它们已经是“最接近 fully replaced”的一组；
- 但还不是“同一个 Verus-native body 同时承担运行时实现与验证对象”的最终形态。

完成标准：

- capability query 三项在模块口径上不再分成“普通 Rust body”和“verify wrapper”两套实现；
- 至少要达到“Verus body 即最终实现体，或最终实现体仅剩极薄的可解释适配层”。

#### 3.5.2 slot-local query / derivation 链还停留在 external + refinement 结构

涉及对象：

- `is_mdb_parent_of`
- `is_final_cap`
- `is_long_running_delete`
- `ensure_no_children`
- `derive_cap`

当前状态：

- 这几项的抽象规格、refined proof entry 与 verify-facing facade 都已经存在；
- `is_mdb_parent_of`、`is_final_cap`、`is_long_running_delete`、`ensure_no_children`、`derive_cap`
  的 verify 路径现已切到签名式 Verus 方法体；
- 它们已经不再主要依赖 `assume_specification` 发布 public contract；
- 但普通构建路径里的 runtime body 仍然与 verify path 分离，
  因而还不能说“运行时实现与被 Verus 检查的对象已经完全合一”。

这意味着：

- 这一簇已经具备重写成 Verus-native body 的前置规格基础；
- 但当前真正被证明的仍主要是 refinement 合同，而不是原始实现体本身。

完成标准：

- 这条链条上的关键函数不再依赖 `verifier::external` 暴露运行时 body；
- public contract 不再主要依赖 `assume_specification`；
- `derive_cap` 的返回状态与返回 capability 由实现体直接满足 `requires/ensures`，而不是仅由 refined wrapper 托住。

#### 3.5.3 lookup 主线已经完成一轮去 assumption 化，但还没有实现路径合一

涉及对象：

- `resolve_address_bits`

当前状态：

- 抽象 lookup 语义、返回结果抽象 core、bridge precondition、refined proof 入口都已存在；
- `resolve_address_bits` 的 verify 路径已经不再依赖 `assume_specification[resolve_address_bits]`；
- 当前 verify body 已切到递归的 Verus-native one-step skeleton，
  再由 bridge 引理推到最终抽象 lookup 合同；
- 但普通构建路径里的 runtime body 仍然与 verify path 分离。

这意味着：

- 当前已经能证明“verify 路径下的这一步返回结果符合抽象 lookup 语义”；
- 但还不能说“同一个实现对象已经同时承担普通构建与 Verus 验证”。

完成标准：

- 普通构建与 verify 构建尽量收敛到同一个 `resolve_address_bits` 实现对象；
- 返回值观察继续保持稳定，不重新把高层 lookup 语义塞回 trusted projection；
- lookup 主线最终达到“Verus body 与模块公共实现口径基本重合”的阶段。

#### 3.5.4 mutating primitive 主线已经完成去 assumption 化，但还没有走到实现体替换

涉及对象：

- `cte_insert`
- `insert_new_cap`
- `cte_move`
- `cte_swap`

当前状态：

- 这几项已经具备：
  - primitive spec
  - local heap transition 语义
  - verify-facing `*_at_pre / *_at`
  - refined proof backend
- 这四个入口的 raw `assume_specification[...]` 已经移除；
- verify 路径当前改为显式的 contract-bearing `#[verifier::external_body]` 入口；
- 但普通构建路径里的 runtime body 仍然与 verify path 分离，
  且 mutator 实现体本身还没有改写成 fully verified Verus body。

这意味着：

- 当前证明的是“某次 concrete 更新满足 slot-local 抽象后置条件”；
- 还不是“Verus 直接验证 mutator 实现体逐步完成这些更新”。

完成标准：

- 四个 mutator 的实现体进一步从当前 contract-bearing `external_body` 推进到更直接的 Verus body；
- 实现体可以直接在 Verus 中表达并维护相应的局部状态变化；
- `interface.rs` 这层 facade 不再只是 proof surface，而能更接近最终替换后的模块公共接口。

#### 3.5.5 bridge / trusted observer 仍然是较重的证明支撑层

涉及对象：

- `trusted_view_*`
- `trusted_extract_*`
- `trusted_concrete_*`
- `trusted_slot_ref_*`
- `trusted_make_*`
- `trusted_range_top_u128_if_small`

当前状态：

- 这些项当前仍承担：
  - opaque runtime value 到抽象模型的投影
  - slot/heap 观察
  - 返回值构造与状态观察
  - 局部算术连接

这意味着：

- 当前 `refinement_bridge.rs` 还是“证明主舞台”的重要组成部分；
- 它还没有退化成“只有薄适配层”的最终边界。

完成标准：

- bridge 继续收缩为表示层适配，而不是高频承担语义恢复；
- 能直接由 Verus 实现体表达的对象，不再经由 trusted observer 重建；
- remaining TCB 缩到可稳定解释、且不再主导核心 CSpace 语义证明。

#### 3.5.6 public contract 仍未完全切换到最终 Verus 模块口径

当前状态：

- `interface.rs` 现在已经提供了稳定的 `*_at_pre / *_at` facade；
- 对于当前仍保留 runtime-shaped contract path 的非删除主线入口，
  `interface.rs` 现在也已经系统补齐了 `*_runtime_at` 包装：
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
  它们都用来显式承接“当前 raw runtime body / contract-bearing verify body”的过渡路径；
- `cte.rs` 中那批 mutator `assume_specification` 已经移除；
- `refinement_bridge.rs` 当前已收回为 crate-private；
- 但当前这些 contract-bearing 入口在 crate 内实现层仍然直接引用 bridge precondition。

这意味着：

- 当前 facade 已经形成；
- 但“最终对外模块合同”与“当前 proof backend contract”还没有完全合一。

当前基线完成口径：

- 模块公共口径已经统一到 `interface.rs` 这层最终保留的 Verus 接口；
- 外部不再需要同时理解 `interface.rs`、`cte.rs` public contract 与 `refinement_bridge.rs` bridge precondition 三套口径；
- 更进一步的工作转入可选加强项：继续把 `external_body` / bridge precondition 这层过渡合同压到更小。

#### 3.5.7 构建路径与验证路径还不是同一个实现对象

当前状态：

- 普通构建已经能通过；
- `cargo xtask verify` 也能通过；
- 但“参与构建的 runtime body”和“被 Verus 检查的主要实现路径”仍未完全重合。

这意味着：

- 当前项目已经是稳定的“验证伴随工程”；
- 还不是“Verus 代码本身就是最终替换模块”的状态。

完成标准：

- 非删除主线范围内，最终参与模块构建的实现与被 Verus 验证的实现尽量重合；
- 至少要做到：不再靠一套 Rust runtime body + 一套 refined wrapper 并行维持主要语义。

#### 3.5.8 非删除主线的推荐收尾顺序

如果先不碰删除主线，那么面向“完全替换模块”的实际推进顺序建议是：

1. 先做 `is_mdb_parent_of / is_final_cap / ensure_no_children / derive_cap` 这一簇，把 `external + assume` 最集中的局部判断链条改造成 Verus-native body。
2. 再做 `resolve_address_bits`，因为它是 lookup 代表，且当前仍明显依赖 trusted return projection。
3. 再做 `cte_insert / insert_new_cap / cte_move / cte_swap`，把 mutator 从“transition contract 已证”推进到“实现体已证”。
4. 然后回头统一 public contract，收掉 `external_body` 过渡合同与 bridge precondition 的三套并存口径。
5. 最后再集中收紧 `refinement_bridge.rs` 中剩余的 trusted observer / extractor / arithmetic helper。

这条顺序的目标不是“先把文档说圆”，而是：

- 先把最有希望真正 Verus-native 化的非删除主线拿下；
- 再回头处理剩余 TCB 与接口口径；
- 最终为删除主线的大规模重构留下一个更干净的基础。

## 4. 本轮证明范围

### 4.1 要证明的内容

本轮只覆盖 `CSpace` 子系统内部、且已经进入当前抽象模型的语义，优先包括：

- capability 基础语义：
  - `sameRegionAs`
  - `sameObjectAs`
  - `isCapRevocable`
  - `isMDBParentOf`
  - `deriveCap`
- lookup 语义：
  - `resolveAddressBits`
- 局部修改原语：
  - `cteInsert`
  - `cteMove`
  - `cteSwap`
- 与删除链路直接相关的局部判断：
  - `ensureNoChildren`
  - `isFinalCapability`

### 4.2 暂不证明的内容

下列内容不属于本轮主证明目标：

- 全局 kernel state 的完整一致性；
- 调度、地址空间、IPC、对象创建/销毁的整系统语义；
- 所有 arch-specific capability 细节；
- generated bitfield 代码本身；
- 指针合法性与底层内存模型的完整证明；
- `deps`/FFI 的内部实现。

## 5. l4v 语义来源

### 5.1 定义来源

当前 primary source 固定为：

- `aux/l4v-master/spec/haskell/src/SEL4/Object/ObjectType.lhs`
- `aux/l4v-master/spec/haskell/src/SEL4/Object/CNode.lhs`
- `aux/l4v-master/spec/haskell/src/SEL4/Kernel/CSpace.lhs`

主要对应关系是：

- `deriveCap`、`isCapRevocable`、`sameRegionAs`、`sameObjectAs`
- `cteInsert`、`cteMove`、`cteSwap`、`isMDBParentOf`、`ensureNoChildren`
- `resolveAddressBits`

### 5.2 证明路线来源

证明分层与证明义务主要参考：

- `aux/l4v-master/proof/invariant-abstract/CSpaceInv_AI.thy`
- `aux/l4v-master/proof/refine/CSpace1_R.thy`
- `aux/l4v-master/proof/refine/RAB_FN.thy`
- `aux/l4v-master/proof/refine/Untyped_R.thy`
- `aux/l4v-master/proof/refine/ARM_HYP/Finalise_R.thy`

这里复用的是它的证明分解方式，而不是直接照搬 Isabelle proof script：

1. 先定义抽象语义
2. 再提炼局部不变量
3. 再证明原语保持这些不变量
4. 最后证明 Rust 实现细化到抽象语义

## 6. CSpace 需要证明什么

如果论文口径要尽量与 l4v 一致，那么“验证 CSpace”不能只停在给几个函数写 `requires/ensures`。更合理的目标分为四层：

### 6.1 基础语义层

需要固定并可复用地表达：

- `sameRegionAs`
- `sameObjectAs`
- `isCapRevocable`
- `isMDBParentOf`
- `deriveCap`

### 6.2 局部不变量层

需要定义只依赖 `CSpace` 本身的 invariant，而不是整个 kernel 的 `invs`，例如：

- slot/cte 的基本 well-formedness；
- `cnode_lookup` / `resolveAddressBits` 依赖的 lookup 一致性；
- MDB 链的局部结构约束；
- 与 `final` / `no-children` / `parent-of` 相关的局部一致性。

### 6.3 原语保持层

对核心原语，不仅要有功能合同，还要证明它们保持这些局部 invariant：

- `cteInsert`
- `cteMove`
- `cteSwap`
- 后续扩展时的 `ensureNoChildren`、delete/finalise 路径

### 6.4 Rust refinement 层

在抽象语义与 invariant 稳定后，再证明 Rust 代码满足这些规格：

- concrete `cap/cte/ret` 能映射到抽象模型；
- `cte.rs` 中的具体函数满足抽象 pre/post；
- bridge 只承接表示差异，不承接新的业务语义。

## 7. TCB 与 trusted boundary

### 7.1 固定 TCB

本轮固定的 TCB 主要包括：

- `sel4_common` 中 generated bitfield/getter 的语义；
- 指针转换与底层内存读写；
- `deps` 中的 FFI；
- 尚未纳入本轮抽象模型的 arch 细节；
- 非 `CSpace` 子系统提供给 `CSpace` 的外部前提。

### 7.2 三层 TCB 设计

当前工程中，trusted boundary 固定分成三层，而不是混成一个“大黑盒”：

| 层次 | 当前代码落点 | 允许承担的职责 | 不应承担的职责 | 取舍策略 |
| --- | --- | --- | --- | --- |
| 固定外部边界 | generated bitfield/getter、底层内存/指针访问、`deps` FFI、非 `CSpace` 子系统前提、受限 arch hook | 提供当前轮次不打算证明的底层表示与外部系统前提 | 直接替代 `CSpace` 高层语义证明 | 长期保留，但必须显式列名并写入台账 |
| 表示/观察 bridge TCB | `sel4_cspace/src/refinement_bridge.rs` 中的 `trusted_view_*`、`trusted_extract_*`、`trusted_slot_ref_*`、`trusted_concrete_*_at`、`trusted_make_*` | 把 concrete `cap/cte/ret` 投影到抽象模型，提供 slot/heap 观察器与小型构造器见证 | `sameRegionAs`、`deriveCap`、`resolveAddressBits`、`cteInsert` 这类高层语义本身 | 可以保留，但要保持“小、局部、可收紧”，优先做 observer，不做 semantic oracle |
| 临时流程假设 | `sel4_cspace/specs/boundary_assumptions.rs` 中的 `assume_*` | 隔离 delete/revoke/finalise 等未完整覆盖路径的局部控制流前提 | 混入当前最小展示子集，或被表述成“核心 CSpace 语义已证” | 只允许出现在未收口路径；后续要么被局部证明替换，要么继续明确保留为边界 |

这三层里，只有第一层适合作为长期稳定 TCB；第二层是当前最主要的工程边界；第三层则是对未完成路径的显式临时收口，而不是主结果本身。

### 7.3 当前 trusted surface 的组织原则

- TCB 必须显式命名，不允许隐式黑盒；
- `interface.rs` 是 verify-facing 公共入口，不属于 TCB；
- bridge 中的 trusted 项只负责表示提取、slot/heap 观察和小型构造见证；
- 不把 bridge 当成新的长期语义层，不让它替代 `specs/*` 中的抽象合同；
- `boundary_assumptions.rs` 只保留未完整覆盖路径的流程前提，不进入最小展示子集；
- 后续优先收缩 `trusted_extract_*`、`trusted_concrete_*_at`、`trusted_range_top_u128_if_small` 这类边界。

### 7.4 新增 TCB 项时的取舍规则

每新增一个 trusted helper，都至少回答下面四个问题：

1. 如果不信任它，当前证明主线是否真的无法推进？
2. 它是否只负责表示恢复/局部观察，而不是高层语义判断？
3. 它的合同是否足够小，错误影响是否被限制在局部？
4. 它是否有清楚的后续收紧方向？

推荐遵循下面这条简单判断：

- 能写成 observer / extractor / constructor witness 的，优先留在 bridge TCB；
- 表达整个高层原语语义的，不要放进 TCB，而要回到 `specs/*` 和 `cte.rs` 证明；
- 只服务于未完成路径控制流的，放进 `boundary_assumptions.rs`，并明确标注它不属于当前最小展示结果。

### 7.5 当前代码中的工程落点

- [interface.rs](/workspace/rel4_kernel/sel4_cspace/src/interface.rs)
  - verify-facing 公共合同层；外部验证代码应优先依赖这里，而不是直接依赖 `*_refined` 或 raw trusted helper。
- [refinement_bridge.rs](/workspace/rel4_kernel/sel4_cspace/src/refinement_bridge.rs)
  - 表示/观察 bridge TCB；只承接 concrete-to-abstract projection、slot/heap 观察与小型构造器。
  - 当前已收回为 crate-private 模块；verify-facing 的
    `ConcreteHeapId` / `ResolveAddressBitsAtRet` 类型名由 `interface.rs` 承接，
    外部代码不再需要直接经过 bridge 模块取这些名字。
- [boundary_assumptions.rs](/workspace/rel4_kernel/sel4_cspace/specs/boundary_assumptions.rs)
  - delete/revoke/finalise 等未完整覆盖路径的临时流程假设层；当前已收紧为 crate 内部模块，不再作为外部 proof API。
- `specs/*` 与 `cte.rs`
  - 核心 `CSpace` 语义、局部 invariant 和 refined proof 主线；它们是要被证明的对象，不应被重新塞回 TCB。

### 7.6 当前拍板后的冻结规则

在当前轮次之后，TCB 设计原则与分类方式视为拍板，不再轻易调整。后续允许的变化只应包括：

- 收缩现有 trusted helper 的合同或可见性；
- 用局部证明替换 `boundary_assumptions.rs` 中的临时流程假设；
- 把 verify-facing 接口继续从内部 bridge 细节中解耦。

后续原则上不再接受下面两类变化，除非文档与代码同时重新评审：

- 新增一类新的 trusted boundary 分类；
- 把新的高层 `CSpace` 语义判定重新塞回 TCB。

## 8. Verus 工程风格

### 8.1 总体风格

- 语义来源对齐 `l4v`
- 证明工程优先参考 `vostd`
- trusted 边界纪律参考 `atmo`
- 抽象模型与原语合同放在 `sel4_cspace/specs`
- proof 主线尽量回到 `sel4_cspace/src/cte.rs`

一句话概括就是：

`l4v 决定证明内容与语义基线，Verus 决定实现形态与证明工程。`

### 8.2 当前固定写法

- 优先签名式 `requires/ensures`
- `#[verus_spec(...)]` 只作为过渡，不作为长期主接口形态
- bridge 只解释数据形状，不承载最终业务语义
- proof 尽量贴近函数 body
- 新增 trusted util 必须小、局部、可收缩

## 9. 主要差距

当前仓库已经完成第一轮工程闭环，但距离“论文里可直接声称沿用 l4v 的 CSpace 证明路线”还有几处差距：

- 抽象 capability 语义仍有压缩建模痕迹；
- `refinement_bridge.rs` 里仍掺杂了部分语义折叠；
- `wf` 还没有完全重组为更贴近 l4v 叙事的 invariant 层；
- `deriveCap` 与部分 `final/no-children` 相关语义仍是子集级完成度；
- 具体函数证明仍较多依赖 refined wrapper。

## 10. 后续阶段计划

后续统一采用下面七个阶段，不再并行维护多套主计划编号。

### P1：冻结论文范围与 TCB 台账

- 固定“只证明 CSpace 局部性质”的边界
- 固定 TCB 清单
- 固定 l4v primary source

状态：

- 已完成

### P2：重构抽象 capability 语义

- 收紧 `sameRegionAs`
- 收紧 `sameObjectAs`
- 收紧 `isCapRevocable`
- 收紧 `mdb_parent_of` / `isMDBParentOf`
- 让 `deriveCap` 更直接对齐 l4v case split

状态：

- 已完成

当前完成口径：

- `sameRegionAs` 已从 `region_id` 近似切换为 capability 关系定义；
- `sameObjectAs`、`isCapRevocable`、`mdb_parent_of` 已统一回收到抽象语义 helper；
- `deriveCap` 的抽象 case split 已与当前 Rust 实现和 l4v generic 子集对齐；
- `ArchCap` 相关分支保留为显式 hook，当前仍属于受限子集/TCB 边界，不做静默近似。

### P3：提炼 CSpace 局部不变量

- 从当前 `wf` 中拆出更贴近 l4v 的 invariant 层
- 为 `ensureNoChildren`、`isFinalCapability`、`resolveAddressBits` 建立明确入口
- 继续把大前提收成可复用 lemma

状态：

- 已完成（第一轮）

当前进展：

- `wf` 已拆分出 `mdb_state_wf` 与 `cspace_lookup_wf` 两组局部 invariant；
- `isFinalCapability` 已切到 `is_final_cap_wf_at(slot)` 入口；
- `deriveCap` 已切到 `derive_cap_wf_at(slot)` 这一类 slot-local 入口；
- `ensureNoChildren` 已切到 `ensure_no_children_wf_at(slot)` 入口；
- `resolveAddressBits` 已切到 `spec_resolve_address_bits_state_wf(state)` 入口；
- 相关 reusable lemma 已开始替代“先要一个完整 `wf`”的证明写法。

### P4：把原语规格改写成 preservation-first 结构

- 功能合同 + frame condition + invariant preservation
- 对 `deriveCap`、`ensureNoChildren`、`isFinalCapability` 补足一致入口

状态：

- 已完成（第一轮）

当前进展：

- `cteInsert`、`insertNewCap`、`cteMove`、`cteSwap` 的 post 已拆成 `frame / invariant preservation / functional` 三层；
- `common.rs` 中已抽出通用 preservation helper；
- `deriveCap`、`ensureNoChildren`、`isFinalCapability`、`longRunningDelete` 已开始收敛到统一的 specs 入口命名；
- `cte.rs` 中的 refined wrapper 已开始复用这些统一入口，而不是直接抓底层 predicate。

### P5：收缩 bridge，只保留表示映射职责

- 清理 bridge 中的语义折叠
- 保留 concrete view 与 heap/state 对应关系
- 把语义定义放回 `specs`

状态：

- 已完成（第二轮）

当前进展：

- `resolve_address_bits` 的 projected-core 语义 helper 已从 `refinement_bridge.rs` 回流到 `specs/cspace_ops/resolve.rs`；
- `resolve_address_bits` 的 cap-level expected/result/core refinement 证明也已回流到 `specs/cspace_ops/resolve.rs`；
- bridge 中一批按 case 命名的 `resolve_address_bits` state-level 包装引理已收缩，只保留 one-step skeleton、raw/state 连接和通用 `result_refines_state` 包装；
- bridge 仍保留 concrete snapshot/view 提取与 heap/state 对应关系；
- 目前 bridge 里剩余的主要高层职责，是 raw input 与 abstract helper 的连接，以及 local heap transition 证明骨架。

### P6：逐个证明 Rust 函数满足抽象 spec

推荐顺序：

1. `resolve_address_bits`
2. `cte_insert`
3. `cte_move`
4. `cte_swap`
5. `derive_cap`
6. `ensure_no_children`
7. `is_final_cap`

状态：

- 已完成

当前进展：

- `resolve_address_bits` 的 trusted 假设已从“直接返回 expected core”收紧成“只满足 one-step control-flow skeleton”，再由 bridge 证明推到最终合同；
- `cteInsert`、`insertNewCap`、`cteMove`、`cteSwap` 的 trusted 假设已从“直接满足完整 exec_contract”收紧成“满足 local heap transition”，最终合同改由 `exec_step + abstract post + bridge lemma` 推导；
- `ensureNoChildren` 的 verified 路径已改走 `via_is_mdb_parent_of`；
- `isFinalCapability` 的 verified 路径已改走 `mdb_prev/mdb_next + same_object_as` 的重建证明；
- `isMDBParentOf` 的 verified 路径已改走 `mdb_revocable + same_region_as_refined + badge/first_badged` 的重建证明；
- `sameObjectAs` 的非 arch 路径已从 `same_region_as` 依赖改写成“cap-kind observer + concrete view object shape”重建，`IRQControlCap` 的 Rust 语义也已修正到与 l4v 一致；
- `sameRegionAs` 的非 arch 路径也已改成“更强的 `valid_cap` 边界 + `trusted_range_top_u128_if_small` + cap-kind observer + concrete view object shape”重建；
- `sameObjectAs` 与 `sameRegionAs` 的整函数 trusted 假设都已从证明主线中移除；
- `sameObjectAs` / `sameRegionAs` 已进一步抽成 bridge-level helper，`isFinalCapability` 与 `isMDBParentOf` 不再依赖 `trusted_slot_cap_clone`；
- `capability::sameObjectAs` / `capability::sameRegionAs` 在 `feature=verify` 下已切换为真实的 Verus 入口，而不再只是旁边保留一个 refined wrapper；
- `capability::is_cap_revocable(...)` 也已在 `feature=verify` 下切换为真实的 Verus 入口，第三个 capability-level query 已从“proof-only wrapper”前进一步变成“真实 rs 接口本身可验证”；
- 为了让上层 query proof 能开始走这条真实接口链路，bridge 新增了更小的 object-local observer `trusted_cap_ref_from_slot(...)`；
- bridge 的 `cap_snapshot_wf` 现已显式纳入 `Endpoint/Notification` 的 `badge_present` 结构约束，避免把这类 tag-to-layout 对应关系隐含在外部提取器体内；
- `cteInsert` 已新增一个不需要手工传入 `new_cap_is_revocable` ghost 的 verified wrapper，开始把“接口里的过渡 ghost”收回到真实 capability query 上；
- 仅用于 `cte.rs` 内部证明分解的 `via_*_refined` helper 已收回为私有函数，不再作为对外 proof 入口暴露；
- bridge 的 `wf` 已显式纳入“supported cap tag”边界，不再把这件事隐含在旧 observer 里；
- 当前 remaining trusted surface 已稳定收敛为：
  - concrete view 提取：
    - `trusted_view_cap`
    - `trusted_view_cte`
    - `trusted_view_resolve_address_bits_ret`
    - `trusted_extract_cap`
    - `trusted_extract_cte`
    - `trusted_extract_resolve_address_bits_ret`
  - heap/state 对应观察器：
    - `trusted_concrete_slot_view_at`
    - `trusted_concrete_cnode_lookup_slot_at`
  - pointer/object-local primitive：
    - `trusted_slot_ref_is_id`
    - `trusted_slot_ref_from_addr`
    - `trusted_cap_ref_from_slot`
  - 小粒度 arithmetic helper：
    - `trusted_range_top_u128_if_small`
  - return/constructor helper：
    - `trusted_make_exception_none`
    - `trusted_make_exception_syscall_error`
    - `trusted_check_exception_is_none`
    - `trusted_make_null_cap`
    - `trusted_clone_cap`
    - `trusted_make_derive_cap_ret`
- 当前 remaining trusted surface 已不再包含 `cte_t::is_mdb_parent_of`、`same_object_as`、`same_region_as`、`trusted_slot_cap_clone`、`trusted_has_mdb_prev/next`、`trusted_follow_mdb_prev/next` 这一层；query 侧主语义已回到“弱 observer + Verus 推导”的结构。

`P6` 完成口径：

- capability/query 侧真实 rs 入口已经基本切到 Verus-native 形态，不再主要依赖“proof-only refined wrapper”；
- `cte` proof 入口里的明显过渡 ghost 参数已经收缩到只剩当前抽象状态/heap 对齐所必需者，`cteInsert` 的显式 revocable ghost 已回收到真实 query；
- remaining trusted surface 已稳定收敛到一份小而清楚的名单，并且每一项都能解释“为什么留在 TCB”；
- 当前已证明入口与对应抽象 spec 的对应关系已经足够稳定，可以直接进入 `P7` 做总表、TCB 清单与论文收口。

### P7：收口论文材料与最终台账

- 已覆盖 / 未覆盖 / TCB 清单
- l4v 对应关系清单
- 当前已证 Rust 入口与对应抽象语义清单

状态：

- 已完成

当前写作出口：

- 独立论文草稿已整理到 `docs/verification/cspace-thesis-draft.md`
- 主计划文件继续负责技术口径与边界台账，不再承载全部正文草稿

`P7` 完成口径：

- 已证入口、未覆盖范围、TCB 与 l4v 对应关系四张总表已冻结成文；
- 论文草稿已独立整理为 `cspace-thesis-draft.md`，并具备摘要、引言、贡献、边界、术语、章节安排与结论/后续工作素材；
- `cargo xtask verify --package sel4_cspace --jobs 1 --max-errors 50` 已于 2026-05-01 再次回归通过，结果为 `249 verified, 0 errors`；
- 因此，`P7` 当前目标可以视为完成，后续工作不再属于本阶段收口，而属于论文润色或下一轮 trusted boundary 收紧 / exec 替换主线。

`P7` 固定口径：

- “已证入口”分成两类：
  - 真实 `feature=verify` Verus 接口
  - 对 `verifier::external_body` / 过渡 exec contract 建立 refinement 合同的证明入口
- 论文中必须明确区分：
  - “现有 Rust 接口本身已经带 Verus 契约”
  - “现有 Rust exec body 仍是 opaque，但已经有一个 refined wrapper 证明它满足抽象 spec”
- 下表中的 l4v 对应点，优先指向：
  - Haskell 定义来源
  - 代表性的 refinement / correspondence 证明文件
  - 不要求逐条复制 Isabelle 证明脚本结构

### P7.1 已证 Rust 入口总表

#### A. 真实 `feature=verify` 接口

| Rust 入口 | 当前形态 | 抽象合同 / 语义入口 | l4v 定义来源 | l4v 证明对齐 |
| --- | --- | --- | --- | --- |
| `capability::same_region_as(&cap, &cap)` | 真实 Verus 接口 | `same_region_as_exec_contract` -> `spec_same_region_as_caps` | `spec/haskell/src/SEL4/Object/ObjectType.lhs::sameRegionAs` | `proof/refine/CSpace1_R.thy::same_region_as_relation` |
| `capability::same_object_as(&cap, &cap)` | 真实 Verus 接口 | `same_object_as_exec_contract` -> `spec_same_object_as_caps` | `spec/haskell/src/SEL4/Object/ObjectType.lhs::sameObjectAs` | `proof/refine/ARM_HYP/Finalise_R.thy::isFinalCapability_corres` 依赖的 `sameObjectAs` 语义 |
| `capability::is_cap_revocable(&cap, &cap)` | 真实 Verus 接口 | `is_cap_revocable_exec_contract` -> `spec_is_cap_revocable` | `spec/haskell/src/SEL4/Object/ObjectType.lhs::isCapRevocable` | `proof/refine/CSpace1_R.thy::is_cap_revocable_eq` |

说明：

- 这三项是当前最“Verus-native”的结果，因为用户侧调用的 Rust 接口本身，在 `feature=verify` 下已经直接带 contract。
- 它们背后仍通过 `cte.rs` 中的 proof backend 完成证明，但论文主体应优先把这三项描述为“真实接口验证”，而不是“旁边另有 wrapper 被验证”。

#### B. `cte.rs` 中的 refined proof 入口

| Rust 入口 | 当前形态 | 抽象合同 / 语义入口 | l4v 定义来源 | l4v 证明对齐 |
| --- | --- | --- | --- | --- |
| `resolve_address_bits_refined` | refined wrapper over opaque body | `resolve_address_bits_exec_contract` -> `spec_resolve_address_bits` | `spec/haskell/src/SEL4/Kernel/CSpace.lhs::resolveAddressBits` | `proof/refine/CSpace1_R.thy` 中的 `resolveAddressBits_*` 引理 |
| `cte_insert_refined` | refined wrapper over opaque body | `cte_insert_exec_contract` -> `spec_cte_insert` / `spec_cte_insert_post` | `spec/haskell/src/SEL4/Object/CNode.lhs::cteInsert` | `proof/refine/CSpace1_R.thy::cteInsert_corres` |
| `insert_new_cap_refined` | refined wrapper over opaque body | `insert_new_cap_exec_contract` -> `spec_insert_new_cap` / `spec_insert_new_cap_post` | `spec/haskell/src/SEL4/Object/CNode.lhs::insertNewCap` | `proof/refine/ARM_HYP/Untyped_R.thy::insertNewCap_corres`（架构族中存在同名 proof family） |
| `cte_move_refined` | refined wrapper over opaque body | `cte_move_exec_contract` -> `spec_cte_move` / `spec_cte_move_post` | `spec/haskell/src/SEL4/Object/CNode.lhs::cteMove` | `proof/refine/CSpace_R.thy::cteMove_corres` |
| `cte_swap_refined` | refined wrapper over opaque body | `cte_swap_exec_contract` -> `spec_cte_swap` / `spec_cte_swap_post` | `spec/haskell/src/SEL4/Object/CNode.lhs::cteSwap` | `proof/refine/CSpace1_R.thy::cteSwap_corres` |
| `derive_cap_refined` | refined wrapper over opaque body | `derive_cap_exec_contract` -> `spec_derive_cap_post` | `spec/haskell/src/SEL4/Object/ObjectType.lhs::deriveCap` | `proof/refine/CSpace_R.thy::deriveCap_corres` |
| `is_mdb_parent_of_refined` | refined wrapper over proof reconstruction | `is_mdb_parent_of_exec_contract` -> `spec_is_mdb_parent_of_post` | `spec/haskell/src/SEL4/Object/CNode.lhs::isMDBParentOf` | `proof/refine/CSpace1_R.thy` 中 `isMDBParentOf` relation / preservation 引理 |
| `ensure_no_children_refined` | refined wrapper over proof reconstruction | `ensure_no_children_exec_contract` -> `spec_ensure_no_children_expected_error` | `spec/haskell/src/SEL4/Object/CNode.lhs::ensureNoChildren` | `proof/refine/CSpace1_R.thy::ensureNoChildren_corres` |
| `is_final_cap_refined` | refined wrapper over proof reconstruction | `is_final_cap_exec_contract` -> `spec_is_final_cap_post` | `spec/haskell/src/SEL4/Object/CNode.lhs::isFinalCapability` | `proof/refine/ARM_HYP/Finalise_R.thy::isFinalCapability_corres` |
| `is_long_running_delete_refined` | refined wrapper over proof reconstruction | `is_long_running_delete_exec_contract` -> `spec_is_long_running_delete_post` | `spec/haskell/src/SEL4/Object/CNode.lhs::slotCapLongRunningDelete` / `longRunningDelete` | `proof/refine/*/Tcb_R.thy::slotCapLongRunningDelete_corres` |

说明：

- 这批入口已经足够支持论文里的“Rust 代码满足抽象 spec”叙述，但表述上必须诚实说明：当前证明对象是 opaque exec body 的 refinement，而不是每个 exec body 都已经改写成 fully verified Verus body。
- `same_region_as_refined`、`same_object_as_refined`、`is_cap_revocable_refined` 目前主要承担上表 A 的 proof backend 角色；论文里可以提到它们是 capability 真实接口的内部证明后端，但不应把它们与用户侧稳定接口混为一谈。

### P7.2 当前可对论文直接主张的结果

- 在“只证明 `CSpace` 局部性质、低层与非 `CSpace` 子系统进入 TCB”的范围约束下，当前仓库已经为选定 `CSpace` 子集建立了抽象语义、局部 invariant、primitive spec 与 Rust refinement 主线。
- 对 `sameRegionAs`、`sameObjectAs`、`isCapRevocable`、`deriveCap`、`isMDBParentOf`、`resolveAddressBits`、`cteInsert`、`insertNewCap`、`cteMove`、`cteSwap`、`ensureNoChildren`、`isFinalCapability`、`slotCapLongRunningDelete`，都已经存在可引用的 Verus 规格入口与证明入口。
- 其中 capability query 三项已经是“真实 Rust 接口本身带 Verus 契约”；其余核心原语当前是“opaque exec body + refined wrapper contract”的完成形态。
- 这条主线在证明内容和分解方式上明确借鉴 l4v：语义来源主要对齐 `ObjectType.lhs` / `CNode.lhs` / `Kernel/CSpace.lhs`，证明叙事则对应 `CSpace1_R.thy`、`CSpace_R.thy`、`Untyped_R.thy`、`Finalise_R.thy`、`Tcb_R.thy` 中的 correspondence / invariant preservation 路线。

### P7.3 当前未覆盖范围

| 范围 | 当前状态 | 备注 |
| --- | --- | --- |
| `cte_insert` / `insert_new_cap` / `cte_move` / `cte_swap` 的 exec body 本体 | 尚未改写为 fully verified Verus body | 当前是 `contract-bearing external_body` + refined wrapper |
| `resolve_address_bits` 的普通构建实现与 verify 实现对象 | 尚未完全合一 | verify 路径已切到 Verus-native one-step skeleton，但普通构建路径仍保留现有 Rust body |
| `cte_t::derive_cap` / `ensure_no_children` / `is_final_cap` 等方法的构建/验证路径 | 尚未完全合一 | verify 路径已切到签名式 Verus 方法体，但普通构建路径仍保留现有 Rust body |
| 删除 / revoke / finalise 主路径 | 未纳入当前完成范围 | 例如 `delete_all`、`delete_one`、`revoke` 仍未形成完整本地证明闭环 |
| 全局 kernel invariant | 未覆盖 | 当前只证明 `CSpace` 局部 invariant，而非整系统 `invs` |
| arch-specific capability 详细语义 | 显式保留为 hook / TCB | 当前抽象模型中 `ArchCap` 不做静默近似补造 |
| bitfield 代码、底层内存模型、FFI 内部实现 | 不在本轮证明对象中 | 属于固定 trusted boundary |

### P7.4 当前 TCB 台账

| TCB 类别 | 当前项 | 保留原因 | 后续方向 |
| --- | --- | --- | --- |
| concrete view 提取 | `trusted_view_cap`、`trusted_view_cte`、`trusted_view_resolve_address_bits_ret`、`trusted_extract_*` | 当前 `cap` / `cte_t` / 返回结构仍是 opaque external type，需要一个小边界把 concrete shape 投影到抽象 `CapSpec` / `SlotEntrySpec` / return core | 继续尝试把更强的结构约束写进 snapshot `wf`，逐步缩小提取器自由度 |
| heap/state 对应观察器 | `trusted_concrete_slot_view_at`、`trusted_concrete_cnode_lookup_slot_at` | 当前 local heap transition 证明仍需要“concrete heap 某位置视图是什么”这一桥接观察器 | 后续可考虑把这层再拆成更细粒度 slot-level observer |
| pointer / object-local primitive | `trusted_slot_ref_is_id`、`trusted_slot_ref_from_addr`、`trusted_cap_ref_from_slot` | 需要把 raw pointer/address 与抽象 `SlotId` 连接，但当前不证明完整内存模型 | 后续若引入更强内存/指针模型，可继续下沉 |
| arithmetic helper | `trusted_range_top_u128_if_small` | 用于把机器整数范围运算连接到抽象 `pow2/range-top`，尤其支撑 untyped containment | 独立的 TCB 收紧小任务，已明确不阻塞当前 `P7` |
| return / constructor helper | `trusted_make_exception_none`、`trusted_make_exception_syscall_error`、`trusted_check_exception_is_none`、`trusted_make_null_cap`、`trusted_clone_cap`、`trusted_make_derive_cap_ret` | 当前只把“构造该返回值 / 克隆该 cap 的 concrete 结果与抽象 view 一致”留在可信边界 | 后续若逐步内化具体结构体字段语义，可进一步移出 TCB |
| fixed external system boundary | generated bitfield/getter、底层内存读写、`deps` FFI、非 `CSpace` 子系统前提、arch 细节 | 不属于当前“只证明 CSpace 局部性质”的目标范围 | 论文中作为显式 TCB/前提列出，不混入已证 claim |

### P7.5 这一阶段结束时希望冻结的论文措辞

- 我们证明的不是整个 kernel，而是在显式 trusted boundary 之内，对 Rust 重写中的 `CSpace` 子集建立了与 l4v 一致的局部语义、局部不变量与 refinement 证明。
- 我们没有把 bridge 扩张成新的长期语义层；bridge 只负责 concrete 表示到抽象模型的最小连接。
- 当前最强的结果是 capability query 三项已经成为真实 Verus 接口；其余 mutating / lookup primitive 目前则以 refined wrapper 形式证明其满足抽象 spec。

### P7.6 论文正文压缩版

#### A. 摘要版主张

- 本文不验证整个 seL4 Rust 重写，而是将证明范围收敛为 `sel4_cspace` 子系统的局部语义与局部不变量。
- 在把底层 bitfield、指针/内存读写、FFI、arch-specific 细节和非 `CSpace` 子系统前提视为 trusted boundary 的条件下，本文为选定 `CSpace` 子集建立了 Verus 抽象模型、primitive specification 与 Rust refinement 证明。
- 所验证的语义内容和证明分解方式主要参考 l4v 中关于 `sameRegionAs`、`sameObjectAs`、`isCapRevocable`、`deriveCap`、`resolveAddressBits`、`cteInsert`、`cteMove`、`cteSwap`、`isMDBParentOf`、`ensureNoChildren`、`isFinalCapability` 的既有定义与 correspondence 路线。
- 当前结果中，capability query 三项已经是“真实 Rust 接口本身带 Verus contract”的形态，其余核心原语则采用“opaque exec body + refined wrapper”方式证明满足抽象 spec。

#### B. 正文段落模板

可直接作为“研究范围与目标”段落的底稿：

`本文关注的是 seL4 Rust 重写中的 CSpace 子系统，而不是整个内核的全系统验证。考虑到当前代码基中大量底层位域访问、指针转换、FFI 调用以及非 CSpace 子系统语义尚未整体迁移到 Verus 中，本文采用局部验证策略：将这些底层实现细节与外部子系统前提明确纳入 trusted boundary，只对 CSpace 子集本身的局部语义、局部不变量与关键原语的 refinement 关系建立形式化证明。`

可直接作为“方法概述”段落的底稿：

`在语义来源上，本文尽量复用 l4v 对 CSpace 的既有设计，主要参考 `SEL4/Object/ObjectType.lhs`、`SEL4/Object/CNode.lhs` 与 `SEL4/Kernel/CSpace.lhs` 中的函数定义，以及对应的 correspondence / refinement 证明文件；在证明工程上，则采用 Verus-native 的模块划分与契约风格。具体而言，本文首先建立抽象 capability 与抽象 CSpace 状态模型，然后提炼只依赖 CSpace 本身的局部 invariant，再为核心原语定义 preservation-first specification，最后证明 Rust 实现通过 bridge 层细化到这些抽象合同。`

可直接作为“当前结果”段落的底稿：

`在当前阶段，本文已经为 `sameRegionAs`、`sameObjectAs`、`isCapRevocable`、`deriveCap`、`resolveAddressBits`、`cteInsert`、`insertNewCap`、`cteMove`、`cteSwap`、`isMDBParentOf`、`ensureNoChildren`、`isFinalCapability` 与 `slotCapLongRunningDelete` 建立了可复用的 Verus 规格入口与证明入口。其中，`sameRegionAs`、`sameObjectAs` 和 `isCapRevocable` 已经在 `feature=verify` 下转化为真实的 Verus 接口；其余原语当前仍保留现有 Rust exec body，并通过 refined wrapper 证明它们满足相应抽象 specification。`

可直接作为“局限性与边界”段落的底稿：

`需要强调的是，本文当前并未证明整个 kernel state 的全局一致性，也未验证所有 arch-specific capability 细节、bitfield 生成代码、底层内存模型和 FFI 内部实现。因此，本文的结论应被理解为：在显式 trusted boundary 之内，选定的 Rust CSpace 子集已经获得与 l4v 语义路线一致的局部形式化保证，而不是对整个系统行为给出端到端完备证明。`

#### C. 图表标题建议

- 表：`已证 Rust 入口与抽象规格对应关系`
- 表：`CSpace 子集的 l4v 语义来源与 Verus 证明入口对应表`
- 表：`当前 trusted boundary（TCB）分类与保留原因`
- 表：`当前未覆盖范围与后续扩展方向`
- 图：`从 concrete CSpace 到 abstract CSpace 的 bridge/refinement 结构`

#### D. 建议避免的表述

- 不要写“本文完成了 Rust 版 seL4 CSpace 的完整验证”。
- 不要写“现有所有 Rust 实现函数都已经被 Verus 直接替换为 fully verified body”。
- 不要把 `*_refined` wrapper 与真实对外 Rust 接口混写成同一层完成度。
- 不要把当前局部 invariant 直接表述成 l4v 的整系统 `invs` 已被复现。
- 不要隐去 TCB；trusted boundary 必须以显式前提形式出现。

### P7.7 论文草稿素材

#### A. 中文摘要草稿

`随着系统软件验证工具的发展，针对已有高可信内核设计进行新的语言重写并复用既有验证思路，成为形式化方法与系统实现结合的重要方向。本文以 seL4 的 Rust 重写中的 CSpace 子系统为对象，研究如何在不追求整系统完备验证的前提下，对局部关键语义建立与 l4v 路线一致的形式化保证。本文将底层 bitfield 访问、指针与内存读写、FFI、arch-specific capability 细节及非 CSpace 子系统前提视为显式 trusted boundary，在此基础上使用 Verus 构建了 CSpace 的抽象 capability 模型、抽象状态模型与局部不变量，并为 `sameRegionAs`、`sameObjectAs`、`isCapRevocable`、`deriveCap`、`resolveAddressBits`、`cteInsert`、`cteMove`、`cteSwap`、`isMDBParentOf`、`ensureNoChildren` 与 `isFinalCapability` 等核心操作建立了规格与 refinement 证明。当前结果表明：选定的 Rust CSpace 子集已经获得与 l4v 语义路线一致的局部形式化保证，其中 capability query 三项已转化为真实的 Verus 接口，其余核心原语则通过 refined wrapper 证明满足抽象 specification。本文展示了一条在显式可信边界内复用 l4v 语义、并以 Verus-native 工程方式推进 Rust 内核子系统验证的可行路径。`

可替换关键词建议：

- `seL4`
- `Rust`
- `Verus`
- `CSpace`
- `形式化验证`
- `Refinement`

#### B. 中文引言开头草稿

第一段可用版本：

`高可信操作系统内核的形式化验证长期以来主要围绕 seL4 及其 l4v 证明体系展开。随着 Rust 在系统软件中的广泛应用，围绕现有高可信内核设计进行 Rust 重写，并尝试将既有验证经验迁移到新的实现载体上，成为一个具有研究意义的问题。相比从零开始为一个全新系统设计验证体系，这一路径的价值在于：一方面可以复用 seL4 在对象模型、能力系统和证明结构上的成熟经验，另一方面也可以检验 Verus 等新一代 Rust 验证工具在真实系统代码上的适用性。`

第二段可用版本：

`然而，直接复刻 l4v 的整系统证明并不现实。当前 Rust 重写代码仍包含大量底层位域操作、指针转换、FFI 调用以及尚未纳入统一抽象模型的外部子系统语义。如果忽略这些现实约束而简单声称“验证整个 Rust 版 seL4”，既不准确，也不利于形成可持续推进的验证工程。因此，本文选择一种更审慎的局部验证策略：先把验证目标收敛到 CSpace 子系统本身，只证明其局部语义、局部不变量与关键原语的 refinement 关系，并将其余底层实现细节与外部依赖显式纳入 trusted boundary。`

第三段可用版本：

`选择 CSpace 作为切入点有两方面原因。首先，能力系统是 seL4 内核设计的核心组成部分，`sameRegionAs`、`sameObjectAs`、`deriveCap`、`cteInsert`、`resolveAddressBits` 等操作直接决定了能力派生、查找与局部更新的语义正确性。其次，l4v 已经为这些操作提供了成熟的定义与 correspondence 证明路线，使得“证明什么、如何分解证明任务”并非完全无据可依。本文的目标不是机械翻译 Isabelle 证明脚本，而是在语义内容上尽量对齐 l4v，在表示方式、模块结构与证明组织上采用更适合 Verus/Rust 的工程形态。`

第四段可用版本：

`基于这一目标，本文首先建立 Rust CSpace 子集对应的抽象 capability 与抽象状态模型，随后提炼只依赖 CSpace 本身的局部 invariant，再为关键原语定义 preservation-first specification，并通过 bridge 层把 concrete `cap/cte/ret` 与抽象模型连接起来。在此基础上，本文分别证明 capability query、lookup 原语以及若干局部修改原语满足相应抽象合同，最终形成一份明确区分“已证入口”“未覆盖范围”与“trusted boundary”的本地证明台账。`

#### C. 贡献列表草稿

可直接作为论文“主要贡献”小节的版本：

1. 本文在显式 trusted boundary 约束下，为 seL4 Rust 重写中的 `CSpace` 子系统建立了一套局部验证框架，将证明目标从“整系统验证”收敛为“局部语义 + 局部不变量 + Rust refinement”，使验证范围、完成度与可信边界都能够被清晰描述。
2. 本文在语义层面尽量复用 l4v 对 `sameRegionAs`、`sameObjectAs`、`isCapRevocable`、`deriveCap`、`resolveAddressBits`、`cteInsert`、`cteMove`、`cteSwap`、`isMDBParentOf`、`ensureNoChildren` 与 `isFinalCapability` 的既有设计，同时在证明工程上采用 Verus-native 的抽象模型、bridge 分层与签名式契约组织方式，形成了“l4v 决定语义、Verus 决定实现形态”的迁移路线。
3. 本文为选定的 Rust CSpace 子集建立了可复用的规格与证明入口，其中 capability query 三项已经成为真实的 Verus 接口，其余关键原语则通过 refined wrapper 证明满足抽象 specification，形成了从 concrete Rust 实现到抽象 CSpace 语义的初步 refinement 闭环。
4. 本文整理了当前阶段的已证入口、未覆盖范围与 TCB 清单，为后续继续局部化 trusted helper、或逐步用 fully verified Verus body 替换现有 opaque exec 实现，提供了可持续扩展的工程基线和论文口径。

#### D. 如果需要更短的摘要版贡献点

- 给出了一个面向 `sel4_cspace` 的 Verus 局部验证框架。
- 在语义上对齐 l4v，在工程上采用 Verus-native 分层。
- 为核心 capability query、lookup 和局部更新原语建立了可复用的规格与 refinement 入口。
- 显式整理了当前已证范围、未证范围与 trusted boundary。

### P7.8 候选定稿版

#### A. 候选摘要定稿

`本文面向 seL4 的 Rust 重写，研究如何在显式可信边界内对 CSpace 子系统建立局部形式化保证。与整系统验证不同，本文将底层 bitfield 访问、指针与内存读写、FFI、arch-specific capability 细节以及非 CSpace 子系统前提视为 trusted boundary，只对 CSpace 子集本身的语义、局部不变量和关键原语的 refinement 关系进行证明。语义来源上，本文尽量复用 l4v 对 `sameRegionAs`、`sameObjectAs`、`isCapRevocable`、`deriveCap`、`resolveAddressBits`、`cteInsert`、`cteMove`、`cteSwap`、`isMDBParentOf`、`ensureNoChildren` 和 `isFinalCapability` 的既有定义与证明分解方式；工程实现上，则采用 Verus-native 的抽象模型、bridge 分层与签名式契约组织。当前结果表明：选定的 Rust CSpace 子集已经获得与 l4v 路线一致的局部语义与 refinement 保证，其中 capability query 三项已成为真实的 Verus 接口，其余核心原语则通过 refined wrapper 证明满足抽象 specification。本文展示了在不回避可信边界的前提下，将 l4v 语义路线迁移到 Rust/Verus 验证工程中的一种可行方法。`

#### B. 候选引言收束段

`因此，本文的核心目标不是给出 Rust 版 seL4 的整系统完备证明，而是在明确可信边界的基础上，为 CSpace 子系统建立一条可持续扩展的局部验证主线。围绕这一目标，本文一方面复用 l4v 已经成熟回答的“证明什么”这一问题，尽量保持对能力关系、派生规则、查找语义和局部更新原语的语义口径一致；另一方面使用 Verus-native 的方式重新组织“如何证明”，将抽象建模、局部 invariant、primitive specification、bridge 映射与 Rust refinement 证明整合到统一工程中。这样的设计既保留了 l4v 的理论来源，也为后续继续局部化 trusted helper、扩展到更多 Rust exec 实现、甚至逐步连接到更大范围的内核验证工作提供了稳定起点。`

#### C. 候选贡献定稿

适合正文“三点贡献”版本：

1. 提出了一种面向 seL4 Rust 重写 `CSpace` 子系统的局部验证路线，在显式 trusted boundary 下把验证目标稳定收敛为“局部语义、局部不变量与 Rust refinement”，从而避免将尚未覆盖的底层机制和外部子系统混入已证结论。
2. 在语义上复用 l4v 的 CSpace 设计，在工程上采用 Verus-native 的抽象模型、bridge 分层与契约组织方式，形成了“l4v 提供语义基线，Verus 承担实现与证明工程”的迁移框架。
3. 为选定的 Rust CSpace 子集建立了可复用的规格与证明入口，并系统整理了已证范围、未证范围与 TCB 台账，为后续继续收紧 trusted boundary 和替换更多 opaque exec body 提供了可持续扩展的基线。

如果导师更偏工程实现导向，可改写为：

1. 完成了 `sel4_cspace` 子集的抽象建模、primitive specification 与 refinement 主线搭建。
2. 打通了 capability query、lookup 和局部更新原语的 Verus 证明入口，并将部分真实 Rust 接口直接提升为 Verus contract 形式。
3. 形成了显式区分“真实接口已证”“refined wrapper 已证”“未覆盖范围”“trusted boundary”的论文口径与工程台账。

#### D. 答辩时可直接使用的一句话概括

`这项工作的重点不是证明整个 Rust 版 seL4，而是在显式可信边界内，把 l4v 对 CSpace 的证明思路迁移到 Verus，并为 Rust CSpace 子集建立一条已经跑通的局部 refinement 验证主线。`

### P7.9 推荐终稿版

这一节不再提供多套候选，而是给出当前最建议直接采用、再按学校格式微调的一版。

#### A. 推荐摘要终稿

`本文面向 seL4 的 Rust 重写，研究如何在显式可信边界内对 CSpace 子系统建立局部形式化保证。与整系统验证不同，本文将底层 bitfield 访问、指针与内存读写、FFI、arch-specific capability 细节以及非 CSpace 子系统前提视为 trusted boundary，只对 CSpace 子集本身的语义、局部不变量和关键原语的 refinement 关系进行证明。语义来源上，本文尽量复用 l4v 对 `sameRegionAs`、`sameObjectAs`、`isCapRevocable`、`deriveCap`、`resolveAddressBits`、`cteInsert`、`cteMove`、`cteSwap`、`isMDBParentOf`、`ensureNoChildren` 和 `isFinalCapability` 的既有定义与证明分解方式；工程实现上，则采用 Verus-native 的抽象模型、bridge 分层与签名式契约组织。当前结果表明：选定的 Rust CSpace 子集已经获得与 l4v 路线一致的局部语义与 refinement 保证，其中 capability query 三项已成为真实的 Verus 接口，其余核心原语则通过 refined wrapper 证明满足抽象 specification。本文展示了在不回避可信边界的前提下，将 l4v 语义路线迁移到 Rust/Verus 验证工程中的一种可行方法。`

如果摘要篇幅受限，优先删减顺序：

1. 先删最后一句“可行方法”的总结性评价。
2. 再把函数名列表压成“核心 capability relation、lookup 与局部更新原语”。
3. 最后才删 trusted boundary 的具体枚举项。

#### B. 推荐引言主线

建议最终引言按下面四段组织：

1. 背景段：
   说明 seL4/l4v 的地位，以及 Rust 重写与 Verus 带来的新问题。
2. 问题段：
   说明为什么当前阶段不能诚实地声称“整系统验证”，并引出显式 trusted boundary。
3. 选题段：
   说明为什么选 CSpace，强调 capability relation、lookup、局部更新语义的重要性，以及 l4v 已提供成熟基线。
4. 方法与结果段：
   概述抽象模型、局部 invariant、primitive spec、bridge、refinement 证明，以及当前已完成到什么程度。

推荐直接采用的引言收束段：

`因此，本文的目标不是给出 Rust 版 seL4 的整系统完备证明，而是在明确可信边界的基础上，为 CSpace 子系统建立一条可持续扩展的局部验证主线。围绕这一目标，本文一方面复用 l4v 已经成熟回答的“证明什么”这一问题，尽量保持对能力关系、派生规则、查找语义和局部更新原语的语义口径一致；另一方面使用 Verus-native 的方式重新组织“如何证明”，将抽象建模、局部 invariant、primitive specification、bridge 映射与 Rust refinement 证明整合到统一工程中。这样的设计既保留了 l4v 的理论来源，也为后续继续局部化 trusted helper、扩展到更多 Rust exec 实现提供了稳定起点。`

#### C. 推荐贡献终稿

推荐正文直接使用三点贡献版本：

1. 提出了一种面向 seL4 Rust 重写 `CSpace` 子系统的局部验证路线，在显式 trusted boundary 下把验证目标收敛为“局部语义、局部不变量与 Rust refinement”，从而使已证范围、未证范围与可信边界都能够被清晰描述。
2. 在语义上复用 l4v 的 CSpace 设计，在工程上采用 Verus-native 的抽象模型、bridge 分层与签名式契约组织方式，形成了“l4v 提供语义基线，Verus 承担实现与证明工程”的迁移框架。
3. 为选定的 Rust CSpace 子集建立了可复用的规格与证明入口，其中 capability query 三项已经成为真实的 Verus 接口，其余关键原语则通过 refined wrapper 证明满足抽象 specification，并进一步整理出已证范围、未证范围与 TCB 台账，为后续继续收紧 trusted boundary 和替换更多 opaque exec body 提供了基线。

#### D. 推荐题目风格

如果论文题目还没完全定，可以优先考虑下面这种风格：

- `基于 Verus 的 seL4 Rust 重写中 CSpace 子系统局部形式化验证`
- `面向 seL4 Rust 重写的 CSpace 子系统局部 Refinement 验证研究`
- `复用 l4v 语义路线的 Rust CSpace 子系统 Verus 验证方法`

推荐原则：

- 题目里优先体现 `CSpace`、`Rust`、`Verus`、`局部验证/Refinement` 四个关键词。
- 不要在题目里写成“seL4 Rust 内核完整验证”。
- 如果学校更偏中文风格，`局部形式化验证` 比 `Refinement` 更稳；如果导师偏形式化方法风格，可保留 `Refinement`。

#### E. 推荐答辩开场 30 秒版本

`我的工作不是去证明整个 Rust 版 seL4，而是在显式可信边界内，选取其中的 CSpace 子系统，复用 l4v 已有的语义路线，用 Verus 为 capability relation、lookup 和若干局部更新原语建立抽象规格与 refinement 证明。当前已经跑通了一个局部验证闭环，并整理出已证范围、未证范围和 TCB 台账，为后续继续扩展提供了基线。`

## 11. P7 后建议

如果按当前主线继续推进，最自然的顺序是：

1. 以 `P7.9` 为主版本，按学校模板和导师偏好做最后的篇幅压缩与术语统一。
2. 如果技术主线继续推进，优先把 `trusted_range_top_u128_if_small`、concrete view 提取器这类独立的 TCB 收紧小任务单独推进。
3. 如果目标转向“最终用 Verus body 替换更多现有 Rust exec 实现”，则下一轮应从 `cte_insert` / `derive_cap` / `ensure_no_children` 这一类仍依赖 opaque body 的入口开始拆。

## 12. 当前代码总结

### 12.1 当前代码分层

当前 `sel4_cspace` 的验证与实现已经稳定成下面这套分层：

- `sel4_cspace/specs/abstract_cspace.rs`
  - 抽象 capability、抽象 `CSpaceState`、局部 `wf` 与基础语义 helper
- `sel4_cspace/specs/cspace_ops/*.rs`
  - `derive / insert / move / swap / resolve / queries / common / smoke`
  - 承接 primitive specification、preservation-first post、query 合同与 smoke proof
- `sel4_cspace/src/refinement_bridge.rs`
  - concrete `cap/cte/ret` 到 abstract model 的最小表示桥
  - 当前仍集中管理 trusted view / extractor / slot-level observer
- `sel4_cspace/src/cte.rs`
  - concrete Rust exec body
  - 与之并列的 refined proof 入口、`*_exec_contract`、bridge-side proof helper
- `sel4_cspace/src/capability/mod.rs`
  - capability query 对外接口
  - 其中一部分在 `feature=verify` 下已切换为真实 Verus 接口
- `sel4_cspace/src/interface.rs`
  - 当前 kernel / 外部模块使用的公共导出入口

一句话概括：

`specs` 负责抽象语义，`refinement_bridge` 负责最小表示映射，`cte.rs` 负责 concrete exec 与 refinement proof 主线。

### 12.2 当前完成度

按“已经证明到哪一层”来分，当前代码状态如下：

- 抽象语义与 primitive spec：
  - 已完成第一轮稳定收口
  - `sameRegionAs / sameObjectAs / isCapRevocable / deriveCap / resolveAddressBits / cteInsert / cteMove / cteSwap / isMDBParentOf / ensureNoChildren / isFinalCapability` 都已有可引用的抽象规格入口
- 真实 Verus 接口：
  - `capability::same_region_as`
  - `capability::same_object_as`
  - `capability::is_cap_revocable`
  - 这三项是当前最接近“运行时接口与验证接口重合”的部分
  - `interface.rs` 现也统一重导出了这组三个 capability query，便于将最终展示子集收口到同一公共接口层
- 稳定 verify-facing 接口：
  - `interface::is_final_cap_at`
  - `interface::ensure_no_children_at`
  - `interface::is_long_running_delete_at`
  - `interface::derive_cap_at`
  - `interface::is_mdb_parent_of_at`
  - `interface::resolve_address_bits_at`
  - `interface::cte_insert_at`
  - `interface::insert_new_cap_at`
  - `interface::cte_move_at`
  - `interface::cte_swap_at`
  - 对应的公开前置条件包装也已补齐到 `interface.rs`：
    - `*_at_pre`
  - 这一层是当前前 7 步的主要收口结果：外部验证代码不再直接依赖 `cte.rs` 中公开的 proof backend 名称
  - 当前第四轮接口收紧后，外部验证代码也不必优先直接依赖 `refinement_bridge::*_call_pre_at`
  - delete 主线当前轮次也已经接入了局部 gate 入口，而不是继续完全停留在内部 helper
  - 其中 `ensure_no_children_at` 与 `derive_cap_at` 现已额外提供稳定的结果观察口径：
    - `exception_status_is_none`
    - `exception_status_is_syscall_error`
    - `derive_cap_ret_status_is_none`
    - `derive_cap_ret_status_is_syscall_error`
    - `derive_cap_ret_capability`
  - 第 3 轮继续推进后，`is_final_cap_at`、`is_long_running_delete_at`、`is_mdb_parent_of_at` 已可直接给出对应抽象布尔语义，而 `resolve_address_bits_at` 也已补出稳定的返回结果观察口径：
    - `resolve_address_bits_ret_core`
    - `resolve_address_bits_expected_ret_core`
    - `resolve_address_bits_ret_status_is_success`
    - `resolve_address_bits_ret_status_is_lookup_fault`
    - `resolve_address_bits_ret_slot`
    - `resolve_address_bits_ret_bits_remaining`
- 内部 proof backend：
  - `resolve_address_bits_refined`
  - `cte_insert_refined`
  - `insert_new_cap_refined`
  - `cte_move_refined`
  - `cte_swap_refined`
  - `derive_cap_refined`
  - `is_mdb_parent_of_refined`
  - `ensure_no_children_refined`
  - `is_final_cap_refined`
  - `is_long_running_delete_refined`
  - 这批入口当前已收回为 `pub(crate)` 或模块私有，不再作为公共 proof 表面暴露
- 当前尚未完成到“fully verified Verus body”的部分：
  - 四个 mutating primitive 的原始 exec body 仍主要是 `verifier::external` 或等价的 opaque 入口
  - `resolve_address_bits`、`derive_cap`、`ensure_no_children`、`is_final_cap` 等入口虽然已有 verify-native body，
    但普通构建路径与 verify 路径仍未完全合一
  - delete/revoke/finalise 主路径当前已被拆成“已证局部 gate + 显式 boundary assumption”，但尚未形成 full local proof
  - 当前证明重心仍然是“原始实现满足抽象合同”，而不是“原始实现体逐句由 Verus 直接验证”

### 12.3 当前最准确的工程判断

当前仓库的 `CSpace` 实现方式，更接近：

- 现有 Rust 实现继续承担运行时主路径
- `specs + bridge + refined wrapper` 在旁边建立局部 refinement 证明

而不是：

- `atmo` 那种 verified core 直接作为 runtime core 被 kernel 主线调用

因此，当前最准确的判断是：

- 已经建立了一条稳定的局部 refinement 证明主线
- capability query 三项已进入“真实 Verus 接口”阶段
- `interface.rs` 已补出一层稳定的 verify-facing facade，用于承接前 7 步中的 slot-local / lookup / mutating proof 接口
- delete 主线当前也已拥有局部 verify-facing gate 与显式 boundary assumption 台账
- build object 与 verified object 仍未完全重合，但现在已经有统一的 build+verify 联合检查脚本

### 12.4 当前最强可主张结果

- 最近一次记录的 Verus 回归结果为 `249 verified, 0 errors`
- `CSpace` 抽象语义、局部 invariant、primitive spec、bridge 和 refined entry 已形成闭环
- capability query 三项已成为真实 Verus 接口
- `cteInsert / cteMove / cteSwap / resolveAddressBits / deriveCap / ensureNoChildren / isFinalCapability` 已拥有 refinement proof 入口
- `interface.rs` 现已提供前 9 步当前轮次对应的稳定 verify-facing 接口层，而 `cte.rs` 中的 proof backend 已收回内部
- `interface.rs` 现同时提供公开的 `*_at_pre` 与 `*_at` facade，verify-facing 代码可以先依赖这层，再下接 bridge / refined proof backend
- delete/revoke/finalise 当前轮次已至少被拆成：
  - `is_mdb_parent_of_at`
  - `is_long_running_delete_at`
  - 以及 `boundary_assumptions.rs` 中对 `reduce_zombie / delete_all / revoke` 的显式边界假设
- `tools/check-cspace-build-and-verify.sh` 已将普通构建与官方 Verus 校验并排固化成一个联合入口
- 当前论文口径已经可以稳定地区分：
  - 真实接口已证
  - refined wrapper 已证
  - remaining TCB
  - 未覆盖范围

### 12.5 当前主要差距

- `cte_insert / insert_new_cap / cte_move / cte_swap` 这四个 mutator 的运行时原始 body 还没有转成 fully verified Verus body
- `resolve_address_bits`、`derive_cap`、`ensure_no_children`、`is_final_cap` 等入口虽然已有 verify-native body，
  但普通构建路径与 verify 路径仍未完全重合
- `refinement_bridge.rs` 中仍保留一批 concrete view / slot observer / arithmetic helper trusted surface
- `refinement_bridge::*_call_pre_at` 这类 bridge 预条件虽然在公共 API 口径上已经退回到 `interface::*_at_pre` 背后，
  但在 crate 内 proof backend 里仍然是实现层依赖
- 当前虽然已经有 `tools/check-cspace-build-and-verify.sh` 联合入口，但 build path 与 verify path 在工具链层仍然部分分离：
  - build 主要走普通 `cargo`
  - verify 主要走 `cargo-verus`
- 删除 / revoke / finalise 主路径虽然已拆出局部 gate 和边界假设，但还未完成完整本地证明闭环

## 13. 毕设最终展示代码要求

### 13.1 目标定位

毕设最终展示代码的目标，不应再只是“旁边有一套 proof artifact”，而应尽量做到：

`选定的 CSpace 子集在参与正式构建的同时，也尽量直接对应到论文中已经证明过的那部分实现。`

这里的关键词是“尽量对齐”，而不是一次性把整个 `CSpace` 全量迁成 `atmo` 模式。

### 13.2 必须满足的要求

最终提交与展示代码，建议至少满足下面这些硬要求：

- kernel / crate 构建可通过
- `cargo xtask verify --package sel4_cspace ...` 验证可通过
- 论文口径与代码口径一致：
  - 只主张 `CSpace` 局部验证
  - 不把 refined wrapper 写成 fully verified body
  - 不隐去 trusted boundary
- 已证范围、未证范围与 TCB 在文档中有一一对应台账
- 最终展示的“已证子集”明确列出，而不是临场口头解释

### 13.3 建议作为最终展示子集的最低范围

如果以“稳妥完成毕设展示”为目标，建议把最终展示子集固定为：

- `same_region_as`
- `same_object_as`
- `is_cap_revocable`
- `derive_cap`
- `ensure_no_children`

原因是：

- 前三项已经最接近“真实 Verus 接口 + 对外可调用接口”重合
- `derive_cap` 与 `ensure_no_children` 能体现 `CSpace` 局部语义闭环
- 这五项合起来已经足以支撑“局部 capability / derivation / no-children 主线”这一毕设展示主题

### 13.4 加强完成线

如果时间允许，建议把下面这些点作为加强版目标：

- `is_final_cap` 也进入最终展示子集
- `resolve_address_bits` 作为 lookup 代表进入最终展示子集
- 进一步减少“运行时导出接口”和“proof-only wrapper 入口”之间的落差
- 让最终展示代码里至少出现一条“查询 + 派生 + lookup”完整局部主线

### 13.5 理想完成线

如果希望最终展示代码更接近 `atmo` 风格，可把下面这些作为理想目标，而不是最低要求：

- 从 `cte_insert / derive_cap / ensure_no_children` 中至少挑 1 个入口，继续把 opaque body 收缩到更直接的 Verus body
- 继续下沉 `trusted_range_top_u128_if_small`、concrete extractor、slot-level observer
- 在最终展示时给出一张“参与构建接口 / 已验证接口 / 差距项”的对照表

### 13.6 当前不必强求的内容

为了保证毕设范围稳定，下面这些内容不应被误设为必须完成：

- 不必把整个 `CSpace` 全量改造成 `atmo` 式 verified runtime core
- 不必在本轮内证明完整 delete / revoke / finalise 主路径
- 接受 TCB 的必要性，但要求它显式、局部、稳定
- 不必把整个 kernel 构建链都改造成直接由 Verus 驱动

### 13.7 推荐验收口径

如果最终代码满足下面这组条件，就已经是一份相当扎实的毕设展示版本：

1. 选定 `CSpace` 子集可以稳定构建与验证
2. capability query 三项作为真实 Verus 接口保留下来
3. `derive_cap` 与 `ensure_no_children` 至少保持 refinement 闭环和清楚的论文口径
4. 文档能明确回答：
   - 哪些代码参与构建
   - 哪些接口已经证明
   - 哪些仍属于 refined wrapper / TCB
5. 论文与答辩中可以诚实说明：
   - 当前成果是局部验证闭环
   - 而不是整个 Rust 版 seL4 的完整验证

### 13.8 推荐实现顺序

为了让“最终展示代码”和“当前已证代码”尽量靠拢，同时避免过早把范围拉到 delete/revoke/finalise 全主线，后续工程实现建议按下面顺序推进。

当前执行状态（2026-04-29 当前轮次）：

- 第 1 步已经完成文档冻结；
- 第 2 步到第 9 步已经完成当前轮次目标：
  - `cte.rs` 中仅供 proof backend 使用的 `*_refined` 入口已收回为 `pub(crate)` 或模块内私有函数；
  - capability query 三项在 `cte.rs` 中对应的 backend 已收缩为 `pub(crate)`，稳定 verify-facing 接口继续只保留在 `capability::*`；
  - `interface.rs` 已补出稳定的 verify-facing facade：
    - `is_final_cap_at`
    - `ensure_no_children_at`
    - `is_long_running_delete_at`
    - `derive_cap_at`
    - `is_mdb_parent_of_at`
    - `resolve_address_bits_at`
    - `cte_insert_at`
    - `insert_new_cap_at`
    - `cte_move_at`
    - `cte_swap_at`
  - 第 3 轮起步后，最终展示子集又进一步收口了一层：
    - capability query 三项现也可统一经 `interface.rs` 导出；
    - `ensure_no_children_at` 的后置条件已直接刻画返回 status 与 `ensure_no_children_blocks(slot)` 的对应关系；
    - `derive_cap_at` 的后置条件已直接刻画返回 status 与返回 capability 对应的抽象结果；
    - `is_final_cap_at` / `is_long_running_delete_at` / `is_mdb_parent_of_at` 的后置条件已直接刻画返回布尔值对应的抽象语义；
    - `resolve_address_bits_at` 的后置条件已直接刻画返回 core、状态、slot 与剩余 bits；
    - 外部展示代码不再需要直接依赖 bridge 层 `trusted_*` 名字来解释这两个入口的结果。
  - `boundary_assumptions.rs` 已补入 delete 主线的显式边界台账：
    - `assume_reduce_zombie_local_progress`
    - `assume_delete_all_local_flow`
    - `assume_revoke_loop_flow`
  - `refinement_bridge.rs` 中仅供 crate 内部使用的一批 trusted constructor / extractor / bridge helper 已进一步收回为 `pub(crate)`；
  - `tools/check-cspace-build-and-verify.sh` 已补出，用于把普通构建与 Verus 校验放在同一条固定检查路径上；
- `bash tools/check-cspace-build-and-verify.sh` 于 2026-05-01 通过，其中 Verus 结果更新为 `249 verified, 0 errors`。

这意味着：

- 第 1 步到第 9 步当前轮次都已经可以视为完成；
- 当前不仅完成了 proof surface shrink，也补出了公共 verify-facing 接口层、delete 主线的局部 gate、显式边界假设与 build+verify 联合检查入口；
- 下一轮技术主线不再是“把第 8、9 步补齐”，而应转向更深入的 opaque body 验证化替换、delete/revoke/finalise full local proof，或继续收紧 remaining TCB。

#### 第 1 步：冻结最小展示子集

先固定最终最小展示范围为：

- `same_region_as`
- `same_object_as`
- `is_cap_revocable`
- `derive_cap`
- `ensure_no_children`

原因是：

- capability query 三项已经是最接近“真实运行时接口 = 验证接口”的部分；
- `derive_cap` 与 `ensure_no_children` 能把 capability 派生和 no-children 主线接起来；
- 这五项已经足以支撑论文与答辩中的局部闭环展示。

完成标准：

- 后续每一项工程推进，都能明确判断自己是在加强这五项中的哪一项，还是在为其服务；
- 不再把 delete/revoke/finalise 全链路误当成当前必须同时完成的最低目标。

#### 第 2 步：优先推进 `ensure_no_children`

对应代码位置：

- `sel4_cspace/src/cte.rs::ensure_no_children_refined`

为什么先做它：

- 它是 slot-local 语义，范围小；
- 当前已经能通过 `is_mdb_parent_of` 路径重建语义；
- 很适合作为“从 refined proof 入口继续向真实接口靠近”的第一站。

建议动作：

- 继续减少它对外部 helper 的暴露；
- 尽量把 proof-only 过渡函数继续收回私有；
- 明确区分“真实入口 contract”与“内部证明后端”。

完成标准：

- `ensure_no_children` 的工程组织更接近“真实接口 + 私有 proof backend”；
- 对外不再继续扩散新的 proof-only 表面。

#### 第 3 步：再推进 `derive_cap`

对应代码位置：

- `sel4_cspace/src/cte.rs::derive_cap_refined`

为什么紧跟在 `ensure_no_children` 后面：

- `derive_cap` 本身依赖 `ensure_no_children` 的局部语义；
- 这一步完成后，可以形成一条完整的 capability derivation 展示主线；
- 它本来就属于最低展示子集，因此收益很高。

建议动作：

- 继续把 `derive_cap` 的 case split 保持为 l4v 可解释的形态；
- 收缩返回值构造和 capability clone/null helper 周边的 trusted 使用面；
- 让“派生成功/失败”的返回值合同更直接贴近真实接口。

完成标准：

- `derive_cap` 与 `ensure_no_children` 之间形成一条清楚的局部 refinement 主线；
- 论文中可以把这两项一起作为 capability 派生闭环展示。

#### 第 4 步：补强 `is_final_cap`

为什么排在这一步：

- 它不是最低展示子集的必需项，但它能把 `same_object_as`、MDB 关系与删除前语义串起来；
- 后续如果要碰 delete/finalise，它是更自然的桥头堡。

建议动作：

- 继续保持 `mdb_prev/mdb_next + same_object_as` 的重建结构；
- 不把它过早扩展成完整 delete/finalise 证明，而是先把局部判定语义做硬。

完成标准：

- `is_final_cap` 可以稳定作为 `ensure_no_children` 之后的加强项；
- 删除链路的后续扩展有了一个干净的局部入口。

#### 第 5 步：推进 `resolve_address_bits`

对应代码位置：

- `sel4_cspace/src/cte.rs::resolve_address_bits_refined`

为什么现在做：

- 前四步主要是 capability/query/no-children 主线；
- `resolve_address_bits` 可以补上 lookup 维度，让最终展示不只停在 capability 层；
- 当前 bridge skeleton 已经比较稳定，适合在这个时点继续收紧。

建议动作：

- 继续保持“one-step skeleton + abstract post + bridge 推导”的结构；
- 尽量不再把高层语义重新塞回 bridge；
- 让 lookup 代表项足够稳定，成为最终展示中的第三条主线。

完成标准：

- 最终展示代码中至少具备一条“查询 + 派生 + lookup”的局部组合叙事；
- `resolve_address_bits` 继续维持 l4v 可解释的语义分解方式。

#### 第 6 步：在 mutating primitive 中先挑 `cte_insert`

对应代码位置：

- `sel4_cspace/src/cte.rs::cte_insert_refined`

为什么先选它：

- 它最适合充当 mutating primitive 的模板；
- 一旦 `cte_insert` 的 proof/backend/heap transition 组织更稳，`insert_new_cap`、`cte_move`、`cte_swap` 都更容易按同一路线推进。

建议动作：

- 继续压缩 ghost 参数和 proof-only 暴露面；
- 保持 `exec_step + abstract post + bridge lemma` 的统一证明结构；
- 优先解决“接口看起来不像最终运行时接口”的组织问题，而不是一口气追求全量 fully verified body。

完成标准：

- `cte_insert` 成为后续 mutating primitive 的模板项；
- 后续三个原语可以在不改证明主线的前提下平推。

#### 第 7 步：平推 `insert_new_cap / cte_move / cte_swap`

为什么放在 `cte_insert` 之后：

- 这三项现在都属于典型的 refined wrapper over opaque body；
- 如果没有一个先行模板，直接并行推进会让接口风格和证明组织继续发散。

建议动作：

- 复用 `cte_insert` 已经固定下来的证明骨架；
- 尽量统一 pre/post、frame、heap transition、bridge 引理的命名与组织；
- 不再为单个函数临时发明新的证明层次。

完成标准：

- 四个 mutating primitive 采用尽量一致的工程风格；
- 文档中可以把它们整体描述为同一 proof family，而不是四段彼此分散的特殊处理。

#### 第 8 步：最后再进入 delete/revoke/finalise 主线

为什么要放到后面：

- 这是当前剩余工作中范围最大、最容易把边界打散的一段；
- 在 capability/query/lookup/mutating template 还没完全稳住前，不适合先碰它。

建议动作：

- 从局部判定和局部辅助语义往前扩；
- 不要一开始就试图完整证明 `delete_all` 或整条 revoke/finalise 链路；
- 先把能复用的局部 invariant 和前置判定入口整理好。

完成标准：

- delete/revoke/finalise 不再只是“未覆盖的大块空白”，而是被拆成可逐段推进的局部任务；
- 不破坏当前最小展示子集和论文口径的稳定性。

当前轮次完成情况：

- 已完成当前轮次目标；
- `interface.rs` 已补出 delete-gate verify-facing 入口：
  - `is_mdb_parent_of_at`
  - `is_long_running_delete_at`
- `boundary_assumptions.rs` 已补出对 `reduce_zombie / delete_all / revoke` 的显式边界台账；
- 但 `delete_all / revoke / finalise` 本体仍未成为 full local proof，这仍属于下一轮深化任务。

#### 第 9 步：最后一轮收紧并固化 TCB，与构建对齐

主要对象：

- `refinement_bridge.rs` 中剩余的 `trusted_view_*`
- `trusted_extract_*`
- `trusted_concrete_*`
- `trusted_range_top_u128_if_small`

为什么放在最后：

- 这一步很重要，但它更像“收紧并固化可信边界”和“让最终展示代码更漂亮”；
- 它不应该阻塞 capability/query/lookup/mutating 主线先稳定下来。

建议动作：

- 把 trusted 项继续拆小、局部化；
- 明确哪些项是短期内保留的 TCB，哪些项是下一轮可继续局部化的对象；
- 同时整理“参与构建接口 / 已验证接口 / 仍属 trusted/opaque 的接口”对照表。

完成标准：

- remaining TCB 有清楚、稳定、可解释的名单；
- build object 与 verified object 的差距得到进一步压缩；
- 最终展示代码能够诚实而明确地说明哪些部分已经真正对齐。

当前轮次完成情况：

- 已完成当前轮次目标；
- `refinement_bridge.rs` 中一批仅供 crate 内部使用的 trusted constructor / extractor / bridge helper 已收回为 `pub(crate)`；
- `tools/check-cspace-build-and-verify.sh` 已把普通构建与官方 Verus 校验固化到同一条联合检查路径中；
- 但 build path 与 verify path 在工具链层仍然分离，remaining TCB 仍需要在后续工作中继续收紧与固化说明。

#### 总体原则

整个顺序遵循三条原则：

- 先做只读或局部返回值型接口，再做 lookup，再做一个 mutating 模板；
- 先把最接近最终展示代码的那部分做硬，再处理范围更大的删除主线；
- 始终区分“真实运行时接口已证”“refined wrapper 已证”“remaining TCB”三类完成度，不混写。

## 14. 历史文档

历史计划、阶段记录与收口报告已移到：

- `docs/verification/archive/`

日常维护时，只以本文件和 `cspace-session-log.md` 为准。
