# Verus Verified Component Kernel Integration

本文记录当前仓库中把 Verus verified component 接入 reL4 kernel 的实际方式。

重点不是“如何让编译器闭嘴”，而是说明三件事：

- 如何验证 `sel4_cspace`。
- 如何把经过 Verus 组织的 exec 代码参与 kernel 构建。
- 如何判断 spike/sel4test 的运行结果是否和 reference 对齐。

本文描述当前状态，日期为 2026-05-18。

## One Sentence

当前 `sel4_cspace` 的接入方式是：

```text
kernel syscall / boot code
  -> sel4_cspace::interface / cte_t method
  -> sel4_cspace::cspace::kernel compatibility dispatch
  -> CSpaceManager verified exec methods
  -> concrete cte_t / cap / mdb_node memory writes
```

`CSpaceManager` 当前是 verified exec method host。它的 ghost/tracked proof state 在普通 Rust kernel 构建中会被擦除；因此当前 exec 路径不需要 boot 阶段把 root cnode 或所有 concrete slot “灌入” manager。

未来如果要证明更强的 whole-kernel initialization invariant，才需要一个 boot/init bridge 来描述初始 slot domain、CDT/original state、MDB graph 等抽象事实。那是 proof bridge，不是当前运行所必需的 runtime initialization。

## Commands

### Bootstrap Verus

如果 `tools/verus/release/cargo-verus` 不存在，先安装官方 Verus release：

```sh
cargo xtask bootstrap-verus
```

这个命令会把 Verus release 工具放到：

```text
tools/verus/release/
```

当前 `xtask verify` 默认会从这里找 `cargo-verus`。

### Verify `sel4_cspace`

当前建议显式使用空 feature 集运行验证，避免被 `xtask` 的默认 feature 值影响：

```sh
cargo xtask verify --package sel4_cspace --features ''
```

如果只想验证某个模块或函数，可以把 Verus 参数放在 `--` 后面：

```sh
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace::manager::impl_insert
```

最近一次完整验证证据为：

```text
259 verified, 0 errors
```

### Build Kernel

只构建 Rust kernel：

```sh
cargo xtask build -p spike --rust-only
```

构建 kernel + sel4test 工程：

```sh
cargo xtask build -p spike
```

`-p spike` 对应 RISC-V spike 平台，`xtask` 会设置：

- target: `riscv64gc-unknown-none-elf`
- `PLATFORM=spike`
- `MARCOS` 中的 kernel 配置宏
- spike 相关 feature，例如 `riscv_ext_d`

### Run Sel4test

运行 spike/sel4test：

```sh
cargo xtask run -p spike
```

当前 reference 输出在 [kernel/rel4.txt](/workspace/rel4_kernel/kernel/rel4.txt:1557) 中：

```text
Test suite passed. 112 tests passed. 51 tests disabled.
All is well in the universe
```

当前 Verus 化后的 `sel4_cspace` 接入 kernel 后，也应当看到同样的 sel4test summary。这里的 `51 tests disabled` 是 reference 本身的配置结果，不是本次接入引入的失败。

有时 QEMU/simulate 在打印 `All is well in the universe` 后不会自动退出；这不改变 sel4test summary 的含义。判断运行是否通过时，优先看测试总结行是否出现并和 reference 对齐。

## Current Integration Shape

### Crate Dependency

`kernel/Cargo.toml` 仍通过 `sel4_cspace` crate 使用 CSpace 代码。workspace 根目录用 `[patch]` 把远端依赖重定向到本仓库本地 crate：

```toml
[patch.'https://github.com/reL4team2/sel4_cspace.git']
sel4_cspace = { path = "sel4_cspace" }
```

因此 kernel 构建时实际链接的是当前 workspace 里的 `sel4_cspace`，不是旧的未 Verus 化版本。

### Kernel Call Sites

kernel 侧仍然使用原有 CSpace API 形状，例如：

- `sel4_cspace::interface::cte_t`
- `sel4_cspace::interface::cte_insert`
- `sel4_cspace::interface::cte_move`
- `sel4_cspace::interface::cte_swap`
- `cte_t::delete_all`
- `cte_t::delete_one`
- `cte_t::revoke`

典型调用点包括：

- `kernel/src/syscall/invocation/invoke_cnode.rs`
- `kernel/src/syscall/invocation/invoke_tcb.rs`
- `kernel/src/syscall/invocation/invoke_irq.rs`
- `kernel/src/syscall/invocation/invoke_untyped.rs`
- `kernel/src/boot/root_server.rs`

这些调用点不需要知道 Verus proof state 的存在。

### Compatibility Dispatch

`sel4_cspace/src/cte.rs` 和 `sel4_cspace/src/interface.rs` 继续保留 kernel-facing compatibility API。它们把调用转到：

```text
sel4_cspace/src/cspace/kernel.rs
```

`cspace::kernel` 再统一转发到 `CSpaceManager` 的 exec methods：

```rust
with_cspace_manager(|manager| manager.cte_insert(new_cap, src, dest));
with_cspace_manager(|manager| manager.cte_move(new_cap, src, dest));
with_cspace_manager(|manager| manager.cte_swap(cap1, slot1, cap2, slot2));
with_cspace_manager(|manager| manager.delete_all(slot, exposed));
with_cspace_manager(|manager| manager.delete_one(slot));
with_cspace_manager(|manager| manager.revoke(slot));
```

