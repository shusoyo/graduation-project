# CSpace Semantics + Invariants Freeze (Pre-Proof Baseline)

Status: Frozen baseline for proof planning (not “proof-complete”)  
Date: 2026-05-14

## 1) Scope and Layer Ownership

- `cspace::cte`: slot-local entry/cap semantics (`cte_*` predicates and slot-local rules).
- `cspace::mdb`: MDB graph/link semantics (`prev/next/payload` + local transition functions).
- `cspace::cdt`: derivation parent/original semantics (`parent_of/is_original` + transitions).
- `cspace::manager`: operation orchestration; composes `cte + mdb + cdt` owner predicates directly.
  It must not become a second semantic proof center or hold manager-level proof-obligation bags.

This file freezes what each operation means at spec level and what must be proved.

## 2) Frozen Post-State Semantics (per owner)

## 2.1 MDB post-state (source of truth: `src/cspace/mdb/state.rs`)

- `state_after_insert_between(parent, slot, old_next, revocable, first_badged)`
- `state_after_move_slot(src, dest)`
- `state_after_unlink(slot)`
- `state_after_delete_slot(slot)`
- `state_after_swap_neighborhood(slot1, slot2)`

Rule: manager-level proof must refer to these functions (or explicit owner lemmas derived from them), not ad-hoc redefinitions.
Manager-level proof still reasons against these owner transitions directly, but the public operation contracts expose only exact post-state and observable slot effects; proof-only owner obligations live in owner lemmas or proof bodies.

## 2.2 CDT post-state (source of truth: `src/cspace/cdt/state.rs`)

- `state_after_cap_insert(src, dest, src_parent, dest_original)`
- `state_after_insert_new_cap(parent, slot)`
- `state_after_move(src, dest)` via `moved_parent_of`
- `state_after_swap(slot1, slot2)` via `swapped_parent_of`
- `state_after_delete(deleted)` via `deleted_parent_of`

Rule: `swap` semantics includes child-parent redirection (`Some(slot1)<->Some(slot2)`), not only swapping `slot1/slot2` entries.
Manager-level proof still reasons against these owner transitions directly, but the public operation contracts expose only exact post-state and observable slot effects; proof-only owner obligations live in owner lemmas or proof bodies.

## 2.3 CTE/cap local semantics (source of truth: `src/cspace/cte/spec.rs`)

- `cte_mdb_parent_of`
- `cte_ensure_no_children_blocks`
- `cte_is_final_cap_at`
- `cte_derive_cap_returns_syscall_error`
- `cte_derive_cap_expected_cap`

Rule: manager proof can depend on these local predicates; it should not duplicate cap-local reasoning.
The visible operation contracts inline slot-local effects directly and refer only to these CTE predicates
when cap-local meaning is needed.

## 3) Frozen Core Invariants (what “well-formed” means)

## 3.1 Structural domain coverage

- MDB: `maps_cover_dom` for `prev/next/payload`.
- CDT: `maps_cover_dom` for `parent_of/is_original`.
- Manager projection consistency: manager slot domain equals owner-state domain.

## 3.2 Link/tree well-formedness

- MDB local link compatibility and parent/derivation semantic constraints (as encoded in `mdb/spec.rs` + manager wf predicates).
- CDT parent slot constraints:
  - empty slot => parent is `None` and `is_original == false`
  - parent (if any) is in domain, not self, and not empty.

## 3.3 Cross-layer semantic consistency

- If CDT says `parent(child)=Some(parent)`, then cap-level parent relation must satisfy `should_be_parent_of(...)`.
- CTE local predicates must align with manager projections used by ops.

## 4) Proof Obligations Matrix (what each op must preserve)

For each operation (`insert`, `move`, `swap`, `delete`, `revoke`):

1. Post-state exactness
- show manager ghost post-state equals composition of frozen owner post-state functions.

2. Domain coverage preservation
- preserve owner `maps_cover_dom` and manager-domain consistency.

3. Owner semantic preservation
- MDB owner invariants preserved by corresponding MDB transition.
- CDT owner invariants preserved by corresponding CDT transition.

