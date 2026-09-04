# 05 Implementation Roadmap

## 1. 总策略

这轮路线图不再按“先扩 spec，再给 verified 层包接口”推进。

新的固定顺序是：

1. 冻结 subsystem 骨架
2. 固化对象层
3. 固化 patch-centered mutator
4. 吸收 derive / resolve
5. 压缩 `specs/cspace_ops/*`
6. 清理 compat / bridge

## 2. 阶段 A：冻结最终骨架

### 2.1 目标

先把“谁是主语”定死，避免边做边漂移。

### 2.2 需要固定的角色

- `abstract_cspace.rs`
  只做抽象模型与全局不变量
- `verified/{cap,mdb,slot}.rs`
  只做局部对象语义
- `verified/cspace.rs`
  只做 subsystem 语义与全局恢复
- `verified/{derive,resolve,insert}.rs`
  只做薄操作壳
- `repr/*`
  只做 view/result/helper
- `specs/cspace_ops/*`
  只做过渡语义锚点

### 2.3 验收

- 新代码默认写进 `verified/*`，不是 `specs/cspace_ops/*`
- 新的全局恢复 lemma 默认写进 `verified/cspace.rs`

## 3. 阶段 B：固化对象层

### 3.1 目标

让对象层真正像 atmo 里的局部对象，而不是 spec façade。

### 3.2 重点

- `CapRef` 的 query 继续本地化
- `MdbRef` 只保留 MDB 局部语义
- `SlotRef` 成为 slot 级 query 和局部 post 的主语
- 能放进对象的 ghost，不再挂在操作层参数里

### 3.3 验收

- 对象层 API 稳定围绕 `view()/wf()/query/post`
- `repr/*` 不再长出第二套对象语义

## 4. 阶段 C：固化 patch-centered mutator

### 4.1 目标

把所有核心 mutator 都改成：

- patch first
- object-local post second
- subsystem recovery last

### 4.2 当前起点

`cte_insert` 和 `insert_new_cap` 已经开始走这条路：

- [verified/cspace.rs](/workspace/rel4_kernel/sel4_cspace/src/verified/cspace.rs)
- [verified/slot.rs](/workspace/rel4_kernel/sel4_cspace/src/verified/slot.rs)
- [verified/insert.rs](/workspace/rel4_kernel/sel4_cspace/src/verified/insert.rs)

### 4.3 继续要做的事

- 抽更统一的 rewiring combinator
- 让 `cte_move` / `cte_swap` 直接复用这套 patch 逻辑
- 把恢复证明继续收口到 `CspaceCtx`

### 4.4 验收

- insert/move/swap 不再各维护一套大的 rewrite skeleton
- patch 成为 mutator 的统一中间语义

## 5. 阶段 D：吸收 derive / resolve

### 5.1 derive

目标形态：

- `SlotRef` 给出 derive 所需局部语义
- `CspaceCtx` 给出上下文
- `verified/derive.rs` 只剩薄壳

### 5.2 resolve

目标形态：

- `CspaceCtx` 成为 `resolve_address_bits` 主语
- `CapRef` 提供 root-cap 局部语义
- `verified/resolve.rs` 只剩薄壳

### 5.3 验收

- `derive/resolve` 的主叙事不再在 `specs/cspace_ops/*`

## 6. 阶段 E：压缩 `specs/cspace_ops/*`

### 6.1 目标

把这些文件从“主证明中心”降成“过渡语义锚点”。

### 6.2 具体原则

保留：

- precondition
- 抽象效果摘要
- 少量必要 bridge/helper

删减或迁出：

- 全局恢复 proof body
- 大段 rewiring 展开
- delegate farm
- 平行操作的重复骨架

### 6.3 验收

- `src/specs/cspace_ops/insert.rs` 不再是本项目最厚的地方
- 调用方主要经由 `verified/*`，不是直接经由操作 spec

## 7. 阶段 F：清理 compat / bridge

### 7.1 目标

让旧边界真的退回边界层。

### 7.2 处理对象

- `refinement_bridge.rs`
- `interface.rs`
- `compatibility.rs`

### 7.3 验收

- 它们只做 observer / glue / façade
- 主证明不再回流到这些文件

## 8. 过程中的硬规则

后续每推进一轮，都遵守下面几条：

- 不再新增厚 `specs/cspace_ops/*` 证明体
- 不为了“先过 proof”临时加大 delegate 层
- 新 helper 优先放对象层或 subsystem 层
- 简单结构优先，不先引入额外 owner/resource 抽象

## 9. 当前阶段判断

从代码现状看，最值得继续推进的不是再堆更多 insert 细节，而是：

1. 让 `CspaceCtx` 成为更明确的全局恢复中心
2. 让 `move/swap` 共用 patch 语义
3. 把 `specs/cspace_ops/*` 继续压成薄层

这三件事做完，整个工程才会真正从“旧 spec 思维”切到 “atmo-style subsystem 思维”。
