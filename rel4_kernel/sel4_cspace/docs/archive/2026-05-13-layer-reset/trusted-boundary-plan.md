# `sel4_cspace` Trusted Boundary Plan

本文档总结 `sel4_cspace` 当前证明边界的状态，并结合 `atmo` 的 `external_body` 使用风格，给出本项目后续应保留哪些 TCB、哪些函数必须继续证明的判断。

## Summary

当前 `sel4_cspace` 的状态不是“只有上层证明了、下层全没证明”，而是：

- `CSpaceManager` 主操作语义已经有较强的 verified core
- `resolve` 这条线已经基本收到 “loop exec + abstract semantics + refinement” 的形态
- 但 `cte`、`capability`、`arch` 里仍有不少 `external_body` 在承载语义

补充进度说明：

- capability generic relation 主体已经基本 verified 化
- `riscv64` 上 `arch_derive_cap` / `arch_mask_cap_rights` 这类 runtime body 已开始退出 whole-function `external_body`
- `cte.rs` 的 self-wrapper runtime forwarding 已改成用 erased local `CSpaceManager` receiver 走同一套 manager exec，不再假装存在 active runtime manager singleton；public wrapper API 暂不铺设 manager-level `requires/ensures`
- delete contract-first 的 witness / projector packaging 已闭合；当前全包验证仍停在既有 3 个非 wrapper proof 点（swap/delete `local_structural_wf` 与 delete rlimit）

因此，后续工作的关键不是机械地减少 `external_body` 数量，而是把 trusted boundary 收敛成：

- 表示桥
- 内存桥
- bitfield / pointer / hardware bridge

而不要继续让 trusted 代码决定高层 CSpace 语义。

## Full-Closeout Status

按当前这轮 “full closeout” 的收口口径，`trusted-boundary-plan` 的执行状态应写成下面这样：

- `manager-level verified core`
  已经收口到当前计划要求的主线强度：`resolve / insert / move / swap` 维持 verified core；`delete` 这条线的 contract-first packaging 已经闭合，caller 依赖的是显式 contract，而不是高层 semantic black box。
- `public wrapper / trusted boundary`
  当前明确降级为 runtime compatibility boundary：`cte.rs` wrapper 保留原 kernel-facing 签名并转发到 manager exec，但不声称 public-wrapper-level proof，也不把 manager `wf` / slot-domain 前提抬成 public API contract。
- `whole-kernel / l4v-level strength`
  本轮仍不声称拿到这一层。剩余 delete residual、arch runtime-match bridge、以及 `aarch64` 目标环境验证都继续单独归档，而不拿来混说成 whole-kernel closeout。

更细一点说：

- Phase 0 已完成：长期 bridge 与 shrink target 已冻结到文档和 checklist。
- Phase 1 已基本完成：generic capability relation 与 arch relation runtime body 已退出 major semantic black box；剩余的是 `get_cap_ptr` 这类长期 bridge，以及 arch relation 的 runtime-vs-spec bridge lemma。
- Phase 2 已基本完成：`update_data / arch_updatedata / arch_derive_cap / arch_mask_cap_rights` 这条 capability-transform 线已经不再是 large trusted semantic layer；剩余小块主要是 `zombie_new` 的 raw-constructor bridge，以及 `aarch64` 目标环境下的专门验证。
- Phase 3 已按“minimal compatibility wrapper”完成，而不是按“public wrapper contract 完成”或“verified thin wrapper”完成；public-wrapper-level claim 留到 whole-kernel proof state 接入之后。
- Phase 4 已完成到“delete dependency cone contracted closeout”这一层；`finalise_slot`、`set_empty -> wf` closeout、`reduce_zombie` immediate bridge 等 residual TCB 继续明确保留，不误说成 delete core 零 TCB。

## Methodology Baseline

本计划默认采用下面这套方法，而不是把 trusted shrink 理解成“把所有 external 逐个硬翻出来”。

- semantic and contract calibration from `l4v`
- Verus organization from `atmo`

这里的具体含义是：

- `l4v` 主要负责校准抽象语义、case coverage、invariant 目标、以及 `requires/ensures` 应强到什么程度
- `atmo` 主要负责校准 manager-based ghost 架构、direct-exec proof 形状、frame / local post / `wf` recovery 分层、以及 trusted bridge 该切在哪一层

本仓库里默认参考的上游位置应明确写死：

