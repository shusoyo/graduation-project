# reL4 CSpace 原生 Verus 重构方案

## 1. 文档定位

本文件保留为总纲版计划，回答三件事：

- 为什么这轮重构要做；
- 重构后的总体风格应该是什么；
- 执行时应优先看哪份细化文档。

本目录中的文档分工如下：

- 总纲说明：`README.md`
- 细化执行计划：[detailed-plan.md](/workspace/rel4_kernel/docs/verification/verus-native-refactor/detailed-plan.md)
- 执行清单：[TODO.md](/workspace/rel4_kernel/docs/verification/verus-native-refactor/TODO.md)

---

## 2. 重构目标

本轮重构的真实目标不是“完全照抄 `l4v` 的结构”，而是：

- 借鉴 `l4v` 对 CSpace 的抽象语义与证明目标；
- 改用更符合 Verus 的原生验证风格组织证明；
- 删除多余的中转 contract、proof wrapper 和 interface 壳；
- 保留必要的 concrete-to-abstract bridge。

换句话说：

**`l4v` 决定证明什么，Verus-native 决定代码怎么长。**

---

## 3. 当前问题

当前代码已经有了比较好的抽象规范基础，但证明组织仍然偏重：

- `interface.rs` 过厚，承担了额外的 proof-surface 职责；
- `cte.rs` 中很多函数仍然优先证明 `*_exec_contract`；
- `refinement_bridge.rs` 作为表示桥接层是必要的，但当前承载内容偏多；
- 查询函数、派生查询函数、mutator 没有统一成一种终局风格。

---

## 4. 目标风格

重构完成后，理想结构应当是：

1. `specs/*`
   只定义抽象语义与状态转移。
2. `refinement_bridge.rs`
   只负责 raw 表示到抽象视图的投影。
3. `cte.rs` / `capability/mod.rs`
   直接给出最终验证入口和最终 postcondition。
4. `interface.rs`
   仅保留真正需要的薄公共层，或者进一步收缩为 re-export。

核心原则：

- 保留桥接层；
- 删除重复语义壳；
- 查询函数直接对齐抽象 spec；
- mutator 直接对齐抽象 state-post；
- 派生查询函数直接结束于最终语义，而不是 contract 名称。
- TCB 命名按语义分类，而不是把所有 `trusted_*` 机械替换成单一前缀。

---

## 5. 执行顺序

建议按以下顺序推进：

1. 查询函数：
   - `same_region_as`
   - `same_object_as`
   - `is_cap_revocable`
2. 派生查询函数：
   - `is_mdb_parent_of`
   - `is_final_cap`
   - `ensure_no_children`
   - `is_long_running_delete`
   - `derive_cap`
3. lookup：
   - `resolve_address_bits`
4. mutator：
   - `cte_insert`
   - `insert_new_cap`
   - `cte_move`
   - `cte_swap`
5. 最后清理：
   - `interface.rs`
   - `refinement_bridge.rs`

---

## 6. 成功标准

完成后应满足：

- public postcondition 一眼能看出抽象语义；
- `*_exec_contract` 不再作为主要对外语义层；
- `interface.rs` 明显变薄；
- `refinement_bridge.rs` 只做表示桥接；
- 读者理解核心函数时，不必跨越 2 到 3 层同义包装。

---

## 7. 对应细化文档

函数级、文件级、证明步骤级的完整执行计划见：

- [detailed-plan.md](/workspace/rel4_kernel/docs/verification/verus-native-refactor/detailed-plan.md)
