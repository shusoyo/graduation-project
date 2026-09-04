# CSpace 面向 l4v 语义对齐的重构计划

状态（2026-04-24）：

- 当前 `sel4_cspace` 已形成一条可运行的 Verus 验证闭环。
- `cargo xtask verify` 当前通过，输出为 `136 verified, 0 errors`。
- 但从论文口径看，当前模型更接近“可验证的工程化子模型”，还不等于“直接沿用 l4v 语义基线”。
- 本计划的目标，是把后续主线从“证明当前抽象模型内部自洽”调整为“尽量直接复用 l4v 的语义分解，并在 Verus/Rust 中建立 refinement”。

## 1. 目标

这次重构的目标不是重新发明一套更复杂的本地语义，而是把 `sel4_cspace` 的验证路线改造成下面这条叙事：

- `aux/l4v` 是语义来源；
- `sel4_cspace/specs` 是对 l4v 语义的 Verus 化重建；
- `sel4_cspace/src/refinement_bridge.rs` 只负责 concrete 到 abstract 的表示映射；
- `sel4_cspace/src/cte.rs` 等具体实现逐步证明满足这些 l4v 风格 spec；
- 论文里证明的重点是“Rust/Verus 实现细化到 l4v 语义”，而不是“实现满足一套项目自定义语义”。

## 1.1 本轮固定参考

本轮语义对齐默认以以下 l4v 文件为 primary source：

- `aux/l4v-master/spec/haskell/src/SEL4/Object/ObjectType.lhs`
- `aux/l4v-master/spec/haskell/src/SEL4/Object/CNode.lhs`
- `aux/l4v-master/spec/haskell/src/SEL4/Kernel/CSpace.lhs`

Verus/Rust 侧的主要落点文件固定为：

- `sel4_cspace/specs/abstract_cspace.rs`
- `sel4_cspace/specs/cspace_ops/common.rs`
- `sel4_cspace/specs/cspace_ops/derive.rs`
- `sel4_cspace/specs/cspace_ops/insert.rs`
- `sel4_cspace/specs/cspace_ops/move.rs`
- `sel4_cspace/specs/cspace_ops/swap.rs`
- `sel4_cspace/specs/cspace_ops/resolve.rs`
- `sel4_cspace/src/refinement_bridge.rs`
- `sel4_cspace/src/cte.rs`

如果后续需要引入新的 l4v 参考文件，应在文档里追加，而不是在代码里隐式扩散。

## 2. 当前状态与主要差距

当前仓库已经具备三个很好的基础：

- 已有稳定的抽象状态层：`CapSpec / SlotEntrySpec / CSpaceState / wf`。
- 已有一套可运行的 refinement bridge 和 exec 合同入口。
- 已有 `cte_insert / cte_move / cte_swap / resolve_address_bits / derive_cap` 等原语的初始证明闭环。

但距离“论文里可直接声称遵循 l4v 证明思路”还有几个关键差距。

### 2.1 基础等价关系仍然是压缩版，不是 l4v 原义

当前最关键的问题不在 proof skeleton，而在基础语义层：

- `sameRegionAs`
- `sameObjectAs`
- `isCapRevocable`
- `isMDBParentOf`

目前这些判断在抽象层和 bridge 层中仍带有“压缩表示”痕迹，例如：

- 用单一 `region_id` 近似多种 cap 的区域关系；
- 用折叠后的 `CapKind` 表示多类 concrete cap；
- 在 bridge 中顺带完成一部分语义裁剪。

这会让后续 `isFinalCap / ensureNoChildren / deriveCap / cte*` 的证明语义也跟着偏离 l4v 原义。

### 2.2 bridge 现在不只是表示映射，还承担了语义折叠

理想状态下，bridge 只应回答：

- 这个 concrete `cap` 在抽象层看起来是什么；
- 这个 concrete `cte_t` 在抽象层看起来是什么；
- 这个 concrete heap 与抽象 `CSpaceState` 是否对应；

而不应回答：

- 哪些 cap 种类被折叠到同一个抽象种类；
- 哪些 l4v 语义分支暂时被抹平；
- 哪些等价关系在抽象层被弱化；

否则论文叙事会变成“我们先自己改了语义，再证明实现满足它”。

### 2.3 覆盖范围边界还不够显式

当前 proof 实际上对 cap coverage 是有范围限制的，但这层限制还不够显式地体现在：

- spec 层的语义适用范围；
- bridge 的 trusted boundary；
- 文档里的“已覆盖 / 未覆盖 / 暂不进入证明入口”的清单。

这在论文里会造成两个问题：

- 不容易清楚陈述“当前结果到底继承了 l4v 的哪些部分”；
- 也不容易诚实地区分“语义一致”和“当前范围内一致”。

### 2.4 `deriveCap` 仍处于 generic 子集阶段

`deriveCap` 的 generic 非 arch 路径已经有闭环，但还不是完整 l4v 结构：

