# reL4 当前构建与验证工具链总览

> 面向第一次接触本仓库、第一次接触 `xtask`、第一次接触 seL4/reL4 构建与 Verus 验证流程的读者。
>
> 本文描述的是当前仓库里的“实际状态”，不是历史设计稿，也不是未来规划图。
>
> 截止时间：2026-04-19

## 1. 先给结论

当前仓库里，构建和验证已经形成了两条相互配合、但还没有完全合并成一个统一入口的流程：

- 普通构建主线是：
  - `rel4_config` 负责读取平台配置并生成代码/链接脚本；
  - 各子模块自己的 `build.rs` 负责做和本模块相关的生成工作；
  - `xtask` 负责把用户输入的高层参数翻译成 `cargo`、`cmake`、`ninja`、环境变量和 feature。
- 形式化验证主线是：
  - 先用 `tools/bootstrap-verus-release.sh` 安装官方 Verus 工具；
  - 再用 `tools/verify-cspace-official.sh` 调用 `cargo-verus verify`；
  - 当前正式接入 Verus 的模块只有 `sel4_cspace`。

如果只记一句话，可以记成：

> 现在的 reL4 是“`xtask` 负责构建编排，`tools/` 脚本负责 Verus 验证，`sel4_cspace` 是当前唯一正式接线的验证目标”。

## 2. 读这份文档前，先分清四件事

很多初学者会把下面几件事混在一起，但它们不是一回事：

### 2.1 `cargo build`

这是普通 Rust 编译。它的目标是把某个 crate 编出来。

### 2.2 `cargo xtask ...`

这是一个“项目级命令入口”。它本身不是内核代码，而是一个 host-side 的辅助程序，用来帮你组织命令、拼参数、设环境变量、调用 `cargo/cmake/ninja`。

可以把它理解成“用 Rust 写的构建驱动器”。

### 2.3 `cmake` / `ninja` / `simulate`

这部分主要是和上游 seL4 project、测试工程、模拟器运行联系在一起。`xtask` 的 `build/run/install` 会去触发它们。

### 2.4 `cargo-verus verify`

这不是普通编译，而是 Verus 形式化验证入口。它会读 Verus 规格和证明代码，尝试证明性质成立。

因此：

- `cargo build` 通过，不等于 Verus 已证明。
- `xtask build` 通过，不等于 Verus 已证明。
- `verify-cspace-official.sh` 通过，才表示当前这条 Verus 验证入口通过了。

## 3. 当前仓库里，工具链是怎么分层的

从上往下看，当前构建/验证工具链大致可以分成 6 层：

1. 用户命令层
- 你输入 `cargo xtask build ...`
- 或者输入 `./tools/verify-cspace-official.sh`

2. 命令编排层
- `xtask` 负责普通构建编排
- `tools/*.sh` 负责当前的 Verus 验证编排

3. 配置与环境层
- `PLATFORM`
- `MARCOS`
- `RUSTFLAGS`
- Cargo feature
- target triple

4. 代码生成层
- `rel4_config`
- 各 crate 的 `build.rs`

5. 模块编译/验证层
- `kernel`
- `sel4_common`
- `sel4_cspace`
- `sel4_ipc`
- `sel4_task`
- `sel4_vspace`
- `rel4-arch`

6. 外部工具层
- `cargo`
- `cmake`
- `ninja`
- `simulate`
- `cargo-verus`

可以把它脑补成两条管线：

```text
普通构建：
用户 -> xtask -> cargo/cmake/ninja -> build.rs + rel4_config -> 各模块产物

形式化验证：
用户 -> tools/bootstrap-verus-release.sh -> cargo-verus 安装
用户 -> tools/verify-cspace-official.sh -> cargo-verus verify -> sel4_cspace/specs
```

## 4. 仓库结构里，哪些东西分别扮演什么角色

### 4.1 workspace 根目录

根 `Cargo.toml` 是整个 workspace 的总入口。它列出了默认成员：

