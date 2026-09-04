# 论文结构批改与初步 Review

本文档用于整理老师已明确指出的三个共性问题，并结合当前稿件 [thesis_full_draft.md](/workspace/rel4_kernel/docs/thesis_full_draft.md) 做一轮结构性 review。这里先聚焦“本科毕业设计论文写作是否清楚、是否方便审稿老师总览”，不评价技术内容本身的对错。

## 一、已确认的三个共性问题

### 1. 章节标题与下级标题缺少清晰对应关系

不少章节的大标题本身包含多个并列任务，但下属小节没有按这些任务展开，导致读者无法从目录直接判断每一节究竟在讲什么。最典型的是第 4 章：

- 第 4 章标题同时提出 `架构设计`、`形式化建模`、`证明框架` 三项内容 [thesis_full_draft.md:320](/workspace/rel4_kernel/docs/thesis_full_draft.md:320)
- 但 4.1 到 4.7 没有按这三项分组，而是平铺成 `验证架构总览`、`抽象建模`、`统一状态表示`、`不变量体系`、`证明义务分解`、`循环型证明`、`可信边界` [thesis_full_draft.md:322](/workspace/rel4_kernel/docs/thesis_full_draft.md:322)

这会造成“章标题是分类式的，子标题却不是分类式的”这一结构问题。

### 2. 各章功能分工不清，实现、结果、分析、边界互相交叉

后半部分没有严格做到“一章只承担一种主要功能”，因此审稿人会分不清哪些地方是在写“做了什么”，哪些地方是在写“得到了什么”，哪些地方是在写“这些结果说明什么、能推到哪里”。

最明显的是：

- 第 5 章标题是 `验证实现与核心结果`，本身就把“实现”和“结果”写在一起 [thesis_full_draft.md:534](/workspace/rel4_kernel/docs/thesis_full_draft.md:534)
- 第 6 章又是 `评估与结果分析`，继续解释结果 [thesis_full_draft.md:614](/workspace/rel4_kernel/docs/thesis_full_draft.md:614)
- 第 7 章是 `讨论、边界与扩展方向`，又在解释结论边界和局限 [thesis_full_draft.md:711](/workspace/rel4_kernel/docs/thesis_full_draft.md:711)

结果就是：同一类“结果解释”被拆散到第 5、6、7 章，整体逻辑不够直观。

### 3. 前两章出现了过多属于“本文自己的工作”的内容

如果按本科毕业设计论文最稳妥的写法，第 1、2 章更适合承担背景、相关研究、基础概念、问题引入等铺垫功能，不宜过早大量展开“本文做了什么”“本文如何设计”“本文方法如何定位”。

当前稿件中，前两章已经较多进入本文自己的方案与方法表达，例如：

- 第 1 章中直接设置 `研究问题`、`本文研究内容`、`本文主要贡献` [thesis_full_draft.md:161](/workspace/rel4_kernel/docs/thesis_full_draft.md:161) [thesis_full_draft.md:172](/workspace/rel4_kernel/docs/thesis_full_draft.md:172) [thesis_full_draft.md:181](/workspace/rel4_kernel/docs/thesis_full_draft.md:181)
- 第 2 章中的 `Verus 对本文验证任务的适配性` 与 `本文的方法论定位` 已经明显转入“本文如何做”的说明 [thesis_full_draft.md:233](/workspace/rel4_kernel/docs/thesis_full_draft.md:233) [thesis_full_draft.md:239](/workspace/rel4_kernel/docs/thesis_full_draft.md:239)

这不一定学术上错误，但从本科论文审稿习惯看，会让前两章显得“太早进入自己的东西”。

## 二、针对当前稿件的初步 Review

下面按严重程度给出当前最需要处理的结构问题。

### 1. 第 4 章的标题体系最容易让审稿老师失去目录把握

位置：

- 第 4 章标题 [thesis_full_draft.md:320](/workspace/rel4_kernel/docs/thesis_full_draft.md:320)
- 4.1 到 4.7 [thesis_full_draft.md:322](/workspace/rel4_kernel/docs/thesis_full_draft.md:322) [thesis_full_draft.md:363](/workspace/rel4_kernel/docs/thesis_full_draft.md:363) [thesis_full_draft.md:375](/workspace/rel4_kernel/docs/thesis_full_draft.md:375) [thesis_full_draft.md:385](/workspace/rel4_kernel/docs/thesis_full_draft.md:385) [thesis_full_draft.md:399](/workspace/rel4_kernel/docs/thesis_full_draft.md:399) [thesis_full_draft.md:488](/workspace/rel4_kernel/docs/thesis_full_draft.md:488) [thesis_full_draft.md:498](/workspace/rel4_kernel/docs/thesis_full_draft.md:498)

