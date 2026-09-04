# MDB Patch Replacement Plan

This document replaces the previous A/B/C/D/E/F/G checklist. The old checklist
mixed three different goals:

- proof-architecture cleanup
- residual trusted-boundary shrink
- code-size reduction

That was not a good refactor plan. The original motivation was that the proof
code was too long, so the plan below is completion-state driven: the shared
`mdb_patch` route is only considered successful when it replaces old
operation-specific closeout paths instead of sitting next to them.

## Current Diagnosis

Current `sel4_cspace/src/cspace_manager` diff at the latest checkpoint:

```text
4588 insertions, 411 deletions
```

Largest growth points:

```text
impl_swap.rs                         +1660 / -55
spec_util/insert.rs                  +1365 / -11
spec_util/delete.rs                  +517  / -12
spec_util/move.rs                    +334  / -2
spec_util/swap/post.rs               +220  / -0
spec_util/swap/wf.rs                 +304  / -124
```

Interpretation:

- `mdb_patch` is useful as a proof-architecture direction.
- The current implementation is still an intermediate state.
- The current problem is not that `mdb_patch` exists; the problem is that the
  new route has not yet replaced enough old operation-specific proof.
- Continuing residual-TCB work now would likely increase code size further.

Latest full-package checkpoint after the narrative-refactor pass:

```sh
cargo xtask verify --package sel4_cspace --features ''
# verification results:: 463 verified, 0 errors
```

## Source Of The Refactor Idea

The architecture is not invented from nowhere, but its code-size benefit still
has to be earned by deletion.

The Verus organization is inspired by atmo-style component recovery:
tracked mutation first, then recover `wf` component by component using reusable
lemmas such as `container_no_change_to_tree_fields_imply_wf(...)` and
`process_no_change_to_trees_fields_imply_wf(...)`.

Relevant references:

- `sel4_cspace_backup/aux/atmosphere-main/kernel/verified/process_manager/impl_base.rs`
- `sel4_cspace_backup/aux/atmosphere-main/kernel/verified/process_manager/impl_drop_enpoints.rs`

The MDB invariant taxonomy is calibrated by l4v, especially the idea that MDB
preservation should live in reusable contexts/lemmas instead of every operation
re-proving the whole story independently.

Relevant references:

- `sel4_cspace_backup/aux/l4v_cspace_extracted/proof/invariant-abstract/CSpace_AI.thy`
- `vmdb_abs`
- `mdb_insert_abs`
- `mdb_insert_abs_sib`

What these sources justify:

```text
tracked mutation -> local patch facts -> reusable invariant recovery
```

What they do not justify by themselves:

```text
this refactor will automatically reduce Verus line count
```

Line-count reduction must come from replacing and deleting old proof routes.

## Completion Definition

The refactor is complete only when each operation has one canonical `wf` route:

```text
operation exact/local post
  -> operation-specific MDB patch obligations
  -> shared mdb_patch closeout
  -> wf()
```

The refactor is not complete if the operation still has two parallel routes:

```text
old component-wise wf recovery
+ new mdb_patch obligations
+ compatibility wrappers
+ final wf closeout
```

Concrete completion criteria:

- Each migrated operation has exactly one documented canonical closeout path.
- Old component-wise wrappers are deleted or clearly demoted to private local
  helpers that feed the canonical route.
- `impl_*` files stay runtime-shaped and do not host large reusable proof
  scripts.
- Every new shared helper must delete more operation-specific proof than it
  introduces.
- Residual trusted-boundary shrink is not part of this replacement phase.
- Full package verification remains green.

## Non-Goals For This Phase

Do not do these during replacement:

- Do not discharge `insert_new_cap_old_next_semantic_admissible(...)`.
- Do not discharge `cte_insert_old_next_semantic_admissible(...)`.
- Do not prove the four remaining `set_empty` caller-admissibility bridges.
- Do not strengthen `reduce_zombie(true)` owner-shape contracts.
- Do not add generic helpers only because they make the proof story prettier.
- Do not split files merely for aesthetics if the proof routes remain duplicated.

