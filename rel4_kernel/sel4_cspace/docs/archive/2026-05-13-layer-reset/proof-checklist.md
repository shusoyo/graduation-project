# `sel4_cspace` Proof Checklist

本文档给出 `sel4_cspace` 当前阶段最合适的证明推进方式，目标是把：

- `atmo` 的 Verus 工程组织
- `l4v` 校准过的不变量 / contract / 语义强度

组合成一条不会频繁返工的路线。

本文档是对 [trusted-boundary-plan.md](/workspace/rel4_kernel/sel4_cspace/docs/trusted-boundary-plan.md) 的补充，重点回答：

- 接下来先补哪些 contract
- contract 应该写到什么强度
- 哪些地方更该学 `l4v`
- 哪些地方更该学 `atmo`

## One-Sentence Rule

对每个关键函数，都做成：

- `atmo` 风格的 Verus exec 组织
- 用 `l4v` 校准过的抽象语义 / 不变量 / `requires/ensures`
- 函数边界上的局部语义闭环

不要先追求一整套 `l4v` 风格的大 refinement tower，也不要让 runtime helper 长期裸 trusted 地承载业务语义。

## 当前结构能否支撑这条路线

对当前阶段来说，答案基本是可以。

现有 [spec_proof.rs](/workspace/rel4_kernel/sel4_cspace/src/cspace_manager/spec_proof.rs) 已经有一批足够关键的 ghost 词汇：

- `CapSpec`
- `SlotEntrySpec`
- `spec_same_region_as_caps`
- `spec_same_object_as_caps`
- `spec_is_cap_revocable`
- `ensure_no_children_blocks`
- slot / MDB 相关局部更新词汇

这意味着：

- `insert/move/swap` 已经有比较像样的抽象层
- `resolve` 已经可以走“exec loop 直接证明 + 抽象语义 refinement”
- 非 arch / non-zombie 的 capability / cte 语义，原则上也可以继续往这套词汇上挂

当前明显还不够的部分主要是：

- arch capability 语义
- zombie capability 语义
- delete / revoke / finalise 的完整语义闭环

所以如果你暂时放弃 arch 和 zombie，当前 ghost 结构是够你把大部分 CSpace 主逻辑继续收下去的。

## 应该更多借鉴谁

答案依然不是二选一，但当前项目的默认重心更偏向 `atmo`。

### 当前默认方法论

默认按下面这套顺序来：

1. 先用 `atmo` 的方式组织 Verus 代码和 proof。
2. 再用 `l4v` 去校准不变量、contract 强度、语义一致性。
3. 暂时不把显式 `l4v` refinement pipeline 当成 blocker。

### 更该学 `l4v` 的地方

这些地方应该优先从 `l4v` 借语义和强度，而不是借代码外形：

- `resolve` 的 fault / success 分类
- `lookup` / `cnode walk` 的前后条件
- `same_region_as` / `same_object_as` / `is_cap_revocable` 这类 capability relation
- `ensure_no_children`
- `derive_cap`
- `is_final_cap`
- 各函数 `requires/ensures` 应强到什么程度

这里的核心不是 monad 写法，而是：

- 输入是什么
- 哪些分支返回 fault
- 哪些分支返回 success
- 返回结果与抽象状态的关系是什么

### 更该学 `atmo` 的地方

这些地方更适合借 `atmo` 的 Verus 工程方式：

- manager 作为 ghost/perms/wf 聚合点
- `external_body + requires/ensures` 的 bridge 风格
- exec 函数直接证明
- “修改很小的 helper + 大量 frame ensures”
- 不把高层业务函数本身 trusted 掉

换句话说：

- 默认设计问题学 `atmo`
- 语义和 contract 强度问题学 `l4v`

## Contract 应该怎么写，才不容易返工

最重要的规则是：

- contract 要描述“语义结果”
- 不要只描述“这次代码刚好算出来了什么中间值”

如果 contract 只贴着当前实现细节写，后面一旦你重排 helper 或调整 exec 结构，就会返工。

### 好的 contract 应该满足三点

#### 1. 对齐现有 spec vocabulary

尽量直接用 [spec_proof.rs](/workspace/rel4_kernel/sel4_cspace/src/cspace_manager/spec_proof.rs) 里已有概念表达结果，比如：

- `trusted_view_cap`
- `trusted_view_cte`
- `spec_same_region_as_caps`
- `spec_same_object_as_caps`
- `spec_is_cap_revocable`

而不是再新造第二套平行术语。

#### 2. 表达抽象 effect，不表达实现步骤

例如对 trusted helper，应该写：

- 这个字段变成什么
- 哪些字段不变
- ghost view 如何变化

而不是写：

- 先读了这个，再写了那个，再调用了某个 setter

#### 3. 为将来“去 trusted”保留同一个边界

也就是说，即使今天一个函数还是 `external_body`，它的 `requires/ensures` 也应该已经像“将来真正证明它时的最终接口”。

这样后面你做的不是“改接口”，而是“把接口背后的实现证明补上”。

## 你现在最适合采用的证明策略

这条路线最适合当前项目：

1. 在函数层面建立局部 refinement。
2. 低层 bridge 允许暂时 trusted，但必须有强 contract。
3. 高层业务语义尽量移出 trusted。
4. 不强行先建完整 global refinement tower。

这其实就是在弥合：

- `l4v` 的“语义和 contract 强度可靠”
- `atmo` 的“Verus exec 直接证明、bridge contract 很强”

## 下一批最值得补的 contract

如果按“收益最大、且不要求先证明 arch / zombie”排序，我建议是下面这批。