- `l4v` 语义与不变量参考：`sel4_cspace_backup/aux/l4v_cspace_extracted/spec/abstract/CSpace_A.thy`
- `l4v` invariant 强度参考：`sel4_cspace_backup/aux/l4v_cspace_extracted/proof/invariant-abstract/CSpace_AI.thy`
- `resolve` 抽象语义镜像：`sel4_cspace_backup/specs/cspace_ops/resolve.rs`
- `atmo` manager / proof structure 参考：`sel4_cspace_backup/aux/atmosphere-main/kernel/verified/process_manager/impl_base.rs`
- `atmo` helper / container 参考：`sel4_cspace_backup/aux/atmosphere-main/kernel/verified/process_manager/container_util_t.rs`
- `atmo` bridge 切分参考：`sel4_cspace_backup/aux/atmosphere-main/kernel/verified/bridge.rs`

因此，本计划的默认要求不是：

- 把 Isabelle monad 和 proof shape 逐段 port 到 Rust / Verus
- 为了“像 `l4v`”而重写当前已收住的 exec
- 要求 `external_body` 绝对清零

而是：

- 用 `l4v` 约束语义边界和 contract 强度
- 用 `atmo` 约束证明架构和 bridge 切分
- 让高层 CSpace 语义尽量退出 trusted 层

## Atmo 对比结论

对比 `sel4_cspace_backup/aux/atmosphere-main` 可以看到，`atmo` 并不是零 `external_body` 项目。

它的 `external_body` 主要集中在：

- 底层容器操作
- 内存/指针桥
- 初始化桥
- trap / hardware / bridge 层
- 一些底层数组、页表、MMU util

也就是说，`atmo` 的风格不是“把所有代码都展开证明”，而是：

- 把 external 留在底层 runtime / memory / hardware bridge
- 尽量不把高层业务语义本身留在 external

对 `sel4_cspace` 而言，这意味着：

- `trusted/*` 和 `impl_base.rs` 可以继续存在
- 但 `capability/*`、`arch/*`、`cte.rs` 里的语义 helper 不应长期保持 large trusted surface

## 当前分层判断

后续讨论 trusted boundary 时，默认继续区分三层：

- manager-level verified core
- public wrapper / trusted boundary
- whole-kernel or `l4v`-level strength

这三层不要混说。

### 1. Verified core

当前已经相对收住的主线包括：

- `src/cspace_manager/spec_util/*`
- `src/cspace_manager/impl_insert.rs`
- `src/cspace_manager/impl_move.rs`
- `src/cspace_manager/impl_swap.rs`
- `src/cspace_manager/impl_resolve.rs`

其中 `resolve` 已经收成：

- cap-centric exec loop
- abstract semantics in `spec_util/resolve.rs`
- concrete-to-abstract refinement
- 最小 trusted primitive set

### 2. Small trusted bridge

当前合理保留为 TCB 的部分主要是：

- `src/trusted/common.rs`
- `src/trusted/exception.rs`
- `src/trusted/resolve.rs`
- `src/trusted/mdb.rs`
- `src/cspace_manager/impl_base.rs` 中的 pointer / permission / slot patch bridge

这些模块的共同特点是：

- 读取 concrete cap/cte/exception/MDB 字段
- 把 runtime pointer / perm 连接到 ghost view
- 做最小内存写入桥
- 不直接决定 CSpace 业务语义

### 3. Residual Semantic TCB Layer

当前仍需要显式托管为 trusted boundary 的高层残余主要包括：

- `src/cte.rs` 中很多 legacy method
- `src/capability/zombie.rs`
- `src/arch/riscv64/mod.rs`
- `src/arch/aarch64/mod.rs`
- `src/cspace_manager/impl_base.rs` / `impl_delete.rs` 中 delete 线最后几处 semantic closeout bridge

这里和早期状态的区别是：

- `src/capability/mod.rs` 的 generic relation / transform 主体已经基本退出 whole-function semantic black box
- `src/cte.rs` 不再是“fake active runtime manager + 弱后置”的旧边界，而是“erased local manager receiver + no public proof contract”的 compatibility boundary
- delete 线剩下的是少数 residual semantic bridge，而不是整条路径的 contract 空洞

## 最终应该保留哪些 TCB

下面这些部分可以作为长期 TCB 保留。

### 表示桥

- `trusted_view_cap`
- `trusted_view_cte`
- `trusted_slot_perm_view`
- `trusted_cnode_lookup_slot_ptr`

理由：

- 这些是 concrete representation 到 ghost/spec view 的 observer bridge
- 它们天然依赖外部内存布局和 bitfield 编码

### 异常码 bridge

- `is_exception_none`
- `is_exception_lookup_fault`
- `is_exception_syscall_error`
- 以及对应的 `runtime_exception_*`

