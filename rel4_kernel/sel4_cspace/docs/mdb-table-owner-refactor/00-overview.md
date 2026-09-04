# MDB Table Owner Refactor Overview

## Goal

Refactor the CSpace MDB layer so that MDB structure ownership lives in an
Atmo-style owner object instead of being projected globally by
`CSpaceManager`.

The target owner is:

```rust
pub struct MdbTable {
    pub entries: Tracked<Map<SlotPtr, PointsTo<cte_t>>>,
    pub order: Ghost<Seq<SlotPtr>>,
    pub live_slots: Ghost<Set<SlotPtr>>,
}
```

`entries` is the only concrete truth. `order` and `live_slots` are proof
summaries maintained by the owner. They are not a full ghost mirror of every
MDB field.

## Non-Goals

- Do not preserve the old `manager.mdb_state()` route as a compatibility layer.
- Do not store `Ghost<MdbState>` inside `MdbTable`.
- Do not move cap/CDT semantics into MDB structural wf.
- Do not create a second MDB node type; use the existing `mdb_node`.
- Do not add broad helper APIs until a repeated proof need is visible.

## Layer Boundaries

`cte_t` remains the runtime slot entry:

- `capability`
- `cteMDBNode`

`mdb_node` remains the raw link/meta representation:

- `prev`
- `next`
- `revocable`
- `first_badged`

`MdbTable` owns the tracked entries and MDB structural proof:

- slot entry accessors
- flat graph accessors
- structural wf
- MDB primitive mutations

`CSpaceManager` composes layers:

- cap semantics
- CDT update parameters
- zombie constraints
- cross-layer wf

## Proof Style

The MDB proof language is a flat graph vocabulary attached to `MdbTable`:

- `dom`
- `prev_of`
- `next_of`
- `revocable_of`
- `first_badged_of`
- `links`
- `live_slots`
- `order`

The table's ghost summaries replace repeated global projection. Mutation
proofs update summaries incrementally and prove concrete links stay aligned
with them.

## First Milestone

The first verified operation after the refactor is `cte_insert`.

The initial runtime target is unchanged:

1. set source untyped cap full when needed
2. write destination entry
3. patch `src.next`
4. patch `old_next.prev` when present
5. update CDT ghost

The ownership target changes: MDB structural mutation moves from manager into
`MdbTable::insert_between`.

Current status: this milestone is reached for the manager-level `cte_insert`
line. The manager now calls `MdbTable::insert_node_after`, and the selected
Verus check for `CSpaceManager::cte_insert` passes. The owner primitive is
still trusted and should be opened in a later step.
