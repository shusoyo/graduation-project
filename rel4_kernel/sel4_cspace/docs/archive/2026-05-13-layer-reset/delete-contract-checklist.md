# `sel4_cspace` Delete Contract Checklist

本文档给出 `sel4_cspace` 下一阶段在 `delete` 方向上的推荐推进顺序。

核心结论不是“立刻去证明 `delete` 本体”，而是：

- 先把 `delete` 会踩到的下层操作收口成有意义的 contract
- 再做 `delete_one` / `delete_all`
- 最后再扩到 `reduce_zombie` / `revoke`

这样做的原因很直接：

- `resolve / insert / move / swap` 之所以已经基本收住，是因为它们主要在 `CSpaceManager` 这层闭合
- `delete` 会直接碰到底层 semantic helper、外部 hook、zombie 递归、preemption bridge
- 如果这些层还是裸 `external_body`，那 `delete` 证明过程里会不断返工 contract

## Summary

推荐顺序如下：

1. 先补 `delete core` 的 contract 闭包
2. 再做 `delete_one`
3. 再做 `delete_all`
4. 然后处理 `reduce_zombie`
5. 最后再做 `revoke`

这里的 “delete core” 指的是：

- `delete_one`
- `delete_all`
- `set_empty`
- `finalise_slot`
- `finalise_cap`
- `preemption_point`
- `post_cap_deletion`
- `is_mdb_parent_of`
- `is_final_cap`
- `same_region_as`
- `same_object_as`
- `cap_cyclic_zombie`

不建议一开始就直接冲 `revoke`，因为它额外引入了：

- `get_volatile_value`
- 遍历型 while loop
- 父子判断的重复调用
- `delete_all + preemption_point` 的交织控制流

这些都不是 `delete_one / delete_all` 的第一批 blocker。

## Current Dependency Graph

当前 `delete` 主线的依赖关系大致是：

1. `CSpaceManager::delete_one`
2. `CSpaceManager::delete_all`
3. `impl_base::finalise_slot`
4. `deps::finalise_cap`
5. `CSpaceManager::is_final_cap`
6. `impl_delete::reduce_zombie`
7. `deps::preemption_point`
8. `impl_delete::set_empty`
9. `deps::post_cap_deletion`

其中还会继续下沉到：

1. `impl_delete::is_mdb_parent_of`
2. `capability::same_region_as`
3. `capability::same_object_as`
4. `capability::zombie::cap_cyclic_zombie`
5. `capability::zombie::{get_zombie_ptr,get_zombie_number,set_zombie_number}`
6. `cte::cte_swap` 或 manager 内部 `cte_swap`

所以如果只盯着 `impl_delete.rs` 自己，是不够的。

## Reusable Spec Vocabulary

这轮 contract 不应该新造一套平行 ghost 结构，应该优先复用现有 spec 词汇。

当前已经存在、而且适合直接复用的词汇包括：

- `trusted::finalise_cap_contract`
- `trusted::post_cap_deletion_preserves_visible_cspace`
- `trusted::preemption_point_preserves_manager`
- `CSpaceManager::mdb_parent_of`
- `CSpaceManager::ensure_no_children_blocks`
- `spec_same_region_as_caps`
- `spec_same_object_as_caps`

这些词汇的意义分别是：

- `finalise_cap_contract`：`finalise_cap` 的抽象语义落点
- `post_cap_deletion_preserves_visible_cspace`：删除后 hook 不改变可见 CSpace
- `preemption_point_preserves_manager`：preemption bridge 不改变 manager 语义状态
- `mdb_parent_of`：manager 层的抽象父子关系
- `ensure_no_children_blocks`：`ensure_no_children` 的规范化阻塞条件
- `spec_same_region_as_caps` / `spec_same_object_as_caps`：cap 语义层的已有抽象判断

如果后面需要新增词汇，优先新增在：

- `src/trusted/common.rs`
- `src/cspace_manager/spec_proof.rs`

而不是新建一份 delete-only 的状态机。

## Contract Writing Rules

