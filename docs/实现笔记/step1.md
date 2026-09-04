
可以，下面这份你可以直接当“第一个任务”的执行清单来用。

**Task 1 Checklist**

**A. 确定分析范围**
- [ ] 明确本阶段只分析三层：`sel4_cspace`、`cnode syscall invocation`、`untyped invocation`
- [ ] 明确本阶段只覆盖 8 个操作：`Copy`、`Mint`、`Move`、`Mutate`、`Rotate`、`Delete`、`Revoke`、`Retype`
- [ ] 建一个笔记文档，准备持续记录“函数入口 / 调用链 / 职责归属 / 验证影响”

**B. 收集代码入口**
- [ ] 阅读 [invoke_cnode.rs](/workspace/rel4_kernel/kernel/src/syscall/invocation/invoke_cnode.rs)，找出 `Copy/Mint/Move/Mutate/Rotate/Delete/Revoke` 对应入口函数
- [ ] 阅读 [invoke_untyped.rs](/workspace/rel4_kernel/kernel/src/syscall/invocation/invoke_untyped.rs)，找出 `Retype` 对应入口函数
- [ ] 阅读 [interface.rs](/workspace/rel4_kernel/sel4_cspace/src/interface.rs)，列出对外 capability 原语接口
- [ ] 阅读 [cte.rs](/workspace/rel4_kernel/sel4_cspace/src/cte.rs)，列出底层 `cte`/mdb 维护函数
- [ ] 阅读 [structures.rs](/workspace/rel4_kernel/sel4_cspace/src/structures.rs)，确认 capability/slot/mdb 相关数据结构

**C. 建“操作-函数映射表”**
- [ ] 为 `Copy` 记录：syscall 入口、调用的 cspace 原语、涉及的数据结构
- [ ] 为 `Mint` 记录：syscall 入口、调用的 cspace 原语、涉及的数据结构
- [ ] 为 `Move` 记录：syscall 入口、调用的 cspace 原语、涉及的数据结构
- [ ] 为 `Mutate` 记录：syscall 入口、调用的 cspace 原语、涉及的数据结构
- [ ] 为 `Rotate` 记录：syscall 入口、调用的 cspace 原语、涉及的数据结构
- [ ] 为 `Delete` 记录：syscall 入口、调用的 cspace 原语、涉及的数据结构
- [ ] 为 `Revoke` 记录：syscall 入口、调用的 cspace 原语、涉及的数据结构
- [ ] 为 `Retype` 记录：untyped 入口、对象创建逻辑、cap 插入逻辑、涉及的数据结构

建议每个操作都按这 6 项记：
- [ ] 入口函数是谁
- [ ] 参数检查在哪里做
- [ ] slot 解析在哪里做
- [ ] 真正修改 cap/slot 的函数是谁
- [ ] mdb/CDT 维护在哪里做
- [ ] 这个操作是否涉及对象创建 / untyped 更新

**D. 画调用链**
- [ ] 给 `Copy` 画出从 invocation 到 `sel4_cspace` 的调用链
- [ ] 给 `Mint` 画出调用链
- [ ] 给 `Move` 画出调用链
- [ ] 给 `Mutate` 画出调用链
- [ ] 给 `Rotate` 画出调用链
- [ ] 给 `Delete` 画出调用链
- [ ] 给 `Revoke` 画出调用链
- [ ] 给 `Retype` 画出调用链，并单独标出“对象创建部分”和“cspace 相关部分”

**E. 做职责归类**
把每段逻辑分类到三层之一。

- [ ] 标出哪些逻辑属于 `sel4_cspace` 核心原语
- [ ] 标出哪些逻辑属于 syscall 编排层
- [ ] 标出哪些逻辑属于 untyped / 对象创建层

判定时重点看：
- [ ] 是否直接修改 `slot/cte`
- [ ] 是否直接修改 capability 内容
- [ ] 是否维护 mdb/CDT 派生关系
- [ ] 是否只是做参数检查和错误处理
- [ ] 是否涉及对象创建和 untyped 状态更新

