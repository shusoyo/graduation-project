# `sel4_cspace` Review Standard

本文档固定 `sel4_cspace` 代码与证明 review 的默认标准。

这套标准适用于：

- `resolve / insert / move / swap / delete / revoke`
- `spec_util/*`
- `impl_*`
- `trusted boundary`
- 论文表述中的完成度判断

## Core Principle

默认先判断三件事，再给结论：

1. exec 语义是否仍然对
2. proof 结构是否合理
3. trusted boundary 是否清晰

不要一开始就只看 proof 能不能过，也不要只看代码是否“像以前”。

## Current Methodology Choice

当前项目默认采用下面这套方法论：

- Verus 架构与证明组织，优先参考 `atmo`
- `l4v` 主要用来校准不变量、`requires/ensures` 强度、以及语义是否一致
- 当前阶段不把“复刻 `l4v` 风格 refinement tower”当成默认目标

也就是说，后续 review 和设计讨论里，默认先问：

- 这是不是一个 `atmo` 风格、可维护的 Verus 实现
- 它的 contract 和语义强度，是否已经被 `l4v` 校准到足够可靠

而不是先问：

- 它是不是已经长成 Isabelle / `l4v` 那套 refinement 组织

## Step 1: Exec First

review 时先看 runtime 主体，而不是先看 lemma。

默认动作：

1. 找当前实现的 canonical exec
2. 对照 `sel4_cspace/reference_0ca248f/src/cte.rs` 或用户指定的 old path
3. 看步骤顺序、局部变量 staging、patch 顺序、断言位置
4. 再把 helper mentally inline 一遍

评判标准：

- 不要求逐 token 一致
- 不要求 helper 展开后与老代码完全同构
- 但要求主语义、关键 patch 顺序、空槽检查、链路修补顺序大体一致

如果 helper inline 后已经看不出和旧实现的对应关系，要把这当成 review 风险点。

## Step 2: Proof Second

proof review 的重点不是“写得长不长”，而是结构是否稳。

优先检查：

- 前提是否清楚且稳定
- 后置是否在表达真正重要的语义
- proof 是否建立了 local post
- frame 语义是否明确
- 最后是否能合理收回 `wf`

偏好的证明形状是：

- runtime mutation
- local post
- changed-slot or changed-set semantic post
- frame preservation
- `wf` recovery

需要警惕的形状是：

- trace-style 证明过重
- 大量一次性 case split
- 把本应放在 spec 里的语义硬塞进 proof block
- contract 太弱，导致 proof 只能靠局部事实堆出来

如果 proof 只是“能过”，但结构明显会卡 solver 或难以维护，也应该作为 review finding。

## How To Use `l4v` And `atmo`

在这个项目里，`l4v` 和 `atmo` 都要参考，但参考方式不同。

### 它们在当前仓库里的位置

后续 AI 不应自己猜 `l4v` 和 `atmo` 在哪里，默认直接看下面这些路径。

`l4v` 参考材料位置：

- `sel4_cspace_backup/aux/l4v_cspace_extracted/spec/abstract/CSpace_A.thy`
- `sel4_cspace_backup/aux/l4v_cspace_extracted/proof/invariant-abstract/CSpace_AI.thy`

如果问题是 `resolve` 的抽象语义，也应优先看：

- `sel4_cspace_backup/specs/cspace_ops/resolve.rs`

`atmo` 参考材料位置：

- `sel4_cspace_backup/aux/atmosphere-main/kernel/verified/process_manager/impl_base.rs`
- `sel4_cspace_backup/aux/atmosphere-main/kernel/verified/process_manager/container_util_t.rs`
- `sel4_cspace_backup/aux/atmosphere-main/kernel/verified/bridge.rs`

默认规则：

- 讨论不变量目标、`requires/ensures` 强度、语义是否和旧实现 / 抽象语义一致时，先读 `l4v`
- 讨论 Verus proof 架构、manager ghost 组织、exec 形状、trusted bridge 切分时，先读 `atmo`

如果只写一句“参考 `l4v`”，很容易让后续 AI 误解成：

- 把 `l4v` 的抽象函数逐行翻译成 Verus
- 把 Isabelle 里的证明结构原样搬过来
- 把 monadic spec 和 proof script 的组织形式也照搬

这不是本项目要的方向。

### `l4v` 主要用来校准什么

`l4v` 主要用来校准：

- 抽象语义是否完整
- case split 是否覆盖全
- invariant 目标是否足够强
- `requires/ensures` 是否够强
- 当前实现语义是否和旧 seL4 / 抽象意图一致

也就是说，`l4v` 在这里首先是：

- semantic reference
- proof-strength reference
- contract-strength reference
- semantic-consistency reference

而不是：

- implementation template
- code structure template
- Verus proof script template

### `atmo` 主要用来校准什么

`atmo` 主要用来校准：

- Verus 里怎样组织 manager-based ghost state
- 如何让 exec 函数尽量保持 runtime 形状
- 如何直接在 exec 本体上证明
- 如何把 proof 分成 local post / frame / wf recovery
- trusted bridge 该收在什么层

也就是说，`atmo` 在这里首先是：

- Verus architecture reference
- proof organization reference
- trusted-boundary engineering reference

而不是：

- seL4 语义金标准

### 默认分工

默认按下面的分工来理解：

- `l4v`：告诉你语义和 contract 至少该强到哪里
- `atmo`：告诉你在 Verus 里该怎么组织实现与证明

更具体一点：

- 当问题是“这个函数最终该满足什么语义、哪些 fault/success case 要覆盖、前后条件该强到什么程度”，优先看 `l4v`
- 当问题是“这套 ghost/manager/lemma 结构在 Verus 里写得对不对、会不会太重、trusted base 该怎么切、exec 该怎么保持贴近 runtime”，优先看 `atmo`

