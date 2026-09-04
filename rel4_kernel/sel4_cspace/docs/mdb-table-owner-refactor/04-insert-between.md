# Step 4: MDB Insert Primitive

## Objective

Move the MDB structural part of `cte_insert` into an `MdbTable` owner
primitive.

This is the first owner primitive and the main proof pilot for the refactor.

## Runtime Effect

The primitive patches the intrusive MDB list:

1. destination entry gets `prev = src`
2. destination entry gets `next = old_next`
3. destination entry gets `revocable/first_badged`
4. source entry gets `next = dest`
5. old next entry gets `prev = dest` if present

The current primitive in code is:

- `MdbTable::insert_node_after`

Cap-field writes, including `setUntypedCapAsFull`, are handled in manager code
before the MDB primitive call.

## Ghost Effect

`order` is updated by inserting `dest` immediately after `src`.

`live_slots` is updated by inserting `dest`.

## Post Relation

Expose one high-level relation:

```rust
pub open spec fn insert_between_rel(
    &self,
    new_table: &Self,
    src: SlotPtr,
    dest: SlotPtr,
    old_next: Option<SlotPtr>,
    revocable: bool,
    first_badged: bool,
) -> bool
```

The relation should state:

- `dom` unchanged
- `live_slots == old.live_slots.insert(dest)`
- `order` is old order with `dest` inserted after `src`
- changed link facts for `src`, `dest`, and `old_next`
- frame facts for all other live slots

## Contract

```rust
ensures
    self.structural_wf(),
    old(self).insert_between_rel(self, src, dest, old_next, revocable, first_badged)
```

Do not expose dozens of manager-facing postconditions unless a caller
genuinely needs them.

## Done Criteria

- `MdbTable::insert_node_after` owns concrete link patching.
- The primitive maintains `order/live_slots`.
- The primitive recovers `MdbTable::structural_wf`.
- Manager insert no longer hand-proves MDB exact post.
