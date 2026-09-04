# MDB Layer Refactor Plan

This document records the next proof-architecture direction for
`sel4_cspace`. It is motivated by the current proof-size problem: the CSpace
manager proof has grown far beyond the proof/code ratio that is acceptable for
this project. The goal is not to make the proof look prettier. The goal is to
move MDB structural reasoning behind a real lower-layer contract, so manager
operations stop re-proving linked-list graph facts by hand.

## Problem Statement

The current CSpace proof mixes four concerns in the same layer:

- MDB structural correctness: `prev` / `next`, cross links, no self links, no
  two-cycles, local link frame.
- CSpace semantic edges: parent, badge, and untyped incoming edge validity.
- Derivation and capability semantics: CDT parent, original bit, same-region,
  untyped-full side effects, revocability.
- Full manager `wf()` recovery: roots, zombies, resolve walk, arch caps, slot
  perms, and other non-MDB frame facts.

This makes small runtime mutations expensive. For example, an insert operation
touches only `src`, `dest`, and maybe `old_next`, but the proof must assemble a
large set of changed-slot and global frame obligations before it can recover
`wf()`.

The current `mdb_patch` vocabulary is useful, but it is still a proof closeout
vocabulary. It is not yet a true MDB abstraction boundary. Operation-specific
adapter lemmas such as `lemma_cte_insert_changed_slots_mdb_patch_ok_from_components`
still expose too much MDB detail to the manager proof.

## Baseline Methodology

This plan keeps the repository's existing methodology:

- Verus proof organization follows the `atmo` style.
- Semantic and contract strength are calibrated by `l4v`.
- Current claims remain manager-level CSpace core claims, not public-wrapper or
  whole-kernel claims.

The relevant `atmo` lesson is not that all helpers should be trusted forever.
The lesson is that low-level data-structure operations should have strong,
local contracts, and manager operations should consume those contracts instead
of expanding the data-structure proof every time.

The relevant `l4v` lesson is that MDB and CSpace semantic invariants must be
strong enough to preserve seL4-style behavior. This plan does not weaken the
semantic target; it changes where the proof is paid.

## Target Architecture

Introduce a real MDB layer between raw slot mutation and CSpace manager
operation proof.

```text
runtime tracked slot writes
  -> MDB layer helper contract
  -> manager operation semantic proof
  -> layered CSpace wf combiner
```

The MDB layer owns structural facts about the MDB graph. The manager operation
proof owns capability semantics and operation-specific postconditions.

### Desired Split

MDB layer proves or contracts:

- changed set for the MDB operation
- `prev` / `next` local shape
- cross-link preservation outside the changed set
- no self link and no two-cycle for changed slots
- `mdb_structural_wf` preservation
- MDB frame facts needed by the manager layer

Manager layer proves:

- functional postconditions, such as inserted cap equals `new_cap`
- same-region and revocability facts
- semantic edge facts for new or changed capability relationships
- CDT parent and original-bit derivation facts
- non-MDB frame facts that are genuinely outside MDB

Final operation wrapper proves:

- all required layers compose to `self.wf()`

## New Layer Vocabulary

The concrete names can still be adjusted during implementation, but the layer
should expose vocabulary like this.

```rust
pub open spec fn mdb_insert_between_post(
    old_mgr: CSpaceManager,
    new_mgr: CSpaceManager,
    prev: SlotPtr,
    slot: SlotPtr,
    next: Option<SlotPtr>,
) -> bool;

pub open spec fn mdb_move_slot_post(... ) -> bool;
pub open spec fn mdb_swap_slots_post(... ) -> bool;
pub open spec fn mdb_remove_slot_post(... ) -> bool;
```

The first pilot only needs `mdb_insert_between_post`.

Suggested ensures for the insert pilot:

```text
mdb_insert_between_post(old_mgr, new_mgr, prev, slot, next)
changed == set![prev, slot] or set![prev, slot, next]
slots_unchanged_except(old_mgr, new_mgr, changed)
mdb_cross_links_unchanged_except(old_mgr, new_mgr, changed)
changed_slots_local_structural_ok(new_mgr, changed)
```

The helper should not talk about CDT derivation, same-region, zombie semantics,
or resolve walk. Those are not MDB structural facts.

## Implementation Boundary

The MDB layer can initially use `external_body` helpers if needed, but only with
narrow MDB-structural contracts. This is an explicit engineering choice to get
the manager proof back to an acceptable size before proving every low-level MDB
primitive internally.

Acceptable temporary trusted boundary:

```text
MDB structural helper contract:
  raw prev/next mutation preserves MDB structural layer as specified
```

Unacceptable trusted boundary:

