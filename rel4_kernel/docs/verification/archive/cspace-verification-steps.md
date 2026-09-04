# CSpace Verus 验证执行步骤（面向 Verus exec 本体化的 6 步版）

## 适用范围

- 只覆盖 CSpace 逻辑。
- 不改生产语义。
- `sel4_common`、FFI、底层指针/位域细节先作为可信边界。
- 长期目标不是停在 “`refinement_bridge.rs` 旁证现有 Rust 实现正确”，而是逐步把
  `sel4_cspace/src/cte.rs` 中的核心函数推进成 Verus 风格的 exec 函数：
  - 函数本体自己携带 `requires/ensures`；
  - proof 尽量贴近函数 body；
  - 大块函数级 trusted wrapper 只作为过渡，不作为终点。

## 风格基线

- 语义来源对齐 `aux/l4v`。
- proof 粒度与合同组织优先参考 `aux/vostd`：
  - 小函数；
  - 小合同；
  - 必要时用小型 `assume_specification[...]` / 小型 `external_body` util；
  - 尽量直接证明 exec body；
  - 默认优先签名式 `requires/ensures`，不把 `#[verus_spec(...)]` 当长期主接口形态。
- 边界管理方式参考 `aux/atmo`：
  - trusted surface 单独命名、单独隔离；
  - verified 层与 concrete 层通过清楚的桥接边界对接；
  - 但当前不引入 `atmo` 式重 tracked/ownership-first 体系。
- 具体编码约束见：
  - `docs/verification/cspace-verus-style-guide.md`

## 新版 6 步

1. 固定门禁与最小 TCB
- 固定回归命令：
  - `RUSTUP_TOOLCHAIN=1.94.0-x86_64-unknown-linux-gnu RUSTC_BOOTSTRAP=1 PLATFORM=spike MARCOS="KERNEL_STACK_BITS=12 FASTPATH=true HAVE_FPU=true RISCV_EXT_D=true" CARGO_BUILD_TARGET=riscv64gc-unknown-none-elf cargo check -p sel4_cspace`
  - `cargo xtask verify --package sel4_cspace --jobs 1 --max-errors 50`
- 固定可信边界：
  - `sel4_common` 位域与 getter；
  - 指针转换与底层内存访问；
  - `deps` FFI；
  - 必要的架构最小操作。
- 完成判定：
  - 无无约束 assume；
  - 每个边界都有输入前提和输出语义；
  - 可映射到调用点与 TCB 台账。

2. 固定抽象模型与全局不变量
- 在 `specs/abstract_cspace.rs` 中固定：
  - `CapSpec`；
  - `SlotEntrySpec`；
  - `CSpaceState`；
  - `wf`；
  - `slots_unchanged_except`；
  - `reachable_slot_from`；
  - `cnode_lookup` 相关抽象。
- 把“大 conjunction”收紧成可复用入口：
  - `lemma_wf_implies_core_invariants`；
  - `lemma_wf_implies_valid_slot_entry`。
- 完成判定：
  - 核心不变量能被目标函数规格直接引用；
  - lookup 语义与图语义不分叉。

3. 固定目标原语的抽象合同
- 给四个目标函数写稳定 spec：
  - `resolve_address_bits`；
  - `cte_insert`；
  - `cte_move`；
  - `cte_swap`。
- 要求：
  - pre/post 先完整；
  - case taxonomy 先完整；
  - frame condition 先完整；
  - packaging lemma 先完整。
- 完成判定：
  - 每个函数都有可复用的抽象 requires/ensures；
  - smoke check 能覆盖主要 success/fault 路径；
  - proof 失败能明确定位到 spec 或调用点。

4. 完成过渡 bridge，但明确它只是脚手架
- 在 `sel4_cspace/src/refinement_bridge.rs` 中建立：
  - raw `cap` / `cte_t` / `resolveAddressBits_ret_t` 到抽象模型的 view；
  - read-only bridge；
  - mutating-op 的 local heap transition bridge；
  - 函数级 refined entry 的过渡入口。
- 约束：
  - bridge 只负责把 concrete 数据形状接到抽象语义；
  - 不继续膨胀成长期最终层；
  - 新增 bridge 词汇必须服务于“把 proof 推回 `cte.rs` 本体”。
- 完成判定：
  - `resolve_address_bits` 与 `cte_*` 都有可复用的桥接入口；
  - bridge vocabulary 稳定；
  - 不再需要为 Stage 5 继续扩巨型 predicate。

