# `swap` Proof Refactor Plan

本文档只讨论 `sel4_cspace` 的 `swap` 证明组织，不直接改代码。

目标不是重新定义 `swap` 语义，而是回答：

- 现在的 `swap` 证明为什么这么重
- 这是不是 `Verus` 表达力不够造成的
- 在保持当前 exec 形状基本不变的前提下，怎样把证明收敛成更适合 `Verus` 的结构

本文档建立在当前仓库中的具体事实之上，而不是纯理论建议。

## Summary

结论先说：

- `Verus` 足够表达当前 `swap` 需要的抽象语义
- 当前困难的主因不是“`Verus` 做不到”
- 当前 `swap` 已经有一套相当完整的 final-state spec vocabulary
- 真正偏重的是：
  - `impl_swap.rs` 中以 `after_*_mgr` 为中心的 runtime trace proof
  - `spec_util/swap.rs` 中大量围绕局部 case split 的 proof lemma
- 因此，更可行的方向不是重写语义层，而是：
  - 保留现有 final-state spec
  - 强化一个中心 permutation 抽象
  - 让 `final_post / exact_post` 成为证明主轴
  - 把 trace proof 从“主结构”降级成“局部实现对接层”

也就是说，问题主要是 **proof organization**，不是 **semantic expressiveness**。

## 0. 当前代码事实

下面这些事实是本文方案成立的基础。

### 0.1 当前已经有 final-state spec，不需要从零开始设计

在 [spec_util/swap.rs](/workspace/rel4_kernel/sel4_cspace/src/cspace_manager/spec_util/swap.rs) 中，当前已经存在：

- `cte_swap_expected_slot1_entry`
- `cte_swap_expected_slot2_entry`
- `cte_swap_expected_neighbor_entry`
- `cte_swap_final_entry`
- `cte_swap_final_post`
- `cte_swap_cap_post`
- `cte_swap_derivation_post`
- `cte_swap_non_mdb_frame_post`
- `cte_swap_exact_post`

其中尤其重要的是：

- [cte_swap_final_entry](/workspace/rel4_kernel/sel4_cspace/src/cspace_manager/spec_util/swap.rs:700)
- [cte_swap_final_post](/workspace/rel4_kernel/sel4_cspace/src/cspace_manager/spec_util/swap.rs:755)
- [cte_swap_exact_post](/workspace/rel4_kernel/sel4_cspace/src/cspace_manager/spec_util/swap.rs:921)

这说明当前工程并不缺“最终状态语义层”；相反，它已经有了较强的 final-state vocabulary。

### 0.2 当前 proof 主线仍然由 trace-state 驱动

在 [impl_swap.rs](/workspace/rel4_kernel/sel4_cspace/src/cspace_manager/impl_swap.rs) 中，当前证明显式维护了多个中间 manager 状态：

- `after_take_slot1_mgr`
- `after_take_slot2_mgr`
- `after_prev1_mgr`
- `after_next1_mgr`
- `after_prev2_mgr`
- `after_next2_mgr`
- `after_put_slot1_mgr`
- `new_mgr`

这在：

- [lemma_cte_swap_exact_runtime_post](/workspace/rel4_kernel/sel4_cspace/src/cspace_manager/impl_swap.rs:473)
- [lemma_cte_swap_finish_after_runtime](/workspace/rel4_kernel/sel4_cspace/src/cspace_manager/impl_swap.rs:1577)
- 以及最终 `cte_swap` 方法本体 ([impl_swap.rs](/workspace/rel4_kernel/sel4_cspace/src/cspace_manager/impl_swap.rs:1755))

中都非常明显。

### 0.3 当前 `swap` 证明 surface 非常大

仅在 [spec_util/swap.rs](/workspace/rel4_kernel/sel4_cspace/src/cspace_manager/spec_util/swap.rs) 中，就已经有大量 `#[verifier::external_body]` proof lemma。

