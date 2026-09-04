# Delete/Revoke Closeout Audit

This audit maps the current `delete + revoke` closeout plan to repository
artifacts. It is intentionally conservative: a passing verifier result is only
counted when it covers the named function or lemma in the plan.

Methodology:

- semantic and contract calibration from `l4v`
- Verus organization from `atmo`
- single-point verification first; full-package verification is a final gate,
  not a substitute for checking each plan item

## Objective

Close the `delete_one / delete_all / revoke` manager-level core as far as the
current repository ghost state allows, while shrinking high-level semantic TCB.

Success criteria from the plan:

- `set_empty` should not rely on an unnamed whole-`wf` semantic jump.
- `reduce_zombie(true)` should expose the l4v-style immediate owner-shape cases.
- `finalise_slot` should be loop-direct verified, not a whole-function external.
- `delete_one`, `delete_all`, and `revoke` should compose verified helpers and
  explicit dependency bridges.
- Remaining bridges should be dependency/representation bridges, or explicitly
  documented residual semantic blockers.

## Prompt-To-Artifact Checklist

| Requirement | Current artifact | Evidence | Status |
| --- | --- | --- | --- |
| `set_empty` runtime patch order remains manager-local and exact | `impl_delete.rs::set_empty`; `set_empty_exact_post`, `set_empty_local_post`, `set_empty_non_cdt_frame_post`, `set_empty_cdt_original_closeout_post` | `lemma_set_empty_exact_post_preserves_wf` and `impl_delete::set_empty` single-point verify; `set_empty` now only promises `wf` when the caller supplies `set_empty_wf_recovery_pre(...)` | Mostly done |
| Remove unnamed `set_empty -> wf()` jump | `set_empty_wf_recovery_pre`, `lemma_set_empty_exact_post_preserves_frame_wf_components`, `lemma_set_empty_exact_post_preserves_empty_slots_wf`, `lemma_set_empty_exact_post_preserves_zombie_slots_sound`, `lemma_set_empty_exact_post_preserves_hard_wf_components`, `lemma_set_empty_wf_recovery_pre_bridge` | easy components are proved; the main `lemma_set_empty_exact_post_preserves_wf` now consumes explicit recovery preconditions instead of projecting them internally; `impl_delete.rs::set_empty` no longer consumes `lemma_set_empty_wf_recovery_pre_bridge` internally; the four remaining non-empty caller facts now sit in `trusted/common.rs` as caller-admissibility bridges, while `spec_util/delete.rs` only has forwarding lemmas | Mostly done |
| Name the remaining `set_empty` admissibility gap | `set_empty_wf_closeout_admissible(old_mgr, slot)` | predicate names root exclusion plus manager no-two-cycle; `mdb_no_two_cycle_wf()` is now part of manager local structural `wf`, and non-root-shaped slot root exclusion is proved from `root_caps_wf`, so only root-shaped CNode-slot root exclusion remains trusted in this closeout group | Done |
| Prove `set_empty` resolve/root preservation under the admissible route | `lemma_set_empty_admissible_preserves_resolve_walk_wf(...)` | single-point verification: `1 verified, 0 errors` | Done |
| Prove first `set_empty` MDB self-link preservation under the admissible route | `lemma_set_empty_admissible_prev_next_distinct(...)`; `lemma_set_empty_admissible_preserves_mdb_no_self_links_wf(...)` | both single-point verified with `1 verified, 0 errors` | Done |
| Prove `set_empty` MDB link preservation under the admissible route | `lemma_set_empty_admissible_preserves_mdb_links_wf(...)` | single-point verification: `1 verified, 0 errors` | Done |
| Prove `set_empty` derivation map-domain preservation | `lemma_set_empty_exact_post_preserves_derivation_domains_wf(...)` | single-point verification: `1 verified, 0 errors` | Done |
| Prove `set_empty` derivation parent-slot shape preservation | `lemma_set_empty_exact_post_preserves_cdt_parent_slots_wf(...)` | single-point verification: `1 verified, 0 errors` | Done |
| Remove `derivation_wf` from the `set_empty` hard closeout | `lemma_set_empty_exact_post_preserves_derivation_wf(...)`; `lemma_set_empty_exact_post_preserves_hard_wf_components(...)` no longer ensures `new_mgr.derivation_wf()` and is no longer itself external | derivation closeout and hard-component closeout both single-point verify | Done |
| Prove full `set_empty -> wf` recovery under explicit admissibility | `set_empty_wf_recovery_pre`, `set_empty_edge_flags_admissible`, `set_empty_semantic_edge_admissible`, `lemma_set_empty_admissible_preserves_wf(...)`, `lemma_set_empty_exact_post_preserves_wf(...)` | single-point verification: `lemma_set_empty_wf_recovery_pre_bridge`, `lemma_set_empty_exact_post_preserves_wf`, and `impl_delete::set_empty` all verify with `0 errors`; `spec_util/delete.rs` now has no `set_empty` external body, and the residual is explicitly the trusted caller-admissibility boundary for non-empty callers | Done as explicit route |
| Remove `reduce_zombie(true)` shape black box | `reduce_zombie_immediate_delete_all_strengthened_success_post`; verified strengthened-success projectors for slot-kind, preserved-zombie/end-empty, self-cycle | strengthened projectors and `*reduce_zombie` single-point verify | Mostly done |
| Name the missing strengthened delete-all post for immediate zombie reduction | `delete_all_zombie_end_success_witness_post`; `reduce_zombie_immediate_delete_all_strengthened_success_post` | specialized witness combines `delete_all_contract(end_slot,false,none)` with owner-shape post | Done |
| Fully prove immediate owner-shape from current `delete_all_contract` | `trusted::lemma_reduce_zombie_immediate_non_null_owner_shape_caller_admissibility_bridge`; forwarding lemma `lemma_reduce_zombie_immediate_non_null_owner_shape_caller_admissibility_bridge`; proved null owner-shape case; proved projector `lemma_reduce_zombie_immediate_delete_all_success_projects_zombie_end_success_witness_bridge`; owner-shape projectors | the remaining trusted fact is now exactly the non-null owner-slot trichotomy inside `reduce_zombie_immediate_owner_shape_post(...)`, located at the trusted caller-admissibility boundary; the generic `delete_all_contract(end_slot,false,none)` fact stays visible in the proved success witness, and the three exact owner-shape projectors are proved above that named shape fact | Incomplete |
| Make `finalise_slot` loop-direct verified | `impl_base.rs::finalise_slot` | single-point verification: `2 verified, 0 errors` | Done |
| Avoid raw direct slot cap write in `finalise_slot` | `write_slot_cap_only_tracked` is used by `finalise_slot` before `reduce_zombie` | source inspection of `impl_base.rs::finalise_slot` | Done |
| Split cap-write `wf` recovery into verified easy components and dependency hard semantics | `lemma_finalise_slot_cap_write_preserves_easy_wf_components`; `lemma_finalise_slot_cap_write_no_affected_incoming_edges_ok`; proved forwarding lemma `lemma_finalise_slot_cap_write_preserves_affected_incoming_edges_bridge`; dependency projector `trusted::lemma_finalise_cap_write_preserves_affected_incoming_edges_dependency_bridge`; `lemma_finalise_slot_cap_write_preserves_semantic_edge_bridge`; `lemma_finalise_slot_cap_write_no_affected_cdt_parent_semantics_ok`; `lemma_finalise_slot_cap_write_preserves_derivation_bridge`; proved forwarding lemma `lemma_finalise_slot_cap_write_preserves_affected_cdt_parent_semantics_bridge`; dependency projector `trusted::lemma_finalise_cap_write_preserves_affected_cdt_parent_semantics_dependency_bridge`; proved root bridge `lemma_finalise_slot_cap_write_rewritten_root_cap_bridge`; dependency projector `trusted::lemma_finalise_slot_cap_write_slot_not_root_dependency_bridge`; `lemma_finalise_slot_cap_write_preserves_root_caps_nonroot`; `lemma_finalise_slot_cap_write_preserves_root_caps_bridge`; `lemma_finalise_slot_cap_write_preserves_resolve_walk_bridge`; `lemma_finalise_slot_cap_write_manager_bridge` | easy component lemma, no-affected incoming-edge case, semantic-edge composition, no-affected CDT-parent case, CDT parent slot-shape proof, CDT parent semantics composition, derivation composition, root-cap preservation, resolve-walk composition, hard-component composition lemma, and manager bridge all single-point verify; affected incoming/CDT/root-slot facts are no longer delete-spec residual bodies and are now explicit dependency/admissibility projectors | Mostly done |
| Move cap-write target facts out of delete-specific semantic TCB | `finalise_slot_cap_write_target_admissible`; proved `lemma_finalise_slot_cap_write_target_admissible_witness_bridge`; dependency projector `trusted::lemma_finalise_cap_non_immediate_nonremovable_projects_reduce_target_admissible`; verified projectors `lemma_finalise_slot_cap_write_immediate_target_admissible_bridge`, `lemma_finalise_slot_cap_write_non_immediate_target_dom_bridge`, `lemma_finalise_slot_cap_write_non_immediate_target_nonroot_bridge`, `lemma_finalise_slot_cap_write_non_immediate_target_nonempty_bridge`, `lemma_finalise_slot_cap_write_non_immediate_loop_no_two_cycle_available_bridge`; composition lemmas `lemma_finalise_slot_cap_write_non_immediate_no_two_cycle_bridge`, `lemma_finalise_slot_cap_write_non_immediate_target_admissible_bridge`, `lemma_finalise_slot_cap_write_target_admissible_bridge`; projector `lemma_finalise_slot_cap_write_target_admissible_projects_target` | delete-spec target witness is now a verified composition over cap-write frame plus a dependency-level `finalise_cap` projector; no-two-cycle availability comes from manager `wf`, and no-two-cycle preservation across cap-only write is proved | Done, modulo dependency TCB |
| Prove cap-only write preserves MDB no-two-cycle when the caller carries it | `lemma_finalise_slot_cap_write_preserves_mdb_no_two_cycle(...)` | single-point verification: `1 verified, 0 errors` | Done |
| Keep finalise dependencies explicit | `deps::finalise_cap`, `preemption_point_bridge`, `post_cap_deletion_bridge` | residual dependency bridges remain named and contracted | Done |
| `delete_all` composes `finalise_slot_contract` and `set_empty`/suspend closeout | `impl_delete.rs::delete_all`; `lemma_delete_all_contract_from_*` | single-point verification: `1 verified, 0 errors`; the non-empty `set_empty` recovery bridge is now consumed at this caller instead of inside `set_empty` | Done, modulo non-empty `set_empty` admissibility bridge |
| `delete_one` contract is local set-empty based | `delete_one_contract(...)`; `impl_delete.rs::delete_one` | single-point verification: `1 verified, 0 errors`; `delete_one_contract` is currently `set_empty_exact_post(...)`; empty-slot `set_empty` discharges via explicit empty-slot recovery, while remaining strength issues are inherited from the non-empty `set_empty` admissibility route rather than a separate remainder-projector bridge | Mostly done |
| `revoke` composes `delete_all` as loop step | `impl_delete.rs::revoke`; `revoke_loop_invariant`, `revoke_step_*`, `revoke_contract` | single-point verification: `2 verified, 0 errors` | Done, modulo delete-all residuals |
| No new whole-loop resolve/insert/move/swap work in this plan | no relevant edits required | scope inspection | Done |