问题：

- 章标题有三个并列维度，但节标题没有按三类内容分别归档。
- 4.3 `管理器层与统一状态表示` 和 4.4 `不变量体系设计` 在功能上更接近“建模”，但它们被并列摆在“架构设计”和“证明框架”之间，读者无法一眼看出关系。
- 4.7 `可信边界、实例化与扩展空间` 又已经接近“边界讨论”或“后续扩展”，与本章前三个任务不完全同类。

影响：

- 读者看目录时必须自己推断节与章之间的归属关系。
- 对不熟悉技术细节的审稿老师而言，这一章会显得“内容很多，但组织方式不直”。

建议：

- 要么缩小第 4 章标题，只保留一个主任务。
- 要么保留现标题，但把小节改成三组，例如“4.1 架构设计”“4.2 形式化建模”“4.3 证明框架”，并把现有内容重组进这三组。

### 2. 第 5 到第 7 章没有形成清楚的“实现-结果-分析/局限”分章

位置：

- 第 5 章 [thesis_full_draft.md:534](/workspace/rel4_kernel/docs/thesis_full_draft.md:534)
- 第 6 章 [thesis_full_draft.md:614](/workspace/rel4_kernel/docs/thesis_full_draft.md:614)
- 第 7 章 [thesis_full_draft.md:711](/workspace/rel4_kernel/docs/thesis_full_draft.md:711)

问题：

- 第 5 章标题写成 `验证实现与核心结果`，把“怎么做”和“得到什么”放在同一章。
- 5.1 `从代表性案例到结果评估的组织原则` 甚至把“结果评估”的说法提前带入第 5 章 [thesis_full_draft.md:536](/workspace/rel4_kernel/docs/thesis_full_draft.md:536)。
- 第 6 章继续写 `评估与结果分析`，6.6 又是 `结果分析与经验总结`，说明结果解释仍在继续 [thesis_full_draft.md:703](/workspace/rel4_kernel/docs/thesis_full_draft.md:703)。
- 第 7 章再从 `结论边界`、`研究局限`、`更广义系统验证目标` 继续解释这些结果能外推到哪里 [thesis_full_draft.md:713](/workspace/rel4_kernel/docs/thesis_full_draft.md:713) [thesis_full_draft.md:729](/workspace/rel4_kernel/docs/thesis_full_draft.md:729) [thesis_full_draft.md:739](/workspace/rel4_kernel/docs/thesis_full_draft.md:739)。

影响：

- 审稿老师无法快速区分“第 5 章到底是实现章，还是结果章”。
- 结果的“呈现、评价、限制解释”被分散到三章，不利于快速总览。

建议：

- 最稳妥的本科写法是明确拆成：
- 第 5 章写“系统/验证实现”
- 第 6 章写“实验/验证结果”
- 第 7 章写“讨论、局限与展望”

### 3. 第 1 章已经过早进入本文自身工作，削弱了“绪论”的铺垫感

位置：

- 1.3 `问题定义与研究问题` [thesis_full_draft.md:161](/workspace/rel4_kernel/docs/thesis_full_draft.md:161)
- 1.4 `本文研究内容` [thesis_full_draft.md:172](/workspace/rel4_kernel/docs/thesis_full_draft.md:172)
- 1.5 `本文主要贡献` [thesis_full_draft.md:181](/workspace/rel4_kernel/docs/thesis_full_draft.md:181)
- 1.6 `论文结构安排` [thesis_full_draft.md:190](/workspace/rel4_kernel/docs/thesis_full_draft.md:190)

问题：

- 第 1 章不仅引出问题，还已经较完整地交代了“本文做了什么、贡献是什么、每章怎么对应研究问题”。
- 对研究型论文这不算离谱，但如果目标是“本科论文审稿友好”，这一章的信息密度偏高，也偏“作者视角”。

影响：

- 绪论会显得不像“先讲背景、再引出问题”，而像“已经开始介绍自己的成果框架”。

建议：

- 保留研究问题与结构安排可以，但要明显压缩。
- `本文研究内容` 和 `本文主要贡献` 可以考虑并入第 3 章或后移为更简短的过渡表述。