**F. 产出职责矩阵**
- [ ] 建一张“操作 × 职责类型”矩阵
- [ ] 对每个操作填写：参数检查、slot 解析、cap 派生、slot 更新、mdb/CDT 维护、对象创建、untyped 更新、错误处理
- [ ] 对每一格标注归属层：`cspace` / `syscall` / `untyped`

**G. 单独分析 `Retype`**
- [ ] 分离 `Retype` 中“对象创建”部分
- [ ] 分离 `Retype` 中“capability 生成/插入”部分
- [ ] 分离 `Retype` 中“派生关系维护”部分
- [ ] 写清楚：`Retype` 是“对象创建 + capability 安装”的复合操作，不是纯 CSpace 原语

**H. 从验证视角分析问题**
- [ ] 写出“边界不稳定”的具体表现
- [ ] 写出“同一语义分散”的具体表现
- [ ] 写出“抽象状态难统一”的具体表现
- [ ] 写出“证明职责不清楚”的具体表现
- [ ] 每一点都配至少一个代码层面的例子

**I. 形成文档初稿**
文档建议至少包含这些小节：

- [ ] 分析范围与对象
- [ ] capability 相关代码当前分布
- [ ] 各操作调用路径与函数映射
- [ ] 职责矩阵
- [ ] 当前结构对验证的影响
- [ ] 对后续重构边界的启示

**J. 验收标准**
你做到下面这些，就算第一个任务完成得比较扎实了：

- [ ] 8 个操作都有明确调用链
- [ ] 8 个操作都有职责归属说明
- [ ] 有一张完整职责矩阵
- [ ] `Retype` 被拆成 cspace 相关部分和 untyped 相关部分
- [ ] 能清楚说明“为什么当前结构不利于验证”
- [ ] 能据此提出后续重构边界，而不是只做代码罗列


**任务目标**
你这一阶段要回答 4 个问题：

1. capability 相关逻辑现在分散在哪几层。
2. 每个操作现在到底是谁在做主逻辑，谁只是在包装调用。
3. 哪些职责应该归 `sel4_cspace`，哪些应该留在 syscall / untyped 层。
4. 现有结构为什么会让后续建模和验证变难。

**建议产出**
最后至少整理出 5 份东西：

1. 一张“代码分布图”
2. 一张“操作-函数映射表”
3. 一张“职责矩阵”
4. 一份“问题分析”
5. 一份“重构前结构分析文档”

---

**第一步：先限定分析范围**
你不要一上来全仓库漫游，先把范围卡住。只看三层：

1. `sel4_cspace`
   重点看 [cte.rs](/workspace/rel4_kernel/sel4_cspace/src/cte.rs)、[interface.rs](/workspace/rel4_kernel/sel4_cspace/src/interface.rs)、[structures.rs](/workspace/rel4_kernel/sel4_cspace/src/structures.rs)
2. cnode syscall invocation
   重点看 [invoke_cnode.rs](/workspace/rel4_kernel/kernel/src/syscall/invocation/invoke_cnode.rs)
3. untyped invocation
   去找 kernel 里和 `Retype`、`Untyped` 相关的 invocation 文件

你这一阶段不要深挖所有对象类型，只聚焦 capability 相关操作：

`Copy`、`Mint`、`Move`、`Mutate`、`Rotate`、`Delete`、`Revoke`、`Retype`

---

**第二步：给每个操作画“调用链”**
你现在最该做的是给每个操作画一条从 syscall 到 cspace 原语的链。

你可以按这个格式记：

- 操作名：`Copy`
- syscall 入口函数：
- invocation 层函数：
- 最终调用的 `sel4_cspace` 原语：
- 是否涉及 `derive_cap` / `cte_insert` / `cte_move` / `delete_all` / `revoke`：
- 是否涉及 untyped 状态更新：
- 当前主职责在哪一层：

你最终要得到类似这样的表：

