# Step 10: Flat-Graph Proof Strategy

This document freezes the proof strategy for opening `MdbTable` owner
primitives. It is written against `insert_node_after` first, then reused for
`move_node`, `swap_nodes`, and `remove_node`.

The goal is not to prove manager semantics directly. The goal is to prove the
owner primitive on the flat MDB graph and then let higher layers consume that
owner post-state.

## Scope

This strategy covers the MDB owner layer only:

- concrete tracked entries: `Tracked<Map<SlotPtr, PointsTo<cte_t>>>`
- flat logical graph: `entry_view`, `prev_of`, `next_of`, `revocable_of`,
  `first_badged_of`, `order`, `live_slots`
- owner relations: `insert_between_rel`, `move_slot_rel`, `swap_slots_rel`,
  `remove_slot_rel`
- owner invariant: `structural_wf()`

It does not directly prove:

- cap semantics
- CDT semantics
- manager combined `wf`

Those remain higher-layer obligations.

## Core Strategy

Use a two-step proof organization:

1. abstract flat-graph delta first
2. concrete write simulates that delta second

Do not prove `structural_wf()` by directly staring at raw writes. First define
the flat transition and prove its properties. Then prove the primitive realizes
that transition.

## Canonical Flat Model

The canonical flat graph is the current `MdbTable` view itself:

- `prev_of(slot)`
- `next_of(slot)`
- `revocable_of(slot)`
- `first_badged_of(slot)`
- `order@`
- `live_slots@`

No separate persistent `MdbState` field is introduced.

## Insert Proof Phases

### Phase A: Flat Transition Vocabulary

For insert, the proof should expose pure transition functions on the flat model.

Minimum vocabulary:

- `insert_footprint(src, dest, old_next)`
- `insert_prev_after(slot, src, dest, old_next)`
- `insert_next_after(slot, src, dest, old_next)`
- `insert_revocable_after(slot, dest, revocable)`
- `insert_first_badged_after(slot, dest, first_badged)`

`order_after_insert_between` and `live_slots.insert(dest)` are already the
summary transition.

These functions define the exact abstract post-state for insert.

### Phase B: Delta Lemmas

Prove exact changed-node and unchanged-node facts on the flat graph before
touching invariant preservation.

Required shape:

- changed nodes exact:
  - `dest`
  - `src`
  - `old_next`, when present
- unchanged outside footprint

The footprint for insert is:

- `{src, dest}` when `old_next` is `None`
- `{src, dest, old_next}` when `old_next` is `Some`

These lemmas should not mention manager semantics.

### Phase C: Abstract Preservation Lemmas

Once the flat delta is available, prove preservation of the owner invariant on
the abstract transition itself.

Split `structural_wf()` into its parts:

1. `entries_wf`
2. `summary_wf`
3. `order_links_wf`

The preservation lemmas should match that split:

- `lemma_insert_preserves_summary_wf`
- `lemma_insert_preserves_order_links_wf`
- `lemma_insert_preserves_structural_wf`

`entries_wf` is expected to be frame-based and light.

`summary_wf` should rely on pure sequence/set lemmas about
`order_after_insert_between`.

`order_links_wf` should be proved from:

- changed-node exact facts
- unchanged-outside-footprint facts
- pure order-neighbor lemmas for `order_after_insert_between`

Do not prove acyclicity using path closure or reachability. Use `order@` as the
acyclic witness and prove link/order agreement.

### Phase D: Concrete-to-Flat Simulation

Only after the abstract transition is in place should `insert_node_after` be
opened.

The primitive proof should have this shape:

1. perform raw writes
2. use local raw bridge facts to recover changed-node flat facts
3. recover unchanged-outside-footprint facts
4. update ghost `order/live_slots`
5. close `insert_between_rel`
6. invoke abstract preservation lemmas
7. conclude `self.structural_wf()`

This keeps concrete-write reasoning separate from invariant reasoning.

## What Stays Trusted

The mature target is not zero trusted code. The mature target is to push trust
down to the raw bridge boundary.

Acceptable raw trusted boundary:

- primitive field reads in `mdb/raw.rs`
- primitive field-write bridge lemmas, if needed

Unacceptable remaining trust for the insert MDB line:

- `insert_node_after` as a whole primitive
- owner-level same-links transfer lemmas that can be proved from flat graph

## Concrete Deliverables For Insert

To call the insert MDB proof line done, the following must hold:

1. `insert_node_after` is no longer `external_body`
2. the insert same-links bridge lemmas are no longer `external_body`
3. the insert relation is closed through flat-graph delta reasoning, not by a
   black-box bridge
4. `cspace::mdb::table` verifies
5. `cspace::manager::impl_insert` verifies using the opened owner primitive

## Follow-On Reuse

After insert, `move_node` and `swap_nodes` should reuse the same proof shape:

- define flat footprint
- prove delta lemmas
- prove abstract preservation
- prove concrete-to-flat simulation

Only the footprint and abstract delta change. The proof architecture must stay
the same.
