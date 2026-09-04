# CSpace Verus 验证阶段性报告（阶段 A 收口）

> 状态说明：本文记录“阶段 A 收口”这一历史检查点。关于 2026-04-18 当前 session
> 中继续推进到 Stage B 抽象模型后的最新状态，请结合
> `docs/verification/cspace-session-log-20260418.md` 与
> `docs/verification/cspace-verification-plan.md` 一并阅读。

## 1. 阶段背景

本阶段工作面向 `sel4_cspace` 模块的形式化验证准备，目标是在**不改写生产语义**的前提下，
先完成验证入口、可信边界（TCB）与最小合同的固化，为后续 CSpace 内部逻辑证明提供稳定起点。

结合当前仓库结构，`sel4_cspace` 的验证并不能直接从纯 Rust 代码展开，需要同时处理：

- `sel4_common` 中自动生成位域代码的可信边界问题；
- 指针转换与底层内存访问的抽象问题；
- `finalise_cap`、`post_cap_deletion`、`preemption_point` 等 FFI 边界问题；
- Verus 工具链、目标平台与构建环境变量之间的耦合问题。

因此，本阶段不直接进入 `cte_insert`、`cte_move`、`cte_swap`、`resolve_address_bits` 的函数级证明，
而是先完成“边界假设可审计、验证门禁可复现、阶段划分清晰可追溯”的基础工作。

## 2. 阶段目标

本阶段的目标可以概括为以下四点：

1. 固定 `sel4_cspace` 的官方验证入口与最小回归门禁。
2. 建立 CSpace 验证的可信边界台账，明确当前哪些部分暂时作为 TCB。
3. 在 `sel4_cspace/specs/lib.rs` 中建立与台账一一对应的边界假设锚点。
4. 保证当前阶段的文档、代码与回归结果三者一致，可作为后续证明工作的“冻结起点”。

## 3. 本阶段已完成工作

### 3.1 建立边界假设模块

在 [sel4_cspace/specs/lib.rs](/workspace/rel4_kernel/sel4_cspace/specs/lib.rs:12) 中新增
`boundary_assumptions` 模块，并补入 SC-BC-01 到 SC-BC-06 对应的合同锚点。

本轮新增或收紧的规格包括：

- `assume_convert_to_mut_type_ref_contract`
- `assume_convert_to_option_mut_type_ref_contract`
- `assume_mask_vm_rights_non_escalation`
- `assume_finalise_cap_pairing`
- `assume_post_cap_deletion_origin`
- `assume_preemption_point_progress`

同时新增 `boundary_assumptions_smoke_check()`，并由 `specs_smoke_check()` 统一接入，
确保这些边界假设确实被当前验证入口编译和检查。

### 3.2 收紧原先过弱的边界占位

在阶段初稿中，部分边界锚点虽然已经存在，但表达能力偏弱，主要问题是：

- `convert_to_mut_type_ref` 只表达了“地址非 0”；
- `convert_to_option_mut_type_ref` 只表达了“0 返回 None，非 0 返回 Some”；
- FFI 边界更多是匿名布尔占位，尚未显式体现调用现场一致性与删除流程语义。

本轮修改后，这些边界假设已经升级为更贴近台账要求的合同骨架：

- 指针转换类边界显式携带“地址对齐、对象有效、无冲突可变别名、返回引用绑定原地址”等条件；
- `finalise_cap` 显式区分 capability、`_final`、`_exposed` 与删除流程结果之间的关联；
- `post_cap_deletion` 显式区分“来源于对应 finalise 输出”与“不会额外修改 capability 状态”；
- `preemption_point` 显式体现“调用点处于可抢占循环，且调用方处理非 NONE 返回”的前提。

这一步的意义在于：当前阶段虽然仍将这些边界视为 TCB，但已经避免了“有锚点但锚点太弱，后续证明无法可靠依赖”的问题。

### 3.3 固化可信边界台账

在 [docs/verification/cspace-verification-tcb.md](/workspace/rel4_kernel/docs/verification/cspace-verification-tcb.md:109)
中，已将边界台账固定为 SC-BC-01 ~ SC-BC-06 六类条目，并同步更新了
`convert_to_mut_type_ref` / `convert_to_option_mut_type_ref` 的最新锚点名称：

- SC-BC-02 -> `assume_convert_to_mut_type_ref_contract`
- SC-BC-03 -> `assume_convert_to_option_mut_type_ref_contract`

台账当前已经具备以下可审计信息：

- 边界符号；
- CSpace 调用点；
- `specs` 锚点；
- 状态；
- 最小 requires；
- 最小 ensures；
- 风险等级；
- 去 TCB 条件。

这意味着边界假设已经不再是散落在代码中的临时占位，而是进入了“文档台账 + 规格锚点 + 调用点映射”的可追踪状态。

### 3.4 修正阶段门禁文档

在 [docs/verification/cspace-verification-steps.md](/workspace/rel4_kernel/docs/verification/cspace-verification-steps.md:19)
和 [docs/verification/cspace-verification-plan.md](/workspace/rel4_kernel/docs/verification/cspace-verification-plan.md:21)
中，已将原先过于简化的 `cargo check -p sel4_cspace` 门禁命令修正为当前仓库中**真实可复现**的形式。

当前可复现的交叉检查命令为：

```bash
RUSTUP_TOOLCHAIN=1.94.0-x86_64-unknown-linux-gnu \
RUSTC_BOOTSTRAP=1 \
PLATFORM=spike \
MARCOS="KERNEL_STACK_BITS=12 FASTPATH=true HAVE_FPU=true RISCV_EXT_D=true" \
CARGO_BUILD_TARGET=riscv64gc-unknown-none-elf \
cargo check -p sel4_cspace
```