| 操作 | syscall/invocation 层职责 | `sel4_cspace` 层职责 | untyped 层职责 |
|---|---|---|---|
| Copy | 参数检查、slot 解析、错误返回 | cap 派生、插入、mdb 更新 | 无 |
| Mint | 参数检查、badge/rights 处理 | 派生、插入、关系维护 | 无 |
| Move | 源/目标检查 | slot 移动、mdb 更新 | 无 |
| Delete | 调用入口、异常处理 | 删除 slot、递归删除相关 cap | 无 |
| Revoke | 调用入口 | 撤销 descendants | 无 |
| Retype | 参数检查、对象创建流程编排 | 新 cap 插入、派生关系维护 | untyped 消耗与对象创建 |

这里先不要怕填不完整，先做“粗表”，后面再补细节。

---

**第三步：区分“谁在做真正的语义”**
这是最关键的一步。你不能只说“函数在哪”，要判断 **这个函数是在做核心 capability 语义，还是只是在组织调用**。

你可以用下面这个标准：

**属于 `sel4_cspace` 核心原语的逻辑**

- 直接修改 slot / cte
- 直接修改 capability 内容
- 维护 capability derivation tree / mdb
- 处理 cap 插入、删除、移动、交换、撤销
- 做 `resolve_address_bits` 这种 CSpace 解析

**属于 syscall 编排层的逻辑**

- 参数合法性检查
- 从 message / syscall 参数中取值
- 把用户传入的 index/depth 解析成 slot
- 错误码转换
- 调用前后的线程状态处理
- 按 syscall 语义组合若干 cspace 原语

**属于 untyped / 对象创建层的逻辑**

- 计算是否有足够 untyped 空间
- 创建新对象
- 更新 untyped 的 free index / 子对象状态
- 在对象创建成功后，把新 cap 插入 cspace

这个判断标准非常重要，因为你论文里“为什么要下沉某些逻辑”就靠它支撑。

---

**第四步：逐个操作做责任归属**
你可以按操作来做，而不是按文件来做。建议顺序：

1. `Copy`
2. `Mint`
3. `Move`
4. `Mutate`
5. `Rotate`
6. `Delete`
7. `Revoke`
8. `Retype`

每个操作都回答这 6 个问题：

1. 用户态发起后，首先进入哪个 syscall / invocation 函数？
2. invocation 层做了哪些检查？
3. 最终落到了哪些 `sel4_cspace` 函数？
4. 哪些状态修改发生在 `sel4_cspace`？
5. 哪些状态修改还留在 syscall / untyped 层？
6. 如果后续要验证，这个操作最自然的“验证对象”应该是哪一层？

---

**第五步：做一张“职责矩阵”**
这个矩阵是你后面论文非常好用的一张图。建议按“操作 × 职责类型”来做。

你可以用这个模板：

| 操作 | 参数检查 | slot 解析 | cap 派生 | slot 更新 | mdb/CDT 维护 | 对象创建 | untyped 更新 | 错误处理/调度 |
|---|---|---|---|---|---|---|---|---|
| Copy | syscall | syscall | cspace | cspace | cspace | - | - | syscall |
| Mint | syscall | syscall | cspace | cspace | cspace | - | - | syscall |
| Move | syscall | syscall | - | cspace | cspace | - | - | syscall |
| Delete | syscall | syscall | - | cspace | cspace | - | - | syscall |
| Revoke | syscall | syscall | - | cspace | cspace | - | - | syscall |
| Retype | syscall | syscall | cspace | cspace | cspace | untyped | untyped | syscall |

这张表会直接支持你后面两件事：

1. 论证“当前边界在哪里”
2. 论证“为什么某些逻辑还应该继续下沉到 `sel4_cspace`”

---

**第六步：专门分析 `Retype`，但不要把它和普通 cap 操作混为一谈**
`Retype` 会是你第一个任务里最容易写乱的地方。你要把它拆成两半：

**属于 cspace 视角的部分**

- 新 capability 的生成
- 新 capability 插入目标 slot
- 建立派生关系 / mdb 关系