这本身并不自动代表错误，但至少说明：

- 当前 `swap` 的 proof debt 很大
- 当前写法已经在借助不少 trusted proof shell 来推动整体闭环

因此，任何可行重构方案都应以“减少 proof surface 和 trace coupling”为目标。

## 1. 当前 `swap` 的本质复杂度

`swap` 本身就比 `insert` / `move` 更复杂，因为它同时影响：

- `slot1` 和 `slot2` 的 capability 内容
- 两个 slot 自己的 `mdb_prev` / `mdb_next`
- 两侧邻接节点的 back-link / forward-link
- `cdt_parent`
- `is_original`
- 连带的 incoming-edge / zombie / resolve ghost 视图

因此，`swap` 不可能像一个“只写两个字段”的小证明那样轻。

这点和 `l4v` 是一致的。

在 [CSpace_A.thy](/workspace/rel4_kernel/sel4_cspace_backup/aux/l4v_cspace_extracted/spec/abstract/CSpace_A.thy:344) 中，`cap_swap` 的抽象定义本身就同时做了：

- `set_cap cap2 slot1`
- `set_cap cap1 slot2`
- `cdt` 交换更新
- `cap_swap_ext`
- `is_original` 更新

所以，“`swap` 本身复杂”是客观事实，不是当前实现独有的问题。

## 2. 现在为什么会卡

当前 `swap` 证明很重，主要不是因为 `spec` 错，而是因为把实现路径 replay 得太细。

### 2.1 当前证明的主负担

当前 [impl_swap.rs](/workspace/rel4_kernel/sel4_cspace/src/cspace_manager/impl_swap.rs) 里引入了大量中间态：

- `after_take_slot1_mgr`
- `after_take_slot2_mgr`
- `after_prev1_mgr`
- `after_next1_mgr`
- `after_prev2_mgr`
- `after_next2_mgr`
- `after_put_slot1_mgr`
- `new_mgr`

这些中间态本身并不是错误的，但它们带来了两个问题：

- prover 要同时维护很多 manager 状态之间的关系
- 大量 lemma 被迫背着长长的 `requires`

这会让 `Verus` / SMT 非常吃力。

### 2.2 当前 post 与 trace 绑定得过紧

当前 `swap` 的 final exact post 方向是对的，但它的证明方式太接近：

- 先 replay runtime patch trace
- 再从 trace 拼出 exact post
- 再从 exact post 逐层恢复 `wf`

这类风格在 Isabelle 中常常能靠现有规则和自动化顶住，但在 `Verus` 里成本更高。

需要特别指出的是：

- 这里的问题不是 `cte_swap_final_post` / `cte_swap_exact_post` 太强
- 而是 **这些 post 没有成为证明主轴**
- 当前主轴仍然是 “如何从 `after_*_mgr` 一步步 replay 到 final state”

### 2.3 重复 case split 太多

当前很多证明在不断展开：

- `slot == slot1`
- `slot == slot2`
- `slot == prev1`
- `slot == next1`
- `slot == prev2`
- `slot == next2`
- 以及 “其它 slot”

这种写法逻辑上没有错，但它把“交换本质是一个有限置换”的结构淹没掉了。

## 3. 问题是不是 `spec` 写坏了

不是。

当前 `swap spec` 的核心部分其实是合理的。

例如：

- `cte_swap_patch_slots`
- `cte_swap_normalize_internal_ref`
- `cte_swap_expected_slot1_entry`
- `cte_swap_expected_slot2_entry`
- `cte_swap_derivation_post`
- `cte_swap_final_post`
- `cte_swap_exact_post`

这些都在表达正确的问题：

- 哪些 slot 会被改
- slot 内部引用怎样归一化
- `slot1/slot2` 的最终状态是什么
- derivation 关系怎样交换
- final exact post 是什么

所以这里的问题不是“spec 完全写错了”，而是：