理由：

- 这些本质是 runtime enum / return code 到 proof classification 的桥

### `resolve` 的最小只读 primitive

保留在 `src/trusted/resolve.rs` 的这组函数是合理的：

- `runtime_cap_is_cnode`
- `runtime_cnode_cap_radix_bits`
- `runtime_cnode_cap_guard_bits`
- `runtime_cnode_cap_guard`
- `runtime_cnode_cap_level_bits`
- `runtime_cnode_lookup_slot_from_cap`
- `runtime_slot_cap_clone`
- `runtime_extract_bits_usize`

理由：

- 这组函数只描述“读到什么”
- 不承载 whole-loop `resolve` 语义
- 当前 `resolve` 已经把业务逻辑证明搬到 verified core

### `impl_base` 中的 pointer / permission bridge

例如：

- `slot_ref`
- `slot_mut`
- `take_slot_perm`
- `put_slot_perm`
- `borrow_slot_with_perm`
- `write_slot_entry_tracked`
- `set_slot_mdb_*_tracked`

理由：

- 这类函数是 manager-based memory bridge
- 它们适合作为局部 patch / permission 层的 trusted base

## 哪些函数必须继续证明

下面这些函数类型不应长期作为 trusted 语义层存在。

### A. Capability 语义函数

位于：

- `src/capability/mod.rs`
- `src/capability/zombie.rs`

优先应证明的函数包括：

- `update_data`
- `get_cap_size_bits`
- `get_cap_is_physical`
- `is_arch_cap`
- `same_region_as`
- `same_object_as`
- `is_cap_revocable`
- `zombie_new`
- `zombie_type_zombie_cnode`
- `cap_cyclic_zombie`

理由：

- 这些函数不是“读字段”
- 它们直接定义 capability 语义、推导规则和 region/object 关系
- `insert/move/delete/revoke/derive` 等高层证明都会依赖它们

### B. Arch capability 语义函数

位于：

- `src/arch/riscv64/mod.rs`
- `src/arch/aarch64/mod.rs`

优先应证明的函数包括：

- `arch_updatedata`
- `arch_is_cap_revocable`
- `get_cap_ptr`
- `arch_derive_cap`
- `arch_mask_cap_rights`
- `arch_same_region_as`
- `arch_same_object_as`

理由：

- 这些函数已经在决定 arch cap 的业务语义
- 它们不只是 concrete bridge
- 如果长期 external，会让 capability semantic layer 过厚

### C. `cte.rs` 中的 legacy semantic methods

位于：

- `src/cte.rs`

优先应证明或重构成 thin wrapper 的函数包括：

- `derive_cap`
- `ensure_no_children`
- `is_final_cap`
- `is_long_running_delete`
- `delete_all`
- `delete_one`
- `revoke`

理由：

- 这些方法本身就是 CSpace 业务判定和操作入口
- 不适合长期把 wrapper `external_body` 加 erased manager receiver 当成 end-to-end wrapper proof；后续应继续收成 verified thin wrapper 或接入 whole-kernel proof state

## 可以暂时 external，但 contract 必须很强的函数

有一类函数虽然暂时可以 external，但必须严格限制它们的职责。

### 局部 slot / MDB patch helper

例如：

- `write_slot_entry_tracked`
- `set_slot_mdb_prev_tracked`
- `set_slot_mdb_next_tracked`
- `set_slot_mdb_first_badged_tracked`
- `clear_slot_entry_tracked`

这类函数可以暂时 external 的前提是：

- contract 必须只描述局部 patch effect
- 不能把整个 insert/move/delete 业务语义黑盒化
- 只能表达“这个字段被改成什么”

这类 external 和 `atmo` 的底层容器 util 更接近，风险相对可控。

## 风险判断标准

判断一个 external 是否危险，不看它“长不长”，而看它在决定什么。

### 风险较低

如果 trusted 代码主要是在做：

- 读字段
- 取 pointer
- clone 值
- 做最小内存 patch
- 连接 concrete 和 ghost view

那么它更接近 bridge，风险相对较低。

### 风险较高

如果 trusted 代码已经在决定：

- 这个 capability 是否和另一个 capability 是 same object
- derive 规则是什么
- revocable 判定是什么
- arch-specific cap 语义是什么
- delete/revoke/finalise 的高层业务条件是什么

那么它实际上已经在承载语义，风险较高。

## 执行原则

执行补强时，默认遵守下面几条：