- `kernel`
- `sel4_common`
- `sel4_cspace`
- `sel4_ipc`
- `sel4_task`
- `sel4_vspace`
- `rel4-arch`
- 其他一些支持 crate

同时也明确把以下内容排除在 workspace 之外：

- `rel4_config`
- `xtask`
- `dv`
- `tools/verus`
- `tools/verus-analyzer`

这意味着：

- `xtask` 是项目工具，但不是 workspace 成员 crate。
- Verus 二进制目录也不是 workspace 成员。
- `cargo build` workspace 时不会顺手把 `xtask` 当作一个普通业务 crate 一起编。

### 4.2 `rel4_config`

`rel4_config` 是当前构建系统里非常关键的一层。

它的职责不是“运行时配置”，而是“编译时平台配置与代码生成”。当前它做的事情包括：

- 读取 `cfg/platform/<platform>.yml`
- 生成 `config.h`
- 生成 `config.rs`
- 生成 `platform_gen.rs`
- 生成 `linker_gen.ld`
- 通过 `cpp` 预处理 `.S` / `.bf` 一类输入文件

因此，`rel4_config` 是“平台配置真源 + 生成工具库”。

### 4.3 各子模块自己的 `build.rs`

当前设计不是让 `xtask` 帮每个模块做所有底层生成，而是让每个模块自己的 `build.rs` 负责自己的那一小块生成。

这正是 v2 编译系统设计的核心思想之一：

> `xtask` 只做参数编排，不替代子模块内部的代码生成逻辑。

### 4.4 `xtask`

`xtask` 是一个独立的小 Rust 程序。它的职责是：

- 解析命令行参数
- 根据平台和选项选择 target
- 拼装 feature
- 设置 `PLATFORM`、`MARCOS`、`RUSTFLAGS`、`LOG`
- 调用 `cargo`
- 调用 `cmake`
- 调用 `ninja`
- 在 `run` 命令里进一步调用 `simulate`

所以，`xtask` 的本质是：

> 一个项目级工作流入口，而不是“内核模块本身的一部分”。

### 4.5 `tools/` 下的两个验证脚本

当前工具链中，和 Verus 相关的正式入口都在 `tools/` 下：

- `bootstrap-verus-release.sh`
- `verify-cspace-official.sh`

它们现在还没有整合进 `xtask`。

### 4.6 `specs/`

`sel4_cspace/specs/` 是当前形式化验证的主战场。

这里放的不是普通运行时代码，而是：

- 可信边界合同
- 抽象模型
- 原语规格
- 小引理
- smoke checks

它服务于 Verus 证明，而不是普通内核运行。

## 5. 最容易踩坑的一点：`xtask` 是 host 工具，但仓库默认 target 是裸机目标

这是当前工具链里最值得先讲清楚的点。

### 5.1 根因

仓库根 `.cargo/config.toml` 默认把构建 target 设成了：

```toml
[build]
target = "riscv64gc-unknown-none-elf"
```

这对内核 crate 来说是合理的，因为它们本来就是要编到目标架构上的。

但是 `xtask` 本身不是目标板上的程序，而是运行在你开发机上的 host-side 工具。它应该编到 host 目标上，例如：

```text
x86_64-unknown-linux-gnu
```

### 5.2 当前仓库怎么绕开这个问题

根 `.cargo/config.toml` 里已经定义了 alias：

```toml
[alias]
xtask = "run --manifest-path ./xtask/Cargo.toml --target=x86_64-unknown-linux-gnu --release --"
```

因此你平时应当优先使用：

```bash
cargo xtask build ...
```

而不是自己手写：

```bash
cargo run --manifest-path ./xtask/Cargo.toml ...
```

后者如果不显式指定 host target，很容易把 `xtask` 也编到 `riscv64gc-unknown-none-elf`，然后报 `std` 不存在之类的问题。

### 5.3 一个额外的小现状

当前 alias 里还有：

- `xrun = "xtask run"`
- `xbuild = "xtask build"`
- `xrelease = "xtask release"`