### 什么叫“参考 `l4v`”

在这个项目里，说“参考 `l4v`”，默认是指：

1. 参考抽象操作的语义分类
2. 参考要保持的不变量种类
3. 参考 `requires/ensures` 应强到什么程度
4. 参考当前实现语义是否和抽象意图一致
5. 在用户明确追问更高层 claim 时，再参考最终可支撑到什么系统性质

不是指：

1. 直接把 `l4v` 函数 port 成 Rust/Verus
2. 强迫当前代码长得像 Isabelle 里的 monad
3. 强迫所有 proof 都照 `l4v` 的 tactic 分解方式重写
4. 为了“像 `l4v`”而牺牲当前 exec 与老实现的接近性

### 什么叫“参考 `atmo`”

在这个项目里，说“参考 `atmo`”，默认是指：

1. 参考 manager ghost 架构
2. 参考 Verus 中的 proof decomposition
3. 参考 direct-exec proof 的组织方式
4. 参考如何把 trusted code 收敛到底层 bridge
5. 参考如何让 exec 代码不被 ghost 结构污染

不是指：

1. 默认 `atmo` 的证明强度就已经足够
2. 只要 proof 组织得像 `atmo`，就算完成 refinement
3. 因为 `atmo` 常直接证明 exec，就可以省掉抽象语义层

### `l4v` Refinement 何时再看

当前阶段，默认不把“有没有显式 `l4v` 风格 refinement tower”当成 review 失败条件。

只有在下面这些问题里，才把 `l4v` refinement 完整度重新拿出来当更高一层的标尺：

- 用户明确问 whole-kernel / `l4v`-level claim
- 用户明确问论文里能不能说 end-to-end / equivalent
- 你要判断当前工作是否已经不只是 manager-level verified core

在默认的日常实现和 review 里，更重要的是：

- 语义有没有讲清
- contract 强度够不够
- manager-level `wf` 有没有稳定收回
- trusted boundary 是否在缩小
- exec 是否仍贴近 old implementation

这时默认继续采用 Verus / `atmo` 风格组织就可以，不要求先补出完整 `l4v` 外形。

### review 时怎么落地

在 review 中，如果看到有人说“这里应该参考 `l4v`”，默认应进一步追问：

- 这里参考的是不变量 / contract / 语义一致性，还是参考代码结构？
- 这里缺的是语义强度，还是只是 proof 组织不好？
- 这里该不该继续保持当前 exec 贴近旧实现？

默认判断规则：

- 如果当前问题是“语义说不清、case 没覆盖、contract 太弱、和旧实现/抽象意图对不上”，就指出应参考 `l4v`
- 如果当前问题是“proof 太重、ghost 太散、TCB 切得不好”，就指出应参考 `atmo`
- 如果两者都涉及，就明确写成：
  `语义和 contract 校准参考 l4v，Verus 组织参考 atmo`

## Step 3: Boundary Third

必须区分三层：

1. manager-level verified core
2. public wrapper / compatibility shell
3. whole-kernel or l4v-level strength

review `external_body` 时，不要只问“有没有 external”，而要问它在决定什么。

风险较低的 trusted code：

- pointer bridge
- slot permission bridge
- bitfield getter / setter bridge
- exception code bridge
- primitive read-only observer

风险较高的 trusted code：

- whole-loop 业务语义
- 高层 capability semantic decision
- 直接承载 insert/move/swap/delete/revoke 主语义的黑盒 helper

如果一个 public wrapper 无前提地调用了一个 manager 内部有强前提的函数，也要明确指出这层断裂。

## Default Review Output

输出顺序固定为：

1. findings
2. assumptions or residual risks
3. brief summary

每条 finding 应满足：

- 给文件/行号
- 说明影响
- 区分语义风险、proof 风险、boundary 风险

如果没有发现实质性问题，要明确说：

- 没有发现明显语义问题

但仍要补一句 residual risk，例如：

- public wrapper 仍只是 runtime compatibility boundary
- semantic bridge 仍太黑
- manager-level 强度还没接到 whole-kernel

## Static Review Default

如果用户说的是 review，而不是修复或验证：

- 默认做静态 inspection
- 默认不跑 `cargo xtask verify`
- 只有在用户明确要求时才跑验证

## Completion Language

在这个项目里，完成度要谨慎表述。

推荐用语：

- `done`
- `mostly done`
- `in progress`
- `planned`
- `manager-level verified`
- `public wrapper still weak`
- `whole-kernel strength not yet claimed`

不推荐直接说：

- `fully verified CSpace`

除非已经同时满足：

- core operations 收齐
- delete core 收齐
- trusted boundary 显式且较小
- public wrapper 与 manager-level proof domain 对齐

## Repository-Specific Rules Of Thumb

当前 repo 里，默认这样判断：

- `resolve` 是最强的一条线，只要它保持 cap-centric loop-direct + abstract refinement
- `insert / move` 主要看 exec 是否仍贴近 old implementation，以及 manager-level `wf` 是否收回
- `swap` 主要看 runtime->exact-post semantic bridge 是否还太黑
- `delete contract` 的完成不等于 delete proof 完成
- `trusted boundary plan` 写成文档不等于已经执行
- 默认设计实践优先参考 `atmo`，不是优先追 `l4v` 的外形
- `l4v` 默认只校准语义、不变量和 contract 强度，不默认要求补齐显式 refinement tower
- 讨论 Verus proof 架构时，默认允许并鼓励更接近 `atmo` 的组织方式，只要语义强度没有退化