### 第一组：capability 关系语义

文件：

- [capability/mod.rs](/workspace/rel4_kernel/sel4_cspace/src/capability/mod.rs)

优先函数：

- `same_region_as`
- `same_object_as`
- `is_cap_revocable`

原因：

- `insert/move/delete` 都已经在依赖这些关系
- [spec_proof.rs](/workspace/rel4_kernel/sel4_cspace/src/cspace_manager/spec_proof.rs) 里已经有对应抽象词汇
- 这三者最容易做成“runtime bool 精化到 spec bool”

建议 contract 形状：

```rust
ensures ret == spec_same_region_as_caps(trusted_view_cap(cap1), trusted_view_cap(cap2))
```

```rust
ensures ret == spec_same_object_as_caps(trusted_view_cap(cap1), trusted_view_cap(cap2))
```

```rust
ensures ret == spec_is_cap_revocable(trusted_view_cap(derived_cap), trusted_view_cap(src_cap))
```

注意：

- 如果 arch 分支暂时不证明，就把 contract 写成“在 non-arch 前提下精化到当前 spec”
- 不要为了省事把 arch 语义偷偷糊进 `Other`

### 第二组：`cte` 的只读语义判断

文件：

- [cte.rs](/workspace/rel4_kernel/sel4_cspace/src/cte.rs)
- [impl_delete.rs](/workspace/rel4_kernel/sel4_cspace/src/cspace_manager/impl_delete.rs)

优先函数：

- `ensure_no_children`
- `is_final_cap`

原因：

- 这两个是 delete / revoke 方向最核心的判定
- 它们比 `derive_cap` 更局部，先收它们最稳
- [spec_proof.rs](/workspace/rel4_kernel/sel4_cspace/src/cspace_manager/spec_proof.rs) 已经有 `ensure_no_children_blocks`

建议 contract 形状：

- `ensure_no_children`：
  - success 当且仅当 `!old(self).ensure_no_children_blocks(slot)`
  - fault 当且仅当 `old(self).ensure_no_children_blocks(slot)`

- `is_final_cap`：
  - 返回值等于一个 spec predicate
  - 这个 predicate 应表达“该 slot 指向对象在当前可见 CSpace 中没有其他 alias”

这里建议先在 `spec_proof.rs` 把 `spec_is_final_cap_slot(...)` 补出来，再给 exec 函数挂 ensures。

### 第三组：`derive_cap`

文件：

- [impl_delete.rs](/workspace/rel4_kernel/sel4_cspace/src/cspace_manager/impl_delete.rs)
- [cte.rs](/workspace/rel4_kernel/sel4_cspace/src/cte.rs)

原因：

- 这是 capability semantic layer 和 delete/revoke layer 的连接点
- 它天然要参考 `l4v` 的分支分类

建议 contract 先不要一次写太大。

先做成分支式 refinement：

- untyped 分支
- zombie / reply / irq control 分支
- identity 分支
- arch 分支先作为单独 deferred case

目标不是一步吃完整个 `derive_cap`，而是先把 non-arch 的语义边界固定住。

## 哪些 trusted helper 现在就该补强 contract

即使这些函数暂时继续 trusted，也不该裸着放。

优先对象：

- [trusted/common.rs](/workspace/rel4_kernel/sel4_cspace/src/trusted/common.rs) 里的 capability / cte view bridge
- [trusted/mdb.rs](/workspace/rel4_kernel/sel4_cspace/src/trusted/mdb.rs) 里的只读 MDB observer
- [trusted/resolve.rs](/workspace/rel4_kernel/sel4_cspace/src/trusted/resolve.rs) 里的 resolve primitive
- [impl_base.rs](/workspace/rel4_kernel/sel4_cspace/src/cspace_manager/impl_base.rs) 里的 slot patch bridge

标准是：

- observer bridge：返回值必须和 ghost view 一致
- patch bridge：只改声明的局部字段，其余保持不变
- pointer bridge：返回的引用 / perm 必须对应到同一 ghost view

## 具体实施顺序

推荐顺序如下：

1. 给 `same_region_as` / `same_object_as` / `is_cap_revocable` 补 contract。
2. 在 `spec_proof.rs` 中补 `is_final_cap` 对应 spec predicate。
3. 给 `ensure_no_children` / `is_final_cap` 补 contract。
4. 把 `derive_cap` 先收成 non-arch refinement。
5. 再考虑 delete / revoke 主线。

这样做的好处是：

- 先把最常被上层使用的 capability relation 固定住
- 再把 delete 判断层固定住
- 最后再去碰更复杂的 mutation / finalisation 逻辑

## 你现在不该做的事

当前阶段不建议：

- 先追求 arch / zombie 全证明
- 先追求全项目统一的大 refinement 图
- 为了“形式统一”把所有函数都塞进 manager method
- 为了赶进度继续增加没有 contract 的 trusted semantic helper

这些都会让你后面更容易返工。

## 最终验收标准

如果后面推进顺利，比较理想的状态是：

- 高层 CSpace 业务语义主要落在 verified exec + spec/refinement 上
- trusted 层只剩 view bridge、pointer bridge、bitfield bridge、局部 patch bridge
- capability relation 不再靠裸 external 语义决定
- `resolve`、`insert/move/swap`、`ensure_no_children`、`is_final_cap`、`derive_cap(non-arch)` 形成一套可讲述的局部 refinement 闭环

这时你的项目就会比较像：

- 工程组织上接近 `atmo`
- 语义和 contract 强度上经过 `l4v` 校准

而不是两边都只学了一半。