但当前 `xtask` 实际只有：

- `build`
- `install`
- `run`
- `clean`

并没有 `release` 子命令。

所以：

- `cargo xbuild`
- `cargo xrun`

可以视为 `cargo xtask build/run` 的快捷形式；

而：

- `cargo xrelease`

更像是一个遗留 alias，当前不应当当作可用正式入口。

## 6. 当前 `xtask` 到底能做什么

### 6.1 当前已有的子命令

当前 `xtask` 只暴露 4 个子命令：

- `build`
- `install`
- `run`
- `clean`

没有：

- `verify`
- `bootstrap-verus`
- `test`
- `release`

这说明当前 `xtask` 的定位仍然是“构建/运行工作流入口”，不是“所有开发任务的统一总控入口”。

### 6.2 `build` 的职责

`xtask build` 的定位是：

- 根据平台和开关设置好构建参数
- 调用 Rust 侧内核编译
- 根据情况与 seL4 project / cmake 流程结合

当前 `xtask build --help` 的主要参数有：

- `-p, --platform <PLATFORM>`
- `-m, --mcs`
- `-s, --smc`
- `--nofastpath`
- `--arm-pcnt`
- `--arm-ptmr`
- `--arm-hypervisor`
- `--rust-only`
- `-B, --bin`
- `-N, --num-nodes <NUM_NODES>`
- `--log <LOG>`
- `--benchmark`

当前支持的平台主要是：

- `spike`
- `qemu-arm-virt`

### 6.3 `build` 做了哪些内部动作

大致流程如下：

1. 按平台决定 target triple
- `spike -> riscv64gc-unknown-none-elf`
- `qemu-arm-virt -> aarch64-unknown-none-softfloat`

2. 组装 feature 和环境变量
- `PLATFORM`
- `MARCOS`
- `RUSTFLAGS`
- `LOG`

3. 在某些模式下选择编二进制还是库
- `--bin`
- `--rust-only`

4. 调用 `cargo build`

5. 在需要时配合 `cmake` / `ninja` 构建 seL4 project 侧内容

### 6.4 `run` 的职责

`xtask run` 会在 build 的基础上继续：

- 触发 `sel4test` / benchmark 对应的构建目录准备
- 调用 `simulate`

因此，`run` 是偏“构建 + 运行模拟测试”的命令。

### 6.5 `install` 的职责

`xtask install` 会：

- 做 Rust 侧构建
- 选择对应平台的 kernel settings cmake preload 文件
- 运行 `cmake`
- 再运行 `ninja all` 和 `ninja install`

它更接近“把整个内核安装到一个产物目录”。

### 6.6 `clean` 的职责

当前 `clean` 非常简单，主要是删掉 `target` 相关目录。

### 6.7 `xtask` 明确不做什么

当前 `xtask` 没有做下面这些事：

- 不直接负责 `kernel/build.rs` 或 `sel4_common/build.rs` 里的底层代码生成
- 不直接执行 Verus 证明
- 不提供统一的“验证所有模块”命令
- 不管理 Verus 二进制安装

也就是说：

> 它负责把“普通构建流程”串起来，但还不是“验证总入口”。

## 7. `build.rs` 与 `rel4_config` 当前是怎么配合的

这部分是当前 v2 编译系统真正落地的地方。

## 7.1 `kernel/build.rs`

`kernel/build.rs` 当前主要负责：

- 读取 `PLATFORM`
- 读取 `MARCOS`
- 调用 `rel4_config::generator::config_gen`
- 生成汇编输入：
  - `head.S`
  - `traps.S`
- 调用 `rel4_config::generator::linker_gen`
- 通过 `cargo:rustc-link-arg` 把 linker script 加到当前 crate 的链接参数里

因此，`kernel` crate 自己知道：

- 怎么生成自己需要的汇编入口
- 怎么接自己的 linker script

而不是让 `xtask` 代替它做这些内部细节。

## 7.2 `sel4_common/build.rs`

