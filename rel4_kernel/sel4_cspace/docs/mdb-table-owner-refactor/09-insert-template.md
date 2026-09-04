# Step 9: Insert Template For Later Ops

This file freezes the workflow and style that worked for `insert`, and should be
used as the template for `move`, `swap`, `delete`, and any later manager op.

## Target Layering

Keep this split fixed:

- `manager` owns cap semantics, CDT semantics, zombie bookkeeping, and final
  bundled op postconditions
- `mdb` owns only MDB links, `cteMDBNode` patching, `order/live_slots`, and MDB
  structural post-state relations
- `cte::payload` owns cap-field writes

Do not move cap semantics into `mdb`.
Do not move manager bundled op semantics into `mdb`.
Do not reintroduce a global `MdbState` projection layer just to state a local
  MDB fact.

## Canonical Operation Shape

Each manager op should follow this order.

1. Read runtime data needed to compute manager-level semantic parameters.
2. Take the needed entry perms from `self.mdb`.
3. Perform cap payload writes through `cte::payload` helpers.
4. Put the entry perms back into `self.mdb`.
5. Snapshot the payload-updated manager/MDB state as `pre_mdb_*`.
6. Use an `MdbTable` owner proof helper to transfer preconditions through
   `same_mdb_links`.
7. Call the `MdbTable` primitive (`insert_node_after`, `move_node`,
   `swap_nodes`, `remove_node`).
8. Update manager-owned ghost state (`cdt`, `zombie_slots`, and any manager
   semantic parameters).
9. Use an `MdbTable` owner proof helper to lift the primitive exact post from
   `pre_mdb_*` back to the old manager snapshot.
10. Finish with CDT structural lemmas and the manager-level `wf` combiner lemma.

## Required Contract Style

Use two layers of postconditions.

### MDB layer

`MdbTable` should expose:

- a primitive relation such as `insert_between_rel`, `move_slot_rel`,
  `swap_slots_rel`, `remove_slot_rel`
- a small owner-local bridge lemma family based on `same_mdb_links`

These lemmas should live as methods on `impl MdbTable`, not in a free proof
module.

### Manager layer

`CSpaceManager` should expose one bundled op relation, such as:

- `cte_insert_rel`
- `cte_move_rel`
- `cte_swap_rel`

That bundled relation may mention:

- cap post-state
- CDT post-state
- one call to the MDB owner relation

It should not restate the exact MDB field patch in manager space.

## Preferred Proof Shape

Prefer expanded local proof blocks over remote helper chains.

Good:

- manager body computes `dest_original`, `src_parent`, `old_next`
- manager body calls `old_mgr.mdb.lemma_same_links_then_insert_between_rel(...)`
- manager body calls `manager_spec::lemma_insert_preserves_manager_semantics_wf(...)`

Avoid:

- free functions whose only job is to rename one manager-local expression
- `mdb` helpers that talk about manager semantics
- duplicated field-by-field bridge logic in manager code when `MdbTable` already
  owns the primitive relation

## Problems Seen During Insert Refactor

These are the failure modes that already happened and should not be repeated.

### 1. Wrong layer absorbed cap semantics

Bad direction:

- pushing cap/original/parent semantics into `mdb`
- inventing `capstate`-style glue inside `mdb`/`cdt`

Correct direction:

- `mdb` talks only about links and summary
- manager computes cap-derived semantic parameters and bundles them

### 2. Free proof modules drifted away from the owner

Bad direction:

- `mdb/proof.rs` carrying owner-local transfer lemmas

Correct direction:

- if the lemma is about `same_mdb_links` preserving or lifting an MDB relation,
  it belongs on `MdbTable`

### 3. Manager restated MDB exact post

Bad direction:

- manager-level ensures or helpers spelling out `prev/next/revocable/first_badged`
  updates directly

Correct direction:

- manager post should call one bundled MDB relation
- exact MDB field patch remains inside `MdbTable::*_rel`

### 4. Compatibility helpers accumulated

Bad direction:

- keeping old bridge helpers alive just to avoid touching callers

Correct direction:

- once the owner API exists, migrate callers and delete the old route

## Review Checklist For The Next Op

Before calling an op migration done, check all of these.

1. The manager body is not `external_body`.
2. Cap writes happen before the MDB primitive call.
3. MDB structural mutation happens only through an `MdbTable` primitive.
4. The manager body calls `MdbTable` owner proof helpers, not a free `mdb_proof` module.
5. The bundled manager relation mentions the MDB relation once, not a field-level rewrite.
6. No new compatibility helper was kept just because an old caller existed.
7. The affected modules verify.

## Default Migration Order

For each later op, use this order:

1. define or clean the MDB primitive relation
2. define or clean the MDB owner bridge lemmas
3. rewrite the manager exec body to call the MDB primitive
4. rewrite the bundled manager relation if needed
5. delete the obsolete compatibility path
6. verify the op module, `mdb::table`, manager spec, and one dependent caller

This is the template unless a later op has a concrete reason to differ.
