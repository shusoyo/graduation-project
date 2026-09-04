# 06 Open Questions

## 1. `specs/cspace_ops/*` 最终是保留还是删除

当前结论：

- 它们不是长期主证明中心。

待决问题：

- 最终保留成极薄 compatibility shim，
- 还是在对象层/subsystem 层完全稳定后继续内联或删除。

## 2. `verified/insert.rs` 能否继续变薄

当前结论：

- 它应该只是操作壳。

待决问题：

- 当 `CspaceCtx + SlotRef` 足够稳定后，
- `insert.rs` 是否还能继续缩，甚至只剩少量 raw write shell。

## 3. `CspaceCtx` 是否最终需要 resource/owner 层

当前结论：

- 现阶段不需要为了风格硬加。

待决问题：

- 当 move/swap 或更复杂 mutator 需要更强独占更新叙事时，
- 是否真的需要 tracked resource/owner 抽象。

## 4. `move/swap` 会不会逼出第二套 patch 结构

当前结论：

- 不应该。

待决问题：

- 现有 `PatchTouchedSlots` / `CspacePatchSpec` 是否足够泛化，
- 还是需要一个更抽象但仍保持简单的 patch core。

## 5. `repr/*` 的最小边界到底在哪里

当前结论：

- `repr/*` 不应再成为主语义中心。

待决问题：

- 哪些 raw-to-view helper 必须留在 `repr/*`
- 哪些其实可以进一步收进对象层。

## 6. `sel4_common` 的长期关系

当前结论：

- 本轮继续把它当 runtime truth。

待决问题：

- 未来如果要把整个 kernel 都做成同风格 verified subsystem 组合，
- 是否需要对 `sel4_common` 再建立更统一的 verified wrapper policy。