Those are trusted-boundary shrink tasks. They may be valuable later, but they are
not the current code-size / replacement objective.

## Phase 0: Route And Deletion Audit

Goal: build a deletion-driven map before any more proof is written.

For each operation, produce a table with this shape:

```text
lemma/function | file | current callers | role | decision | reason
```

Allowed decisions:

```text
keep       required proof, not duplicated
move       reusable proof currently in the wrong file
merge      duplicate pattern should be represented once
delete     no callers or compatibility-only wrapper
postpone   deletion would require a large new proof
```

Audit commands:

```sh
git diff --numstat -- sel4_cspace/src/cspace_manager
rg "lemma_name" sel4_cspace/src
rg "external_body" sel4_cspace/src/cspace_manager
rg "lemma_patch_recovers_wf_from_obligations|patch_derivation_obligations|changed_slots_.*_ok" sel4_cspace/src/cspace_manager
```

Acceptance criteria:

- The audit identifies concrete deletion candidates before implementation.
- The audit separates code-size work from trusted-boundary shrink.
- No code is changed in this phase except the audit document if one is created.

Status: complete as a document-level audit.

Audit result:

```text
lemma/function | file | current callers | role | decision | reason
lemma_cte_move_frame_from_components | impl_move.rs, spec_util/move.rs | cte_move closeout | structural frame producer | keep/postpone | still feeds the local patch_frame assertion; replacing it requires a small incoming-parent-cap frame helper
lemma_cte_move_changed_slots_ok_from_components | spec_util/move.rs | lemma_cte_move_changed_slots_mdb_patch_ok_from_components | private changed-slot assembly | keep | it is already private and feeds the canonical mdb_patch obligation wrapper
lemma_cte_move_changed_slots_mdb_patch_ok_from_components | impl_move.rs, spec_util/move.rs | cte_move closeout | canonical changed-slot obligation producer | keep | this is the current mdb_patch route entry for move
lemma_cte_move_patch_non_mdb_frame_from_components | impl_move.rs, spec_util/move.rs | cte_move closeout | non-MDB frame producer | keep | still required by lemma_patch_recovers_wf_from_obligations
lemma_cte_move_derivation_wf | impl_move.rs, spec_util/move.rs | cte_move closeout | derivation obligation producer | keep | already outputs patch_derivation_obligations
lemma_cte_move_changed_zombie_slots_sound | impl_move.rs, spec_util/move.rs | cte_move closeout | zombie sound obligation producer | keep | still feeds patch_non_mdb_frame

lemma_insert_new_cap_frame_from_tracked_ops | impl_insert.rs, spec_util/insert.rs | insert_new_cap closeout | patch_frame producer | keep | already consumes lemma_patch_frame_from_components and feeds canonical route
lemma_cte_insert_frame_from_tracked_ops | impl_insert.rs, spec_util/insert.rs | cte_insert closeout | patch_frame producer | keep | already consumes lemma_patch_frame_from_components and feeds canonical route
lemma_insert_new_cap_patch_non_mdb_frame_from_components | impl_insert.rs, spec_util/insert.rs | insert_new_cap closeout | non-MDB frame producer | keep/merge candidate | paired with cte_insert version; merge only if net code decreases
lemma_cte_insert_patch_non_mdb_frame_from_components | impl_insert.rs, spec_util/insert.rs | cte_insert closeout | non-MDB frame producer | keep/merge candidate | paired with insert_new_cap version; merge only if net code decreases
lemma_insert_new_cap_changed_slots_ok_from_components | spec_util/insert.rs | lemma_insert_new_cap_changed_slots_mdb_patch_ok_from_components | private changed-slot assembly | keep | already private and feeds canonical wrapper
lemma_cte_insert_changed_slots_ok_from_components | spec_util/insert.rs | lemma_cte_insert_changed_slots_mdb_patch_ok_from_components | private changed-slot assembly | keep | already private and feeds canonical wrapper
lemma_insert_new_cap_changed_slots_mdb_patch_ok_from_components | impl_insert.rs, spec_util/insert.rs | insert_new_cap closeout | canonical changed-slot obligation producer | keep | current shared-route entry
lemma_cte_insert_changed_slots_mdb_patch_ok_from_components | impl_insert.rs, spec_util/insert.rs | cte_insert closeout | canonical changed-slot obligation producer | keep | current shared-route entry
lemma_insert_new_cap_old_next_slot_ok | spec_util/insert.rs | insert_new_cap changed-slot lemmas | old-next local obligation | keep/postpone | do not prove or remove old-next residual during replacement
lemma_cte_insert_old_next_slot_ok | spec_util/insert.rs | cte_insert changed-slot lemmas | old-next local obligation | keep/postpone | do not prove or remove old-next residual during replacement

lemma_swap_runtime_implies_cap_post_bridge | impl_swap.rs | lemma_swap_runtime_implies_local_semantic_post | runtime-to-cap-post bridge | move | reusable proof is in impl_swap.rs; target home is spec_util/swap/runtime_bridge.rs or spec_util/swap/post.rs
lemma_swap_runtime_implies_local_semantic_post | impl_swap.rs | cte_swap closeout | runtime-to-local-semantic bridge | move | required by current proof but should not live in runtime implementation
lemma_swap_cap_post_implies_changed_slot_local_structural_ok | impl_swap.rs | lemma_swap_cap_post_implies_changed_slot_components_bridge | changed-slot structural producer | move | reusable proof belongs under spec_util/swap
lemma_swap_cap_post_implies_changed_slot_semantic_edge_ok_bridge | impl_swap.rs | lemma_swap_cap_post_implies_changed_slot_components_bridge | changed-slot semantic-edge producer | move | required proof-strength work; not counted as code-size win
lemma_swap_cap_post_implies_changed_slot_components_bridge | impl_swap.rs | lemma_swap_runtime_implies_local_semantic_post | canonical obligation bridge | move | keep the role, move out of impl_swap.rs
lemma_swap_changed_slots_semantic_post_from_mdb_patch_components | spec_util/swap/post.rs | lemma_swap_runtime_implies_local_semantic_post | compatibility projection to old semantic post | keep/postpone | delete only after callers stop needing swap_changed_slots_semantic_post
lemma_swap_exact_post_recovers_wf_via_mdb_patch | spec_util/swap/wf.rs | lemma_swap_exact_post_implies_wf | canonical shared closeout | keep | final mdb_patch route for swap
lemma_swap_exact_post_implies_wf | impl_swap.rs, spec_util/delete.rs, spec_util/swap/wf.rs | cte_swap and delete users | public swap wf wrapper | keep | public wrapper over canonical route; old component-wise wrappers already removed

lemma_set_empty_exact_post_implies_patch_frame | spec_util/delete.rs | lemma_set_empty_exact_post_preserves_wf | patch_frame producer | keep | part of current canonical set_empty route
lemma_set_empty_exact_post_implies_patch_non_mdb_frame | spec_util/delete.rs | lemma_set_empty_exact_post_preserves_wf | non-MDB frame producer | keep | part of current canonical set_empty route
lemma_set_empty_admissible_implies_changed_slot_mdb_patch_ok | spec_util/delete.rs | lemma_set_empty_exact_post_preserves_wf | changed-slot obligation producer | keep | consumes explicit admissibility; do not discharge residual here
lemma_set_empty_exact_post_preserves_derivation_wf | spec_util/delete.rs | lemma_set_empty_exact_post_preserves_wf | derivation obligation producer | keep | already feeds patch_derivation_obligations
lemma_set_empty_empty_slot_preserves_wf | impl_delete.rs, spec_util/delete.rs | set_empty empty path | empty-slot special closeout | keep | separate path, not duplicated by non-empty mdb_patch route
lemma_set_empty_exact_post_preserves_wf | impl_delete.rs, spec_util/delete.rs | set_empty callers | canonical non-empty closeout | keep | current shared route for set_empty
```