- spec 已经足够强
- final-state 层已经存在
- 但 proof 没有把 final-state 层作为中心接口
- 反而回头围着 runtime trace 展开了很多低层细节

换句话说：

- **semantic layer 大体合理**
- **proof layer 过于 operational**

## 4. `l4v` 为什么看起来没这么炸

`l4v` 不是没有复杂度，而是它用更好的中间抽象吸收了复杂度。

最关键的是 [CSpace_AI.thy](/workspace/rel4_kernel/sel4_cspace_backup/aux/l4v_cspace_extracted/proof/invariant-abstract/CSpace_AI.thy:2376) 里的：

- `s_d_swap`

它把“slot 引用交换”先抽象成一个纯函数：

- `src -> dest`
- `dest -> src`
- 其它值保持不变

然后围绕这个函数建立基础引理：

- involution
- injective
- preserves `0`
- parency / descendants 如何跟着变化

再往上用 `mdb_swap_abs` 这一层，把 `cdt` / descendants / parent relation 的变化收成抽象结论。

也就是说，`l4v` 并不是“证明不复杂”，而是：

- 复杂度被压缩进少数中心抽象
- 后续很多 invariant proof 复用这些抽象引理

这正是当前 `Verus` 版本最值得借鉴的地方。

## 5. `Verus` 能不能做到同样的抽象

可以做到，但要写成更 SMT-friendly 的形状。

`Verus` 不像 Isabelle 那样擅长大规模 relation reasoning 自动化，但它完全可以表达：

- 一个 slot ref permutation
- final state function
- finite changed-set frame
- extensional map equality
- `forall` 量化的点态后条件

当前代码中已经有一个很接近 `l4v::s_d_swap` 的雏形：

- [cte_swap_normalize_internal_ref](/workspace/rel4_kernel/sel4_cspace/src/cspace_manager/spec_util/swap.rs:101)

它已经在表达：

- 若 link 指向 `slot1`，变成 `slot2`
- 若 link 指向 `slot2`，变成 `slot1`
- 否则保持不变

这说明 `Verus` 并不是没有表达力，而是当前工程还没有把这个函数提升成整套证明的中心抽象。

此外，当前仓库已经在很多地方使用了适合 `Verus` 的表达手段：

- extensional map / set equality
- `forall` 量化的点态 post
- `slots_unchanged_except`
- `mdb_cross_links_unchanged_except`
- `cte_swap_final_post`

见：

- [spec_util/common.rs](/workspace/rel4_kernel/sel4_cspace/src/cspace_manager/spec_util/common.rs:17)
- [spec_util/common.rs](/workspace/rel4_kernel/sel4_cspace/src/cspace_manager/spec_util/common.rs:124)
- [spec_util/swap.rs](/workspace/rel4_kernel/sel4_cspace/src/cspace_manager/spec_util/swap.rs:755)

所以，这个问题更像是“已有表达能力没有被用在最简洁的主线位置上”。

## 6. 更适合 `Verus` 的证明骨架

下面给出推荐的 `swap` 证明结构。

### Phase A. 建立一个中心 permutation 抽象

建议把当前的 `cte_swap_normalize_internal_ref` 提升成整个证明的中心。

目标是把它当作 `l4v::s_d_swap` 的 Verus 对应物。

它至少应承担下面几类基础引理：

- `swap_ref_none`：
  - `None` 在 swap 后保持 `None`
- `swap_ref_involution`：
  - 交换两次回到原值
- `swap_ref_preserves_other`：
  - 非 `slot1/slot2` 的引用保持不变
- `swap_ref_eq_slot1` / `swap_ref_eq_slot2`
- `swap_ref_nonzero_preserved`
- `swap_ref_injective`

这些 lemma 都应该是：

- 小
- 纯
- 不依赖 manager 全状态

这是最值得新增的基础抽象层。

注意：

