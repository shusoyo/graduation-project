# Step 7: Cleanup Rules

## Objective

Keep the refactor from preserving old proof debt under new names.

## Delete Rather Than Wrap

When a new owner path exists, delete the old manager-level route instead of
wrapping it.

Examples:

- delete active uses of `CSpaceManager::mdb_state()`
- delete manager-dependent MDB structural preservation lemmas
- delete helper functions that only bridge old `slot_perms` ownership

## Helper Budget

Add a helper only when:

- the same proof obligation appears in at least two operations, or
- the helper names a stable layer-local concept, or
- the helper hides unavoidable solver trigger details

Do not add helpers merely to make one failing assert pass.

## Forbidden Shapes

Avoid:

- `Ghost<MdbState>` inside `MdbTable`
- `MdbTable` carrying CDT information
- MDB structural wf mentioning `spec_should_be_parent_of`
- manager owning both `MdbTable` and raw `slot_perms`
- public postconditions that expose every internal patch when a relation is enough

## Review Checklist

For each migrated operation:

1. Does runtime still match `reference_0ca248f`?
2. Is MDB structural mutation owned by `MdbTable`?
3. Does manager avoid direct MDB projection proof?
4. Are cap/CDT semantics still in manager?
5. Did the refactor delete old responsibility instead of adding a shim?

## Completion Criterion

The refactor is complete when insert, move, swap, and delete no longer depend
on manager-owned MDB projection and the manager no longer stores raw
`slot_perms`.

## Current Cleanup Boundary

Do not keep compatibility shims once an operation migrates. For now,
`cte_insert` is the migrated operation. The old projection route may remain
only for unmigrated operations, and every remaining use should be treated as a
deletion target for the corresponding operation migration.
