# CSpace 局部验证计划书（论文范围版）

状态：草案，2026-04-24

## 1. 项目定位

本课题当前不追求复刻整个 seL4/l4v 的全系统证明，而是把目标收敛为：

- 只证明 `sel4_cspace` 这一子系统本身的语义与局部不变量；
- 底层指针、位域、FFI、非 `CSpace` 子系统先作为 TCB；
- 对已选定的 `CSpace` 子集，语义尽量严格对齐 `aux/l4v`；
- 表示方式、模块分层与证明工程采用 Verus-native 设计。

论文中希望表达的核心命题是：

`在把底层实现细节与非 CSpace 子系统视为可信边界的前提下，对 Rust 重写中的 CSpace 子集建立与 l4v 一致的局部语义、局部不变量与 refinement 证明。`

## 2. 范围与非目标

### 2.1 本轮证明范围

本轮只覆盖 `CSpace` 子系统内部、且已经进入当前抽象模型的那部分语义，优先包括：

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
- 与删除链路紧密相关、但可单独分批推进的局部判断：
  - `ensureNoChildren`
  - `isFinalCapability`

### 2.2 当前明确不证明的内容

下列内容不属于本轮主证明目标：

- 全局 kernel state 的完整一致性；
- 调度、地址空间、IPC、对象创建/销毁的整系统语义；
- 所有 arch-specific capability 细节；
- 底层内存布局、指针合法性、bitfield 生成代码本身；
- `deps`/FFI 里的外部实现。

这些内容要么作为后续工作，要么明确进入 TCB。

### 2.3 TCB 边界

本轮固定的 TCB 主要包括：

- `sel4_common` 中 generated bitfield/getter 的语义；
- 指针转换与底层内存读写；
- `deps` 中的 FFI；
- 尚未纳入本轮抽象模型的 arch 细节；
- 非 `CSpace` 子系统提供给 `CSpace` 的外部前提。

原则是：

- 不在 bridge 中“悄悄证明”这些边界；
- 不把它们伪装成已验证结论；
- 每一项都要能在文档中点名。

## 3. 语义来源与方法论

### 3.1 l4v 侧的参考基线

本轮“证什么、怎么证”的 primary source 固定为 `aux/l4v` 中的 CSpace 相关定义与证明分层。

语义定义侧主要参考：

- `aux/l4v-master/spec/haskell/src/SEL4/Object/ObjectType.lhs`
- `aux/l4v-master/spec/haskell/src/SEL4/Object/CNode.lhs`
- `aux/l4v-master/spec/haskell/src/SEL4/Kernel/CSpace.lhs`

证明分层与证明义务侧主要参考：

- `aux/l4v-master/proof/invariant-abstract/CSpaceInv_AI.thy`
- `aux/l4v-master/proof/refine/CSpace1_R.thy`
- `aux/l4v-master/proof/refine/RAB_FN.thy`
- `aux/l4v-master/proof/refine/Untyped_R.thy`
- `aux/l4v-master/proof/refine/ARM_HYP/Finalise_R.thy`

这里的原则不是照搬 Isabelle proof script，而是复用它的证明分解方式：

- 先定义抽象语义；
- 再陈述局部不变量；
- 再证明原语保持这些不变量；
- 最后把具体实现细化到抽象语义。

### 3.2 Verus 侧的工程原则

Verus 代码继续采用本项目已经形成的风格：

- 语义来源对齐 l4v；
- 抽象模型与原语合同放在 `sel4_cspace/specs`；
- concrete 到 abstract 的表示映射放在 bridge；
- 具体函数证明尽量回到 `sel4_cspace/src/cte.rs` 的函数本体；
- 长期目标是签名式 `requires/ensures`，而不是长期依赖大块 wrapper。

一句话概括就是：

`l4v 决定证明内容与语义基线，Verus 决定实现形态与证明工程。`

## 4. CSpace 本身需要证明什么

如果只证明 `CSpace`，并且希望论文口径尽量与 l4v 一致，那么不能只停在“给几个函数加 `requires/ensures`”。更合理的证明目标应分成四层。

### 4.1 第一层：Capability 与 CTE 的基础语义

需要固定并证明可复用的基础判断，包括：

- 哪些 cap 被视为指向同一区域：`sameRegionAs`
- 哪些 cap 被视为指向同一对象：`sameObjectAs`
- 新 cap 是否可撤销地派生自旧 cap：`isCapRevocable`
- MDB 父子关系如何判定：`isMDBParentOf`
- `deriveCap` 的语义分支是什么

这一层决定后面几乎所有删除、派生、插入相关规则的含义，是整个 CSpace 证明的语义底座。