## Verification Evidence

Commands run during this closeout pass:

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::spec_util::delete \
  --verify-function lemma_set_empty_exact_post_preserves_wf \
  --triggers-mode silent
```

Result: `3 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::spec_util::delete \
  --verify-function lemma_set_empty_admissible_preserves_resolve_walk_wf \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::spec_util::delete \
  --verify-function lemma_set_empty_admissible_prev_next_distinct \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::spec_util::delete \
  --verify-function lemma_set_empty_admissible_preserves_mdb_no_self_links_wf \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::spec_util::delete \
  --verify-function lemma_set_empty_admissible_preserves_mdb_links_wf \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::spec_util::delete \
  --verify-function lemma_set_empty_exact_post_preserves_derivation_domains_wf \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::spec_util::delete \
  --verify-function lemma_set_empty_exact_post_preserves_cdt_parent_slots_wf \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::spec_util::delete \
  --verify-function lemma_set_empty_exact_post_preserves_derivation_wf \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::spec_util::delete \
  --verify-function lemma_set_empty_edge_flags_admissible_preserves_local_structural_wf \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::spec_util::delete \
  --verify-function lemma_set_empty_semantic_edge_admissible_preserves_semantic_edge_wf \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::spec_util::delete \
  --verify-function lemma_set_empty_admissible_preserves_wf \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::spec_util::delete \
  --verify-function lemma_set_empty_wf_closeout_admissibility_bridge \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::spec_util::delete \
  --verify-function lemma_set_empty_exact_post_preserves_hard_wf_components \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::impl_delete \
  --verify-function set_empty \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::spec_util::delete \
  --verify-function lemma_finalise_slot_cap_write_preserves_easy_wf_components \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::spec_util::delete \
  --verify-function lemma_finalise_slot_cap_write_rewritten_root_cap_bridge
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::spec_util::delete \
  --verify-function lemma_finalise_slot_cap_write_preserves_root_caps_bridge
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::spec_util::delete \
  --verify-function lemma_finalise_slot_cap_write_preserves_affected_incoming_edges_bridge
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::spec_util::delete \
  --verify-function lemma_finalise_slot_cap_write_preserves_affected_cdt_parent_semantics_bridge
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::spec_util::delete \
  --verify-function lemma_finalise_slot_cap_write_preserves_hard_wf_components_bridge \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::spec_util::delete \
  --verify-function lemma_finalise_slot_cap_write_target_admissible_witness_bridge
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::impl_base \
  --verify-function finalise_slot
