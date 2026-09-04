# Step 3: Manager Ownership Migration

## Objective

Replace manager-owned raw slot permissions with `MdbTable`.

Old shape:

```rust
pub struct CSpaceManager {
    pub slot_perms: Tracked<Map<SlotPtr, PointsTo<cte_t>>>,
    pub zombie_slots: Ghost<Set<SlotPtr>>,
    pub cdt: Ghost<CdtState>,
}
```

Target shape:

```rust
pub struct CSpaceManager {
    pub mdb: MdbTable,
    pub zombie_slots: Ghost<Set<SlotPtr>>,
    pub cdt: Ghost<CdtState>,
}
```

## Migration Rule

Do not keep `slot_perms` in manager as a compatibility field.

Manager accessors should delegate to `self.mdb`:

- `slot_dom()`
- `get_slot_view()`
- `get_cap()`
- `get_prev()`
- `get_next()`
- `slot_is_empty()`

The old `mdb_state()` accessor should be removed from the active proof route.

## Manager WF Split

Manager wf becomes:

```rust
self.mdb.structural_wf()
&& cdt_proof::structural_wf_on(self.cdt@)
&& self.cross_layer_wf()
```

`cross_layer_wf` owns:

- null slot and CDT empty/original relation
- CDT parent cap semantics
- zombie slot constraints
- cap-specific MDB semantic constraints if they are not purely structural

## Done Criteria

- Manager no longer owns `slot_perms`.
- Manager no longer defines active `mdb_state()`.
- Manager proof code reads slot information through `self.mdb`.
