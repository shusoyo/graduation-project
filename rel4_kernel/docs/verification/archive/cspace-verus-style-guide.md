# CSpace Verus 编写风格约束

状态（2026-04-24）：

- 本文档服务于 `sel4_cspace` 后续第 6 步推进。
- 目标是把 `sel4_cspace/src/cte.rs` 中的核心函数逐步收敛成“原生 Verus 风格”的 exec 函数。
- 核心偏好是：
  - 签名优先；
  - attribute 最小化；
  - 语义对齐 `l4v`；
  - proof 工程风格优先靠近 `vostd`；
  - trusted 边界纪律借鉴 `atmo`。

## 1. 目标

- 长期目标不是让 `refinement_bridge.rs` 永久承载主要语义，也不是让 `#[verus_spec(...)]` 永久承载主合同。
- 长期目标是让 `cte.rs` 中的核心函数尽量直接写成：
  - 同名 exec 函数；
  - 自身携带 `requires/ensures`；
  - proof 尽量贴近函数 body；
  - 只在不可避免的边界保留 `external_body` / attribute contract。

## 2. 总原则

### 2.1 签名优先，attribute 只做过渡

首选写法：

```rust
verus! {

pub fn helper(...)
    requires ...
    ensures ...
{
    ...
}

} // verus!
```

不把下面这种形态作为长期终点：

```rust
#[cfg_attr(feature = "verify", verus_spec(...))]
pub fn helper(...) { ... }
```

`#[verus_spec(...)]` 仅允许作为：

- 旧 Rust body 暂时还不能直接进入 Verus 时的过渡合同；
- 必须保持原 Rust 函数签名不变的边界函数；
- 调用点临时注入 ghost / tracked 实参的过渡桥。

### 2.2 同名函数优先，不长期维持双层 wrapper

优先目标是：

- `cte_insert` 证明 `cte_insert`；
- `cte_move` 证明 `cte_move`；
- `cte_swap` 证明 `cte_swap`；
- `resolve_address_bits` 证明 `resolve_address_bits`。

不希望长期停在：

- `trusted_call_*`
- `*_refined`
- 外围 wrapper 证明同名 exec 正确

这类双层入口只允许作为阶段性脚手架。

### 2.3 bridge 只解释数据形状，不承载最终业务语义

`refinement_bridge.rs` 应继续承担：

- raw `cap` / `cte_t` / 返回值 到抽象模型的 view；
- concrete heap 与抽象状态之间的 refinement relation；
- 必要的桥接引理。

`refinement_bridge.rs` 不应继续扩张为：

- 永久函数语义层；
- 巨型 proof orchestration 层；
- 主函数合同的最终落点。

### 2.4 合同粒度要小，proof 尽量贴近 body

优先写法：

- 小 helper；
- 小 contracts；
- object-local lemma；
- case split 跟着函数控制流走；
- 局部 frame condition 明确。

避免：

- 一个函数依赖一个超大 predicate 才能调用；
- 证明主要发生在远离 body 的巨型 bridge 文件里；
- 每个调用点都手拆大 conjunction。

### 2.5 trusted surface 必须单独命名、单独收口

借鉴 `atmo` 的边界纪律：

- trusted 项必须显式命名；
- trusted 项必须解释为什么暂时不能下推；
- trusted 项必须有收缩方向。

推荐命名：

- `trusted_*`：确实属于可信边界；
- `lemma_*`：普通证明引理；
- 不再新增语义不透明但看不出可信性质的中间名。

## 3. 函数迁移阶梯

每个目标函数默认按下面四段推进，不要求一步到位。

### 3.1 阶段 A：抽象 spec 与 bridge 已稳定

此时函数已有：

- 抽象 `pre/post`；
- bridge precondition；
- refinement relation；
- `*_refined(...)` 入口。

这是进入本体化之前的起点。

### 3.2 阶段 B：同名函数挂临时 attribute 合同

如果真实 Rust body 还不够 Verus-friendly，可以先写成：

- 同名函数；
- `external_body`；
- `verus_spec(...)`。

这一步允许存在，但它只是过渡层。

判定标准：

- 合同已经回到同名函数；
- 但 proof 还没有进入函数 body。

### 3.3 阶段 C：拆出可直接验证的小 helper

下一步不是继续加 wrapper，而是把 body 中难点拆开：

- bitfield getter 语义封装成小 helper；
- 指针读写封装成最小 trusted util；
- 分支逻辑拆成局部 helper；
- case-local lemma 贴近 helper 写。