为了保证后面不返工，建议所有新 contract 都遵守下面四条规则。

### 1. `requires` 写语义适用域

不要把 solver 临时需要的事实直接写成永久前提。

好的前提例子是：

- slot 在 `slot_dom`
- `old(self).wf()`
- 输入 cap 和 slot view 良构
- 调用发生在 manager 当前抽象状态允许的域里

不好的前提例子是：

- 某个中间局部变量已经等于某个特定表达式
- 只是为了当前 proof script 方便拆 case 的辅助事实

### 2. `ensures` 写抽象效果或精确局部 patch

contract 的后置条件应只做两种事：

- 描述这个操作精化到哪个抽象语义
- 描述它局部修改了哪些字段、哪些地方保持 frame

不要把整段高层业务语义黑盒化。

### 3. 优先复用已有抽象语义

例如：

- `same_region_as` 应落回 `spec_same_region_as_caps`
- `same_object_as` 应落回 `spec_same_object_as_caps`
- `ensure_no_children` 应落回 `ensure_no_children_blocks`

这样上层 proof 不会再维护两套术语。

### 4. manager 内部优先调用 manager 版本，而不是 public wrapper

例如 `reduce_zombie` 里如果需要复用 swap，更推荐直接用 manager 内部 `cte_swap` 语义，而不是经过 `cte.rs` 的 public `external_body` wrapper。

理由是：

- manager 内部版本已经有更强的 proof interface
- public wrapper 更适合 kernel-facing 兼容层
- 证明时绕过 public wrapper，可以减少一层 TCB 壳

## Phase 0: 先补 Spec 落点

在正式补 exec contract 之前，建议先把下面两个 spec 落点补齐。

### P0.1 为 `is_final_cap` 建立 spec 谓词

当前没有现成的 `spec_is_final_cap_at` 一类定义。

建议新增一个 manager 只读谓词，语义贴合当前代码：

- 如果前驱存在且与当前 cap `same_object_as`，则不是 final
- 否则如果后继不存在，则是 final
- 否则只有当后继与当前 cap `!same_object_as` 时才是 final

这个谓词应该放在：

- `src/cspace_manager/spec_proof.rs`

这样 `is_final_cap`、`is_long_running_delete`、`finalise_slot` 都能复用。

### P0.2 为 `finalise_slot` 建立抽象结果关系

如果这轮还不准备立刻把 `finalise_slot` body 全证明出来，就应该先定义一个抽象 relation。

建议新增类似：

- `finalise_slot_contract(old_mgr, slot, immediate, new_mgr, ret)`

它至少要表达：

- 返回的 `status / success / cleanupInfo`
- 哪些可见 slot 可能变化
- 哪些全局 ghost 量必须保持 frame
- 什么情况下返回 success
- 什么情况下只是“中断/挂起”而不是彻底完成删除

这个 relation 应该放在：

- `src/cspace_manager/spec_util/delete.rs`

如果暂时不想新建文件，也可以先放在 `impl_base.rs` 对应的 spec 区域，但长期更建议单独成模块。

## Phase 1: Delete Core Blocker Checklist

这一批是 `delete_one / delete_all` 的第一优先级 blocker。

### 1. `deps::finalise_cap`

位置：

- `src/deps.rs`
- 抽象落点已在 `src/trusted/common.rs`：`finalise_cap_contract`

当前状态：

- 只有裸 `external_body`
- 没有 `requires`
- 没有 `ensures`

建议 contract 目标：

- `ensures trusted_view_cap(&ret.remainder) == finalise_cap_contract(trusted_view_cap(capability), is_final, exposed).0`
- `ensures trusted_view_cap(&ret.cleanupInfo) == finalise_cap_contract(trusted_view_cap(capability), is_final, exposed).1`

建议说明：

- 不要在 `deps.rs` 里过早塞进大量 delete-specific case split
- “返回的 remainder 是什么类型”“cleanupInfo 何时非空” 这类性质，优先作为 `finalise_cap_contract` 上的 lemma 去组织

