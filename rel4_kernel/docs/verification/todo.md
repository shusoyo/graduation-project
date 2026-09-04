# reL4 CSpace 验证项目（毕设）高层进度总览

本文档用于追踪整个毕业设计项目在宏观大方向上的进度。底层的具体改进细节请参考 `cspace-improvement-plan.md` 和 `cspace-verification-plan.md`。

## 🟢 已完成里程碑 (Completed)

- [x] **基础设施与门禁工程确立**
  - 构建了稳定的 Verus 验证工具链 (`check-cspace-build-and-verify.sh`)
  - 成功跑通 200+ 个验证目标，实现 0 error 的基线。
- [x] **核心原语闭环 (Engineering Loop)**
  - 完成抽象模型 (Abstract Model) 和 `wf` 状态定义。
  - 完成 primitive spec 规格编写。
- [x] **语义对齐与 TCB 冻结 (Semantic & TCB Freeze)**
  - 完成 10 步 verify-native 化计划的当前基线收口（`10 / 10`）。
  - facade 层接口隔离，稳定了 verify-facing 与 runtime path。
- [x] **达成毕设基线能力 (Baseline Delivered)**
  - 完成局部规约（Local Spec）向 l4v 的对应语义对齐。
  - 完成基本的精化桥接（Refinement Bridge）。

## 🟡 当前进行中：非删除主线 Verus 替换执行轮 (In Progress)

当前新的主执行面不是继续补“10 步 verify-native 化基线”，而是按 `cspace-improvement-plan.md` 推进“非删除主线最终替换”：

- [ ] **Step 1：模块边界重画**
  - [ ] 把 `specs / repr / body / interface` 四层职责在代码结构上拉开。
- [ ] **Step 2：表示层落地**
  - [ ] 建最小 `repr / owner / view / model` 骨架。
  - [ ] 为 `cap / cte / mdb / slot / resolve ret` 安排稳定落点。
  - [ ] 引入独立的 `memory_axioms`，收口地址/指针/小算术假设。
- [ ] **Step 3：capability query 主实现收合**
  - [ ] `same_region_as`
  - [ ] `same_object_as`
  - [ ] `is_cap_revocable`
- [ ] **Step 4：slot-local query / derive 主线替换**
  - [ ] `is_mdb_parent_of`
  - [ ] `is_final_cap`
  - [ ] `ensure_no_children`
  - [ ] `derive_cap`
- [ ] **Step 5：lookup 主线替换**
  - [ ] `resolve_address_bits`
- [ ] **Step 6：mutator family 替换**
  - [ ] `cte_insert`
  - [ ] `insert_new_cap`
  - [ ] `cte_move`
  - [ ] `cte_swap`
- [ ] **Step 7：delete 主线**
  - [ ] 当前冻结，不纳入本轮执行。

## 🟡 并行进行：论文与口径收尾 (In Progress)

- [ ] **毕业论文正文撰写 (`cspace-thesis-draft.md`)**
  - [ ] 按学校模板继续压缩摘要、引言、贡献与结论。
  - [ ] 绘制更清楚的 TCB 架构与状态转换图。
  - [ ] 保持论文口径与“非删除主线替换计划”一致。

## ⚪ 待启动：终验与答辩准备 (To Do)

- [ ] **全量代码清理与开源整理**
  - 剔除不再需要的历史调试注释、旧版本 bridge 废弃物。
- [ ] **编写答辩宣讲材料 (PPT)**
  - 重点突出分离逻辑的缺失导致的困境、契约设计的巧思以及 Refinement 对齐 l4v 的贡献。
- [ ] **最终预演与验收提交**