```

Result: `2 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::spec_util::delete \
  --verify-function lemma_finalise_slot_cap_write_target_admissible_bridge \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::spec_util::delete \
  --verify-function lemma_finalise_slot_cap_write_target_admissible_projects_target \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::spec_util::delete \
  --verify-function lemma_finalise_slot_cap_write_preserves_cdt_parent_slots_bridge \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::spec_util::delete \
  --verify-function lemma_finalise_slot_cap_write_preserves_cdt_parent_semantics_bridge \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::spec_util::delete \
  --verify-function lemma_finalise_slot_cap_write_preserves_derivation_bridge \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::spec_util::delete \
  --verify-function lemma_finalise_slot_cap_write_non_immediate_target_admissible_bridge \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::spec_util::delete \
  --verify-function lemma_finalise_slot_cap_write_target_admissible_bridge \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::spec_util::delete \
  --verify-function lemma_finalise_slot_cap_write_preserves_semantic_edge_bridge \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::spec_util::delete \
  --verify-function lemma_finalise_slot_cap_write_preserves_hard_wf_components_bridge \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::spec_util::delete \
  --verify-function lemma_finalise_slot_cap_write_preserves_resolve_walk_bridge \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::spec_util::delete \
  --verify-function lemma_finalise_slot_cap_write_manager_bridge \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::impl_base \
  --verify-function finalise_slot \
  --triggers-mode silent