优先级：

- `P0`

### 2. `deps::preemption_point`

位置：

- `src/deps.rs`
- 抽象落点已在 `src/trusted/common.rs`：`preemption_point_preserves_manager`

当前状态：

- 裸 `external_body`

建议 contract 目标：

- 说明返回的 `exception_t` 是唯一可见结果
- 说明它不改变 manager 的可见抽象状态

推荐做法：

- 可以保留 free function 版本
- 但更适合再补一个 manager-side bridge helper，用 `preemption_point_preserves_manager(old_mgr, new_mgr, status)` 来承接证明

优先级：

- `P0`

### 3. `deps::post_cap_deletion`

位置：

- `src/deps.rs`
- 抽象落点已在 `src/trusted/common.rs`：`post_cap_deletion_preserves_visible_cspace`

当前状态：

- 裸 `external_body`

建议 contract 目标：

- 说明它不会修改可见 slot/MDB/resolve/cdt 语义
- 如果需要，允许它影响 CSpace 外部世界，但那部分不进入 `CSpaceManager` 语义

优先级：

- `P0`

### 4. `capability::same_region_as`

位置：

- `src/capability/mod.rs`

当前状态：

- `external_body`
- 被 `is_mdb_parent_of` 直接依赖

建议 contract 目标：

- 返回值与 `spec_same_region_as_caps(trusted_view_cap(cap1), trusted_view_cap(cap2))` 一致

组织建议：

- 如果直接给 exec 函数写强 ensures 不方便，就补一个 refinement lemma
- 但不要长期停留在“纯 runtime bool、无 spec 对应”的状态

优先级：

- `P0`

### 5. `capability::same_object_as`

位置：

- `src/capability/mod.rs`

当前状态：

- `external_body`
- 被 `is_final_cap` 直接依赖

建议 contract 目标：

- 返回值与 `spec_same_object_as_caps(trusted_view_cap(cap1), trusted_view_cap(cap2))` 一致

优先级：

- `P0`

### 6. `CSpaceManager::is_mdb_parent_of`

位置：

- `src/cspace_manager/impl_delete.rs`

当前状态：

- `external_body`
- 其实是一个只读查询

建议 contract 目标：

- `requires old(self).wf()`
- `requires old(self).slot_dom().contains(slot)`
- `requires old(self).slot_dom().contains(next)`
- `ensures ret == old(self).mdb_parent_of(slot, next)`
- `ensures self =~= old(self)` 在可见 manager 语义上保持不变

这一步非常重要，因为：

- `ensure_no_children`
- `revoke`
- `delete` 路径上的父子关系判断

都会通过它收口到同一个抽象 predicate。

优先级：

- `P0`

### 7. `CSpaceManager::ensure_no_children`

位置：

- `src/cspace_manager/impl_delete.rs`

当前状态：

- 有 exec body
- 但还没有把语义明确写进 contract

建议 contract 目标：

- `requires old(self).wf()`
- `requires old(self).slot_dom().contains(slot)`
- `ensures ret == runtime_exception_syscall_error() <==> old(self).ensure_no_children_blocks(slot)`
- `ensures ret == runtime_exception_none() <==> !old(self).ensure_no_children_blocks(slot)`
- `ensures self =~= old(self)` 在 manager 语义上不变

优先级：

- `P0`

### 8. `CSpaceManager::is_final_cap`

位置：

- `src/cspace_manager/impl_delete.rs`

当前状态：

- `external_body`
- 目前没有对应的 spec predicate

建议 contract 目标：

- 先补 `spec_is_final_cap_at` 一类 ghost 定义
- 再给 `is_final_cap` 写：
  `ensures ret == old(self).spec_is_final_cap_at(slot)`
- 同时要求它是只读查询，不改变 manager 语义状态

优先级：

- `P0`

### 9. `CSpaceManager::set_empty`

位置：

- `src/cspace_manager/impl_delete.rs`

当前状态：

- 有 exec body
- 但还没有形成像 `insert/move/swap` 那样的局部 post 语义层