### 4. 第 2 章后半已经从“背景基础”转入“本文方案预告”

位置：

- 2.4 `Verus 对本文验证任务的适配性` [thesis_full_draft.md:233](/workspace/rel4_kernel/docs/thesis_full_draft.md:233)
- 2.5 `本文的方法论定位` [thesis_full_draft.md:239](/workspace/rel4_kernel/docs/thesis_full_draft.md:239)

问题：

- 2.4 已经不是纯背景，而是在解释“为什么本文选这个工具、这个任务和这个方法匹配”。
- 2.5 更明确提出“本文的方法论定位”，已经是作者自己的方案论证。

影响：

- 第 2 章作为“背景与相关技术基础”的边界被冲淡。
- 第 3 章还没开始，读者已经提前接触到本文的方法设计判断。

建议：

- 若老师强调“前两章尽量不要有自己的东西”，则 2.4 和 2.5 需要明显收缩。
- 最简单的处理方式，是把第 2 章收回到“技术基础”，把方法选择与方案定位放入第 3 章或第 4 章。

### 5. 第 3 章承担了“研究对象、范围、边界、成果口径”四种功能，作为过渡章略重

位置：

- 3.1 到 3.4 [thesis_full_draft.md:245](/workspace/rel4_kernel/docs/thesis_full_draft.md:245) [thesis_full_draft.md:255](/workspace/rel4_kernel/docs/thesis_full_draft.md:255) [thesis_full_draft.md:290](/workspace/rel4_kernel/docs/thesis_full_draft.md:290) [thesis_full_draft.md:302](/workspace/rel4_kernel/docs/thesis_full_draft.md:302)

问题：

- 第 3 章作为“自己的东西开始出现”的入口是合理的，但目前同时承担对象说明、范围划定、难点分析、成果边界说明，内容偏满。
- 尤其 3.4 `研究范围与成果边界` 已经提前引入大量结论口径，和第 7 章边界讨论存在前后呼应，但也有一定重复。

影响：

- 第 3 章容易显得“既像问题分析章，又像结论边界预告章”。

建议：

- 第 3 章保留“研究对象 + 问题分析 + 范围界定”即可。
- 更强的“成果边界解释”可以放到第 7 章统一收口。

### 6. 目录中的若干标题使用了抽象并列词，审稿友好度不够高

位置：

- `架构设计、形式化建模与证明框架` [thesis_full_draft.md:320](/workspace/rel4_kernel/docs/thesis_full_draft.md:320)
- `验证实现与核心结果` [thesis_full_draft.md:534](/workspace/rel4_kernel/docs/thesis_full_draft.md:534)
- `评估与结果分析` [thesis_full_draft.md:614](/workspace/rel4_kernel/docs/thesis_full_draft.md:614)
- `讨论、边界与扩展方向` [thesis_full_draft.md:711](/workspace/rel4_kernel/docs/thesis_full_draft.md:711)

问题：

- 这些标题都不是错，但都在同时容纳两到三个论述任务。
- 对熟悉领域的人来说能看懂，对非本方向审稿老师来说不够“一眼明白”。

建议：

- 尽量改成更单功能、更朴素的标题。
- 本科论文里，“方法设计”“系统实现”“实验结果”“讨论与局限”这类标题通常更稳。

## 三、整体判断

当前稿件的主要问题不是内容不足，而是“结构表达密度过高、审稿路径不够直”。更具体地说：

- 技术内容已经比较完整，但章节承担的功能还没有完全分开。
- 作者自己能顺着研究逻辑看懂，但审稿老师未必能顺着目录快速抓住“每章只做什么”。
- 如果后续修改优先级有限，应优先处理目录和章标题，而不是先增补更多理论细节。

## 四、建议的修改优先级

1. 先重排第 4 到第 7 章的章标题和节标题，让“方法、实现、结果、讨论/局限”分开。
2. 再压缩第 1、2 章中的“本文自己的东西”，把方案性表达后移到第 3 章及以后。
3. 最后处理各章内部重复表述，尤其是“边界、结论口径、外推限制”在第 3、6、7 章之间的重复。

## 五、下一步可直接开展的工作

如果继续往下改，最适合的顺序是：

1. 先重写第 1 章到第 7 章目录。
2. 再根据新目录决定哪些段落前移、后移、合并或删减。
3. 最后再润色语言，不然容易在旧结构上反复修词。