4. Cross-layer consistency preservation
- after op, CDT-parent semantics and CTE/cap compatibility still hold.

5. Operation-specific side conditions
- e.g. no-children constraints, final-cap constraints, badge/revocable conditions.

## 5) TCB Boundary Freeze (pre-proof)

- Trusted runtime/kernel effects are confined to raw mutation primitives and projection assumptions.
- All semantic correctness claims above are proved in ghost/spec layers.
- Any new trusted assumption must be recorded explicitly in docs before use.

## 6) Done vs Not Done (current state)

Done:
- layer ownership split is stable.
- MDB/CDT state models and `state_after_*` functions exist.
- manager `wf()` composes owner-layer `mdb_layer_wf` and `cdt_layer_wf` instead of spelling the full graph/tree story itself.
- manager-level `insert/move/swap/delete/revoke_proof_obligations` wrappers have been removed.
- operation contracts inline owner obligations directly for insert/move/swap/delete-side paths.
- CDT swap/move/delete domain-preservation lemmas scaffolded in `cdt/proof.rs`.

Not done yet:
- full per-op preservation proofs (`insert/move/swap/delete/revoke`) are not closed.
- TCB freeze/shrink is explicitly out of scope for this baseline.

## 7) l4v Alignment Baseline (Strength/Meaning Source)

Primary references:
- `sel4_cspace_backup/aux/l4v_cspace_extracted/proof/invariant-abstract/Invariants_AI.thy`
- `sel4_cspace_backup/aux/l4v_cspace_extracted/proof/invariant-abstract/CSpace_AI.thy`

## 7.1 Directly aligned (already reflected)

- `valid_mdb` is a composite predicate family in l4v (not a single trivial check):
  includes `mdb_cte_at`, `untyped_mdb`, `descendants_inc`, `no_mloop`, `untyped_inc`,
  `ut_revocable`, `irq_revocable`, `reply_master_revocable`, `reply_mdb`, `valid_arch_mdb`
  (see `Invariants_AI.thy`, `valid_mdb` definition sites).
- `cdt` and `is_original_cap` are first-class semantic state in l4v and participate in `valid_mdb`.
  Keeping `is_original` in our CDT owner state is therefore semantically consistent with source strength.
- Parent semantics linkage through `should_be_parent_of` is central in l4v CSpace proof chain
  (see `CSpace_AI.thy`, `should_be_parent_of`-related lemmas and `safe_parent_for` flow).

## 7.2 Partially aligned (model exists, proof strength pending)

- MDB owner model exists (`prev/next/payload` + transitions), but l4v-level invariant closure
  over all operations is not proved yet in this repo.
- CDT owner model exists (`parent_of/is_original` + transitions), but descendants/loop-avoidance
  strength is not yet closed to l4v-equivalent level.
- Manager composition exists structurally, but not all operation-level preservation lemmas are closed.

## 7.3 Explicit gaps to close before claiming l4v-equivalent strength

1. Close per-op preservation for the full invariant family, not just domain coverage.
2. Add/close acyclicity witness usage (`rank/depth`) where needed to replace transitive-closure-heavy reasoning.
3. Finish cross-layer refinement lemmas:
   - manager post-state == composition of frozen owner `state_after_*`
   - owner invariants imply manager-level `valid_mdb`/derivation obligations.
4. Ensure revoke/delete paths preserve the same semantic commitments that l4v enforces
   for descendants and revocability conditions.

## 7.4 Policy

- l4v is the semantic-strength authority.
- Any new invariant weakening must be justified against l4v reference lemmas/defs in review notes.
- “Proof complete” is only declared when operation-level closure matches this l4v-aligned matrix.

## 7.5 Code Mapping Freeze

This is the code-level surface that must stay aligned with the l4v-strength plan.