建议 contract 目标：

- 当前 slot 变成 empty entry
- `prev.next` 被改到原来的 `next`
- `next.prev` 被改到原来的 `prev`
- `next.first_badged` 得到 `old_first_badged || next_first_badged`
- patch set 之外保持 frame
- `post_cap_deletion` 不改变可见 CSpace 语义

这一步建议像 `swap` 一样整理成：

- local post
- patch set
- exact post

优先级：

- `P0`

### 10. `impl_base::finalise_slot`

位置：

- `src/cspace_manager/impl_base.rs`

当前状态：

- 仍然是 whole-loop `external_body`

建议做法有两个选择。

选择 A：

- 先保留 external
- 但补一个明确的 `finalise_slot_contract`
- 让 `delete_one / delete_all` 先建立在这个 contract 上

选择 B：

- 直接拆循环
- 把 `finalise_cap`
- `cap_cyclic_zombie`
- `reduce_zombie`
- `preemption_point`

这些步骤拆成较小 helper，再对 loop 本体证明

推荐判断：

- 如果目标是尽快把 `delete_one / delete_all` 收到和 `move/swap` 同一层级，先走选择 A
- 如果目标是继续 aggressively 缩 TCB，直接走选择 B

无论选哪条路，`finalise_slot` 都是 `delete` 的总 blocker。

优先级：

- `P0`

## Phase 2: Zombie Reduction Checklist

这一批是 `delete core` 收住之后，继续向 `reduce_zombie` 推进时要补的 contract。

### 11. `capability::zombie::cap_cyclic_zombie`

位置：

- `src/capability/zombie.rs`

当前状态：

- `external_body`

建议 contract 目标：

- 返回值应落回一个明确的 ghost zombie 判定
- 至少能表达“当前 zombie 是否指回当前 slot”

优先级：

- `P1`

### 12. `zombie_func::{get_zombie_ptr,get_zombie_number,set_zombie_number}`

位置：

- `src/capability/zombie.rs`
- `src/cspace_manager/impl_base.rs` 的 `set_slot_zombie_number_runtime`

当前状态：

- 都是低层语义 helper

建议 contract 目标：

- `get_zombie_ptr/get_zombie_number` 是只读 refinement bridge
- `set_zombie_number` / `set_slot_zombie_number_runtime` 只修改 zombie number，不改其他 cap 语义字段

优先级：

- `P1`

### 13. `CSpaceManager::reduce_zombie`

位置：

- `src/cspace_manager/impl_delete.rs`

当前状态：

- 已经不是 whole-function `external_body`
- 当前形态是：
- verified outer dispatch
- `immediate == false` 分支直接复用 manager 内部 `cte_swap`
- `immediate == true` 分支仍通过 branch-local trusted helper 收口
- 另保留一条小的 non-immediate ghost bridge，用来承接 swap 前的 domain / root 约束
- `immediate == false` 这条已验证路径当前还显式依赖 `cte_swap` 带来的
  `mdb_no_two_cycle_wf` side condition；这在当前 closeout 阶段可接受，但还不应被表述成
  “纯粹只靠 `reduce_zombie_pre_from_mgr` 就能调用”

建议 contract 目标：

- `immediate == false` 分支应表达为“与一次受控 swap 等价”或“把当前 zombie 与其目标 slot 做指定变换”
- `immediate == true` 分支应表达为“对子项递归 delete 后，当前 zombie 计数减少或消失”
- 返回非 `EXCEPTION_NONE` 时，状态必须落到明确的中断/失败分类

重要建议：

- 如果后续要证明它，优先直接复用 manager 内部 `cte_swap`
- 不建议继续通过 `cte.rs::cte_swap` 这个 public wrapper 过桥

优先级：

- `P1`

## Phase 3: Revoke-Specific Checklist

这一批可以明确延后到 `delete_one / delete_all / reduce_zombie` 之后。

### 14. `impl_base::get_volatile_value`

位置：

- `src/cspace_manager/impl_base.rs`