### 4.2 第二层：CSpace 局部不变量

需要定义并维护只依赖 CSpace 本身的局部不变量，而不是整个 kernel 的全局不变量。当前适合纳入的包括：

- slot/cte 的基本 well-formedness；
- `cnode_lookup` / `resolveAddressBits` 依赖的 lookup 一致性；
- MDB 链的局部结构约束；
- 与 final/no-children/parent-of 相关的局部一致性。

这一步的目标不是复刻整个 `invs`，而是提炼出“只证明 CSpace 也必须拥有”的那一层。

### 4.3 第三层：原语保持局部不变量

对 `CSpace` 修改原语，不仅要给出功能合同，还要证明它们在前置条件下保持局部不变量。核心原语包括：

- `cteInsert`
- `cteMove`
- `cteSwap`
- 后续扩展时的 `ensureNoChildren`、删除与 finalise 路径

这一步对应 l4v 里“操作正确”更实质的含义：不只是结果值对，还包括状态更新后 CSpace 仍然保持合法。

### 4.4 第四层：Rust 实现对抽象语义的 refinement

在抽象语义与局部不变量稳定后，再证明 Rust 代码中的具体函数满足对应 spec：

- concrete `cap/cte/ret` 能映射到抽象模型；
- `cte.rs` 中的具体函数满足抽象 pre/post；
- 需要时，通过 bridge 承接底层 TCB 与抽象层之间的表示差异。

因此，本课题中的“验证 CSpace”至少意味着：

1. 定义 CSpace 局部语义；
2. 定义 CSpace 局部不变量；
3. 证明核心原语保持这些不变量；
4. 证明 Rust 实现细化到这套抽象语义。

## 5. 当前实现的基础与差距

### 5.1 已有基础

当前仓库已经形成一条可运行的第一轮验证闭环，主要基础包括：

- `sel4_cspace/specs/abstract_cspace.rs`
  - 已有 `CapSpec`、`SlotEntrySpec`、`CSpaceState` 与 `wf`
- `sel4_cspace/specs/cspace_ops/*.rs`
  - 已有 `derive_cap`、`cte_insert`、`cte_move`、`cte_swap`、`resolve_address_bits` 等原语规格
- `sel4_cspace/src/refinement_bridge.rs`
  - 已有 concrete 到 abstract 的桥接层
- `sel4_cspace/src/cte.rs`
  - 已有一批 refined proof 入口

也就是说，工程上已经不是“从零开始”，而是已经完成了第一轮可验证骨架。

### 5.2 主要差距

但距离“论文里可以有把握地说，本工作沿用了 l4v 的 CSpace 证明路线”还有几处关键差距：

- 抽象 capability 语义仍有压缩建模痕迹；
- `refinement_bridge.rs` 里仍掺杂了部分语义折叠，而不只是表示映射；
- 局部不变量目前更多体现为工程化 `wf`，还没有完全重组为更接近 l4v 叙事的 invariant 层；
- `deriveCap` 与部分 final/no-children 相关语义还停留在子集级完成度；
- 具体函数证明仍较多依赖 refined wrapper，而不是“函数本体 + 小型 helper 合同”的终态风格。

所以，当前进度更准确的定位是：

- 已完成第一轮工程化验证闭环；
- 尚未完成“面向论文口径的 l4v 语义对齐与证明重组”。

## 6. 分阶段执行计划

### 阶段 P1：冻结论文范围与 TCB 台账

目标：

- 固定“本论文只证明 CSpace 局部性质”的边界；
- 固定 TCB 清单；
- 固定本轮 primary source 的 l4v 文件。

主要工作：

- 把已证明、暂不证明、作为 TCB 的部分写成清单；
- 明确 generic 与 arch 的覆盖边界；
- 固定后续所有 spec/helper 的语义出处。

完成标志：

- 任一 spec 都能回答“它对应 l4v 的哪段定义”；
- 任一未覆盖分支都能回答“它为什么还不在本轮证明里”。

### 阶段 P2：重构抽象 capability 语义

目标：

- 让 `abstract_cspace.rs` 中最关键的 capability 关系直接对齐 l4v。

主要工作：

- 重构 `sameRegionAs`；
- 重构 `sameObjectAs`；
- 重构 `isCapRevocable`；
- 重构 `mdb_parent_of`/`isMDBParentOf`；
- 为 `deriveCap` 的 generic 语义提供更直接的 l4v-style case split。

完成标志：

- bridge 不再需要替 capability 语义“补解释”；
- capability 基础判断本身就可以作为后续 proof 的直接依据。

### 阶段 P3：提炼 CSpace 局部不变量

目标：