- 这里不是要求新造完全不同的 API
- 更现实的做法是保留 `cte_swap_normalize_internal_ref`
- 再补一组围绕它的纯 lemma

这样既贴近当前代码，也不会破坏已有 spec vocabulary。

### Phase B. 让现有 final-state spec 成为主接口

建议 future proof 主要依赖“最终状态函数”，而不是多个 `after_*_mgr`。

例如：

当前代码里其实已经有这一步的大部分内容：

- `cte_swap_final_entry`
- `cte_swap_final_post`
- `cte_swap_derivation_post`

因此，这一 phase 的实际工作不是“再定义一套 final-state spec”，而是：

- 把 `cte_swap_final_post` / `cte_swap_exact_post` 提升成 proof 主接口
- 让 runtime 对接层只负责证明它们成立

这一步的目标是：

- 对任意 `slot`
- 直接说出 new state 应该是什么

而不是：

- 先说 `after_prev1_mgr` 应该是什么
- 再说 `after_next1_mgr` 应该是什么
- ...

也就是说，proof 的主接口应是 **extensional final-state spec**，而且当前代码已经基本具备这个接口。

### Phase C. runtime proof 只证明局部 patch facts

exec 部分跑完后，只需要抽取少量局部事实：

- `slot1` 的最终 entry 正确
- `slot2` 的最终 entry 正确
- patch 集内其他 changed slot 的最终 entry 正确
- patch 集外 slot 完全不变
- `cdt_parent` / `is_original` 的 ghost map 正确

然后用一个总 lemma：

- `local_swap_facts_imply_exact_post`

把这些局部事实提升成：

- `cte_swap_exact_post(old_mgr, new_mgr, ...)`

这样 proof 的中心就从：

- runtime patch trace

变成：

- final-state extensional exactness

在当前工程里，比较可行的落点是：

- 保留一个 runtime-to-final-post 主 lemma
- 把 `after_*_mgr` 中间态尽量只留在这一个主 lemma 的内部
- 其他恢复 lemma 不再感知这些 trace-state

### Phase D. 所有 `wf` 恢复都从 `exact_post` 出发

一旦有了 `exact_post`，后面的恢复 lemma 应该统一走共享 `mdb_patch` closeout：

- `lemma_swap_exact_post_recovers_wf_via_mdb_patch`
- `lemma_swap_exact_post_implies_wf`

旧的 component-wise wrapper 已经不再保留为公共 proof surface；structural / semantic-edge / non-MDB frame recovery 由 `mdb_patch::lemma_patch_recovers_wf_from_obligations(...)` 统一拼回。

当前 [impl_swap.rs](/workspace/rel4_kernel/sel4_cspace/src/cspace_manager/impl_swap.rs:1418) 这类 lemma 已经有点接近这个方向，但前面还背着太多 trace-state 负担。

因此，更现实的改法不是重写所有 `wf` lemma，而是：

- 先把它们的 `requires` 从 trace-state 剥离
- 让它们只依赖 `cte_swap_exact_post` 和少量 frame / domain facts

## 7. 推荐保留的内容

下列内容建议保留，不必推倒：

- 当前 exec 主体的大顺序
- `cte_swap_patch_slots`
- `cte_swap_normalize_internal_ref`
- `cte_swap_expected_slot1_entry`
- `cte_swap_expected_slot2_entry`
- `cte_swap_final_entry`
- `cte_swap_final_post`
- `cte_swap_derivation_post`
- `cte_swap_exact_post`

这些已经形成了一套可以讲清语义的 vocabulary。

特别是：

- `cte_swap_final_entry`
- `cte_swap_final_post`

应被视为当前重构最值得保住的部分，而不是需要推翻重来。

## 8. 推荐删减或降级的内容

下列内容建议逐步退出 proof 主线中心：