`sel4_common/build.rs` 当前主要负责两类工作：

1. 解析和生成位域/结构相关代码
- `structures.bf`
- `shared_types.bf`
- `pbf_parser`

2. 生成平台相关 Rust 代码
- `platform_gen.rs`

因此，`sel4_common` 承担了很多“公共底层结构与平台信息承载层”的职责。

## 7.3 `rel4_config` 现在提供的生成能力

`rel4_config/src/generator.rs` 里当前比较关键的生成接口有：

- `linker_gen(platform)`
- `platform_gen(platform)`
- `asm_gen(dir, name, inc_dir, defs, out)`
- `config_gen(platform, custom_defs)`

从职责上看：

- `rel4_config` 提供通用生成能力
- `build.rs` 决定当前 crate 具体要调用哪一个生成器
- `xtask` 负责把平台和宏定义喂给它们

这三者刚好形成一个清晰分层。

## 8. 当前 Verus 验证工具链长什么样

当前 Verus 验证链路还比较“手工”，但已经有一个官方入口。

## 8.1 第一个脚本：`bootstrap-verus-release.sh`

这个脚本只做一件事：

> 下载并安装官方 Verus release 二进制。

它的行为可以概括为：

1. 解析仓库根目录
2. 确定安装目录，默认是：

```text
tools/verus/release
```

3. 决定要下载哪个 release tag
4. 用 `curl` 拉取 release 信息
5. 找到 `x86-linux.zip`
6. 下载并解压
7. 把里面的：
   - `cargo-verus`
   - `verus`
   - `rust_verify`
   复制到安装目录

它不做：

- 项目构建
- 项目验证
- crate 选择

也就是说，它只是“装工具”，不是“跑证明”。

## 8.2 第二个脚本：`verify-cspace-official.sh`

这个脚本才是当前官方验证入口。

它的默认行为是：

- 使用 `tools/verus/release/cargo-verus`
- 设置验证用 Rust toolchain
- 设置 target triple
- 设置 `PLATFORM` 和 `MARCOS`
- 执行：

```bash
cargo-verus verify -p sel4_cspace --features verify
```

默认参数大致是：

- `VERIFY_TOOLCHAIN=1.94.0-x86_64-unknown-linux-gnu`
- `VERIFY_TARGET=riscv64gc-unknown-none-elf`
- `VERIFY_PACKAGE=sel4_cspace`
- `VERIFY_FEATURES=verify`
- `PLATFORM=spike`
- `MARCOS="KERNEL_STACK_BITS=12 FASTPATH=true HAVE_FPU=true RISCV_EXT_D=true"`

这说明两件事：

1. 当前验证工具链已经尽量复用了普通构建那一套环境约定
- 例如仍然用 `PLATFORM`
- 仍然用 `MARCOS`

2. 但它仍然是一个独立脚本入口
- 不是 `cargo xtask verify`

## 8.3 当前推荐的验证命令

第一次在这台机器上使用 Verus 时，先运行：

```bash
./tools/bootstrap-verus-release.sh
```

然后运行当前官方验证入口：

```bash
./tools/verify-cspace-official.sh
```

## 8.4 旧 `dv` 路线的状态

仓库里还留有一些历史痕迹，例如：

- `dv`
- `xtask.log.json`
- `docs/verification/archive/...`

它们说明仓库过去做过其他验证尝试。

但就“当前正式入口”而言，应该以：

- `tools/bootstrap-verus-release.sh`
- `tools/verify-cspace-official.sh`

这条官方 `cargo-verus` 路线为准，而不是旧 `dv` 路线。

## 9. 当前各模块在工具链中的状态

下面这张表描述的是“当前实际接线情况”，不是长期规划。

