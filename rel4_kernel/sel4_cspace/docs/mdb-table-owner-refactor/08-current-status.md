# Step 8: Current Status

## Insert Milestone

`CSpaceManager::cte_insert` and `CSpaceManager::insert_new_cap` are now on the
new owner path.

Current shape:

- manager owns `mdb: MdbTable`
- both insert operations call `MdbTable::insert_node_after`
- MDB structural mutation is no longer hand-expanded in manager insert code
- manager insert code computes cap/CDT parameters and updates `self.cdt`
- manager insert code recovers total `wf` through a manager-level combiner lemma
- `mdb` same-links bridge lemmas now live on `MdbTable` itself instead of a free
  `mdb::proof` module

Verified evidence:

```text
cargo xtask verify --package sel4_cspace --jobs 1 --max-errors 20 -- \
  --verify-only-module cspace::manager::impl_insert \
  --verify-function CSpaceManager::cte_insert
```

Result:

```text
verification results:: 1 verified, 0 errors
```

Additional insert-line check:

```text
cargo xtask verify --package sel4_cspace --jobs 1 --max-errors 20 -- \
  --verify-only-module cspace::manager::impl_insert \
  --verify-function CSpaceManager::insert_new_cap
```

Result:

```text
verification results:: 1 verified, 0 errors
```

Full insert module check:

```text
cargo xtask verify --package sel4_cspace --jobs 1 --max-errors 20 -- \
  --verify-only-module cspace::manager::impl_insert
```

Result:

```text
verification results:: 2 verified, 0 errors
```

Build evidence:

```text
cargo check -p sel4_cspace
```

Result:

```text
Finished dev profile
```

## Trusted Boundary

The current insert line intentionally keeps two trusted bridges:

- `MdbTable::insert_node_after`
- `lemma_insert_preserves_manager_semantics_wf`

This is the staged boundary for the owner refactor. The manager no longer owns
MDB link patch proof for insert, but the table primitive itself is still a
trusted owner primitive until its internal proof is opened.

## Removed Old Route

The old-state MDB projection route has been removed from the current source:

- `CSpaceManager::mdb_state()`
- `cspace::mdb::state`
- `cspace::mdb::spec`
- old `MdbState` transition lemmas

The temporary free `cspace::mdb::proof` bridge file is also gone now. Those
owner-local bridge lemmas were moved onto `MdbTable` so manager code calls the
owner directly.

The remaining unmigrated operations no longer expose old `MdbState` post-state
contracts. Their MDB post-state must be rebuilt with `MdbTable` owner
primitives instead of reintroducing a compatibility projection.

## Next Migration Order

1. Open or strengthen `MdbTable::insert_node_after`.
2. Add owner primitives for move and swap.
3. Migrate delete/unlink last, because it has the widest dependency cone.
