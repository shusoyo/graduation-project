# Step 5: Manager `cte_insert`

## Objective

Rewrite manager `cte_insert` so it orchestrates layers instead of proving MDB
structure directly.

## Target Responsibilities

Manager insert should:

1. read source cap and old next through `self.mdb`
2. compute cap-derived booleans:
   - `revocable`
   - `first_badged`
   - `src_parent`
   - `dest_original`
3. call MDB owner primitive:
   - `write_entry` or equivalent
   - `insert_between`
4. update CDT ghost:
   - `state_after_cap_insert`
5. recover manager wf from:
   - `self.mdb.structural_wf()`
   - CDT structural preservation
   - manager cross-layer conditions

## What To Remove

Remove from active insert proof:

- manager-local `old_mdb/new_mdb`
- manager-level `mdb_state() == old.mdb_state().state_after_*`
- direct calls to MDB state preservation lemmas from manager
- expanded concrete MDB link assertions except where needed for cap semantics

## Expected Contract Shape

`cte_insert` should expose:

- `self.wf()`
- domain unchanged
- destination cap/link result
- source cap frame
- patch frame for unrelated slots
- CDT post-state
- MDB post as `MdbTable::insert_between_rel(...)`, if needed by callers

The contract should not repeat the whole low-level proof script in ensures.

## Done Criteria

- `cte_insert` calls `MdbTable::insert_node_after`.
- MDB structural proof is owned by `MdbTable`.
- Manager proof is shorter and focused on cap/CDT/cross-layer facts.

## Current Result

The current version uses `MdbTable::insert_node_after` for MDB link insertion,
while keeping cap payload writes in manager code. `cte_insert` is verified as a
manager-level orchestrator:

- compute `old_next`
- compute cap-derived `revocable` / `dest_original`
- call the MDB owner primitive
- update CDT ghost state
- call the manager combiner for cross-layer `wf`

The old manager-local slot-permission patch sequence has been removed from the
active `cte_insert` body.

`insert_new_cap` has also been moved to the same owner primitive. The current
split keeps `setUntypedCapAsFull` decisions entirely in manager code rather
than encoding them as an MDB primitive mode flag.