- 先冻结已经收住的 verified line，避免为了 shrink 而破坏 `resolve / insert / move / swap` 当前的 exec 形状
- 先收“谁在决定语义”，再收“external 数量”
- 优先补 contract 和 spec 对齐，再决定是否立刻把对应实现去掉 `external_body`
- 对外表述始终区分 manager-level 完成度与 public-wrapper 完成度

特别是：

- `resolve` 默认保持当前最小只读 trusted primitive 形态，不把它当作下一轮主要重写对象
- `move` 默认保持 manager-level 证明主线，后续主要补 public wrapper 与 semantic dependency，而不是大改 exec
- `delete` 路径默认继续按 contract-first 推进，不在 contract 还弱时抢着声称 core completion

## 执行路线

下面的顺序，不只是“建议阅读顺序”，而是推荐的实际施工顺序。

### Phase 0: Freeze 已收住主线并标注边界

目标：

- 明确哪些 external 是长期 bridge，哪些 external 是临时 semantic trusted layer
- 固定当前论文口径，不因 shrink 计划而提前拔高 claim

动作：

- 把 `trusted/*`、`impl_base.rs` 中可长期保留的 bridge 列为 allowed TCB
- 把 `capability/*`、`arch/*`、`cte.rs` 中承载业务语义的 external 列为 shrink target
- 对 `resolve / insert / move / swap` 标明“保持 exec 贴近 old implementation，不做非 blocker 级重写”

完成标志：

- 每个主要 external 已被归类为 bridge 或 semantic trusted layer
- 后续工作不会再把“文档已写 plan”误说成“trusted boundary 已经缩完”

当前状态：

- 已完成，并由 `residual-tcb-checklist.md` 继续充当 source of truth。

### Phase 1: Capability relation layer shrink

目标：

- 先把 capability 之间最核心的 relation 语义从 trusted 层往 verified semantic layer 挪

语义来源：

- `l4v` 中关于 capability relation、覆盖关系、revocable 判定的抽象语义与不变量目标

证明架构：

- 继续用 `atmo` 风格，让 runtime helper 保持贴近当前实现；proof 侧单独补 relation spec、case split、frame 使用者 contract

优先对象：

- `same_region_as`
- `same_object_as`
- `is_cap_revocable`
- `get_cap_ptr`
- `get_cap_size_bits`
- `get_cap_is_physical`
- `is_arch_cap`

原因：

- 这层最直接决定 `insert / move / delete / revoke / derive` 上层语义边界
- 如果这层继续 external，manager-level proof 以上的叙事会一直偏弱

完成标志：

- relation 函数已有稳定 spec / refinement 依据
- 上层 contract 不再把 capability relation 当成黑盒 trusted fact 使用

当前状态：

- manager-level 上已基本完成；当前残余主要是 arch relation runtime-vs-spec bridge lemma，以及长期允许保留的 `get_cap_ptr` bridge。

### Phase 2: Capability transform and arch semantic layer shrink

目标：

- 收掉 capability transformation 和 arch-specific semantic dispatch 这层高杠杆 trusted 语义

语义来源：

- `l4v` 中 derive / update / rights / arch case 的抽象意图与 contract 强度

证明架构：

- 仍按 `atmo` 风格分成 runtime mutation / local semantic post / frame / `wf` story
- 避免把 arch 语义揉回大而混杂的 wrapper

优先对象：

- `update_data`
- `arch_updatedata`
- `arch_derive_cap`
- `arch_mask_cap_rights`
- `arch_same_region_as`
- `arch_same_object_as`
- `arch_is_cap_revocable`
- zombie semantic helper

完成标志：

- `capability/*` 与 `arch/*` 更接近 verified semantic layer，而不再是 large trusted semantic helper layer

当前状态：

- 已基本完成。`riscv64` 这条线已在当前环境验证通过；`aarch64` 源码侧 shrink 已落地，但 arm-target 专门验证仍受当前工具链环境限制。

### Phase 3: Public wrapper thinning

目标：

- 把 `cte.rs` 从承载语义的 legacy shell，收成 minimal compatibility shell；当前不做 public-wrapper-level verification claim

重点对象：

- `derive_cap`
- `ensure_no_children`
- `is_final_cap`
- `is_long_running_delete`
- `delete_all`
- `delete_one`
- `revoke`

执行方式：

- 删除 kernel-facing wrapper 上非必要的 raw pre/post，避免把 manager 内部强前提误包装成 public API obligation
- 只保留 `get_ptr` / `get_offset_slot` 这类表示桥 contract，以及 manager/delete proof 仍然依赖的内部 vocabulary
- 对当前仍依赖 public wrapper `external_body` 的入口，后续等 kernel proof state 存在后再改成 verified thin wrapper 或给出 end-to-end proof domain
- 明确当前 public wrapper 的 claim 是“单一路径 runtime forwarding”，不是“public API 已证明”