| l4v-strength item / op | Owner | Code predicate or transition | Status |
| --- | --- | --- | --- |
| `mdb_cte_at` | MDB | `mdb_cte_at_wf` | target specified |
| `untyped_mdb` | MDB | `untyped_mdb_wf` | target specified |
| `untyped_inc` | MDB | `untyped_inc_wf` | target specified |
| `ut_revocable` | MDB | `ut_revocable_wf` | target specified |
| `irq_revocable` | MDB | `irq_revocable_wf` | target specified |
| `reply_master_revocable` | MDB | `reply_master_revocable_wf` | target specified |
| `reply_mdb` | MDB | `reply_mdb_wf` | target specified |
| `valid_arch_mdb` | MDB | `valid_arch_mdb_wf` | target specified, arch detail pending |
| `descendants_inc` | CDT | `descendants_inc_wf` | target specified |
| `no_mloop` | CDT | `no_mloop_wf` via `CdtDepthWitness` | target specified, proof pending |
| full MDB layer | MDB | `mdb_layer_wf` | composed in manager `wf` |
| full CDT layer | CDT | `cdt_layer_wf` | composed in manager `wf` |
| insert graph target | MDB | exact `state_after_insert_between` post-state with owner preservation discharged in proof/body | expanded in `impl_insert.rs` |
| insert derivation target | CDT | expanded in `impl_insert.rs` using `state_after_cap_insert`, `state_after_insert_new_cap`, and `should_be_parent_of` | in op contract |
| insert slot target | CTE | explicit slot-entry postconditions | expanded in `impl_insert.rs` |
| move graph target | MDB | exact `state_after_move_slot` post-state with owner preservation discharged in proof/body | expanded in `impl_move.rs` |
| move derivation target | CDT | exact `state_after_move` post-state with owner preservation discharged in proof/body | expanded in `impl_move.rs` |
| move slot target | CTE | explicit source-empty/destination-copy postconditions | expanded in `impl_move.rs` |
| swap graph target | MDB | exact `state_after_swap_neighborhood` post-state with owner preservation discharged in proof/body | expanded in `impl_swap.rs` |
| swap derivation target | CDT | exact `state_after_swap` post-state with owner preservation discharged in proof/body | expanded in `impl_swap.rs` |
| swap slot target | CTE | explicit cap/payload swap postconditions | expanded in `impl_swap.rs` |
| delete graph target | MDB | exact `state_after_delete_slot` post-state with owner preservation discharged in proof/body | expanded in delete path contract |
| delete derivation target | CDT | exact `state_after_delete` post-state with owner preservation discharged in proof/body | expanded in delete path contract |
| delete slot target | CTE | explicit empty/unchanged slot postconditions | expanded in delete path contract |
| revoke graph target | MDB | descendant/delete cascade over MDB owner semantics | target specified, no wrapper |
| revoke derivation target | CDT | descendant/delete cascade over CDT owner semantics | target specified, no wrapper |
| derive local target | CTE | `cte_derive_cap_returns_syscall_error` + `cte_derive_cap_expected_cap` | expanded in derive contract |
| no-children local target | CTE | `cte_ensure_no_children_blocks` | semantic predicate, no wrapper |
| final-cap local target | CTE | `cte_is_final_cap_at` | semantic predicate, no wrapper |

Public manager-op policy:
- public `requires/ensures` expose call constraints, exact owner post-state, and observable slot effects
- proof-only facts like `maps_cover_dom`, owner layer wf subgoals, and local pointwise projection facts live in owner lemmas or proof bodies, not in public manager contracts

Policy: add new proof facts to the owner layer first. Manager contracts may mention owner predicates and
transitions directly, but must not repackage them into parallel manager-level obligation helpers.

## 8) Detailed Operation Semantics Freeze (Implementation-Anchored)

This section is the exact pre/post contract baseline to prove against.  
Primary implementation anchors:
- `src/cspace/mdb/state.rs`
- `src/cspace/cdt/state.rs`
- `src/cspace/cte/spec.rs`

## 8.1 `insert(src,parent,dest/slot)` semantics

MDB owner:
- Function: `MdbState::state_after_insert_between(parent, slot, old_next, revocable, first_badged)`
- Preconditions:
  - `parent, slot ∈ dom`
  - `old_next = Some(n) => n ∈ dom`