Phase 0 conclusion:

- `move` is the best first implementation target because its route is already single and small.
- `insert` should focus on paired-wrapper consolidation, not old-next residual proof.
- `swap` should focus on moving reusable proof out of `impl_swap.rs`; this may improve file shape even if total line count does not drop immediately.
- `set_empty` should only clean existing shared-route closeout; admissibility residuals remain deferred.

## Phase 1: Move As The Small Replacement Template

Why first: `move` is smaller than `insert`, `swap`, and `delete`, and already
uses `lemma_patch_recovers_wf_from_obligations(...)` in the final closeout.

Status: complete for the replacement pass.

Target canonical route:

```text
cte_move local/exact facts
  -> cte_move_patch_slots(...)
  -> patch_frame / patch_non_mdb_frame
  -> changed_slots_local_structural_ok
  -> changed_slots_semantic_edge_ok
  -> patch_derivation_obligations
  -> lemma_patch_recovers_wf_from_obligations
  -> wf()
```

Work items:

1. Audit `spec_util/move.rs` wrappers around changed-slot, frame, derivation,
   and non-MDB closeout.
2. Delete wrappers that only repackage facts already consumed by the canonical
   route.
3. Keep the local `assert(patch_frame(...))` in `impl_move.rs` unless a small
   incoming-parent-cap frame helper removes more code than it adds.