这层的设计目标是让 kernel call sites 稳定，同时让 CSpace 主操作在 `manager` 层保持 verified exec body。

### Manager Runtime Meaning

当前 `CSpaceManager` 不拥有真实 kernel resource。它主要承载：

- Verus 证明中的 owner/view/wf 结构。
- `cte / mdb / cdt / manager` 分层 post-state 和 invariant 组合。
- 与 reference CSpace 操作对齐的 exec method。

普通 Rust kernel 构建中，`Ghost<T>`、`Tracked<T>`、proof/spec 内容不进入运行时语义。也就是说，当前 runtime 行为的真实效果仍然是 concrete `cte_t`、`cap`、`mdb_node` 上的读写。

因此 `cspace::kernel` 中的 fallback：

```text
如果全局 CSpaceKernel 未初始化，就临时创建 CSpaceManager::new() 并调用 exec method
```

是当前 exec 语义下合理的兼容路径。它不是在“丢失 manager 资源”，因为 manager 在当前运行模型下没有需要长期保存的 concrete resource。

## Allocator Note

当前 `sel4_cspace` 依赖：

```toml
vstd = { git = "https://github.com/verus-lang/verus.git", default-features = false, features = ["alloc"] }
```

原因是当前 Verus release 中 `vstd::simple_pptr` 挂在 `alloc` feature 后面。即使 CSpace exec 路径不主动做 heap allocation，普通 Rust 链接也需要 kernel 提供 allocator symbol。

因此 kernel 侧有最小 allocator 注册：

```text
kernel/src/heap.rs
```

并在：

```text
kernel/src/lib.rs
kernel/src/main.rs
```

中声明：

```rust
mod heap;
```

这一步是为了满足当前工具链和 `vstd` feature 的链接需求，不表示 CSpace manager 在 runtime 依赖 heap 来维护资源。后续如果迁移到不需要 `vstd::simple_pptr` 的组织，或者换到类似 atmo 使用的 Verus fork/feature 组合，可以重新评估是否移除这个 allocator bridge。

## Atmo Comparison

atmo 的重要启发不是“照抄一个 manager init 形状”，而是：

- verified component 通过普通 Rust crate 参与 kernel 构建。
- proof/ghost 组织留在 verified crate 内部。
- 与未验证 kernel 部分交互时，保留明确的 bridge / trusted boundary。
- 验证入口和普通运行入口可以是两条工具链。

本仓库当前做法与 atmo 的相同点：

- `sel4_cspace` 像一个 verified subsystem crate，由 kernel 正常依赖。
- Verus proof state 不要求 kernel boot 阶段持有一个额外 runtime object graph。
- kernel-facing API 保持稳定，verified core 在 crate 内部演进。

差异点：

- atmo 的 `verified` crate 使用 `mars-research/verus` 的 `mars` branch，`vstd` 没有启用 `alloc` feature。
- 本仓库当前使用 `verus-lang/verus` release，且因 `vstd::simple_pptr` 需要保留 `alloc` feature。
- atmo 的 build-tool 通过 package metadata 搜索 verified crate，并直接调用 `rust_verify`。
- 本仓库当前通过 `cargo xtask verify` 调用官方 `cargo-verus verify`，普通 kernel 构建仍走 `cargo xtask build/run`。

这意味着接入策略应该参考 atmo 的分层和 bridge 思路，但工具链细节必须按当前 reL4/Verus release 的实际约束处理。

## Validation Checklist

修改 verified component 后，推荐按下面顺序确认：

1. 先确认普通 crate 能编：

```sh
cargo check -p sel4_cspace
```

2. 再跑 Verus 验证：

```sh
cargo xtask verify --package sel4_cspace --features ''
```

3. 再跑 kernel/sel4test：

```sh
cargo xtask run -p spike
```

4. 最后对照 reference summary：

```text
Test suite passed. 112 tests passed. 51 tests disabled.
All is well in the universe
```

只有第 2 步通过，才能说 `sel4_cspace` 当前 Verus verification entrypoint 通过。只有第 3 和第 4 步对齐，才能说当前 exec 代码已经接入 kernel 并通过 spike/sel4test reference 级运行测试。

## Claim Boundary

当前可以安全表述为：

- `sel4_cspace` 的 verified exec core 已经能够参与 kernel 构建。
- 当前 spike/sel4test 运行结果与 reference `kernel/rel4.txt` 对齐。
- manager-level verified core 与 kernel-facing compatibility API 已经完成当前运行接入。

当前不应表述为：

- whole-kernel CSpace invariant 已经完全证明。
- public wrapper 到 manager 的所有语义边界都已经 end-to-end verified。
- boot 阶段已经证明了完整 initial CSpace abstract state。

更准确的说法是：

```text
当前完成的是 verified CSpace component 的 kernel runtime integration：
Verus 验证入口通过，exec 代码参与 kernel 构建，并通过 spike/sel4test reference summary。
更强的 whole-kernel proof bridge 仍属于后续工作。
```