5. 将 proof 从 bridge/wrapper 推回 `cte.rs` 函数本体
- 这是新的核心步骤，也是当前主线。
- 目标不是继续停在 `trusted_call_* -> *_refined(...)` 这一层，而是：
  - 逐步缩小函数级 trusted wrapper；
  - 把 proof 推回 `sel4_cspace/src/cte.rs` 的同名函数；
  - 让这些函数最终尽量直接带 `requires/ensures`。
- 推荐顺序：
  - `resolve_address_bits`
  - `cte_insert`
  - `cte_move`
  - `cte_swap`
- 收敛方式：
  - 先把函数级 `trusted_call_*` 缩成更小的 util 假设；
  - 再让函数 body 直接消费这些 util 合同；
  - 最后删掉或最小化函数级 wrapper。
- 完成判定：
  - 目标函数的主要语义由函数本体自身导出；
  - proof 结构更像 `vostd` 的 exec proof，而不是“外层 wrapper + 大块桥接”。

6. 收口最小 trusted surface 与最终台账
- 当四个核心函数都已尽量靠近 `cte.rs` 本体证明后，再回头压缩过渡层。
- 最终只保留最小可信边界：
  - 不可避免的 raw pointer/bitfield 转换；
  - FFI；
  - 极少量平台相关操作；
  - 少量不可替代的 object-local util。
- 输出：
  - 已直接验证的 exec 函数清单；
  - 仍保留的 trusted util 清单；
  - 未完成项与继续下推建议；
  - 最终 TCB 台账。

## 当前阶段位置（截至 2026-04-24 本 session）

- 第 1-6 步现已全部完成。
- 第 1 步：
  - 门禁与最小 TCB 已固定；
  - `boundary_assumptions` 已接入验证入口。
- 第 2 步：
  - `abstract_cspace` / `wf` / lookup / reachability 抽象已稳定。
- 第 3 步：
  - `resolve_address_bits`、`cte_insert`、`cte_move`、`cte_swap` 的抽象合同与 packaging lemma 已稳定。
- 第 4 步：
  - `refinement_bridge.rs` 已收口为桥接脚手架，不再继续扩成长期语义层。
- 第 5 步：
  - proof 已推回 `sel4_cspace/src/cte.rs` 的 refined 入口；
  - `resolve_address_bits_refined(...)`、`cte_insert_refined(...)`、`cte_move_refined(...)`、`cte_swap_refined(...)` 与配套只读入口均已位于 `cte.rs`。
- 第 6 步：
  - 最小 trusted surface 与当前 TCB 台账已整理；
  - 当前已证入口 / 保留 trusted util / 后续收缩方向已在
    `docs/verification/cspace-stage6-closeout-20260424.md` 收口。
- 当前 verify 实测通过：
  - `cargo xtask verify`
  - 输出为 `136 verified, 0 errors`。

## 下一步执行建议

1. 进入下一轮“signature-first”收缩
- 优先减少 `#[verus_spec(...)]` 覆盖面，把主函数合同逐步迁移成直接写在函数签名上的 `requires/ensures`。

2. 继续削减 bridge-local trusted util
- 优先拆 `trusted_extract_*`、`trusted_concrete_*_at` 这类黑盒视图；
- 保持新增 util 只做 object-local concrete 观察，不重新膨胀成函数级语义黑盒。

3. 把覆盖面从当前原语闭环扩到 delete/finalise/revoke 路径
- 当前 6 步已经收住了 cspace 原语主线；
- 下一轮更值得投入的是高副作用删除链路，而不是回头再扩 Stage 5 bridge 词汇。

## 当前起点

- 当前这套 6 步计划已经闭环。
- 如果继续推进，起点不再是“完成第 6 步”，而是新一轮：
  - 去 `wrapper`；
  - 去 `attribute-heavy`；
  - 去多余 trusted util；
  - 把更多主函数推进成更原生的 Verus exec 形式。
- 但如果目标切换到“论文里尽量直接复用 l4v 语义口径”，则还需要进入第二轮重构主线：
  - 先把 capability 基础语义、bridge 语义边界、原语 spec 重新对齐 `aux/l4v`；
  - 再在这套更贴近 l4v 的语义基线上继续推进 exec 本体化。
- 这条新主线见：
  - `docs/verification/cspace-l4v-alignment-refactor-plan.md`