- generic 路径与 arch 路径尚未形成同一套语义框架；
- bridge 还没有提供足够精确的 arch 语义载体；
- 文档上也还没有把“generic 对齐完成”和“full l4v 对齐完成”明确拆开。

## 3. 重构原则

这轮重构遵循下面五条原则。

### 3.1 语义来源优先对齐 l4v

优先对齐的不是 proof script，而是语义定义与 case split：

- `sameRegionAs`
- `sameObjectAs`
- `deriveCap`
- `isCapRevocable`
- `isMDBParentOf`
- `ensureNoChildren`
- `isFinalCapability`
- `resolveAddressBits`

只要 l4v 已经给出清楚定义，Verus 侧默认先翻译这些定义，而不是重新设计等价替代物。

### 3.2 bridge 只做表示映射，不再做语义重写

`refinement_bridge.rs` 的职责应收敛为：

- raw 类型到 spec 类型的 view；
- concrete heap 到抽象状态的对应关系；
- local transition 的对接引理；

不再继续在 bridge 里完成：

- cap 种类折叠；
- 区域关系重解释；
- 业务语义的“简化版重建”。

### 3.3 明确区分“语义未覆盖”和“暂时 trusted”

若某些 cap 类别暂时不进入第一轮证明，应明确落在下列二选一中：

- 暂不覆盖：不进入本轮 verified 入口；
- 暂时 trusted：进入入口，但以显式假设承接，并记录收缩方向；

不能再使用“在 bridge 中悄悄压平”这种中间状态。

### 3.4 generic 与 arch 分层保持和 l4v 相近

建议保持以下结构：

- 先定义 generic capability 语义；
- 再为 arch capability 留出明确 hook；
- bridge 负责把 concrete arch cap 映射到这些 hook 需要的抽象信息；
- proof 文档明确区分 generic 完成度与 arch 完成度。

### 3.5 proof 工程继续保持 Verus-native 风格

语义来源跟 l4v，但证明工程不回退到 Isabelle 风格。

继续保留当前已经比较合适的工程选择：

- spec-first；
- relational contracts；
- packaging lemma；
- `vostd` 风格的小合同、小 helper；
- `atmo` 风格的可信边界隔离；
- 主函数长期目标仍然是签名式 `requires/ensures`。

## 4. 分阶段执行计划

为了保证每一步都能形成稳定中间结果，这轮重构按 7 个阶段推进。

### 阶段 R1：固定语义基线与覆盖范围

目标：

- 把本轮对齐的 l4v 参考文件固定下来；
- 把“当前必须跟 l4v 对齐的语义项”列成清单；
- 把“暂不覆盖的 cap 种类 / arch 分支 / 例外路径”明确写成范围边界。

主要工作：

- 在文档中固定语义来源文件；
- 把 generic 与 arch 分别列出；
- 为 bridge 和 spec 层补一份“当前覆盖范围”说明。

产出：

- 本文档；
- 后续阶段中引用的一份 coverage/tcb 口径。

完成判定：

- 任何一个 spec helper 都能说清楚它对应 l4v 的哪一段语义；
- 任何一个未覆盖分支都能在文档中定位。

### 阶段 R2：重建抽象 capability 语义层

目标：

- 让 `specs/abstract_cspace.rs` 中最核心的 capability 关系尽量直接对应 l4v。

优先重构项：

- `sameRegionAs`
- `sameObjectAs`
- `isPhysicalCap` 或等价辅助概念
- `isCapRevocable`
- `mdb_parent_badge_compatible`
- `mdb_parent_of`

主要工作：

- 扩充 `CapSpec`，让它携带实现 l4v 语义所必需的信息；
- 把当前基于单一 `region_id` 的压缩表示拆开；
- 明确 generic cap 与 arch cap 的关系入口。

产出：

- 一版更贴近 l4v 的 `CapSpec`；
- 一组直接对应 l4v 的基础 helper 和引理。

完成判定：

- `isFinalCap / ensureNoChildren` 可以直接建立在这些 l4v 风格 helper 上；
- 不再需要通过 bridge 的额外折叠才能表达这些关系。

### 阶段 R3：收缩并重写 bridge 表示层

目标：

- 让 `refinement_bridge.rs` 回到“表示映射层”。

主要工作：

- 调整 `trusted_extract_cap` / `view_cap` / `trusted_view_cap` 所需字段；
- 去掉不必要的 cap tag 折叠；
- 把 unsupported/opaque 情况移出语义层，转成显式边界；
- 保持 `trusted_cspace_heap_matches_state_at` 这类关系词汇，但不让它们定义业务语义。

产出：

- 一版更窄的 trusted bridge surface；
- 一版更明确的 `trusted_view_*` 语义边界。

完成判定：

- bridge 本身不再解释 `sameRegionAs / sameObjectAs / isCapRevocable`；
- bridge 只负责把 concrete 信息送到 spec 可消费的形状。

### 阶段 R4：重构原语规格，使其直接消费 l4v 基础语义

目标：

- 让 `derive / insert / move / swap / resolve` 等 spec 文件消费新的 capability 语义。

主要工作：