修正原因是：

- `sel4_common/build.rs` 依赖 `PLATFORM` 与 `MARCOS`；
- 交叉构建路径会涉及 `-Ztls-model=local-exec`，因此需要 `RUSTC_BOOTSTRAP=1`；
- 如果文档仍保留裸命令，就会出现“文档写法与实际仓库行为不一致”的问题。

通过本轮修正，阶段文档与真实执行入口已经保持一致。

## 4. 关键代码变更总结

本阶段与代码直接相关的核心变更集中在以下文件：

### 4.1 `sel4_cspace/specs/lib.rs`

主要变化：

- 新增 `boundary_assumptions` 模块；
- 将边界假设由松散占位改为合同骨架；
- 新增 smoke check，并接入 `specs_smoke_check()`。

这部分变更的意义是为后续 Verus 证明建立**可引用、可扩展、可审计**的边界规格入口。

### 4.2 `docs/verification/cspace-verification-tcb.md`

主要变化：

- 固化 SC-BC-01 ~ SC-BC-06 台账条目；
- 明确每个边界的调用点、风险级别与去 TCB 条件；
- 同步更新边界锚点名称。

这部分变更的意义是为“当前证明覆盖范围”与“当前仍信任哪些边界”提供正式说明，
避免后续把阶段 A 误解为“已经完成全链路证明”。

### 4.3 `docs/verification/cspace-verification-steps.md`

主要变化：

- 修正阶段 A 门禁命令；
- 明确边界阶段的验收口径。

这部分变更的意义是让当前阶段具备稳定的操作手册属性。

### 4.4 `docs/verification/cspace-verification-plan.md`

主要变化：

- 将阶段 A 的现状、门禁命令与本周目标同步到最新状态；
- 保持 A -> B -> C -> D -> E 的推进顺序不变。

这部分变更的意义是保证“计划文档”和“当前落地状态”一致。

## 5. 阶段验证结果

本阶段完成后，当前回归结果如下：

### 5.1 交叉目标检查

- 命令：

```bash
RUSTUP_TOOLCHAIN=1.94.0-x86_64-unknown-linux-gnu \
RUSTC_BOOTSTRAP=1 \
PLATFORM=spike \
MARCOS="KERNEL_STACK_BITS=12 FASTPATH=true HAVE_FPU=true RISCV_EXT_D=true" \
CARGO_BUILD_TARGET=riscv64gc-unknown-none-elf \
cargo check -p sel4_cspace
```

- 结果：通过。

### 5.2 官方验证入口

- 命令：`./tools/verify-cspace-official.sh`
- 结果：通过。
- 当前输出：`2 verified, 0 errors`

这说明：

- `specs` 模块已被官方验证入口纳入；
- 新增边界假设 smoke check 已成功进入验证路径；
- 本轮修改没有破坏当前 `sel4_cspace` 的验证闭环。

## 6. 阶段结论

截至 2026-04-18，`sel4_cspace` 的 Verus 验证工作已经完成阶段 A 的主要收口任务，
可以形成如下阶段性结论：

1. 当前验证入口已经稳定，具备可复现的执行命令。
2. 边界台账已经建立，且具备可追踪的 `specs` 锚点。
3. 边界假设不再是过弱的匿名占位，而是升级为与台账一致的最小合同骨架。
4. 当前仍未进入 CSpace 内部逻辑证明，但已经具备进入下一阶段的基础条件。

换言之，本阶段的核心成果不是“完成了 CSpace 的函数级证明”，而是完成了
**形式化验证前的验证工程收口**：把验证边界、验证入口、文档口径和规格锚点统一到同一个基线上。

## 7. 当前局限与风险

尽管阶段 A 已基本收口，但当前工作仍然存在以下局限：

1. `boundary_assumptions` 目前仍属于 TCB 假设层，而非已证明引理层。
2. 指针转换与 FFI 语义尚未被真实内存模型或导出实现证明所替代。
3. 当前 `2 verified, 0 errors` 主要说明“边界规格锚点与 smoke check 已接通”，
   还不能等同于 `cte_insert`、`cte_move`、`cte_swap`、`resolve_address_bits`
   已完成行为正确性证明。
4. `sel4_common` 仍存在独立 warning，例如 `repr(C)` enum 判别值过大问题，
   该问题不影响当前阶段通过判定，但后续需要视整体工程要求决定是否处理。

因此，本阶段成果应被准确表述为：

- 已完成验证工程准备与边界合同化；
- 尚未完成 CSpace 内部逻辑的正式证明。

## 8. 下一阶段工作计划

下一阶段将进入阶段 B / C 的准备与落地，重点包括：

1. 在 `sel4_cspace/specs/lib.rs` 中建立 slot/cte 抽象模型。
2. 定义 mdb 双向一致性、空槽约束等核心不变量。
3. 为 `cte_insert`、`cte_move`、`cte_swap`、`resolve_address_bits` 编写规格骨架。
4. 将当前边界假设与目标原语规格建立明确引用关系。
5. 在每完成一个原语规格或证明节点后，继续执行官方验证入口回归。

从毕业设计角度看，下一阶段的工作重点将从“验证工程与边界合同化”
转向“CSpace 核心原语的抽象建模与行为证明”，这也是后续论文或报告中最具技术含量的部分。

## 9. 阶段成果概括

本阶段的成果可以概括为一句话：

**完成了 `sel4_cspace` 形式化验证的阶段 A 收口，使验证入口、可信边界、规格锚点与回归文档达成一致，为后续 CSpace 内部逻辑证明提供了稳定起点。**