**不属于 cspace 核心原语的部分**

- 新对象实际创建
- untyped 资源划分
- untyped 剩余空间/状态更新

也就是说，你在第一阶段就要明确一句话：

`Retype` 不是一个纯粹的 CSpace 原语，而是“对象创建 + capability 安装”的复合操作。

这句话对你后面做规约边界非常重要。

---

**第七步：从“验证视角”写问题分析**
这一部分不是重复“代码分散”，而是要明确说出 **为什么分散会妨碍验证**。建议你从这 4 点写：

1. **边界不稳定**
   同一个操作的一部分语义在 syscall 层，一部分在 `sel4_cspace`，导致验证对象不好切。
2. **同一语义分散**
   比如 capability 插入、移动、删除相关语义如果既出现在 invocation 又出现在 cspace，就很难写统一规约。
3. **抽象状态难统一**
   你很难直接说“cspace 的抽象状态是什么”，因为有些关键状态变化发生在外层。
4. **证明职责不清楚**
   不知道该证明 syscall 组合正确，还是证明 cspace 原语正确，还是两层都要证明。

这里建议你每一点都写成“现象 + 导致的验证困难”两句结构，不要只写空泛结论。

---

**第八步：把结果整理成一份“重构前结构分析文档”**
你最终文档可以直接按这个结构写：

**1. 分析对象与范围**

- 本文分析 `sel4_cspace`、cnode syscall invocation、untyped invocation 中 capability 相关逻辑。
- 聚焦 `Copy`、`Mint`、`Move`、`Mutate`、`Rotate`、`Delete`、`Revoke`、`Retype`。

**2. 当前 capability 相关代码分布**

- `sel4_cspace` 当前承载哪些原语
- cnode invocation 当前承载哪些 syscall 级包装逻辑
- untyped invocation 当前承载哪些与 `Retype` 相关的逻辑

**3. 各操作当前调用路径与职责分布**

- 逐个操作列调用链
- 列操作-函数映射表

**4. 职责矩阵**

- 给出一张总表

**5. 当前结构对验证的影响**

- 边界不稳定
- 同一语义分散
- 抽象状态难统一
- 证明职责不明确

**6. 对后续重构的启示**

- 哪些逻辑可以继续下沉到 `sel4_cspace`
- 哪些逻辑应保留在 syscall 层
- `Retype` 哪些部分只纳入 cspace 视角，不整体下沉

---

**你实际执行时可以这样安排**
建议你按 5 个小阶段推进：

1. 先做“文件和函数清单”
   把相关文件、核心函数、操作入口先列出来。
2. 再做“逐操作调用链”
   每个操作都从入口一路跟到 `sel4_cspace`。
3. 然后做“职责归类”
   判断每段逻辑属于 cspace、syscall 还是 untyped。
4. 接着做“职责矩阵 + 验证困难分析”
   这是论文内容的核心。
5. 最后写成文档
   不要等全部做完才写，边分析边记表格。

---

**你现在最值得马上做的具体事项**
如果你今天就开始，建议按这个顺序：

1. 先把 [invoke_cnode.rs](/workspace/rel4_kernel/kernel/src/syscall/invocation/invoke_cnode.rs) 里和 `Copy/Mint/Move/Mutate/Rotate/Delete/Revoke` 对应的入口函数找出来。
2. 再在 [interface.rs](/workspace/rel4_kernel/sel4_cspace/src/interface.rs) 和 [cte.rs](/workspace/rel4_kernel/sel4_cspace/src/cte.rs) 里找到它们最终调用的原语。
3. 再去找 untyped invocation 里 `Retype` 的路径，拆出“对象创建部分”和“cap 插入部分”。
4. 先做出一版“操作-函数映射表”和“职责矩阵”。
5. 最后再写“为什么当前结构不利于验证”。

如果你愿意，我下一步可以直接帮你把这个第一任务进一步整理成一份 **可执行 checklist**，或者直接给你一份 **“重构前结构分析文档”模板**，你可以照着往论文里填。