完成标志：

- `cte.rs` 对外接口保留原 runtime 签名，不再暴露大量 verification-only `requires/ensures`
- public wrapper level 与 manager-level 的差距被明确记录为 future whole-kernel work，而不是当前 manager-level claim 的一部分

当前状态：

- 已按“minimal compatibility wrapper”口径完成。wrapper 不再依赖 fake active runtime manager pointer，也不再携带大面积 public proof contract；`get_ptr` / `get_offset_slot` 是当前保留的必要表示桥。

### Phase 4: Delete dependency cone closure

目标：

- 在 trusted boundary 叙事上，把 delete 路径从“高风险 semantic black box”推进到“强 contract 下的可控 bridge + 可继续证明的 core”

优先对象：

- `finalise_slot`
- `preemption_point_bridge`
- `post_cap_deletion_bridge`
- `delete_one`
- `delete_all`
- `revoke`

执行顺序：

- 先补 `finalise_slot` 及其下层 bridge 的强 contract
- 再补 delete core 第一批 contract 闭包
- 最后再谈 `revoke` 的 nontrivial success / error split 叙事

完成标志：

- delete path 的 contract-first closure 已基本收住
- delete 线保留的高层 TCB 已进一步收缩到：
- `finalise_slot`
- `reduce_zombie` 的 immediate helper
- `reduce_zombie(false)` 进入 `cte_swap` 前的一条小 ghost bridge
- `preemption_point` / `post_cap_deletion` 的正式语义边界明确停在 manager bridge
- 可以更稳地声称 delete path 处于 contracted closeout，而不是单纯 planned

当前状态：

- 已完成到 contracted closeout。当前全包 direct Verus checkpoint 仍有 3 个既有 swap/delete proof 点；delete witness/projector packaging 的闭合继续以 module-scoped 记录支撑。剩余未收缩项继续按 checklist 中的 residual TCB 处理，而不是再被当成未建模黑箱。

## 当前不建议做的事

- 不为“看起来更像 `l4v`”去重写 `resolve`
- 不对 `move / swap` 做非 blocker 级别的 exec 重构
- 不把“减少 `external_body` 数量”当作高于“减少 semantic trusted surface”的目标
- 不在 public wrapper 仍弱时提前声称 whole-kernel 或 `l4v`-equivalent completion

## 推荐的后续收敛顺序

推荐按下面顺序继续削薄 trusted base。

### 第一优先级

- `same_region_as`
- `same_object_as`
- `is_cap_revocable`
- `arch_same_region_as`
- `arch_same_object_as`

原因：

- 它们最核心地决定 capability semantic relation

### 第二优先级

- `arch_updatedata`
- `arch_derive_cap`
- `arch_mask_cap_rights`
- `update_data`

原因：

- 它们决定 capability transformation 语义

### 第三优先级

- `derive_cap`
- `ensure_no_children`
- `is_final_cap`
- `is_long_running_delete`

原因：

- 它们是 CSpace 操作的 legacy semantic helper

### 第四优先级

- zombie semantic helper
- delete/revoke 相关逻辑

原因：

- 这些路径往往更复杂，可以放在 derive / relation 函数之后推进

## 最终目标

本项目最终应收成下面这种形态：

- `trusted/*`
  只保留表示桥、内存桥、bitfield bridge、异常桥、硬件桥
- `impl_base.rs`
  只保留 manager-based pointer / perm / local patch bridge
- `capability/*`
  成为 verified semantic layer
- `arch/*`
  成为 verified arch semantic layer
- `cte.rs`
  尽量退成 thin wrapper / compat shell
- `cspace_manager/*`
  继续作为 verified core

如果达到这个状态，那么 `sel4_cspace` 的 trusted boundary 就会更接近 `atmo` 的成熟风格：

- external 不为零
- 但 external 只留在 bridge 和底层 util
- 高层 CSpace 语义尽量留在 verified core

## Practical takeaway

对当前 `sel4_cspace` 来说，最重要的不是继续压缩 `resolve` 这一条线，而是：

- 保持 `resolve` 当前这套最小 trusted primitive 形态
- 把 `capability` / `arch` / `cte` 中承载语义的 external 继续往 verified core 挪

也就是说，后续工作的主轴应从“继续补 `resolve`”切换到“削薄 capability / arch / cte 的 semantic trusted layer”。