4. Avoid proving new semantic facts.

Validation:

```sh
cargo xtask verify --package sel4_cspace --features '' -- --verify-only-module cspace_manager::impl_move --verify-function CSpaceManager::cte_move
```

Exit criteria:

- `cte_move` has one closeout route.
- No old duplicate `wf` recovery path remains for move.
- Net code size for move-related files is lower or the remaining non-deletable
  proof is explicitly documented.

Result:

- `cte_move` already uses the canonical `mdb_patch` closeout route.
- The remaining `lemma_cte_move_frame_from_components(...)` plus local
  `assert(patch_frame(...))` is intentionally kept for now because replacing it
  requires an incoming-parent-cap frame helper and would add proof surface.
- No duplicate old `wf` recovery path was found for move.
- Validation passed:
  `cargo xtask verify --package sel4_cspace --features '' -- --verify-only-module cspace_manager::impl_move --verify-function CSpaceManager::cte_move`
  reported `1 verified, 0 errors`.

## Phase 2: Insert Replacement Without New Old-Next Proof

Why second: insert has two similar operation variants and a large amount of
operation-specific proof. It is a good candidate for replacing duplicated
wrappers, but it must not drift into old-next residual shrink.

Status: complete for the current replacement pass.

Target canonical routes:

```text
insert_new_cap exact/local facts
  -> insert_new_cap_patch_slots(...)
  -> mdb_patch obligations
  -> shared closeout
  -> wf()

cte_insert exact/local facts
  -> cte_insert_patch_slots(...)
  -> mdb_patch obligations
  -> shared closeout
  -> wf()
```

Work items:

1. Keep old-next admissibility as an explicit precondition for now.
2. Do not try to prove caller-side old-next admissibility in this phase.
3. Compare paired insert lemmas and merge only when the merged helper deletes
   more code than it adds.
4. Delete compatibility wrappers whose only role is preserving the old closeout
   shape.
5. Prefer direct consumption of `patch_frame`, `patch_non_mdb_frame`,
   changed-slot obligations, and `patch_derivation_obligations` over
   operation-specific repackaging.

Primary candidates to audit:

```text
lemma_insert_new_cap_frame_from_tracked_ops
lemma_cte_insert_frame_from_tracked_ops
lemma_insert_new_cap_patch_non_mdb_frame_from_components
lemma_cte_insert_patch_non_mdb_frame_from_components
lemma_insert_new_cap_changed_slots_mdb_patch_ok_from_components
lemma_cte_insert_changed_slots_mdb_patch_ok_from_components
lemma_insert_new_cap_old_next_slot_ok
lemma_cte_insert_old_next_slot_ok
```

Validation:

```sh
cargo xtask verify --package sel4_cspace --features '' -- --verify-only-module cspace_manager::impl_insert --verify-function CSpaceManager::insert_new_cap
cargo xtask verify --package sel4_cspace --features '' -- --verify-only-module cspace_manager::impl_insert --verify-function CSpaceManager::cte_insert
```

Exit criteria:

- Both insert operations use one canonical `mdb_patch` closeout route.
- Paired insert wrappers are deleted, merged, or justified as truly distinct.
- No new old-next residual proof is added.

Result:

- `insert_new_cap` and `cte_insert` both already route final `wf` recovery
  through `lemma_patch_recovers_wf_from_obligations(...)`.
- The paired non-MDB frame wrappers were inspected and kept distinct. They share
  the same final `lemma_patch_non_mdb_frame_from_components(...)` closeout, but
  their cap preservation obligations differ enough that a generic wrapper would
  likely add more proof surface than it removes.
- The paired changed-slot wrappers are already private assembly helpers feeding
  the public `*_changed_slots_mdb_patch_ok_from_components(...)` route.
- Old-next admissibility remains an explicit deferred residual; no new
  caller-side old-next proof was added.
- Validation passed:
  `CSpaceManager::insert_new_cap` and `CSpaceManager::cte_insert` each reported
  `1 verified, 0 errors` under `cspace_manager::impl_insert`.

## Phase 3: Swap Replacement And Impl Cleanup

Why third: `swap` is the largest size regression. It should not be attacked
first because it is also the riskiest, but it must be cleaned before the refactor
can be called complete.

Status: complete for the replacement pass.

Current audit result:

- The final `wf` route already goes through
  `lemma_swap_exact_post_recovers_wf_via_mdb_patch(...)`.
- The placement cleanup is now done: reusable bridge proof lives under
  `spec_util/swap/runtime_bridge.rs`.
- This phase was implemented as a code movement / wrapper cleanup slice, with
  no new residual-TCB proof.

Original issue:

```text
impl_swap.rs contains large reusable proof blocks.
```

This was not a good final shape. Even though the total line count was not
reduced below the original baseline, reusable proof now lives in
`spec_util/swap/*`, not inside the runtime implementation file.

Target shape:

```text
impl_swap.rs
  -> runtime mutation
  -> concrete runtime view facts
  -> call spec_util/swap proof wrappers
```

Canonical proof route:

```text
swap exact/cap post
  -> swap_patch_set(...)
  -> mdb_patch obligations
  -> lemma_patch_recovers_wf_from_obligations
  -> wf()
```

Work items:

1. Move reusable proof out of `impl_swap.rs` into `spec_util/swap/post.rs`,
   `spec_util/swap/wf.rs`, or a new `spec_util/swap/runtime_bridge.rs`.
2. After moving, delete wrappers in `impl_swap.rs` rather than re-exporting both
   old and new names.
3. Audit semantic-edge helpers. If they are required to replace a trusted
   bridge, keep them, but do not count them as code-size wins.
4. Delete old component-wise closeout wrappers that are superseded by
   `lemma_swap_exact_post_recovers_wf_via_mdb_patch(...)`.
5. Keep `impl_swap.rs` focused on the mutation and the minimal bridge from
   runtime fields to ghost postconditions.

Primary candidates to audit:

```text
lemma_swap_runtime_implies_cap_post_bridge
lemma_swap_runtime_implies_local_semantic_post
lemma_swap_cap_post_implies_changed_slot_local_structural_ok
lemma_swap_cap_post_implies_changed_slot_semantic_edge_ok_bridge
lemma_swap_cap_post_implies_changed_slot_components_bridge
lemma_swap_changed_slots_semantic_post_from_mdb_patch_components
lemma_swap_exact_post_recovers_wf_via_mdb_patch
lemma_swap_exact_post_implies_wf
```

