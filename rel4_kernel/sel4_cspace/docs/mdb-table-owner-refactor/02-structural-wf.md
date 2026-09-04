# Step 2: Structural WF

## Objective

Move MDB structural well-formedness into `MdbTable`.

This wf is about the intrusive doubly-linked list shape only. It must not
include cap derivation semantics or CDT semantics.

## WF Decomposition

Use small spec predicates instead of one large opaque expression:

```rust
pub open spec fn entries_wf(&self) -> bool;
pub open spec fn summary_wf(&self) -> bool;
pub open spec fn order_links_wf(&self) -> bool;
pub open spec fn structural_wf(&self) -> bool;
```

## `entries_wf`

Expected facts:

- every entry in `dom()` is initialized
- every entry address equals its slot pointer
- slot pointer `0` is not in `dom()`

## `summary_wf`

Expected facts:

- `order` has no duplicates
- `live_slots == order.to_set()`
- `live_slots` is a subset of `dom()`

## `order_links_wf`

Expected facts:

- if `order` is empty, `live_slots` is empty
- the first live slot has no prev
- the last live slot has no next
- adjacent slots in `order` agree with concrete `next` and `prev`
- live slots have link endpoints inside `live_slots` when present
- non-live slots do not participate in the live MDB chain

## Derived Properties

These should be proved as small lemmas only when needed:

- no self link
- no local two-cycle
- no cycle in the live chain
- link endpoints stay in `live_slots`

Do not add a large lemma suite up front.

## Done Criteria

- `MdbTable::structural_wf()` is the MDB owner wf.
- `CSpaceManager::wf()` can refer to `self.mdb.structural_wf()`.
- Existing `mdb/proof.rs` manager-dependent structural lemmas become obsolete
  for the insert path.