- 从当前 `wf` 中抽出更贴近 l4v 叙事的 invariant 层。

主要工作：

- 区分基础 well-formedness 与 MDB/lookup/finality 层的不变量；
- 为 `ensureNoChildren`、`isFinalCapability`、`resolveAddressBits` 建立明确依赖的不变量入口；
- 把“大 conjunction”继续压成可复用 lemma。

完成标志：

- 对每个原语都能说清楚它依赖哪些局部不变量；
- “原语正确”不再只是 post-state 结果正确，而是能陈述 invariant preservation。

### 阶段 P4：把原语规格改写成 preservation-first 结构

目标：

- 把 `cteInsert`、`cteMove`、`cteSwap`、`resolveAddressBits` 的 spec 组织成更像 l4v 的证明义务。

主要工作：

- 明确功能合同；
- 明确 frame condition；
- 明确哪些局部不变量必须被保持；
- 对 `deriveCap`、`ensureNoChildren`、`isFinalCapability` 补足与主线一致的规格入口。

完成标志：

- 每个原语都能以“功能 + 保持不变量”的形式表述；
- smoke 或小引理能够直接覆盖这些入口。

### 阶段 P5：收缩 bridge，只保留表示映射职责

目标：

- 让 `refinement_bridge.rs` 回到“表示桥”而不是“语义层”的角色。

主要工作：

- 清理 bridge 中与 capability 语义本身有关的压缩逻辑；
- 保留 concrete `cap/cte/ret` 到抽象对象的 view；
- 保留 concrete heap 到抽象 `CSpaceState` 的对应关系；
- 把语义定义放回 `specs`，把 proof 主线放回 `src/cte.rs`。

完成标志：

- bridge 可以被描述为 representation refinement layer；
- 不再需要依赖 bridge 去解释“什么叫 parent-of / same-region / revocable”。

### 阶段 P6：逐个证明 Rust 函数满足抽象 spec

目标：

- 把证明真正压回 Rust 实现本体。

推荐顺序：

1. `resolve_address_bits`
2. `cte_insert`
3. `cte_move`
4. `cte_swap`
5. `derive_cap`
6. `ensure_no_children`
7. `is_final_cap`

主要工作：

- 缩小函数级 wrapper；
- 把证明从 bridge/refined entry 推回 `cte.rs` 的函数 body；
- 尽量把主函数写成更直接的 Verus `requires/ensures` 风格。

完成标志：

- 主要语义由函数本体自身导出；
- trusted util 只剩 object-local 或 bitfield/FFI 级别的最小边界。

### 阶段 P7：收口论文材料与最终台账

目标：

- 形成可直接写入论文的方法论与结果陈述。

主要工作：

- 产出“已覆盖 / 未覆盖 / TCB”清单；
- 产出“l4v 对应关系”清单；
- 产出“当前已证的 Rust 函数与其对应抽象语义”清单；
- 记录 remaining gaps 与可继续扩展方向。

完成标志：

- 可以明确回答：
  - 证明了哪些 CSpace 性质；
  - 这些性质为什么有 l4v 依据；
  - 哪些部分还未证明、为何未证明。

## 7. 预期交付物

本计划收口后，论文与仓库层面应至少有以下结果：

- 一套与 l4v 对齐的 CSpace 子集抽象语义；
- 一组只依赖 CSpace 本身的局部不变量；
- 对核心 CSpace 原语的 invariant-preservation 证明；
- Rust 具体实现到抽象语义的 refinement 证明；
- 一份清楚的 TCB/未覆盖项台账。

## 8. 成功判据

当下面四点都成立时，可以认为本轮计划达到目标：

1. 对选定的 CSpace 子集，抽象语义能够逐项对照到 l4v 的定义来源。
2. 原语证明不再只是“有 pre/post”，而是包含局部不变量保持。
3. Rust 代码的主要验证结论来自函数本体或其紧邻 helper，而不是大块语义 wrapper。
4. 论文中可以诚实、清楚地说明：
   - 本工作证明了什么；
   - 没有证明什么；
   - 为什么这种范围选择仍然是沿用 l4v 路线的、而不是随意自造的。

## 9. 与现有文档的关系

这份计划书是“论文范围版主计划”，与已有文档的关系如下：

- `docs/verification/cspace-verification-steps.md`
  - 保留工程执行轨迹与阶段收口记录
- `docs/verification/cspace-l4v-alignment-refactor-plan.md`
  - 保留面向代码重构的差距分析
- `docs/verification/cspace-stage5-bridge-design.md`
  - 保留 bridge 风格与分层约束

后续如果要继续推进实现，建议以本文档作为“总目标与范围约束”，以上三份文档作为“工程执行细则”。
