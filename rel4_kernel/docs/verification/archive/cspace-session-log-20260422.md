# CSpace Verus 验证 Session 记录（2026-04-22，Stage 4 收尾）

## 1. 本 session 目的

- 把“第 4 步：补小引理，把规格收紧成可复用入口”收口为可复述、可回归的阶段完成点。
- 更新阶段文档，使当前仓库状态不再停留在“Stage C 继续收紧中”的旧口径。
- 为下一步 concrete -> abstract view / refinement bridge 提供更稳定的规格入口。

## 2. 本 session 完成的代码收紧

### 2.1 `wf` 已收紧成稳定入口

在 [sel4_cspace/specs/abstract_cspace.rs](/workspace/rel4_kernel/sel4_cspace/specs/abstract_cspace.rs) 中新增：

- `lemma_wf_implies_core_invariants`
- `lemma_wf_implies_valid_slot_entry`

这一步的意义是：

- 不再要求后续 proof 在每个调用点手工展开 `wf` 这个大 conjunction；
- `valid_slots`、`mdb_prev_next_consistent`、`cnode_lookup_wf`、`cspace_roots_wf` 等核心模型层事实，
  现在已经有统一入口可以复用。

### 2.2 `resolve_address_bits_pre` 已收紧成 bridge 前入口

在 [sel4_cspace/specs/cspace_ops/resolve.rs](/workspace/rel4_kernel/sel4_cspace/specs/cspace_ops/resolve.rs) 中新增：

- `lemma_resolve_pre_implies_base_invariants`
- `lemma_resolve_pre_implies_root_lookup_ready`

这一步的意义是：

- `spec_resolve_address_bits_pre` 不再只是一个“需要手拆的大前提”；
- 对 root 为 `CNodeCap` 的情形，后续 refinement proof 现在可以直接取到：
  - `state.cnode_lookup_wf()`
  - `root_cap.cnode is Some`
  - `root_cap.object is Some`
  - `0 < spec_cnode_level_bits(root_cap)`
  - `spec_cnode_cap_lookup_total(state, root_cap)`

换句话说，`resolve_address_bits` 这条线已经从“smoke 中手工展开”提升为“bridge 可直接调用的入口”。

### 2.3 规格层噪声已收口到可接受水平

本 session 还做了两类纯规格层整理：

1. 在 [sel4_cspace/specs/cspace_ops/smoke.rs](/workspace/rel4_kernel/sel4_cspace/specs/cspace_ops/smoke.rs) 中，
   将 `assert forall ... ==> ... by` 改成当前 Verus 推荐的 `implies` 写法。
2. 在 [sel4_cspace/specs/abstract_cspace.rs](/workspace/rel4_kernel/sel4_cspace/specs/abstract_cspace.rs)
   与 [sel4_cspace/specs/cspace_ops/resolve.rs](/workspace/rel4_kernel/sel4_cspace/specs/cspace_ops/resolve.rs) 中，
   对当前接受的自动量词选择显式标注 `#![auto]`。

结果是：

- 之前 verify 输出中的主要 trigger 提示已不再刷屏；
- `assert forall` 的已知行为警告也已消失；
- 当前剩余 warning 主要回到 Rust 侧 `cfg(test)` 与 `repr(C)` enum 这类非本轮 Stage 4 阻塞项。

## 3. 本 session 实测结果

### 3.1 官方验证入口

- 命令：

```bash
cargo xtask verify --package sel4_cspace --jobs 1 --max-errors 1
```

- 结果：通过。
- 当前输出：`46 verified, 0 errors`

相较于上一轮文档中记录的 `42 verified, 0 errors`，本 session 新增通过项主要来自：

- `wf` 解包入口引理；
- `resolve_address_bits_pre` 解包入口引理；
- 收口后的规格层 smoke / 量词结构。

### 3.2 交叉目标检查

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

## 4. 当前阶段判断

截至 2026-04-22，建议把当前阶段判断更新为：

1. 第 1 步：可信边界与门禁，已完成。
2. 第 2 步：抽象模型与全局不变量，已完成本轮最小冻结。
3. 第 3 步：四个目标原语抽象合同，已完成。
4. 第 4 步：首批可复用小引理与 smoke 收口，已完成。
5. 第 5 步：concrete -> abstract view / refinement bridge，下一步。
6. 第 6 步：逐函数 refinement / 规格满足性证明，尚未开始。
7. 第 7 步：整体回归、文档与 TCB 清单收口，尚未开始。

如果用项目内部阶段语言来表达，更准确的口径是：

- Stage A：已完成。
- Stage B：已完成本轮最小冻结。
- Stage C：已完成第一轮收口。
- 当前下一步：进入 bridge。
- 函数级证明：尚未开始。

## 5. 下一步建议

建议从以下最小 bridge 切口开始：

1. 先为 `cap`、`cte_t`、`resolveAddressBits_ret_t` 建 ghost view。
2. 先打通 `resolve_address_bits` 的 refinement 闭环。
3. 等 bridge 稳定后，再推进 `cte_insert` / `cte_move` / `cte_swap`。