- 过多的 `after_*_mgr`
- 依赖长 trace 的 “从 runtime patch 顺序推 exact post” 大 lemma
- 大量重复的 slot 分类讨论
- 为了推动主证明而引入的过重 `external_body` proof shell

它们不是完全错误，但应该从“主结构”降级为：

- 局部实现注释
- 或少数必要 helper

而不是继续作为整体证明的骨架。

## 9. 一个更适合 `Verus` 的分层模板

推荐后续将 `swap` 证明收成下面 4 层。

### Layer 1. Pure permutation lemmas

只讨论：

- `Option<SlotPtr>` 的 swap
- `SlotPtr` 的 swap
- injective / involutive / nonzero / not-self

不依赖 manager。

### Layer 2. Final-state spec lemmas

只讨论：

- final slot view
- final `cdt_parent`
- final `is_original`

不回放 runtime。

在当前工程中，这一层的核心应直接对应：

- `cte_swap_final_entry`
- `cte_swap_final_post`
- `cte_swap_derivation_post`

### Layer 3. Runtime-to-exact-post lemma

只证明：

- 当前 exec 实现满足 final-state spec

它是唯一需要看 runtime patch 顺序的层。

如果要控制复杂度，最关键的工程约束是：

- 尽量只保留一个这样的主 lemma
- 不让 trace-state 渗透进所有恢复 lemma

### Layer 4. Exact-post-to-wf recovery lemmas

只证明：

- `exact_post` 足以恢复各类 `wf`

它们不应该再关心 exec 的中间 patch 顺序。

## 10. 迁移顺序建议

如果以后真的做重构，建议顺序如下：

1. 先围绕 `cte_swap_normalize_internal_ref` 补纯 permutation lemma 集。
2. 明确把 `cte_swap_final_post` 视为主接口，不再新增平行 final-state 术语。
3. 收缩 `runtime -> exact_post`，减少 `after_*_mgr` 的外泄。
4. 把 `exact_post -> wf` 系列 lemma 改成完全不依赖 trace-state。

这样重构是渐进的，不需要一次重写整个 `swap`。

## 11. 可行性判断

这套方案之所以可行，依赖以下几点：

### 11.1 它不要求改 exec 主体

当前 exec 主体已经与 `reference_0ca248f` 的 old-style `cte_swap` 大体同形。

因此，本方案不要求你先动 mutation 顺序，而主要是重组 proof。

### 11.2 它不要求重建整套 spec vocabulary

当前工程已经有：

- `cte_swap_final_entry`
- `cte_swap_final_post`
- `cte_swap_exact_post`
- `slots_unchanged_except`
- `mdb_cross_links_unchanged_except`

所以方案更像“以现有 spec 为中心重排 proof”，而不是“重写 spec 再重写 proof”。

### 11.3 它符合已有项目总路线

这也和 [proof-checklist.md](/workspace/rel4_kernel/sel4_cspace/docs/proof-checklist.md) 的总原则一致：

- 语义上学 `l4v`
- 组织上学 `atmo`
- 以局部 refinement 闭环为主

当前 `swap` 的问题正是“局部 refinement 已有雏形，但 proof 组织偏 operational”；本方案正好是对这一点做收束。

## 12. 最终目标

理想状态下，`swap` 应达到：

- exec 形状继续接近旧 kernel 实现
- final semantic post 清晰
- proof 主体围绕 permutation + exact post 展开
- runtime trace 不再主导整个证明
- `Verus` 能稳定处理，而不是依赖大量 prover “硬推”

## Final Takeaway

一句话总结：

- `Verus` 不是做不到 `swap`
- 当前重，不是因为表达力不够
- 而是因为当前写法把“交换本质是一个置换”这件事埋在了长 trace 和大量 case split 下面

因此，更好的方向不是削弱语义目标，而是：

- 强化 permutation 抽象
- 弱化 trace 中心性
- 把证明重心移回现有的 final-state exactness

这才是更适合 `Verus` 的 `swap` 证明组织方式。
