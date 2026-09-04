# Step 6: Follow-Up Operations

## Objective

After insert is stable, migrate the remaining MDB-affecting operations to the
same owner pattern.

## Recommended Order

1. `move_slot`
2. `swap_neighborhood`
3. `unlink/delete`

## `move_slot`

Move is the next operation because it mostly transfers one slot's cap/link
state into another slot and clears the source.

Expected MDB primitive:

```rust
MdbTable::move_slot(src, dest)
```

Expected ghost effect:

- `order` replaces `src` with `dest`
- `live_slots` removes `src` and inserts `dest`

## `swap_neighborhood`

Swap is more case-heavy because neighboring and non-neighboring swaps differ.

Expected MDB primitive:

```rust
MdbTable::swap_neighborhood(slot1, slot2)
```

Expected ghost effect:

- `order` swaps positions of `slot1` and `slot2`
- `live_slots` unchanged

## `unlink/delete`

Delete is last because it interacts with cap finalization, zombie logic, and
cleanup effects.

Expected MDB primitive:

```rust
MdbTable::unlink(slot)
```

Expected ghost effect:

- `order` removes `slot`
- `live_slots` removes `slot`

## Done Criteria

- All MDB structural mutations live in `MdbTable`.
- Manager ops call MDB primitives and focus on cross-layer semantics.
- Old manager-level MDB projection route is deleted.