- Post effects:
  - `prev[slot] = Some(parent)`, `next[slot] = old_next`
  - `payload[slot] = {revocable, first_badged}`
  - `next[parent] = Some(slot)`
  - `old_next = Some(n) => prev[n] = Some(slot)`
- Non-effects:
  - other slots unchanged.

CDT owner:
- Function: `CdtState::state_after_cap_insert(src, dest, src_parent, dest_original)`
- Preconditions: `src, dest ∈ dom`
- Post effects:
  - `dest_original = spec_is_cap_revocable(new_cap, src_cap)`
  - `src_parent = should_be_parent_of(src_cap, is_original[src], new_cap, dest_original)`
  - `parent_of[dest] = if src_parent { Some(src) } else { parent_of[src] }`
  - `is_original[dest] = dest_original`
- Non-effects:
  - `dom` unchanged, other entries unchanged.

Strong child-insert special case:
- Function: `CdtState::state_after_insert_new_cap(parent, slot)`
- Post effects:
  - `parent_of[slot] = Some(parent)`
  - `is_original[slot] = true`

CTE/cap local checks:
- Uses `cte_mdb_parent_of`, `cte_derive_cap_expected_cap`, `cte_ensure_no_children_blocks` where applicable.

## 8.2 `move(src,dest)` semantics

MDB owner:
- Function: `MdbState::state_after_move_slot(src, dest)`
- Preconditions:
  - `src, dest ∈ dom`
  - `prev[src]`/`next[src]` targets (if any) are in dom
- Post effects:
  - `dest` receives old `{prev,next,payload}` of `src`
  - `src` becomes structural empty (`prev=None,next=None,payload=empty`)
  - old neighbors of `src` now point to `dest`

CDT owner:
- Function: `CdtState::state_after_move(src, dest)` via `moved_parent_of`
- Preconditions: `src, dest ∈ dom`
- Post effects:
  - `parent_of[src] = None`
  - `parent_of[dest] = old parent_of[src]`
  - any child with old `parent_of[child]=Some(src)` becomes `Some(dest)`
  - `is_original[src] = false`
  - `is_original[dest] = old is_original[src]`

## 8.3 `swap(slot1,slot2)` semantics

MDB owner:
- Function: `MdbState::state_after_swap_neighborhood(slot1, slot2)`
- Preconditions:
  - `slot1, slot2 ∈ dom`, `slot1 != slot2`
  - neighbor endpoints (if any) in dom
- Post effects:
  - neighborhood redirection around `slot1/slot2`
  - payload exchanged (`payload[slot1] <-> payload[slot2]`)
  - all links touching `slot1/slot2` retargeted consistently

CDT owner:
- Function: `CdtState::state_after_swap(slot1, slot2)` via `swapped_parent_of`
- Preconditions: `slot1, slot2 ∈ dom`
- Post effects:
  - `parent_of[slot1] = old parent_of[slot2]`
  - `parent_of[slot2] = old parent_of[slot1]`
  - all nodes with parent `Some(slot1)` become `Some(slot2)` and vice versa
  - `is_original[slot1] <-> is_original[slot2]`

## 8.4 `delete(slot)` semantics

MDB owner:
- Function: `MdbState::state_after_delete_slot(slot)` (defined as unlink + first_badged carry)
- Preconditions:
  - `slot ∈ dom`
  - neighbor endpoints (if any) in dom
- Post effects:
  - unlink slot from prev/next chain
  - slot becomes structural empty
  - if old `next=Some(n)`: `payload[n].first_badged` OR old `first_badged(slot)`

CDT owner:
- Function: `CdtState::state_after_delete(deleted)` via `deleted_parent_of`
- Preconditions: `deleted ∈ dom`
- Post effects:
  - `parent_of[deleted] = None`
  - children of deleted lose parent (`None`)
  - `is_original[deleted] = false`

## 8.5 `revoke(slot)` semantics (freeze target; implementation closure pending)