```text
whole cte_insert is correct
whole move/swap/delete is correct
semantic edge validity magically holds
derivation_wf magically holds
```

This keeps the trusted surface local and eventually removable.

## Layered `wf()` Plan

The current `wf()` remains the final public manager invariant, but proof should
stop recovering full `wf()` inside every intermediate lemma.

Introduce or expose stable layer predicates:

```text
basic_manager_wf
mdb_structural_wf
semantic_edge_wf
derivation_wf
non_mdb_frame_wf
```

The exact names should follow `spec_proof.rs`, but the split should be clear.

The final combiner should look conceptually like this:

```rust
pub proof fn lemma_cspace_wf_from_layers(mgr: CSpaceManager)
    requires
        mgr.basic_manager_wf(),
        mgr.mdb_structural_wf(),
        mgr.semantic_edge_wf(),
        mgr.derivation_wf(),
        mgr.non_mdb_frame_wf(),
    ensures
        mgr.wf();
```

Intermediate lemmas should require and ensure only the layer they actually
touch. For example, an MDB insert helper should not require or prove resolve
walk facts.

## Insert Pilot

Use `cte_insert` as the first pilot because it clearly shows the current problem:
the runtime mutation is small, but the changed-slot proof is large.

### Current Shape

Current closeout shape:

```text
tracked writes for dest/src/old_next
  -> cte_insert_local_post
  -> expected src/dest/old_next entries
  -> lemma_cte_insert_frame_from_tracked_ops
  -> lemma_cte_insert_changed_slots_mdb_patch_ok_from_components
  -> lemma_cte_insert_patch_non_mdb_frame_from_components
  -> lemma_cte_insert_derivation_wf
  -> lemma_patch_recovers_wf_from_obligations
```

This is single-route, but `lemma_cte_insert_changed_slots_mdb_patch_ok_from_components`
is too heavy. It performs MDB structural reasoning that should belong to the MDB
layer.

### Target Shape

Target closeout shape:

```text
tracked writes through mdb_insert_between helper
  -> mdb_insert_between_post
  -> manager insert semantic post
  -> derivation post for dest
  -> non-MDB frame post
  -> semantic-edge post for changed capability edges
  -> layered wf combiner
```

The `impl_insert.rs` proof block should no longer need to manually establish
old-next structural facts such as local double links, no self links, or no
two-cycles.

### Pilot Work Items

1. Add an MDB-layer module.

   Suggested file:

   ```text
   sel4_cspace/src/cspace_manager/spec_util/mdb_layer.rs
   ```

   Add it to `spec_util.rs` with minimal exports.

2. Define `mdb_insert_between_post` and `mdb_insert_between_changed_slots`.

   Keep this predicate structural. It should cover only slot entry MDB fields
   and frame outside the changed set.

3. Add a helper contract for the runtime mutation.

   Possible helper names:

   ```text
   mdb_insert_between_tracked
   mdb_link_insert_between_tracked
   ```

   The helper can initially be `external_body` if proving it would block the
   architecture experiment.

4. Rewrite `cte_insert` to consume the helper contract.

   Preserve the runtime order and old implementation correspondence. This is a
   proof refactor, not permission to obscure runtime semantics.

5. Delete or bypass the MDB-structural parts of
   `lemma_cte_insert_changed_slots_mdb_patch_ok_from_components`.

   The success criterion is not that the lemma gets renamed. The success
   criterion is that old-next structural case splits leave `spec_util/insert.rs`.

6. Keep semantic and derivation proof in `insert.rs`.

   `insert.rs` should still prove facts about:

   - `self.get_cap(dest_slot) == trusted_view_cap(new_cap)`
   - same-region after untyped-full source update
   - destination CDT parent
   - destination original bit
   - semantic edge validity not covered by MDB structure

## Handling `old_next_semantic_admissible`

The `old_next_semantic_admissible` precondition must not become a permanent
unexplained manager API burden.

Treat it as a named decision point with two possible outcomes.

### Preferred Outcome

Prove that the current manager preconditions imply it:

```text
old_mgr.wf()
old_mgr.get_next(src) == Some(old_next)
spec_same_region_as_caps(old_mgr.get_cap(src), new_cap)
operation-specific revocability facts
  ==> cte_insert_old_next_semantic_admissible(old_mgr, new_cap, old_next)
```

If this proof works, remove the requires clause from `cte_insert` and
`insert_new_cap`, and keep the lemma private to `spec_util/insert.rs`.

### Residual Outcome

If the implication does not hold, record it explicitly as a residual semantic
precondition:

```text
insert old-next semantic admissibility residual
```

Do not hide it inside a broad trusted bridge. Do not continue growing hundreds
of lines of caller-side admissibility proof during the MDB-layer pilot.