| 模块 | 当前角色 | 普通构建状态 | Verus 验证状态 | 说明 |
| --- | --- | --- | --- | --- |
| `rel4_config` | 平台配置与生成工具库 | 已使用 | 未作为独立验证目标接线 | 为 `build.rs`/`xtask` 提供生成能力 |
| `xtask` | host-side 构建编排工具 | 已使用 | 未接 Verus | 当前只有 `build/install/run/clean` |
| `sel4_common` | 公共底层结构、平台、公用桥接层 | 已使用 | 暂未作为正式验证目标 | 已有 `verify_bridge`，但更像后续 refinement 基础设施 |
| `sel4_cspace` | 当前 Verus 验证主目标 | 已使用 | 已正式接线 | 当前唯一明确接入 `[package.metadata.verus]` 的目标 |
| `kernel` | 顶层内核 crate | 已使用 | 未正式接线 | 通过 feature 聚合多个子模块 |
| `sel4_ipc` | IPC 相关模块 | 已使用 | 未正式接线 | 当前没有 Verus metadata |
| `sel4_task` | task 相关模块 | 已使用 | 未正式接线 | 当前没有 Verus metadata |
| `sel4_vspace` | vspace 相关模块 | 已使用 | 未正式接线 | 当前没有 Verus metadata |
| `rel4-arch` | 架构相关支持模块 | 已使用 | 未正式接线 | 当前没有 Verus metadata |

## 10. 为什么说目前只有 `sel4_cspace` 正式接入了 Verus

这是因为它同时满足了几件事。

### 10.1 它有明确的 Verus feature 线路

`sel4_cspace/Cargo.toml` 当前定义了：

- `verus = ["dep:vstd"]`
- `verify = ["verus"]`

这表示：

- `verus` feature 会引入 `vstd`
- `verify` 是当前实际用于验证入口的 feature

### 10.2 它有 package metadata

`sel4_cspace` 还声明了：

- `[package.metadata.verus]`
- `[package.metadata.rel4.verification]`

这说明它已经被明确视为“一个验证目标包”，而不是只是随便写了一点 Verus 风格代码。

### 10.3 它有实际的规格入口

当前 `sel4_cspace/src/lib.rs` 中有：

```rust
#[path = "../specs/lib.rs"]
pub mod specs;
```

这意味着 `specs/` 不是一个孤立草稿目录，而是已经和 crate 主入口接通了。

### 10.4 它有实际的 `specs` 结构

当前 `sel4_cspace/specs/` 里已经包括：

- `boundary_assumptions.rs`
- `abstract_cspace.rs`
- `cspace_ops/`
  - `common.rs`
  - `insert.rs`
  - `move.rs`
  - `resolve.rs`
  - `swap.rs`
  - `smoke.rs`

这表明它已经不再是“只有几个零散想法”，而是已经具备了：

- 边界合同层
- 抽象模型层
- 原语规格层
- 小引理/烟雾检查层

## 11. `sel4_common` 当前为什么还不能算“正式验证目标”

这是另一个很容易误判的地方。

`sel4_common` 当前确实已经出现了验证相关内容，例如：

- `verify` feature
- `verus` feature
- `verify_bridge.rs`

但它还不能被视为“当前已正式接入的验证目标”，原因在于：

1. 它没有 `sel4_cspace` 那样的 `[package.metadata.verus]`
2. 当前没有专门的官方脚本去运行 `cargo-verus verify -p sel4_common`
3. `sel4_cspace` 当前也没有把 `sel4_common/verify` 当作一个正式依赖链一路打开

因此更准确的描述应当是：

> `sel4_common` 目前是“验证辅助基础设施”和“未来 refinement 桥接层”的一部分，而不是当前官方验证主目标。

## 12. 当前 `sel4_cspace` 的验证做到哪一步了

从代码结构上看，当前 `sel4_cspace` 的 Verus 工作大致处在：

> 已完成可信边界、抽象模型、原语规格和首批可复用小引理包；第 4 步已收口，下一大步是 concrete view/refinement bridge，然后才是逐个 `src` 函数证明。

可以按下面这条路线理解：

1. 固定可信边界和门禁
- 已有 `boundary_assumptions.rs`

2. 建抽象模型和全局不变量
- 已有 `abstract_cspace.rs`