- 用新的 `sameRegionAs / sameObjectAs / isCapRevocable` 重写相关 pre/post；
- 检查 `spec_cte_insert_derivable`、`spec_cte_move_cap_compatible` 等 helper 是否仍然只是“工程近似”；
- 补齐必要的 l4v case split；
- 保持 smoke check 可跑通。

产出：

- 一版与 l4v 更一致的原语 spec；
- 一批新的 packaging lemma。

完成判定：

- `deriveCap` 的 generic 语义可直接对齐 l4v；
- `cte_insert / cte_move / cte_swap / resolve_address_bits` 的合同不再依赖压缩语义。

### 阶段 R5：重新验证 concrete helper 与 refined 入口

目标：

- 检查 `src/cte.rs` 当前实现到底是“本来就和 l4v 一致”，还是“需要修正代码 / 收窄入口范围 / 增加 trusted boundary”。

主要工作：

- 逐项比对 concrete `same_region_as / same_object_as / is_cap_revocable / derive_cap`；
- 对照 `l4v` 标出一致项、缺口项、临时约束项；
- 更新 `*_exec_contract` 与 `*_refined(...)` 的逻辑前提。

产出：

- 一版经过重对齐的 refined 入口；
- 一份局部差距清单。

完成判定：

- 每个 refined 入口都能说清楚“它证明的是 l4v 哪条定义”；
- 不再把“内部自洽”误写成“语义已对齐”。

### 阶段 R6：继续把 proof 推回 exec 本体

目标：

- 在新的 l4v 语义基线下，继续推进 `cte.rs` 的签名式本体化。

主要工作：

- 优先从 `resolve_address_bits` 开始，因为它更接近 l4v 控制流；
- 再处理 `cte_insert / cte_move / cte_swap`；
- 最后整理 `derive_cap`。

产出：

- 更少的 wrapper；
- 更小的 trusted util；
- 更贴近函数 body 的 proof 结构。

完成判定：

- 主函数的关键语义由本体合同直接导出；
- `refinement_bridge.rs` 更像 supporting layer，而不是主证明入口。

### 阶段 R7：论文口径收口

目标：

- 把最终结果整理成适合论文叙述的成果边界。

主要工作：

- 列出已对齐的 l4v 语义项；
- 列出当前仍未覆盖的路径；
- 列出最小 trusted surface；
- 列出 generic/arch 两条主线的完成度。

产出：

- 收口文档；
- TCB 台账；
- 已证/未证清单；
- 适合论文正文和附录引用的术语与边界说明。

完成判定：

- 可以清晰陈述“我们复用了 l4v 的哪些语义、在哪些位置用 Verus 重建并证明 refinement”。

## 5. 推荐执行顺序

为了尽快得到论文上最关键的收益，建议执行顺序如下：

1. 先做 `R2` 的 generic capability 语义重建。
2. 再做 `R3` 的 bridge 收缩。
3. 接着做 `R4`，把 spec 文件接到新的基础语义上。
4. 然后做 `R5`，重新审视 concrete helper 与 refined 入口。
5. 最后再继续 `R6` 的本体化 proof 推进。

理由：

- 如果不先修 capability 基础语义，后面所有原语 proof 都会建立在压缩模型上；
- 如果不先收 bridge，论文叙事仍然会显得像“本地模型自证”；
- 只有先把 `R2-R5` 收住，`R6` 推回函数本体才真正有论文价值。

## 6. 每阶段的验收命令

每完成一个阶段，都至少保留以下回归：

- `cargo xtask verify`

在发生较大接口调整时，再补：

- `cargo check -p sel4_cspace`

如果阶段只改文档，不强制要求运行回归；但一旦进入 `specs/` 或 `src/` 改动，就必须保持验证结果可回归。

## 7. 当前起点

截至 2026-04-24，建议把当前仓库视为：

- 原“6 步 exec 本体化计划”已经形成第一轮工程闭环；
- 但为了毕业设计/论文的语义口径，现在进入第二轮主线：
  - 从工程可验证闭环，转向 l4v 语义对齐闭环。

这意味着接下来的第一步，不是继续堆新 proof，而是优先做：

- `R2`：重建 `sameRegionAs / sameObjectAs / isCapRevocable` 这层 capability 语义基线。

## 8. 第一批执行项

为了尽快进入可验证的重构节奏，第一批任务建议按下面顺序执行：

1. 在 `abstract_cspace.rs` 中重构 capability 语义表示。
2. 先把 `sameRegionAs / sameObjectAs / isCapRevocable / mdb_parent_of` 改成更贴近 l4v 的定义。
3. 再收缩 `refinement_bridge.rs` 中的 cap kind 折叠与 `region_id` 近似。
4. 然后重写 `derive.rs` 与 `common.rs` 中直接依赖这些 helper 的规格入口。
5. 每做完一小批，就跑一次 `cargo xtask verify`，防止语义层和 proof 层同时漂移。

第一批的阶段目标不是一次性覆盖全部 arch 语义，而是先把 generic capability 语义基线对齐并稳定下来。