```

Result: `2 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::spec_util::delete \
  --verify-function lemma_reduce_zombie_immediate_strengthened_success_projects_slot_kind \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::spec_util::delete \
  --verify-function lemma_reduce_zombie_immediate_strengthened_success_preserved_old_zombie_projects_end_slot_empty \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::spec_util::delete \
  --verify-function lemma_reduce_zombie_immediate_strengthened_success_nonpreserved_zombie_projects_self_cycle \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::spec_util::delete \
  --verify-function lemma_reduce_zombie_immediate_delete_all_success_projects_owner_shape \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::spec_util::delete \
  --verify-function lemma_reduce_zombie_immediate_delete_all_success_projects_strengthened_success \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::impl_delete \
  --verify-function 'CSpaceManager::derive_cap' \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::impl_delete \
  --verify-function delete_one \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::impl_delete \
  --verify-function delete_all \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

Follow-up after making `set_empty`'s `wf` guarantee explicitly depend on
`set_empty_wf_recovery_pre(...)`:

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::impl_delete \
  --verify-function set_empty \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::impl_delete \
  --verify-function delete_one \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::impl_delete \
  --verify-function delete_all \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

Follow-up after moving the remaining `set_empty` admissibility external bodies
from `spec_util/delete.rs` to `trusted/common.rs`:

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::spec_util::delete \
  --verify-function lemma_set_empty_wf_recovery_pre_bridge \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::impl_delete \
  --verify-function set_empty \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::impl_delete \
  --verify-function delete_all \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::impl_delete \
  --verify-function delete_one \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

