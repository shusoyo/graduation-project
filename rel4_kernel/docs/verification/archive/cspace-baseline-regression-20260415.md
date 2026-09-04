# CSpace Verus 验证基线记录（2026-04-15）

## 1. 验证分支与冻结点

- 验证分支：`verif/cspace-verus-stage1-20260415`
- 冻结提交：`78330223bc5d11c3a37286d44e78b4956a789b7a`（短哈希：`7833022`）
- 冻结标签：`baseline/prod-freeze-20260415-cspace`
- 备注：该阶段分支由先前验证分支 `verif/cspace-verus-baseline-20260415` 切出。
- 说明：验证改动后续仅在验证分支进行，发布分支不直接承载验证实验性变更。

## 2. 基线回归命令与结果

### 2.1 Kernel 集成编译（xtask）

- 命令：`cargo xtask build --platform spike --rust-only`
- 结果：**通过（exit code = 0）**
- 备注：存在若干 warning（包括 dead_code / static mut refs / cfg 拼写警告），但不影响本次基线通过判定。

### 2.2 项目测试流程（xtask）

- 命令：`cargo xtask run --platform spike`
- 结果：**通过（exit code = 0）**
- 备注：本仓库以 xtask 为主流程，已按项目标准命令完成测试基线记录。

## 3. 基线结论

- 结论：在冻结点 `7833022` 上，按项目标准 xtask 流程，
  - Kernel 集成编译通过；
  - xtask run 流程通过。
- 后续所有验证改动均以本文件结果作为对照基线。

## 4. 第 11 步运行方式与版本决策（2026-04-15）

- 验证工作流：采用官方 `cargo-verus`（dv 路线已下线）。
- Verus 来源：upstream Verus release zip（通过 `tools/bootstrap-verus-release.sh` 下载并安装到 `tools/verus/release`）。
- 目标版本：默认 `VERUS_RELEASE=release/0.2026.04.12.f1166c4`，可切换为 `VERUS_RELEASE=latest` 或指定 tag。
- 说明：生产内核构建与验证工具链分离管理，默认生产构建路径不受影响。

## 5. 第 11 步历史记录归档（2026-04-16）

- 历史 dv 排障记录已迁移到归档文档：
  - `docs/verification/archive/cspace-step11-dv-history-20260415.md`
- 主文档仅保留当前官方执行入口与结果，避免与现行流程混淆。

## 6. 2026-04-16 更新（dv 删除后）

- 当前推荐入口（官方）：
  1. `./tools/bootstrap-verus-release.sh`
  2. `./tools/verify-cspace-official.sh`
- 默认验证目标：`riscv64gc-unknown-none-elf`。
- 实测结果：`sel4_cspace` 验证可稳定得到 `1 verified, 0 errors`。
- 说明：历史 dv 排障细节已迁移到 archive；本页仅作为当前官方执行手册。