当前状态：

- 纯 external volatile 读

建议 contract 目标：

- 只描述“从 slot 的当前 next pointer 位置读到什么”
- 不承载高层 revoke 语义

优先级：

- `P2`

### 15. `CSpaceManager::revoke`

位置：

- `src/cspace_manager/impl_delete.rs`

当前状态：

- whole-function `external_body`

建议 contract 目标：

- while loop 每次只删除直接子项
- 遇到非 child 或空链停止
- `preemption_point` 只影响返回状态，不改变 CSpace 语义

但这一步明确不是当前第一批 delete blocker。

优先级：

- `P2`

## Phase 4: Public Wrapper Cleanup

这些不是 manager 内核证明的第一 blocker，但如果以后要让 public API 也进入已证明域，就需要补。

位置：

- `src/cte.rs`

建议后续收口的 wrapper 包括：

- `ensure_no_children`
- `is_final_cap`
- `is_long_running_delete`
- `delete_all`
- `delete_one`
- `revoke`

建议方向：

- 让它们成为 thin wrapper
- `requires` 明确 mirror manager proof domain
- `ensures` 明确 mirror manager postcondition

不建议继续保持“无前置约束的 public external shell”。

## Recommended Execution Order

推荐的实际执行顺序如下：

1. 在 `spec_proof.rs` 补 `is_final_cap` 的 ghost predicate
2. 在 `trusted/common.rs` 固定 `finalise_cap_contract / preemption_point_preserves_manager / post_cap_deletion_preserves_visible_cspace` 的使用方式
3. 给 `deps.rs::{finalise_cap,preemption_point,post_cap_deletion}` 补 contract
4. 给 `same_region_as / same_object_as` 补 refinement bridge
5. 给 `is_mdb_parent_of / ensure_no_children / is_final_cap` 补 contract
6. 给 `set_empty` 整理 local post / exact post
7. 选择 `finalise_slot` 的推进方式：
8. 临时 abstract contract，或直接拆 loop 证明
9. 在此基础上先做 `delete_one`
10. 再做 `delete_all`
11. 然后处理 `cap_cyclic_zombie / zombie_number patch`
12. 再做 `reduce_zombie`
13. 最后做 `revoke`

## Definition Of Done

这一轮真正完成的标准建议定义为：

1. `delete_one` 拥有 manager-level proof，强度接近当前 `move/swap`
2. `delete_all` 拥有 manager-level proof，强度接近当前 `move/swap`
3. `delete` 依赖的第一批 external helper 不再是裸函数，而是都有明确 `requires/ensures`
4. 不再新增新的 whole-operation black-box helper 来承载 delete 业务语义
5. `revoke` 可以继续暂缓，但其 blocker 已被明确隔离到第二批

## Bottom Line

下一步最合适的工作，不是直接去写 `delete` 大证明，而是先把下面这几项补齐：

1. `finalise_cap`
2. `preemption_point`
3. `post_cap_deletion`
4. `same_region_as`
5. `same_object_as`
6. `is_mdb_parent_of`
7. `ensure_no_children`
8. `is_final_cap`
9. `set_empty`
10. `finalise_slot`

其中前八项是“contract 闭包”，第九和第十项是“delete core 自身的证明入口”。

只要这一批收住，后面的 `delete_one / delete_all` 就不会像现在这样被一圈裸 external 卡住。

## Current Closeout Snapshot

当前 closeout 已经达到下面这个状态：

1. `delete` 依赖的第一批 contract 闭包已基本完成。
2. `delete_one` / `delete_all` 已经能够建立在这些 contract 之上。
3. `preemption_point` / `post_cap_deletion` 的正式 delete 语义边界固定在 manager bridge，而不是 raw extern。
4. 当前明确保留的 delete 线高层 TCB 主要是 `finalise_slot`，以及 `reduce_zombie` 的 immediate/helper 残余 bridge。
5. 如果下一轮继续推进，重点应从“补 contract”转向“是否要继续 shrink 这两个 retained TCB”。