Follow-up after moving the `reduce_zombie(true)` non-null owner-shape external
body from `spec_util/delete.rs` to `trusted/common.rs`:

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::spec_util::delete \
  --verify-function lemma_reduce_zombie_immediate_delete_all_success_projects_owner_shape_caller_admissibility_bridge \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::impl_delete \
  --verify-function '*reduce_zombie*' \
  --triggers-mode silent
```

Result: `3 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::impl_delete \
  --verify-function revoke \
  --triggers-mode silent
```

Result: `2 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::impl_delete \
  --verify-function '*reduce_zombie' \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::spec_util::delete \
  --verify-function lemma_finalise_slot_cap_write_preserves_mdb_no_two_cycle \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::spec_util::delete \
  --verify-function lemma_finalise_slot_cap_write_non_immediate_no_two_cycle_bridge \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::spec_util::delete \
  --verify-function lemma_finalise_slot_cap_write_target_admissible_bridge \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::spec_util::delete \
  --verify-function lemma_set_empty_wf_recovery_pre_bridge \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::spec_util::delete \
  --verify-function lemma_set_empty_exact_post_preserves_wf \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::spec_util::delete \
  --verify-function lemma_raw_derive_cap_untyped_next_view_alignment_some_bridge \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::spec_util::delete \
  --verify-function lemma_raw_derive_cap_untyped_next_view_alignment_bridge \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::impl_delete \
  --verify-function lemma_raw_derive_cap_untyped_matches_manager \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::impl_delete \
  --verify-function delete_all \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::impl_delete \
  --verify-function delete_one \
  --triggers-mode silent
```

Result: `1 verified, 0 errors`.

```bash
cargo xtask verify --package sel4_cspace --features '' -- \
  --verify-only-module cspace_manager::impl_delete \
  --verify-function revoke \
  --triggers-mode silent