Current freeze intent (must match l4v descendant semantics):
- remove/revoke descendant-derived caps under `slot` according to CDT/MDB parent semantics
- preserve global invariants (`valid_mdb` family) after cascading delete/revoke effects
- keep parent/original and revocability semantics aligned with l4v `descendants_of` and revocable rules

Status:
- semantic intent frozen here
- full owner-level executable post-state composition still pending proof closure.

## 9) Formal Invariant Matrix (Frozen, Proof-Obligation Oriented)

Legend:
- `Req`: required target invariant.
- `Src`: where semantics/strength comes from.
- `Now`: current repo status.

1. `INV-DOM-MDB`
- Req: `mdb.maps_cover_dom()`.
- Src: impl `mdb/state.rs`; l4v `valid_mdb` context requires coherent slot/cap presence.
- Now: modeled; per-op closure partial.

2. `INV-DOM-CDT`
- Req: `cdt.maps_cover_dom()`.
- Src: impl `cdt/state.rs`; l4v keeps `cdt` and `is_original_cap` as coherent state components.
- Now: modeled; swap/move/delete domain lemmas scaffolded.

3. `INV-MDB-LINK`
- Req: prev/next local compatibility and structural emptiness semantics preserved by transitions.
- Src: impl `mdb/state.rs` transition definitions; l4v `mdb_cte_at`/`valid_mdb` family.
- Now: semantics present; full preservation proofs pending.

4. `INV-CDT-PARENT-WF`
- Req:
  - empty slot => no parent and not original
  - parent in dom, non-self, parent slot non-empty
- Src: manager/cdt proof predicates; l4v `cdt_parent_defs` usage.
- Now: specified; not globally closed per-op.

5. `INV-CDT-CAP-PARENT-SEM`
- Req: `parent_of` relation implies `should_be_parent_of(...)` on cap semantics.
- Src: `cdt/proof.rs` + `cte/spec.rs`; l4v `safe_parent_for`/`should_be_parent_of` chain.
- Now: predicate exists; full op preservation pending.

6. `INV-UNTYPED/REVOCABLE-FAMILY`
- Req: preserve untyped/irq/reply revocability and descendant monotonic conditions.
- Src: l4v `valid_mdb` conjuncts:
  - `untyped_mdb`, `descendants_inc`, `untyped_inc`,
  - `ut_revocable`, `irq_revocable`, `reply_master_revocable`, `reply_mdb`.
- Now: partially represented in current spec surface; full mapping/proofs pending.

7. `INV-ACYCLIC-WITNESS-MDB`
- Req: witness rank strictly increases along `next`.
- Src: `rank_witness_valid_for`; replacement strategy for heavy transitive-closure reasoning.
- Now: witness defined; integration into op proofs pending.

8. `INV-ACYCLIC-WITNESS-CDT`
- Req: witness depth strictly increases from parent to child.
- Src: `depth_witness_valid_for`; replacement strategy for descendants loop pressure.
- Now: witness defined; integration into manager-level proofs pending.

9. `INV-POSTSTATE-EXACTNESS`
- Req: manager post-state equals composition of owner `state_after_*` functions.
- Src: this freeze document + owner states.
- Now: not fully closed; this is the main proof-construction axis next.

## 10) Per-Operation Obligation Checklist (Execution Plan)

For each op (`insert/move/swap/delete/revoke`) prove in this order:

1. Exact post-state composition (`INV-POSTSTATE-EXACTNESS`).
2. Domain invariants (`INV-DOM-MDB`, `INV-DOM-CDT`).
3. Owner-local semantic invariants (`INV-MDB-LINK`, `INV-CDT-PARENT-WF`).
4. Cross-layer parent/cap semantics (`INV-CDT-CAP-PARENT-SEM`).
5. l4v-strength conjunct family touchpoints (`INV-UNTYPED/REVOCABLE-FAMILY`).
6. Optional witness closure where needed (`INV-ACYCLIC-WITNESS-*`).

---

Change policy:
- If semantics change, update this document first, then update owner `state_after_*`, then proof obligations.
- No silent semantic drift inside manager proof files.