Validation:

```sh
cargo xtask verify --package sel4_cspace --features '' -- --verify-only-module cspace_manager::impl_swap
cargo xtask verify --package sel4_cspace --features '' -- --verify-only-module cspace_manager::spec_util::swap::runtime_bridge
cargo xtask verify --package sel4_cspace --features '' -- --verify-only-module cspace_manager::spec_util::swap::wf --verify-function lemma_swap_exact_post_implies_wf
```

Exit criteria:

- `impl_swap.rs` is runtime-shaped again.
- Swap has one canonical final `wf` route.
- Any remaining large proof block is in `spec_util/swap/*` and has a clear role.
- No operation-side `external_body` is reintroduced without an explicit decision.

Result:

- The reusable runtime-to-cap-post and changed-slot bridge proof was moved out
  of `impl_swap.rs` into `spec_util/swap/runtime_bridge.rs`.
- `impl_swap.rs` is runtime-shaped again: it contains the swap mutation and
  calls `lemma_swap_runtime_implies_local_semantic_post(...)` plus
  `lemma_swap_exact_post_implies_wf(...)`.
- The large remaining proof block is intentionally under
  `spec_util/swap/runtime_bridge.rs`; it is proof-library code, not runtime
  implementation code.
- The final swap `wf` closeout still goes through
  `lemma_swap_exact_post_recovers_wf_via_mdb_patch(...)` and then
  `lemma_swap_exact_post_implies_wf(...)`.
- No residual trusted-boundary proof was added, and no operation-side
  `external_body` was introduced.
- Validation passed:
  `cspace_manager::impl_swap` reported `2 verified, 0 errors`;
  `cspace_manager::spec_util::swap::runtime_bridge` reported
  `11 verified, 0 errors`;
  `lemma_swap_exact_post_implies_wf` reported `1 verified, 0 errors`.

## Phase 4: SetEmpty Replacement Only For The Existing MDB Patch Route

Why fourth: `set_empty` is delete-adjacent and risky. Only clean the part that
already participates in the shared `mdb_patch` route.

Status: complete for the existing shared-route closeout.

Target route:

```text
set_empty exact post
  -> set_empty_patch_slots(...)
  -> patch_frame / patch_non_mdb_frame
  -> changed-slot obligations
  -> patch_derivation_obligations
  -> lemma_patch_recovers_wf_from_obligations
  -> wf()
```

Work items:

1. Keep the four caller-admissibility residuals explicit.
2. Do not try to prove them in this phase.
3. Delete old closeout assembly that is now duplicated by the shared route.
4. Keep the non-empty and empty paths clearly separated.

Validation:

```sh
cargo xtask verify --package sel4_cspace --features '' -- --verify-only-module cspace_manager::spec_util::delete --verify-function lemma_set_empty_exact_post_preserves_wf
```

Exit criteria:

- The existing set_empty MDB patch route is single-line and explicit.
- Residual admissibility facts remain named, not hidden.
- No reduce-zombie work is started.

Result:

- `lemma_set_empty_exact_post_preserves_wf(...)` already uses the shared
  `mdb_patch` closeout route for the non-empty path.
- The empty-slot path remains separate through
  `lemma_set_empty_empty_slot_preserves_wf(...)`.
- The four caller-admissibility residuals remain explicit and deferred.
- No `reduce_zombie` work was started.
- Validation passed:
  `cargo xtask verify --package sel4_cspace --features '' -- --verify-only-module cspace_manager::spec_util::delete --verify-function lemma_set_empty_exact_post_preserves_wf`
  reported `1 verified, 0 errors`.

## Phase 5: Full Verification And Replacement Report

Goal: prove that the refactor is no longer stuck in an added-layer state.

Status: complete.

Run:

