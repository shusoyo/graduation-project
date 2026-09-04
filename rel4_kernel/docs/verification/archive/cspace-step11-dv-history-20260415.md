# CSpace Step11 历史 DV 排障记录（归档）

本文档保存 2026-04-15 阶段使用 cargo dv 路线时的历史排障信息，仅用于审计追溯。
当前执行入口请以主文档为准：
- docs/verification/cspace-baseline-regression-20260415.md
- docs/verification/cspace-verification-steps.md

## 1. 第 11 步执行记录更新（2026-04-15）

### 1.1 早期阻塞（已定位）

- 命令：`RUSTUP_TOOLCHAIN=1.94.0-x86_64-unknown-linux-gnu CARGO_BUILD_TARGET=x86_64-unknown-linux-gnu cargo dv bootstrap --upstream-verus`
- 结果：**部分完成**
  - `verusdoc` 可生成；
  - `vargo build --release --features singular` 曾在 `vir` 编译阶段出现 `signal: 9 (SIGKILL)`。
- 影响：早期阶段出现 `Cannot find the Verus binary`。

### 1.2 运行链路恢复（已完成）

- 经验证，以下组合可使验证进入 Verus 执行阶段，而非停在“找不到二进制”：
  - `RUSTUP_TOOLCHAIN=1.94.0-x86_64-unknown-linux-gnu`
  - `RUSTC_BOOTSTRAP=1`
  - `PLATFORM=spike`
  - `MARCOS='KERNEL_STACK_BITS=12 FASTPATH=true HAVE_FPU=true RISCV_EXT_D=true'`
- 对 fresh build 场景，`target/sel4_common.deps.toml` 可能退化为空，需强制重建（见 1.3 命令）。

### 1.3 当前可复现命令（收敛到 vstd 阻塞）

```bash
cd /workspace/rel4_kernel

cargo clean
rm -f target/sel4_common.deps.toml

RUSTUP_TOOLCHAIN=1.94.0-x86_64-unknown-linux-gnu \
RUSTC_BOOTSTRAP=1 \
PLATFORM=spike \
MARCOS='KERNEL_STACK_BITS=12 FASTPATH=true HAVE_FPU=true RISCV_EXT_D=true' \
cargo build -p sel4_common --target riscv64gc-unknown-none-elf

OUT_DIR="$(ls -d target/riscv64gc-unknown-none-elf/debug/build/sel4_common-*/out | tail -n 1)"

RUSTUP_TOOLCHAIN=1.94.0-x86_64-unknown-linux-gnu \
RUSTC_BOOTSTRAP=1 \
PLATFORM=spike \
MARCOS='KERNEL_STACK_BITS=12 FASTPATH=true HAVE_FPU=true RISCV_EXT_D=true' \
OUT_DIR="$OUT_DIR" \
cargo dv verify --targets sel4_common --max-errors 1 -- \
  --target riscv64gc-unknown-none-elf \
  -L dependency=target/riscv64gc-unknown-none-elf/debug/deps
```

- 现象：错误已从早期 bootstrap/二进制缺失，收敛到 `vstd` 工件链。

### 1.4 当前主阻塞（未解除）

1. 不注入 `OUT_DIR` 时，`include!(env!("OUT_DIR"))` 相关路径缺失会导致大量失败。
2. 注入 `OUT_DIR` 且补充 target 依赖搜索路径后，错误前移为：`could not open .../target-verus/debug/vstd.vir`。
3. 使用 `--no-vstd` 时，错误变为：`verus_builtin crate was not imported`。
4. 手动尝试 `vstd_build` 目前仍有大量内部编译错误，尚未形成稳定产物链。

### 1.5 本轮最小生产代码兼容修复

- 文件：`sel4_common/src/structures.rs`
- 变更：`CStr::from_ptr(self.get_ptr::<i8>())` 调整为 `CStr::from_ptr(self.get_ptr::<c_char>())`。
- 目的：消除 `c_char` 有符号性平台差异导致的类型不匹配，不改变业务语义。

### 1.6 低并发 bootstrap 与精简版 Step-11 脚本（2026-04-15）

- 新增脚本：`tools/bootstrap-verus-release.sh`
- 用途：
  - 通过 `BOOTSTRAP_JOBS` 控制 bootstrap 并发，默认 `1`；
  - 固定使用 release 模式构建 Verus；
  - 固定使用 `CARGO_BUILD_TARGET=x86_64-unknown-linux-gnu`；
  - 默认按文档版本执行 `cargo dv bootstrap --upstream-verus --branch release/0.2026.04.12.f1166c4`。

- 精简脚本：`tools/verify-step11-sel4-common.sh`
- 用途：
  - 假设 release bootstrap 已完成；
  - 自动补齐 `OUT_DIR`；
  - 固定 release 模式的 cross-target 参数；
  - 直接执行 `cargo dv verify --targets sel4_common`。

- 当时结果：
  - step11 脚本复杂度已明显降低；
  - rel4 侧只剩 release build、`OUT_DIR` 注入和 cross-target 参数这层适配；
  - 是否闭环取决于 `cargo dv bootstrap` 能否成功产出 release Verus/vstd 工件。