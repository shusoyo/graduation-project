# Step 1: MDB Table Shape

## Objective

Introduce `cspace::mdb::table::MdbTable` as the owner of CSpace slot entries.
This step establishes the shape only; it should not preserve old manager
projection APIs except as temporary call sites being migrated.

## Target Struct

```rust
pub struct MdbTable {
    pub entries: Tracked<Map<SlotPtr, PointsTo<cte_t>>>,
    pub order: Ghost<Seq<SlotPtr>>,
    pub live_slots: Ghost<Set<SlotPtr>>,
}
```

## Required Accessors

`MdbTable` exposes slot-level and graph-level spec accessors:

```rust
pub open spec fn dom(&self) -> Set<SlotPtr>;
pub open spec fn entry_view(&self, slot: SlotPtr) -> SlotEntrySpec;
pub open spec fn cap_of(&self, slot: SlotPtr) -> CapSpec;
pub open spec fn prev_of(&self, slot: SlotPtr) -> Option<SlotPtr>;
pub open spec fn next_of(&self, slot: SlotPtr) -> Option<SlotPtr>;
pub open spec fn revocable_of(&self, slot: SlotPtr) -> bool;
pub open spec fn first_badged_of(&self, slot: SlotPtr) -> bool;
pub open spec fn links(&self, left: SlotPtr, right: SlotPtr) -> bool;
pub open spec fn slot_is_empty(&self, slot: SlotPtr) -> bool;
```

These accessors are the flat graph interface. No separate public `MdbState`
object is needed for the first refactor pass.

## Constructor

Add `MdbTable::from_entries` as the population boundary:

```rust
pub fn from_entries(
    Tracked(entries): Tracked<Map<SlotPtr, PointsTo<cte_t>>>,
    Ghost(order): Ghost<Seq<SlotPtr>>,
    Ghost(live_slots): Ghost<Set<SlotPtr>>,
) -> (ret: MdbTable)
```

This can be `external_body` initially, but its contract must describe the new
owner invariant. Do not route through `CSpaceManager`.

## Done Criteria

- `MdbTable` exists in `cspace::mdb::table`.
- `mdb/mod.rs` re-exports `MdbTable`.
- Accessors compile independently of `CSpaceManager`.
- No cap/CDT cross-layer condition appears in `MdbTable`.