3. 为关键原语写抽象合同
- 已有 `cspace_ops/{insert,move,swap,resolve}.rs`

4. 补可复用的小引理和 smoke checks
- 已有 `cspace_ops/common.rs`
- 已有 `cspace_ops/smoke.rs`
- 当前 `cspace_ops` 也已经从单文件拆成模块化目录
- 当前还额外有：
  - `lemma_wf_implies_core_invariants`
  - `lemma_wf_implies_valid_slot_entry`
  - `lemma_resolve_pre_implies_base_invariants`
  - `lemma_resolve_pre_implies_root_lookup_ready`
- 这一步目前可以视为已收口

5. 建 concrete -> abstract 的 view / refinement bridge
- 这一步还没有成为当前仓库里的正式主线入口

6. 逐个证明 `src` 里的真实函数满足对应 spec
- 这一步还没成为当前代码结构里的已完成部分

7. 回归、文档、TCB 清单收口
- 文档侧已有不少材料，但代码主线仍主要停留在前半段

所以如果你想用一句更短的话来记当前状态，可以记成：

> `sel4_cspace` 目前已经把 spec 世界搭起来了，但 concrete `src` 世界还没有系统接桥、也还没有逐函数证明完。

## 13. 现在真正应该怎么使用这套工具链

如果你是第一次进入仓库，建议按下面顺序使用。

## 13.1 想做普通构建

优先使用：

```bash
cargo xtask build -p spike
```

如果想构建 binary 模式：

```bash
cargo xtask build -p spike --bin
```

如果只想偏 Rust 侧地构建：

```bash
cargo xtask build -p spike --rust-only
```

如果想直接构建并运行模拟：

```bash
cargo xtask run -p spike
```

如果想查看帮助：

```bash
cargo xtask build -h
```

## 13.2 想做 Verus 验证

先安装官方 Verus：

```bash
./tools/bootstrap-verus-release.sh
```

再跑当前官方验证入口：

```bash
./tools/verify-cspace-official.sh
```

## 13.3 想直接对单个模块用 `cargo build`

理论上可以，但不建议新手一开始就这样做。

原因是：

- 你需要自己处理 target
- 你需要自己处理 `PLATFORM`
- 你需要自己处理 `MARCOS`
- 你可能还要自己处理 feature

而这些本来就是 `xtask` 帮你统一组织的。

因此对新手来说，更推荐的心智模型是：

- 普通构建尽量走 `cargo xtask ...`
- Verus 验证尽量走 `tools/*.sh`

## 14. 常见误区

### 14.1 “我看到 `sel4_common` 里有 `verify_bridge.rs`，是不是它已经验证了？”

不是。

它更像是“为了后续验证准备的桥接基础设施”，而不是“当前已经正式接入并持续回归验证的目标”。

### 14.2 “我 `cargo build` 通过了，是不是就说明 `sel4_cspace` 证明了？”

不是。

普通编译通过和 Verus 证明通过是两回事。

### 14.3 “既然有 `xtask`，为什么不直接 `cargo xtask verify`？”

因为当前仓库还没有这个子命令。

现在验证入口仍然是 `tools/verify-cspace-official.sh`。

### 14.4 “为什么 `xtask` 自己会受 target 影响？”

因为仓库根默认 target 指向裸机目标，而 `xtask` 本身又是一个 Rust 程序。只要你用的是 `cargo run`，Cargo 就必须先把 `xtask` 编出来。

### 14.5 “`MARCOS` 是不是拼错了，应该叫 `MACROS`？”

从命名上看，它确实像是一个历史拼写遗留。

但当前代码和脚本里用的就是：

```text
MARCOS
```

所以在当前仓库里，这个名字不是“可以随便改写的注释”，而是实际接口的一部分。

### 14.6 “是不是整个 kernel 都在走 Verus 了？”

不是。

当前正式验证主线仍然集中在 `sel4_cspace`。

## 15. 如果后面要把别的模块也接进验证，大概要补哪些东西