```sh
cargo xtask verify --package sel4_cspace --features ''
```

Report:

```text
operation | canonical route | old route removed? | remaining wrappers | reason
move      | yes             | yes                | frame/non-MDB/derivation wrappers | still feed lemma_patch_recovers_wf_from_obligations directly
insert    | yes             | yes                | paired insert wrappers and old-next obligations | wrappers differ enough that merging would likely add proof; old-next proof is deferred residual work
swap      | yes             | yes                | runtime_bridge proof under spec_util/swap | large proof remains, but no longer lives in impl_swap.rs
set_empty | yes             | yes                | admissibility obligations and empty-slot path | residual admissibility proof is explicitly deferred
```

Also report size metrics:

```sh
git diff --numstat -- sel4_cspace/src/cspace_manager
wc -l sel4_cspace/src/cspace_manager/impl_swap.rs       sel4_cspace/src/cspace_manager/spec_util/insert.rs       sel4_cspace/src/cspace_manager/spec_util/move.rs       sel4_cspace/src/cspace_manager/spec_util/delete.rs       sel4_cspace/src/cspace_manager/spec_util/mdb_patch.rs
```

Final validation:

```sh
cargo xtask verify --package sel4_cspace --features ''
# verification results:: 463 verified, 0 errors
```

Final size metrics at this checkpoint:

```text
impl_swap.rs                                      463 lines
spec_util/swap/runtime_bridge.rs                1803 lines
spec_util/insert.rs                             3027 lines
spec_util/move.rs                               2442 lines
spec_util/delete.rs                             8547 lines
spec_util/mdb_patch.rs                           441 lines
```

Tracked `cspace_manager` diff still shows proof growth from the broader
verification/refactor work, and the newly added proof-library files must be
counted separately when judging total size. The honest final size conclusion is:

- `impl_swap.rs` was substantially reduced and returned to runtime shape.
- Total proof code did not become smaller in this checkpoint; the main gain is
  route replacement and proof placement, not net line-count reduction.
- The added-layer problem is resolved for the targeted operations because the
  old duplicate closeout routes are either removed or explicitly justified as
  wrappers feeding the canonical `mdb_patch` closeout.

Completion criteria:

- Full package verification reports `0 errors`.
- Each migrated operation has a single canonical closeout route.
- Old duplicate closeout routes are deleted or explicitly justified.
- The plan records whether code size actually improved.
- If code size did not improve, the final claim must say so plainly.

## Deferred Work: Trusted-Boundary Shrink

These tasks remain valuable, but they are not part of completing the current
replacement refactor.

Insert residuals:

- Discharge `insert_new_cap_old_next_semantic_admissible(...)`.
- Discharge `cte_insert_old_next_semantic_admissible(...)`.

Set-empty residuals:

- Discharge root-shaped CNode-slot root exclusion.
- Discharge head next-edge flag clearance.
- Discharge non-head revocable patched semantic-edge admissibility.
- Discharge no-two-cycle patch admissibility.

Reduce-zombie residual:

- Strengthen `delete_all_contract(end_slot, false, none)` so success exposes the
  non-null owner-slot trichotomy needed by `reduce_zombie(true)`.
- Remove or prove
  `lemma_reduce_zombie_immediate_non_null_owner_shape_caller_admissibility_bridge`.

Longer-term trusted-boundary work:

- `same_region_as / same_object_as` refinement.
- `finalise_cap / preemption_point / post_cap_deletion` stronger contracts.
- Public wrapper proof alignment.

## Rules For Future Changes

1. Deletion first.

   Do not add a helper until the old code it replaces is identified.

2. One operation at a time.

   Do not migrate insert, swap, and delete in one patch.

3. No new residual proof during replacement.

   Residual proof is proof-strength work, not replacement work.

4. Verify small slices.

   Prefer operation-level verification before full-package verification.

5. Report bad news directly.

   If a route cannot be shortened without losing proof strength, document that
   fact instead of hiding it behind a new abstraction.