这一步新增的 helper 默认必须用签名式 `requires/ensures`，而不是继续堆 attribute。

### 3.4 阶段 D：主函数变成原生 Verus 签名风格

最终目标是：

- 同名主函数本身进入 `verus!` 风格；
- 主合同写在函数签名；
- `#[verus_spec(...)]` 从主入口删除；
- wrapper 退化或消失。

## 4. 新代码的默认规则

### 4.1 新增 helper 的默认写法

凡是新增的 proof-carrying helper，默认写成：

- `verus!` 块内；
- 函数签名直接写 `requires/ensures`；
- 如果是纯规格辅助，则写 `spec fn` / `proof fn`。

不默认写成 attribute contract。

### 4.2 新增 trusted util 的门槛

只有满足下面任一条件，才能新增 trusted util：

- Verus 当前确实无法直接吃下该 Rust 特性；
- raw pointer / bitfield / FFI 边界无法在本轮继续下推；
- 这是收缩现有更大 trusted wrapper 的中间步骤。

同时必须满足：

- 合同尽量小；
- 影响面尽量 object-local；
- 明确记录后续如何继续下推。

### 4.3 不再新增函数级 `trusted_call_*`

默认禁止再新增新的“大函数级 trusted call wrapper”。

例外只允许在下面情况出现：

- 当前必须保留旧签名；
- 同名函数暂时无法承载合同；
- 且该 wrapper 是短期过渡，不是长期 API。

优先顺序应改成：

- 同名函数临时 attribute 合同
- 小 trusted util
- 原生 Verus helper

而不是：

- 再包一层新的 `trusted_call_*`

### 4.4 proof 位置优先级

优先级从高到低如下：

1. 函数 body 同文件、同名附近。
2. 同文件 object-local helper / lemma。
3. `specs/` 中的抽象合同与 packaging lemma。
4. `refinement_bridge.rs` 中真正不可避免的桥接引理。

也就是说，除非某个结论本质上是 bridge 级别的，否则不要优先往 `refinement_bridge.rs` 塞。

## 5. 当前 `cspace` 的阶段判断

截至 2026-04-24，`sel4_cspace` 当前处在“阶段 B 到阶段 C 之间”。

已经具备的部分：

- `cte_insert / cte_move / cte_swap / resolve_address_bits` 已经把合同挂回同名函数；
- `*_refined(...)` 已经位于 `cte.rs`，并且本身是签名式合同；
- `refinement_bridge.rs` 已经从“入口承载层”收缩成“关系与引理层”。

仍然属于过渡态的部分：

- 主函数合同当前仍大量依赖 `#[verus_spec(...)]`；
- 主函数 body 还没有完全进入原生 Verus 证明；
- `external_body` 还在帮助隔离当前 Verus 不擅长的 Rust 特性。

因此下一阶段的正确方向不是再加 bridge，而是：

- 缩小 attribute 合同；
- 把 proof 推进到 body-local helper；
- 逐步把主函数收敛成签名式 Verus 本体。

## 6. 对四个目标函数的直接约束

### 6.1 `resolve_address_bits`

优先作为第一条“从 attribute 过渡到签名式本体”的样板。

原因：

- 分支语义最贴近 `l4v`；
- read-only；
- 适合先把 getter / step relation 拆小。

### 6.2 `cte_insert`

优先作为第一条 mutating-op 的签名式本体化路线。

原因：

- 局部影响面最清楚；
- `src / dest / old-next` 结构已经稳定；
- 适合先练通“小 trusted util + 本地 proof”。

### 6.3 `cte_move`

在 `cte_insert` 模板稳定后推进。

重点：

- `src` 清空；
- `dest` 接管；
- 邻接节点更新。

### 6.4 `cte_swap`

最后推进。

原因：

- 两侧邻接关系一起动；
- case bookkeeping 最多；
- 更适合在前两个 mutating-op 风格收敛后再做。

## 7. 一条简化判断规则

以后写 `cspace` proof 时，先问自己三个问题：

1. 这个合同能不能直接写在函数签名里？
2. 这个 trusted 点能不能缩成更小的 object-local util？
3. 这个 proof 是不是应该离 body 更近，而不是继续放进 bridge？

如果三个问题的答案分别是：

- 能；
- 能；
- 是；

那就不要继续加 `#[verus_spec(...)]`、不要继续加大 wrapper、也不要继续把主要语义留在 bridge 里。