假设未来要把某个模块也按当前风格接进 Verus，通常至少需要补下面几类东西：

1. 先定义验证边界
- 哪些外部模块先信任
- 哪些 FFI/底层内存访问暂时进 TCB

2. 建规格入口
- 为该模块建立自己的 `specs/`
- 在 crate 主入口接入 spec 模块

3. 配 feature
- 至少区分 `verus`
- 和实际验证入口 feature，例如 `verify`

4. 配 package metadata
- 类似 `[package.metadata.verus]`
- 以及项目内部的验证元数据

5. 如有需要，补 bridge 层
- 尤其是从 concrete bitfield / generated types 到抽象规格世界的桥

6. 提供官方验证入口
- 至少先有脚本
- 更进一步才是并入 `xtask verify`

7. 先从单模块开始
- 不要一上来试图“验证整个 kernel”

当前仓库的现实状态其实已经说明了一件事：

> 最可行的路线不是“先统一验证整个系统”，而是“先把一个模块从 spec 入口到 proof 流程打通，再逐步推广”。

## 16. 当前工具链还缺什么

如果从“一个更完整、更顺手的开发者体验”来看，当前工具链还缺的主要是下面几项：

### 16.1 缺统一的验证入口

当前普通构建走 `xtask`，形式化验证走 `tools/*.sh`。

这已经能用，但心智模型仍然是分裂的。

理想状态通常会是：

```bash
cargo xtask verify cspace
```

或者：

```bash
cargo xtask verify -p sel4_cspace
```

### 16.2 缺按模块推广的验证元数据约定

现在只有 `sel4_cspace` 比较完整地接上了这一套。

其他模块还没有形成统一的“接 Verus 的最小模板”。

### 16.3 缺更清晰的新手文档

这正是本文试图补的空白：

- `xtask` 是什么
- `tools` 脚本是什么
- `build.rs` 是什么
- 哪些模块已经验证
- 哪些模块还没有

## 17. 给新同学的最短上手建议

如果你是第一次碰这个仓库，我建议你先只记住下面 6 条：

1. 普通构建先用 `cargo xtask ...`，不要先自己拼一大串 `cargo build`。
2. `xtask` 是 host-side 工具，不是目标板上的程序。
3. `rel4_config + build.rs` 才是底层代码生成核心。
4. 当前 Verus 官方入口不在 `xtask`，而在 `tools/verify-cspace-official.sh`。
5. 当前正式接入 Verus 的模块只有 `sel4_cspace`。
6. `sel4_cspace` 当前主要完成了 spec/lemma 这半边，`src` 具体函数证明还不是主线已完成部分。

## 18. 一张最终总图

最后可以把当前状态总结成下面这张简图：

```text
普通构建世界
--------------
cargo xtask build/run/install
    -> xtask 解析参数
    -> 设置 PLATFORM / MARCOS / feature / target
    -> cargo build
    -> 各 crate build.rs
    -> rel4_config 生成 config/platform/linker/asm
    -> cmake/ninja/simulate（按命令需要）

形式化验证世界
--------------
./tools/bootstrap-verus-release.sh
    -> 安装官方 cargo-verus

./tools/verify-cspace-official.sh
    -> 设置验证 toolchain / target / PLATFORM / MARCOS
    -> cargo-verus verify -p sel4_cspace --features verify
    -> 读取 sel4_cspace/specs

模块覆盖现状
------------
sel4_cspace：正式验证主目标
sel4_common：桥接基础设施，尚非正式验证目标
其他模块：当前仍主要处于普通构建路径
```

---

如果你读完本文后还想继续往下走，最自然的下一步通常有两个方向：

- 想学“怎么构建/跑起来”：
  - 从 `cargo xtask build -h`
  - `cargo xtask build -p spike`
  - `cargo xtask run -p spike`
  开始。
- 想学“当前 Verus 到哪一步了”：
  - 从 `sel4_cspace/specs/`
  - `tools/verify-cspace-official.sh`
  开始。