The residual must be documented as manager-level, not public-wrapper-level.

## Public Wrapper And Trusted Boundary Policy

During the MDB-layer pilot, do not work on public wrapper proof or broad trusted
boundary shrink.

In scope:

- manager-level `cte_insert` and `insert_new_cap`
- narrow MDB structural helper contracts
- local semantic and derivation proof needed by insert

Out of scope:

- proving `cte.rs` public wrappers
- whole-kernel or l4v-level refinement claims
- deleting all trusted helper contracts
- `set_empty` caller-admissibility residuals
- `reduce_zombie(true)` owner-shape residual

This is not a retreat. It is how the work stays comparable to the `atmo` style:
strong helper contracts first, then selected boundary shrink after the manager
core becomes maintainable.

## Success Metrics

The MDB-layer pilot succeeds only if it improves the engineering numbers and
the proof shape.

Required indicators:

- `impl_insert.rs` remains runtime-shaped.
- `impl_insert.rs` proof block gets shorter or simpler.
- `spec_util/insert.rs` old-next structural case split code is substantially
  reduced.
- Manager APIs do not gain new unexplained semantic-admissibility preconditions.
- `mdb_layer.rs` is smaller than the proof it replaces for the insert pilot.
- Full package verification remains green, or any temporary verification gap is
  explicitly documented with a narrow failing target.

Failure indicators:

- `mdb_layer.rs` becomes another large adapter file with the same case splits.
- `lemma_cte_insert_changed_slots_mdb_patch_ok_from_components` is merely
  renamed and not reduced.
- More operation-specific preconditions leak into `impl_insert.rs`.
- The pilot increases total proof size without deleting old proof.

## Rollout Order

Do not migrate all operations at once.

1. Freeze current `insert` proof surface.
2. Add the MDB-layer vocabulary and one `insert_between` helper.
3. Convert only `cte_insert` to consume the helper.
4. Measure line count and proof complexity.
5. If successful, convert `insert_new_cap`.
6. If both insert paths improve, migrate `move`.
7. Migrate `swap` only after the insert/move pattern is stable.
8. Touch `delete/set_empty` last, and only for already-isolated MDB structural
   proof. Do not enter delete residual semantics during this phase.

## Measurement Commands

Use these before and after the pilot:

```sh
wc -l sel4_cspace/src/cspace_manager/impl_insert.rs \
      sel4_cspace/src/cspace_manager/spec_util/insert.rs \
      sel4_cspace/src/cspace_manager/spec_util/mdb_patch.rs \
      sel4_cspace/src/cspace_manager/spec_util/mdb_layer.rs

rg -n "old_next_slot|changed_slots_mdb_patch_ok|old_next_semantic_admissible" \
      sel4_cspace/src/cspace_manager/impl_insert.rs \
      sel4_cspace/src/cspace_manager/spec_util/insert.rs \
      sel4_cspace/src/cspace_manager/spec_util/mdb_layer.rs

rg -n "lemma_patch_recovers_wf_from_obligations|lemma_cspace_wf_from_layers" \
      sel4_cspace/src/cspace_manager
```

Verification target for the first pilot:

```sh
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::impl_insert \
  --verify-function CSpaceManager::cte_insert
```

After the pilot succeeds, run the full package verification:

```sh
cargo xtask verify --package sel4_cspace --features ''
```

## Documentation Updates After Pilot

After `cte_insert` is converted, update these documents:

- `project-verification-map.md`: record whether the MDB-layer pilot reduced
  proof surface and whether `insert` remains manager-level only.
- `trusted-boundary-plan.md`: add the MDB structural helper contract as a named
  temporary trusted boundary if it uses `external_body`.
- `residual-tcb-checklist.md`: record the status of
  `old_next_semantic_admissible`.
- `mdb-patch-refactor-todo.md`: mark the old patch-closeout route as either
  superseded for insert or still retained as a final wf combiner.

## Non-Goals

Do not use this plan to justify unrelated rewrites.

- Do not port l4v proof script structure into Rust/Verus.
- Do not make runtime code less recognizable compared with the old seL4-shaped
  implementation.
- Do not expand delete/revoke semantics while proving the MDB-layer pilot.
- Do not make public wrappers part of the success criteria.
- Do not add generic abstractions unless they remove concrete insert proof.

## Expected End State

The end state for this phase is modest and concrete:

```text
insert manager proof = runtime-shaped code + small semantic proof
MDB structural proof = isolated helper layer
full wf recovery = layered combiner
trusted boundary = narrow, named, and later removable
```

If this cannot be achieved for `cte_insert`, the MDB-layer direction should be
stopped before migrating other operations.