```

Result: `2 verified, 0 errors`.

## Current Blockers

### `set_empty` admissibility closeout

Residual trusted caller-admissibility bridge:

- `trusted::lemma_set_empty_root_shaped_slot_not_root_caller_admissibility_bridge`
- `trusted::lemma_set_empty_head_next_edge_flags_clearance_caller_admissibility_bridge`
- `trusted::lemma_set_empty_revocable_patched_semantic_edge_caller_admissibility_bridge`

Reason:

- `CSpaceManager::wf()` now includes `mdb_no_two_cycle_wf()` as part of local
  structural well-formedness, so the old no-two-cycle availability residual has
  been eliminated.
- Clearing a slot whose old prev and next point to the same neighbor still
  explains why this stronger MDB invariant belongs in base manager `wf` rather
  than in an ad-hoc delete-only bridge.
- `set_empty` also preserves `roots`, so clearing a registered root can violate
  `resolve_walk_wf/root_caps_wf` unless the precondition excludes registered
  roots or the ghost root set is updated.
- If the old cap is not root-shaped (`kind != CNodeCap` or `cnode is None`),
  `root_caps_wf` is enough to prove the slot is not a registered root. The
  `lemma_set_empty_slot_not_root_available_bridge(...)` wrapper is proved, and
  the residual root fact is now only the trusted caller-admissibility bridge for
  a root-shaped CNode.
- A call-surface audit shows why this is not a wrapper-local proof obligation:
  the public `cte_t::{delete_all, delete_one, revoke}` wrappers are currently
  runtime compatibility shims, not verified public contracts, and they do not
  expose the manager's `roots()` ghost state. Eliminating this bridge therefore
  requires either a manager-level caller admissibility precondition or a future
  whole-kernel/public-wrapper proof state; it cannot be recovered locally from
  the current public wrapper surface.
- Calibration against the reference material points in the same direction:
  l4v discharges delete/MDB obligations under global predicates such as
  `valid_mdb`/CDT/descendants facts, while the atmo-style Verus organization
  keeps comparable ownership and tree facts in manager/permission state. The
  remaining `set_empty` admissibility facts therefore belong in manager-level
  admissibility or ghost-state strengthening, not in a raw wrapper-only proof.
- The previous residual post bridge has been moved one layer earlier:
  `lemma_set_empty_exact_post_preserves_hard_wf_components(...)` is now a proved
  composition and the trusted steps are only the projection of three explicit
  admissibility fact groups from the current generic closeout pre.

Named future route:

- `set_empty_wf_closeout_admissible(old_mgr, slot)`
- `lemma_set_empty_admissible_preserves_resolve_walk_wf(...)` already proves the
  root/resolve part under that route.
- `lemma_set_empty_wf_closeout_admissible_bridge(...)` is now a verified
  composition over proved no-two-cycle availability from `wf`, proved
  non-root-shaped slot root exclusion via
  `lemma_set_empty_non_root_shaped_slot_not_root(...)`, proved root-exclusion
  dispatch via `lemma_set_empty_slot_not_root_available_bridge(...)`, and the
  remaining trusted root-shaped CNode-slot root-exclusion bridge.
- `lemma_set_empty_admissible_preserves_mdb_no_self_links_wf(...)` already
  proves the first MDB local-structural subcase under that route.
- `lemma_set_empty_admissible_preserves_mdb_links_wf(...)` already proves the
  full MDB link subcase under that route.
- `lemma_set_empty_exact_post_preserves_derivation_domains_wf(...)` already
  proves the derivation map-domain subcase.
- `lemma_set_empty_exact_post_preserves_cdt_parent_slots_wf(...)` already
  proves the derivation parent-slot shape subcase.
- `lemma_set_empty_exact_post_preserves_derivation_wf(...)` keeps
  `derivation_wf` outside the set-empty admissibility bridge.
- `lemma_set_empty_edge_flags_admissible_preserves_local_structural_wf(...)`
  now proves local structural recovery under the named edge-flag admissibility
  route.
- `lemma_set_empty_edge_flags_admissible_bridge(...)` is now a verified wrapper:
  ordinary cases follow from the predicate shape directly. In the head-deletion
  case, `lemma_set_empty_head_slot_first_badged_false(...)` proves the removed
  slot's `first_badged=false` fact from `incoming_edge_flags_wf`, and
  `lemma_set_empty_head_next_edge_flags_already_clear(...)` proves the
  already-clear next-node case directly. Only the next-node clearance case where
  `revocable || first_badged` is still set remains trusted at the caller-
  admissibility boundary.
- `lemma_set_empty_semantic_edge_admissible_preserves_semantic_edge_wf(...)`
  proves the semantic-edge closeout under an explicit side condition for the
  patched incoming edge.
- `lemma_set_empty_semantic_edge_admissible_bridge(...)` is now a verified
  wrapper: if `next(slot)` is absent, the predicate is immediate; if the delete
  is at the head of the MDB chain, `lemma_set_empty_head_patched_semantic_edge_admissible(...)`
  proves the patched edge from the edge-flag admissibility bridge. It delegates
  to `lemma_set_empty_patched_semantic_edge_admissible_bridge(...)` only for the
  non-head case where `prev(slot)` is present.
- `lemma_set_empty_patched_semantic_edge_admissible_bridge(...)` is now a
  verified case split: `lemma_set_empty_nonrevocable_patched_semantic_edge_admissible(...)`
  proves the non-head/non-revocable patched edge directly, and the remaining
  trusted residual is narrowed to the non-head revocable caller-admissibility
  bridge.
- `lemma_set_empty_admissible_preserves_wf(...)` composes the already-proved
  frame, local-structural, semantic-edge, derivation, zombie, and resolve-walk
  closeouts into a complete non-bridge `wf()` recovery route.
- `lemma_set_empty_exact_post_preserves_hard_wf_components(...)` now also
  follows this route and single-point verifies.

### `finalise_slot` cap-write hard compatibility

Residual bridge:

- none in `spec_util/delete.rs` for cap-write root/affected-edge/target facts;
  the remaining assumptions have been moved to dependency/admissibility
  projectors in `trusted::common`.

Reason:

- `write_slot_cap_only_tracked` changes only the cap field, and the easy frame /
  local-structural components are now proved.
- The cap-only write post now explicitly records that `cdt_parent_map` and
  `is_original_map` are unchanged, matching the tracked helper's actual effect.
- The current `finalise_cap_contract(...)` only projects a non-removable
  remainder to a reduce-ready zombie. It does not yet prove that replacing the
  old slot cap with the remainder preserves incoming-edge compatibility for the
  rewritten slot or its direct children, or CDT parent semantics for the CDT
  edge directly touching the rewritten slot. Root-cap visibility is now handled
  by the explicit dependency/admissibility projector
  `trusted::lemma_finalise_slot_cap_write_slot_not_root_dependency_bridge(...)`,
  which states that the cap-write path is not applied to a registered root slot.
- The affected incoming-edge and affected CDT compatibility obligations are now named as
  `finalise_slot_cap_write_affected_incoming_edges_ok(...)` and
  `finalise_slot_cap_write_affected_cdt_parent_semantics_ok(...)`, so the
  hard assumptions are stated as compatibility predicates rather than anonymous
  forall postconditions. The no-affected incoming-edge subcase is now
  proved by `lemma_finalise_slot_cap_write_no_affected_incoming_edges_ok(...)`,
  and the affected case is no longer a delete-spec `external_body`; it is a
  proved forwarding lemma over
  `trusted::lemma_finalise_cap_write_preserves_affected_incoming_edges_dependency_bridge(...)`.
  The no-affected CDT-parent subcase is likewise proved by
  `lemma_finalise_slot_cap_write_no_affected_cdt_parent_semantics_ok(...)`, so
  the affected CDT case is also a proved forwarding lemma over
  `trusted::lemma_finalise_cap_write_preserves_affected_cdt_parent_semantics_dependency_bridge(...)`.
- The resolve-walk table/cnode-slot part is now proved by frame. Root-cap
  preservation now also proves the frame part directly; the former rewritten-root
  bridge is proved by first projecting that the cap-write slot is not registered
  as a root, so `finalise_slot_cap_write_rewritten_root_cap_ok(...)` is vacuous
  in the verified delete-side proof.
- The target side now has a named predicate,
  `finalise_slot_cap_write_target_admissible(...)`, but it is no longer a
  delete-specific semantic bridge. The former witness bridge is proved from the
  cap-only write frame plus the dependency-level finalise projector
  `trusted::lemma_finalise_cap_non_immediate_nonremovable_projects_reduce_target_admissible(...)`.
  The individual target bridges are verified projectors over that proved
  witness, and the caller-facing target facts are still projected by the
  verified `lemma_finalise_slot_cap_write_target_admissible_projects_target(...)`.
  The remaining trusted claim is now explicitly attached to `finalise_cap`: the
  non-removable non-immediate remainder has a zombie target/end slot in the
  manager domain, and the non-immediate target is non-root and nonempty. The
  swap-side `mdb_no_two_cycle_wf()` precondition is available from manager
  `wf()` and is not part of this dependency projector.

Verified shrink:

- `lemma_finalise_slot_cap_write_preserves_easy_wf_components(...)`
- `lemma_finalise_slot_cap_write_preserves_semantic_edge_bridge(...)` is now a
  proved composition: unaffected incoming edges are discharged by frame, and
  the remaining semantic assumption is only the dependency-level affected
  incoming-edge projector.
- `lemma_finalise_slot_cap_write_preserves_cdt_parent_slots_bridge(...)` is now
  proved from the cap-only write frame and the fact that the rewritten remainder
  is non-null.
- `lemma_finalise_slot_cap_write_preserves_cdt_parent_semantics_bridge(...)` is
  now a proved composition: unaffected CDT edges are discharged by frame, and
  only the edge whose child or parent is the rewritten slot remains in the
  dependency-level affected-edge predicate.
- `lemma_finalise_slot_cap_write_preserves_derivation_bridge(...)` is now proved
  over the domain, CDT-slot, and CDT-semantics sublemmas.
- `lemma_finalise_slot_cap_write_preserves_mdb_no_two_cycle(...)` now proves the
  frame fact that cap-only write preserves `mdb_no_two_cycle_wf()` whenever the
  loop state already has that stronger MDB invariant. Since the invariant is now
  part of manager `wf()`, availability is no longer a target-witness residual.
- `lemma_finalise_slot_cap_write_non_immediate_no_two_cycle_bridge(...)` is now
  a verified composition over manager `wf()` and the proved cap-write
  preservation lemma.
- `lemma_finalise_slot_cap_write_preserves_root_caps_bridge(...)` is now a
  proved composition over unchanged roots/slots plus the proved rewritten-root
  predicate.
- `lemma_finalise_slot_cap_write_preserves_resolve_walk_bridge(...)` is now a
  proved composition over unchanged lookup tables plus proved root-cap recovery.
- `lemma_finalise_slot_cap_write_preserves_hard_wf_components_bridge(...)` is
  now a proved composition over the three smaller compatibility bridges.
- `lemma_finalise_slot_cap_write_manager_bridge(...)`
- `lemma_finalise_slot_cap_write_target_admissible_witness_bridge(...)` is now
  proved from the dependency-level finalise target projector.
- `lemma_finalise_slot_cap_write_preserves_affected_incoming_edges_bridge(...)`
  and `lemma_finalise_slot_cap_write_preserves_affected_cdt_parent_semantics_bridge(...)`
  are now proved from dependency-level finalise compatibility projectors.
- `lemma_finalise_slot_cap_write_rewritten_root_cap_bridge(...)` is now proved
  from the dependency/admissibility projector that excludes registered root
  slots from the cap-write path.

Remaining route:

- strengthen the dependency-level `finalise_cap_contract(...)` with proved cap
  compatibility facts, replacing the current affected-incoming-edge and
  affected-CDT dependency projectors when the finalise-cap dependency cone is
  opened; separately replace the root-slot exclusion projector with a proved
  caller-admissibility lemma.
- replace the current dependency-level finalise target projector with a proved
  low-level finalise-cap/slot-allocation lemma when the dependency cone is
  opened.

### `reduce_zombie(true)` owner-shape projection

Residual trusted caller-admissibility bridge:

- `trusted::lemma_reduce_zombie_immediate_non_null_owner_shape_caller_admissibility_bridge`

Reason:

- Current `delete_all_contract(...)` is sufficient for frame/wf composition, but
  does not expose the non-null owner-slot trichotomy needed by immediate zombie
  reduction.
- The null owner-slot case is now proved directly by
  `lemma_reduce_zombie_immediate_null_owner_shape(...)`; the residual starts
  only once `mid_mgr.get_cap(slot).kind != NullCap`.
- A source audit of `delete_all_contract(...)` / `delete_all_success_witness_post(...)`
  confirms that the success witness exposes `finalise_slot_contract(...)` for
  the deleted end slot plus the `delete_all_success_stage_post(...)`; it does
  not frame arbitrary owner slots strongly enough to classify `mid_mgr.get_cap(slot)`.
- After `delete_all(end_slot, false)` succeeds, `reduce_zombie(true)` reads the
  owner slot and case-splits on runtime cap shape. The present contract does not
  prove that this slot is either Null or Zombie, nor that a preserved old zombie
  implies `end_slot` is empty, nor that the non-preserved zombie is a self-cycle.
- The case projectors themselves are no longer the trusted part: they now consume
  `delete_all_zombie_end_success_witness_post(...)` through
  `reduce_zombie_immediate_delete_all_strengthened_success_post(...)` and are
  verified. The success-witness projector is also proved from the visible
  generic `delete_all_contract(...)` fact plus the remaining non-null owner-shape
  bridge. The residual is now exactly the non-null owner-slot trichotomy, not a
  whole generic-to-specialized contract implication.

Named future route:

- `reduce_zombie_immediate_delete_all_strengthened_success_post(...)`
- `delete_all_zombie_end_success_witness_post(...)`
- strengthen the success witness of `delete_all_contract(...)` for the
  `exposed == false` zombie-end-slot use case so it directly exposes
  `reduce_zombie_immediate_owner_shape_post(...)`, instead of trying to derive
  the owner trichotomy from the current generic frame/nonempty-subset post.

## Completion Judgment

The closeout is not complete.

Current state is suitable to describe as:

- manager-level delete/revoke core is mostly verified and loop-direct for
  `finalise_slot`, `delete_all`, and `revoke`
- remaining high-level semantic TCB has been narrowed and explicitly named
- full Paper-Max closeout still requires proving or structurally eliminating the
  residual semantic bridges above
