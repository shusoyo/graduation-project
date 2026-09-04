# Stage 3 Todo

## Goal

Build a seL4-style abstract state model for `sel4_cspace`, define shared abstract
semantics for primitive and capability-op layers, and draft the concrete/abstract
view.

## Deliverables

- Abstract state model document
- Core semantic definitions
- Concrete/abstract view draft

## Checklist

- [ ] Keep the current Rust implementation in `src/` as the concrete layer.
- [ ] Use `proof/` as the Verus workspace for stage 3 work.
- [ ] Define `ObjId`, `SlotId`, `CapKind`, `Rights`, `AbsCap`, `AbsState`.
- [ ] Decide the abstract derivation model (`parent_of` / `children_of`).
- [ ] Write `wf_slots`, `wf_derivation`, `wf_objects`, `wf_cspace`.
- [ ] Write primitive-layer `pre/post` for:
  - `derive_cap`
  - `cte_insert`
  - `cte_move`
  - `delete_all`
  - `revoke`
  - `insert_new_cap`
- [ ] Write capability-op-layer `pre/post` for:
  - `Copy`
  - `Mint`
  - `Move`
  - `Delete`
  - `Revoke`
  - `Retype` cspace-install part
- [ ] Draft `cap_view`, `slot_view`, `state_view`.
- [ ] Prepare a concrete-to-abstract mapping note for:
  - concrete `cte`
  - concrete `cap`
  - concrete `mdb`
  - object references
  - rights fields

## Notes

- Stage 3 focuses on model, semantics, and view.
- Stage 3 does not require full proofs of all invariants.
- Stage 3 does not require replacing the whole `src/` tree with Verus code.
