#[cfg(verus_keep_ghost)]
use super::state::depth_witness_valid_for;
#[cfg(verus_keep_ghost)]
use super::spec::{parent_semantics_wf_on, parent_slots_wf_on, spec_should_be_parent_of};
#[cfg(verus_keep_ghost)]
use crate::capability::spec::{
    lemma_same_region_after_untyped_full, lemma_same_region_preserved_when_rhs_same_except_untyped,
    same_cap_except_untyped_free_index, CapSpec,
};
use super::state::{CdtDepthWitness, CdtState};
use crate::cspace::types::SlotPtr;
use vstd::prelude::*;

verus! {

pub open spec fn cdt_parent_dom_wf_on(state: CdtState) -> bool {
    state.parent_of.dom() =~= state.dom()
}

pub open spec fn is_original_dom_wf_on(state: CdtState) -> bool {
    state.is_original.dom() =~= state.dom()
}

pub open spec fn parent_graph_wf_on(state: CdtState) -> bool {
    forall|slot: SlotPtr| #![auto]
        state.dom().contains(slot) ==> {
            let parent = state.parent(slot);
            parent is Some ==> {
                let parent_slot = parent.unwrap();
                &&& state.dom().contains(parent_slot)
                &&& parent_slot != slot
            }
        }
}

pub open spec fn no_mloop_wf_on(state: CdtState) -> bool {
    exists|witness: CdtDepthWitness| #[trigger] depth_witness_valid_for(state, witness)
}

pub open spec fn structural_wf_on(state: CdtState) -> bool {
    &&& state.maps_cover_dom()
    &&& cdt_parent_dom_wf_on(state)
    &&& is_original_dom_wf_on(state)
    &&& parent_graph_wf_on(state)
    &&& no_mloop_wf_on(state)
}

pub proof fn lemma_state_after_cap_insert_maps_cover_dom(
    state: CdtState,
    src: SlotPtr,
    dest: SlotPtr,
    src_parent: bool,
    dest_original: bool,
)
    requires
        state.maps_cover_dom(),
        state.dom().contains(src),
        state.dom().contains(dest),
    ensures
        state.state_after_cap_insert(src, dest, src_parent, dest_original).maps_cover_dom(),
{
    let new_state = state.state_after_cap_insert(src, dest, src_parent, dest_original);
    assert(new_state.dom() == state.dom());
    assert(new_state.parent_of.dom() =~= state.dom()) by {
        assert(new_state.parent_of.dom() =~= state.parent_of.dom());
        assert(state.parent_of.dom() =~= state.dom());
    }
    assert(new_state.is_original.dom() =~= state.dom()) by {
        assert(new_state.is_original.dom() =~= state.is_original.dom());
        assert(state.is_original.dom() =~= state.dom());
    }
}

pub proof fn lemma_state_after_insert_new_cap_preserves_structural_wf(
    old_state: CdtState,
    new_state: CdtState,
    parent: SlotPtr,
    slot: SlotPtr,
)
    requires
        structural_wf_on(old_state),
        old_state.dom() =~= new_state.dom(),
        old_state.dom().contains(parent),
        old_state.dom().contains(slot),
        parent != slot,
        old_state.parent(slot) is None,
        forall|child: SlotPtr| #[trigger]
            old_state.dom().contains(child) ==> old_state.parent(child) != Some(slot),
        new_state == old_state.state_after_insert_new_cap(parent, slot),
    ensures
        structural_wf_on(new_state),
{
    assert(new_state == old_state.state_after_cap_insert(parent, slot, true, true));
    lemma_state_after_cap_insert_preserves_structural_wf(
        old_state,
        new_state,
        parent,
        slot,
        true,
        true,
    );
}

pub proof fn lemma_state_after_cap_insert_preserves_structural_wf(
    old_state: CdtState,
    new_state: CdtState,
    src: SlotPtr,
    dest: SlotPtr,
    src_parent: bool,
    dest_original: bool,
)
    requires
        structural_wf_on(old_state),
        old_state.dom() =~= new_state.dom(),
        old_state.dom().contains(src),
        old_state.dom().contains(dest),
        src != dest,
        old_state.parent(dest) is None,
        forall|slot: SlotPtr| #[trigger]
            old_state.dom().contains(slot) ==> old_state.parent(slot) != Some(dest),
        new_state == old_state.state_after_cap_insert(src, dest, src_parent, dest_original),
    ensures
        structural_wf_on(new_state),
{
    let witness = choose|witness: CdtDepthWitness| depth_witness_valid_for(old_state, witness);
    let new_witness = CdtDepthWitness {
        depth: Map::new(
            |slot: SlotPtr| old_state.dom().contains(slot),
            |slot: SlotPtr|
                if slot == dest {
                    if src_parent {
                        witness.depth_of(src) + 1
                    } else {
                        witness.depth_of(src)
                    }
                } else {
                    witness.depth_of(slot)
                },
        ),
    };

    assert(old_state.maps_cover_dom());
    lemma_state_after_cap_insert_maps_cover_dom(old_state, src, dest, src_parent, dest_original);
    assert(new_state.maps_cover_dom());
    assert(cdt_parent_dom_wf_on(new_state));
    assert(is_original_dom_wf_on(new_state));

    assert(parent_graph_wf_on(new_state)) by {
        assert forall|slot: SlotPtr| #[trigger] new_state.dom().contains(slot) implies {
            let parent = new_state.parent(slot);
            parent is Some ==> {
                let parent_slot = parent.unwrap();
                &&& new_state.dom().contains(parent_slot)
                &&& parent_slot != slot
            }
        } by {
            if new_state.dom().contains(slot) {
                let parent = new_state.parent(slot);
                if parent is Some {
                    let parent_slot = parent.unwrap();
                    if slot == dest {
                        if src_parent {
                            assert(parent_slot == src);
                            assert(parent_slot != slot);
                        } else {
                            assert(old_state.parent(src) == Some(parent_slot));
                            assert(old_state.dom().contains(parent_slot));
                            assert(parent_slot != src);
                            assert(parent_slot != dest) by {
                                if parent_slot == dest {
                                    assert(old_state.parent(src) == Some(dest));
                                    assert(false);
                                }
                            }
                            assert(parent_slot != slot);
                        }
                    } else {
                        assert(old_state.parent(slot) == Some(parent_slot));
                        assert(old_state.dom().contains(parent_slot));
                        assert(parent_slot != slot);
                        assert(parent_slot != dest) by {
                            if parent_slot == dest {
                                assert(old_state.parent(slot) == Some(dest));
                                assert(false);
                            }
                        }
                    }
                }
            }
        }
    }

    assert(depth_witness_valid_for(new_state, new_witness)) by {
        assert(new_state.maps_cover_dom());
        assert(new_witness.dom() =~= new_state.dom()) by {
            assert(new_witness.dom() =~= old_state.dom());
            assert(old_state.dom() =~= new_state.dom());
        }
        assert forall|slot: SlotPtr| #[trigger]
            new_state.dom().contains(slot) && new_state.parent(slot) is Some implies {
                let parent = new_state.parent(slot).unwrap();
                &&& new_state.dom().contains(parent)
                &&& new_witness.depth_of(parent) < new_witness.depth_of(slot)
            } by {
            if new_state.dom().contains(slot) && new_state.parent(slot) is Some {
                let parent = new_state.parent(slot).unwrap();
                if slot == dest {
                    if src_parent {
                        assert(parent == src);
                        assert(new_witness.depth_of(parent) == witness.depth_of(src));
                        assert(new_witness.depth_of(slot) == witness.depth_of(src) + 1);
                    } else {
                        assert(old_state.parent(src) == Some(parent));
                        assert(witness.depth_of(parent) < witness.depth_of(src));
                        assert(new_witness.depth_of(parent) == witness.depth_of(parent));
                        assert(new_witness.depth_of(slot) == witness.depth_of(src));
                    }
                } else {
                    assert(old_state.parent(slot) == Some(parent));
                    assert(witness.depth_of(parent) < witness.depth_of(slot));
                    assert(new_witness.depth_of(parent) == witness.depth_of(parent));
                    assert(new_witness.depth_of(slot) == witness.depth_of(slot));
                }
            }
        }
    }

    assert(no_mloop_wf_on(new_state));
    assert(structural_wf_on(new_state));
}

pub proof fn lemma_cap_insert_parent_pointwise(
    state: CdtState,
    src: SlotPtr,
    dest: SlotPtr,
    src_parent: bool,
    dest_original: bool,
    x: SlotPtr,
)
    requires
        state.maps_cover_dom(),
        state.dom().contains(src),
        state.dom().contains(dest),
        state.dom().contains(x),
        src != dest,
    ensures
        state.state_after_cap_insert(src, dest, src_parent, dest_original).parent(x)
            == if x == dest {
                if src_parent {
                    Some(src)
                } else {
                    state.parent(src)
                }
            } else {
                state.parent(x)
            },
{
    let new_state = state.state_after_cap_insert(src, dest, src_parent, dest_original);
    assert(new_state.parent_of[x]
        == if x == dest {
            if src_parent {
                Some(src)
            } else {
                state.parent(src)
            }
        } else {
            state.parent(x)
        });
}

pub proof fn lemma_cap_insert_parent_locality(
    state: CdtState,
    src: SlotPtr,
    dest: SlotPtr,
    src_parent: bool,
    dest_original: bool,
    x: SlotPtr,
)
    requires
        state.maps_cover_dom(),
        state.dom().contains(src),
        state.dom().contains(dest),
        state.dom().contains(x),
        src != dest,
        x != dest,
    ensures
        state.state_after_cap_insert(src, dest, src_parent, dest_original).parent(x) == state.parent(x),
{
    lemma_cap_insert_parent_pointwise(state, src, dest, src_parent, dest_original, x);
}

pub proof fn lemma_cap_insert_original_pointwise(
    state: CdtState,
    src: SlotPtr,
    dest: SlotPtr,
    src_parent: bool,
    dest_original: bool,
    x: SlotPtr,
)
    requires
        state.maps_cover_dom(),
        state.dom().contains(src),
        state.dom().contains(dest),
        state.dom().contains(x),
        src != dest,
    ensures
        state.state_after_cap_insert(src, dest, src_parent, dest_original).original(x)
            == if x == dest { dest_original } else { state.original(x) },
{
    let new_state = state.state_after_cap_insert(src, dest, src_parent, dest_original);
    assert(new_state.is_original[x] == if x == dest { dest_original } else { state.original(x) });
}

pub proof fn lemma_state_after_cap_insert_preserves_parent_slots_wf(
    old_state: CdtState,
    new_state: CdtState,
    src: SlotPtr,
    dest: SlotPtr,
    src_parent: bool,
    dest_original: bool,
    old_nonempty: Set<SlotPtr>,
    new_nonempty: Set<SlotPtr>,
)
    requires
        structural_wf_on(old_state),
        parent_slots_wf_on(old_state, old_nonempty),
        old_state.dom() =~= new_state.dom(),
        old_state.dom().contains(src),
        old_state.dom().contains(dest),
        src != dest,
        old_state.parent(dest) is None,
        forall|slot: SlotPtr| #[trigger]
            old_state.dom().contains(slot) ==> old_state.parent(slot) != Some(dest),
        new_state == old_state.state_after_cap_insert(src, dest, src_parent, dest_original),
        old_nonempty.contains(src),
        !old_nonempty.contains(dest),
        new_nonempty =~= old_nonempty.insert(dest),
    ensures
        parent_slots_wf_on(new_state, new_nonempty),
{
    assert(parent_slots_wf_on(new_state, new_nonempty)) by {
        assert forall|slot: SlotPtr| #[trigger] new_state.dom().contains(slot) implies {
            let parent = new_state.parent(slot);
            &&& (!new_nonempty.contains(slot) ==> {
                &&& parent is None
                &&& !new_state.original(slot)
            })
            &&& (parent is Some ==> {
                let parent_slot = parent.unwrap();
                &&& new_nonempty.contains(parent_slot)
                &&& parent_slot != slot
            })
        } by {
            if new_state.dom().contains(slot) {
                if slot == dest {
                    lemma_cap_insert_parent_pointwise(
                        old_state, src, dest, src_parent, dest_original, dest,
                    );
                    lemma_cap_insert_original_pointwise(
                        old_state, src, dest, src_parent, dest_original, dest,
                    );
                    assert(new_nonempty.contains(dest));
                    if new_state.parent(dest) is Some {
                        let parent_slot = new_state.parent(dest).unwrap();
                        if src_parent {
                            assert(parent_slot == src);
                            assert(new_nonempty.contains(src));
                            assert(parent_slot != dest);
                        } else {
                            assert(old_state.parent(src) == Some(parent_slot));
                            assert(parent_slots_wf_on(old_state, old_nonempty));
                            assert(old_nonempty.contains(parent_slot));
                            assert(parent_slot != src);
                            assert(parent_slot != dest) by {
                                if parent_slot == dest {
                                    assert(old_state.parent(src) == Some(dest));
                                    assert(false);
                                }
                            }
                            assert(new_nonempty.contains(parent_slot));
                        }
                    }
                } else {
                    lemma_cap_insert_parent_locality(
                        old_state, src, dest, src_parent, dest_original, slot,
                    );
                    lemma_cap_insert_original_pointwise(
                        old_state, src, dest, src_parent, dest_original, slot,
                    );
                    assert(new_state.parent(slot) == old_state.parent(slot));
                    assert(new_state.original(slot) == old_state.original(slot));
                    if !new_nonempty.contains(slot) {
                        assert(slot != dest);
                        assert(!old_nonempty.contains(slot)) by {
                            if old_nonempty.contains(slot) {
                                assert(new_nonempty.contains(slot));
                                assert(false);
                            }
                        }
                        assert(parent_slots_wf_on(old_state, old_nonempty));
                        assert(old_state.parent(slot) is None);
                        assert(!old_state.original(slot));
                    }
                    if new_state.parent(slot) is Some {
                        let parent_slot = new_state.parent(slot).unwrap();
                        assert(old_state.parent(slot) == Some(parent_slot));
                        assert(parent_slots_wf_on(old_state, old_nonempty));
                        assert(old_nonempty.contains(parent_slot));
                        assert(parent_slot != slot);
                        assert(new_nonempty.contains(parent_slot));
                    }
                }
            }
        }
    }
}

pub proof fn lemma_state_after_insert_new_cap_preserves_parent_slots_wf(
    old_state: CdtState,
    new_state: CdtState,
    parent: SlotPtr,
    slot: SlotPtr,
    old_nonempty: Set<SlotPtr>,
    new_nonempty: Set<SlotPtr>,
)
    requires
        structural_wf_on(old_state),
        parent_slots_wf_on(old_state, old_nonempty),
        old_state.dom() =~= new_state.dom(),
        old_state.dom().contains(parent),
        old_state.dom().contains(slot),
        parent != slot,
        old_state.parent(slot) is None,
        forall|child: SlotPtr| #[trigger]
            old_state.dom().contains(child) ==> old_state.parent(child) != Some(slot),
        new_state == old_state.state_after_insert_new_cap(parent, slot),
        old_nonempty.contains(parent),
        !old_nonempty.contains(slot),
        new_nonempty =~= old_nonempty.insert(slot),
    ensures
        parent_slots_wf_on(new_state, new_nonempty),
{
    assert(new_state == old_state.state_after_cap_insert(parent, slot, true, true));
    lemma_state_after_cap_insert_preserves_parent_slots_wf(
        old_state,
        new_state,
        parent,
        slot,
        true,
        true,
        old_nonempty,
        new_nonempty,
    );
}

pub proof fn lemma_state_after_move_preserves_parent_slots_wf(
    old_state: CdtState, new_state: CdtState, src: SlotPtr, dest: SlotPtr,
    old_nonempty: Set<SlotPtr>,
)
    requires
        structural_wf_on(old_state),
        parent_slots_wf_on(old_state, old_nonempty),
        old_state.dom().contains(src),
        old_state.dom().contains(dest),
        src != dest,
        new_state == old_state.state_after_move(src, dest),
        old_nonempty.contains(src),
        !old_nonempty.contains(dest),
    ensures
        parent_slots_wf_on(new_state, old_nonempty.remove(src).insert(dest)),
{
    let new_nonempty = old_nonempty.remove(src).insert(dest);
    assert(parent_slots_wf_on(new_state, new_nonempty)) by {
        assert forall|slot: SlotPtr| #[trigger] new_state.dom().contains(slot) implies {
            let parent = new_state.parent(slot);
            &&& (!new_nonempty.contains(slot) ==> {
                &&& parent is None
                &&& !new_state.original(slot)
            })
            &&& (parent is Some ==> {
                let parent_slot = parent.unwrap();
                &&& new_nonempty.contains(parent_slot)
                &&& parent_slot != slot
            })
        } by {
            if new_state.dom().contains(slot) {
                assert(old_state.dom().contains(slot));
                lemma_move_parent_pointwise(old_state, src, dest, slot);
                lemma_move_original_pointwise(old_state, src, dest, slot);
            }
        }
    }
}

pub proof fn lemma_state_after_swap_preserves_parent_slots_wf(
    old_state: CdtState, new_state: CdtState, slot1: SlotPtr, slot2: SlotPtr,
    nonempty: Set<SlotPtr>,
)
    requires
        structural_wf_on(old_state),
        parent_slots_wf_on(old_state, nonempty),
        old_state.dom().contains(slot1),
        old_state.dom().contains(slot2),
        slot1 != slot2,
        new_state == old_state.state_after_swap(slot1, slot2),
        nonempty.contains(slot1),
        nonempty.contains(slot2),
    ensures
        parent_slots_wf_on(new_state, nonempty),
{
    assert(parent_slots_wf_on(new_state, nonempty)) by {
        assert forall|slot: SlotPtr| #[trigger] new_state.dom().contains(slot) implies {
            let parent = new_state.parent(slot);
            &&& (!nonempty.contains(slot) ==> {
                &&& parent is None
                &&& !new_state.original(slot)
            })
            &&& (parent is Some ==> {
                let parent_slot = parent.unwrap();
                &&& nonempty.contains(parent_slot)
                &&& parent_slot != slot
            })
        } by {
            if new_state.dom().contains(slot) {
                assert(old_state.dom().contains(slot));
                let donor = CdtState::swap_slot(slot, slot1, slot2);
                assert(old_state.dom().contains(donor));
                lemma_swap_parent_pointwise(old_state, slot1, slot2, slot);
                lemma_swap_original_pointwise(old_state, slot1, slot2, slot);
            }
        }
    }
}

pub proof fn lemma_state_after_delete_preserves_parent_slots_wf(
    old_state: CdtState, new_state: CdtState, deleted: SlotPtr,
    old_nonempty: Set<SlotPtr>,
)
    requires
        structural_wf_on(old_state),
        parent_slots_wf_on(old_state, old_nonempty),
        old_state.dom().contains(deleted),
        new_state == old_state.state_after_delete(deleted),
        old_nonempty.contains(deleted),
    ensures
        parent_slots_wf_on(new_state, old_nonempty.remove(deleted)),
{
    let new_nonempty = old_nonempty.remove(deleted);
    assert(parent_slots_wf_on(new_state, new_nonempty)) by {
        assert forall|slot: SlotPtr| #[trigger] new_state.dom().contains(slot) implies {
            let parent = new_state.parent(slot);
            &&& (!new_nonempty.contains(slot) ==> {
                &&& parent is None
                &&& !new_state.original(slot)
            })
            &&& (parent is Some ==> {
                let parent_slot = parent.unwrap();
                &&& new_nonempty.contains(parent_slot)
                &&& parent_slot != slot
            })
        } by {
            if new_state.dom().contains(slot) {
                lemma_delete_parent_pointwise(old_state, deleted, slot);
                lemma_delete_original_pointwise(old_state, deleted, slot);
            }
        }
    }
}

pub proof fn lemma_parent_slots_wf_implies_empty_slot_is_parentless(
    state: CdtState,
    nonempty: Set<SlotPtr>,
    slot: SlotPtr,
)
    requires
        parent_slots_wf_on(state, nonempty),
        state.dom().contains(slot),
        !nonempty.contains(slot),
    ensures
        state.parent(slot) is None,
        !state.original(slot),
        forall|child: SlotPtr| #[trigger]
            state.dom().contains(child) ==> state.parent(child) != Some(slot),
{
    assert(state.parent(slot) is None);
    assert(!state.original(slot));
    assert forall|child: SlotPtr| #[trigger]
        state.dom().contains(child) implies state.parent(child) != Some(slot) by {
        if state.dom().contains(child) {
            if state.parent(child) == Some(slot) {
                assert(parent_slots_wf_on(state, nonempty));
                assert(nonempty.contains(slot));
                assert(false);
            }
        }
    }
}

proof fn lemma_should_be_parent_of_preserved_when_parent_cap_same_except_untyped(
    old_parent_cap: CapSpec,
    new_parent_cap: CapSpec,
    parent_original: bool,
    child_cap: CapSpec,
    child_original: bool,
)
    requires
        same_cap_except_untyped_free_index(old_parent_cap, new_parent_cap),
        spec_should_be_parent_of(old_parent_cap, parent_original, child_cap, child_original),
    ensures
        spec_should_be_parent_of(new_parent_cap, parent_original, child_cap, child_original),
{
    lemma_same_region_after_untyped_full(old_parent_cap, new_parent_cap, child_cap);
    assert(new_parent_cap.kind == old_parent_cap.kind);
    assert(new_parent_cap.badge == old_parent_cap.badge);
}

proof fn lemma_should_be_parent_of_preserved_when_child_cap_same_except_untyped(
    parent_cap: CapSpec,
    parent_original: bool,
    old_child_cap: CapSpec,
    new_child_cap: CapSpec,
    child_original: bool,
)
    requires
        same_cap_except_untyped_free_index(old_child_cap, new_child_cap),
        spec_should_be_parent_of(parent_cap, parent_original, old_child_cap, child_original),
    ensures
        spec_should_be_parent_of(parent_cap, parent_original, new_child_cap, child_original),
{
    lemma_same_region_preserved_when_rhs_same_except_untyped(parent_cap, old_child_cap, new_child_cap);
    assert(new_child_cap.kind == old_child_cap.kind);
    assert(new_child_cap.badge == old_child_cap.badge);
}

pub proof fn lemma_parent_semantics_wf_on_implies_parent_original(
    state: CdtState,
    caps: Map<SlotPtr, CapSpec>,
    slot: SlotPtr,
)
    requires
        parent_semantics_wf_on(state, caps),
        state.dom().contains(slot),
        state.parent(slot) is Some,
    ensures
        state.dom().contains(state.parent(slot).unwrap()),
        state.original(state.parent(slot).unwrap()),
{
    let parent = state.parent(slot).unwrap();
    assert(state.dom().contains(parent)) by {
        assert(caps.dom() =~= state.dom());
        assert(caps.dom().contains(parent));
    }
    assert(state.original(parent)) by {
        assert(parent_semantics_wf_on(state, caps));
    }
}

pub proof fn lemma_state_after_cap_insert_preserves_parent_semantics_wf(
    old_state: CdtState,
    new_state: CdtState,
    src: SlotPtr,
    dest: SlotPtr,
    src_parent: bool,
    dest_original: bool,
    old_caps: Map<SlotPtr, CapSpec>,
    new_caps: Map<SlotPtr, CapSpec>,
)
    requires
        structural_wf_on(old_state),
        parent_semantics_wf_on(old_state, old_caps),
        old_state.dom() =~= new_state.dom(),
        old_state.dom().contains(src),
        old_state.dom().contains(dest),
        src != dest,
        old_state.parent(dest) is None,
        forall|slot: SlotPtr| #[trigger]
            old_state.dom().contains(slot) ==> old_state.parent(slot) != Some(dest),
        new_state == old_state.state_after_cap_insert(src, dest, src_parent, dest_original),
        old_caps.dom() =~= old_state.dom(),
        new_caps.dom() =~= new_state.dom(),
        same_cap_except_untyped_free_index(old_caps[src], new_caps[src]),
        forall|slot: SlotPtr| #![auto]
            old_state.dom().contains(slot) && slot != src && slot != dest ==> new_caps[slot] == old_caps[slot],
        src_parent ==> spec_should_be_parent_of(
            new_caps[src], old_state.original(src), new_caps[dest], dest_original,
        ),
        !src_parent ==> {
            &&& old_state.parent(src) is Some
            &&& {
                let parent = old_state.parent(src).unwrap();
                spec_should_be_parent_of(
                    old_caps[parent], old_state.original(parent), new_caps[dest], dest_original,
                )
            }
        },
    ensures
        parent_semantics_wf_on(new_state, new_caps),
{
    assert(parent_semantics_wf_on(new_state, new_caps)) by {
        assert(new_caps.dom() =~= new_state.dom());
        assert forall|slot: SlotPtr| #[trigger]
            new_state.dom().contains(slot) && new_state.parent(slot) is Some implies {
                let parent = new_state.parent(slot).unwrap();
                &&& new_caps.dom().contains(parent)
                &&& spec_should_be_parent_of(
                    new_caps[parent],
                    new_state.original(parent),
                    new_caps[slot],
                    new_state.original(slot),
                )
            } by {
            if new_state.dom().contains(slot) && new_state.parent(slot) is Some {
                let parent = new_state.parent(slot).unwrap();
                if slot == dest {
                    if src_parent {
                        assert(parent == src);
                        lemma_should_be_parent_of_preserved_when_parent_cap_same_except_untyped(
                            old_caps[src],
                            new_caps[src],
                            old_state.original(src),
                            new_caps[dest],
                            dest_original,
                        );
                    } else {
                        assert(old_state.parent(src) == Some(parent));
                        assert(spec_should_be_parent_of(
                            old_caps[parent],
                            old_state.original(parent),
                            new_caps[dest],
                            dest_original,
                        ));
                        assert(new_caps[parent] == old_caps[parent]);
                        assert(new_state.original(parent) == old_state.original(parent));
                    }
                } else if slot == src {
                    assert(old_state.parent(src) == Some(parent));
                    assert(new_caps[parent] == old_caps[parent]);
                    assert(new_state.original(parent) == old_state.original(parent));
                    assert(new_state.original(src) == old_state.original(src));
                    lemma_should_be_parent_of_preserved_when_child_cap_same_except_untyped(
                        old_caps[parent],
                        old_state.original(parent),
                        old_caps[src],
                        new_caps[src],
                        old_state.original(src),
                    );
                    assert(parent_semantics_wf_on(old_state, old_caps));
                } else if old_state.parent(slot) == Some(src) {
                    assert(parent == src);
                    assert(new_caps[slot] == old_caps[slot]);
                    assert(new_state.original(slot) == old_state.original(slot));
                    assert(new_state.original(src) == old_state.original(src));
                    lemma_should_be_parent_of_preserved_when_parent_cap_same_except_untyped(
                        old_caps[src],
                        new_caps[src],
                        old_state.original(src),
                        old_caps[slot],
                        old_state.original(slot),
                    );
                    assert(parent_semantics_wf_on(old_state, old_caps));
                } else {
                    assert(old_state.parent(slot) == Some(parent));
                    assert(new_caps[parent] == old_caps[parent]);
                    assert(new_caps[slot] == old_caps[slot]);
                    assert(new_state.original(parent) == old_state.original(parent));
                    assert(new_state.original(slot) == old_state.original(slot));
                    assert(parent_semantics_wf_on(old_state, old_caps));
                }
            }
        }
    }
}

pub proof fn lemma_state_after_insert_new_cap_preserves_parent_semantics_wf(
    old_state: CdtState,
    new_state: CdtState,
    parent: SlotPtr,
    slot: SlotPtr,
    old_caps: Map<SlotPtr, CapSpec>,
    new_caps: Map<SlotPtr, CapSpec>,
)
    requires
        structural_wf_on(old_state),
        parent_semantics_wf_on(old_state, old_caps),
        old_state.dom() =~= new_state.dom(),
        old_state.dom().contains(parent),
        old_state.dom().contains(slot),
        parent != slot,
        old_state.parent(slot) is None,
        forall|child: SlotPtr| #[trigger]
            old_state.dom().contains(child) ==> old_state.parent(child) != Some(slot),
        new_state == old_state.state_after_insert_new_cap(parent, slot),
        old_caps.dom() =~= old_state.dom(),
        new_caps.dom() =~= new_state.dom(),
        new_caps[parent] == old_caps[parent],
        forall|s: SlotPtr| #![auto]
            old_state.dom().contains(s) && s != slot ==> new_caps[s] == old_caps[s],
        spec_should_be_parent_of(new_caps[parent], old_state.original(parent), new_caps[slot], true),
    ensures
        parent_semantics_wf_on(new_state, new_caps),
{
    assert(new_state == old_state.state_after_cap_insert(parent, slot, true, true));
    lemma_state_after_cap_insert_preserves_parent_semantics_wf(
        old_state,
        new_state,
        parent,
        slot,
        true,
        true,
        old_caps,
        new_caps,
    );
}

pub proof fn lemma_move_parent_pointwise(state: CdtState, src: SlotPtr, dest: SlotPtr, x: SlotPtr)
    requires
        state.maps_cover_dom(),
        state.dom().contains(src),
        state.dom().contains(dest),
        state.dom().contains(x),
        src != dest,
    ensures
        state.state_after_move(src, dest).parent(x) == state.moved_parent_of(x, src, dest),
{
    let new_state = state.state_after_move(src, dest);
    assert(new_state.parent_of[x] == state.moved_parent_of(x, src, dest));
}

pub proof fn lemma_move_parent_locality(state: CdtState, src: SlotPtr, dest: SlotPtr, x: SlotPtr)
    requires
        state.maps_cover_dom(),
        state.dom().contains(src),
        state.dom().contains(dest),
        state.dom().contains(x),
        src != dest,
        x != src,
        x != dest,
        state.parent(x) != Some(src),
    ensures
        state.state_after_move(src, dest).parent(x) == state.parent(x),
{
    lemma_move_parent_pointwise(state, src, dest, x);
    assert(state.moved_parent_of(x, src, dest) == state.parent(x));
}

pub proof fn lemma_move_original_pointwise(state: CdtState, src: SlotPtr, dest: SlotPtr, x: SlotPtr)
    requires
        state.maps_cover_dom(),
        state.dom().contains(src),
        state.dom().contains(dest),
        state.dom().contains(x),
        src != dest,
    ensures
        state.state_after_move(src, dest).original(x)
            == if x == src {
                false
            } else if x == dest {
                state.original(src)
            } else {
                state.original(x)
            },
{
    let new_state = state.state_after_move(src, dest);
    assert(new_state.is_original[x]
        == if x == src {
            false
        } else if x == dest {
            state.original(src)
        } else {
            state.original(x)
        });
}

pub proof fn lemma_swap_parent_pointwise(state: CdtState, slot1: SlotPtr, slot2: SlotPtr, x: SlotPtr)
    requires
        state.maps_cover_dom(),
        state.dom().contains(slot1),
        state.dom().contains(slot2),
        state.dom().contains(x),
    ensures
        state.state_after_swap(slot1, slot2).parent(x) == state.swapped_parent_of(x, slot1, slot2),
{
    let new_state = state.state_after_swap(slot1, slot2);
    assert(new_state.parent_of[x] == state.swapped_parent_of(x, slot1, slot2));
}

pub proof fn lemma_swap_original_pointwise(state: CdtState, slot1: SlotPtr, slot2: SlotPtr, x: SlotPtr)
    requires
        state.maps_cover_dom(),
        state.dom().contains(slot1),
        state.dom().contains(slot2),
        state.dom().contains(x),
        slot1 != slot2,
    ensures
        state.state_after_swap(slot1, slot2).original(x)
            == if x == slot1 {
                state.original(slot2)
            } else if x == slot2 {
                state.original(slot1)
            } else {
                state.original(x)
            },
{
    let new_state = state.state_after_swap(slot1, slot2);
    assert(new_state.is_original[x]
        == if x == slot1 {
            state.original(slot2)
        } else if x == slot2 {
            state.original(slot1)
        } else {
            state.original(x)
        });
}

pub proof fn lemma_delete_parent_pointwise(state: CdtState, deleted: SlotPtr, x: SlotPtr)
    requires
        state.maps_cover_dom(),
        state.dom().contains(deleted),
        state.dom().contains(x),
    ensures
        state.state_after_delete(deleted).parent(x) == state.deleted_parent_of(x, deleted),
{
    let new_state = state.state_after_delete(deleted);
    assert(new_state.parent_of[x] == state.deleted_parent_of(x, deleted));
}

pub proof fn lemma_delete_original_pointwise(state: CdtState, deleted: SlotPtr, x: SlotPtr)
    requires
        state.maps_cover_dom(),
        state.dom().contains(deleted),
        state.dom().contains(x),
    ensures
        state.state_after_delete(deleted).original(x) == if x == deleted {
            false
        } else {
            state.original(x)
        },
{
    let new_state = state.state_after_delete(deleted);
    assert(new_state.is_original[x] == if x == deleted {
        false
    } else {
        state.original(x)
    });
}

pub proof fn lemma_state_after_delete_identity(state: CdtState, deleted: SlotPtr)
    requires
        state.maps_cover_dom(),
        state.dom().contains(deleted),
        state.parent(deleted) is None,
        !state.original(deleted),
        forall|x: SlotPtr| #[trigger] state.dom().contains(x) ==> state.parent(x) != Some(deleted),
    ensures
        state.state_after_delete(deleted) == state,
{
    let new_state = state.state_after_delete(deleted);
    assert(new_state.parent_of =~= state.parent_of) by {
        assert forall|x: SlotPtr| #[trigger] state.parent_of.dom().contains(x)
            implies new_state.parent_of[x] == state.parent_of[x] by {
            assert(state.dom().contains(x));
            assert(new_state.parent_of[x] == state.deleted_parent_of(x, deleted));
        }
    }
    assert(new_state.is_original =~= state.is_original) by {
        assert forall|x: SlotPtr| #[trigger] state.is_original.dom().contains(x)
            implies new_state.is_original[x] == state.is_original[x] by {
            assert(new_state.is_original[x] == if x == deleted { false } else { state.original(x) });
        }
    }
    assert(new_state == state);
}

pub proof fn lemma_state_after_swap_maps_cover_dom(state: CdtState, slot1: SlotPtr, slot2: SlotPtr)
    requires
        state.maps_cover_dom(),
        state.dom().contains(slot1),
        state.dom().contains(slot2),
    ensures
        state.state_after_swap(slot1, slot2).maps_cover_dom(),
{
    let new_state = state.state_after_swap(slot1, slot2);
    assert(new_state.dom() == state.dom());
    assert(new_state.parent_of.dom() =~= state.dom()) by {
        assert forall|x: SlotPtr| #[trigger] new_state.parent_of.dom().contains(x)
            implies state.dom().contains(x) by {};
        assert forall|x: SlotPtr| #[trigger] state.dom().contains(x)
            implies new_state.parent_of.dom().contains(x) by {};
    }
    assert(new_state.is_original.dom() =~= state.dom()) by {
        assert(new_state.is_original.dom() =~= state.is_original.dom());
        assert(state.is_original.dom() =~= state.dom());
    }
}

pub proof fn lemma_state_after_move_maps_cover_dom(state: CdtState, src: SlotPtr, dest: SlotPtr)
    requires
        state.maps_cover_dom(),
        state.dom().contains(src),
        state.dom().contains(dest),
    ensures
        state.state_after_move(src, dest).maps_cover_dom(),
{
    let new_state = state.state_after_move(src, dest);
    assert(new_state.dom() == state.dom());
    assert(new_state.parent_of.dom() =~= state.dom()) by {
        assert forall|x: SlotPtr| #[trigger] new_state.parent_of.dom().contains(x)
            implies state.dom().contains(x) by {};
        assert forall|x: SlotPtr| #[trigger] state.dom().contains(x)
            implies new_state.parent_of.dom().contains(x) by {};
    }
    assert(new_state.is_original.dom() =~= state.dom()) by {
        assert(new_state.is_original.dom() =~= state.is_original.dom());
        assert(state.is_original.dom() =~= state.dom());
    }
}

pub proof fn lemma_state_after_move_preserves_structural_wf(
    old_state: CdtState,
    new_state: CdtState,
    src: SlotPtr,
    dest: SlotPtr,
)
    requires
        structural_wf_on(old_state),
        old_state.dom() =~= new_state.dom(),
        old_state.dom().contains(src),
        old_state.dom().contains(dest),
        old_state.parent(dest) is None,
        forall|slot: SlotPtr| #[trigger]
            old_state.dom().contains(slot) ==> old_state.parent(slot) != Some(dest),
        new_state == old_state.state_after_move(src, dest),
    ensures
        structural_wf_on(new_state),
{
    let witness = choose|witness: CdtDepthWitness| depth_witness_valid_for(old_state, witness);
    let new_witness = CdtDepthWitness {
        depth: Map::new(
            |slot: SlotPtr| old_state.dom().contains(slot),
            |slot: SlotPtr|
                if slot == src {
                    0
                } else if slot == dest {
                    witness.depth_of(src)
                } else {
                    witness.depth_of(slot)
                },
        ),
    };

    assert(old_state.maps_cover_dom());
    lemma_state_after_move_maps_cover_dom(old_state, src, dest);
    assert(new_state.maps_cover_dom());
    assert(cdt_parent_dom_wf_on(new_state));
    assert(is_original_dom_wf_on(new_state));

    assert(parent_graph_wf_on(new_state)) by {
        assert forall|slot: SlotPtr| #[trigger] new_state.dom().contains(slot) implies {
            let parent = new_state.parent(slot);
            parent is Some ==> {
                let parent_slot = parent.unwrap();
                &&& new_state.dom().contains(parent_slot)
                &&& parent_slot != slot
            }
        } by {
            if new_state.dom().contains(slot) {
                let parent = new_state.parent(slot);
                if parent is Some {
                    let parent_slot = parent.unwrap();
                    if slot == src {
                        assert(false);
                    } else if slot == dest {
                        assert(old_state.parent(src) == Some(parent_slot));
                        assert(old_state.dom().contains(parent_slot));
                        assert(parent_slot != src);
                        assert(parent_slot != dest) by {
                            if parent_slot == dest {
                                assert(old_state.parent(src) == Some(dest));
                                assert(false);
                            }
                        }
                        assert(parent_slot != slot);
                    } else if old_state.parent(slot) == Some(src) {
                        assert(parent_slot == dest);
                        assert(old_state.dom().contains(dest));
                        assert(slot != src);
                        assert(slot != dest) by {
                            if slot == dest {
                                assert(old_state.parent(dest) == Some(src));
                                assert(false);
                            }
                        }
                    } else {
                        assert(old_state.parent(slot) == Some(parent_slot));
                        assert(old_state.dom().contains(parent_slot));
                        assert(parent_slot != slot);
                        assert(parent_slot != dest) by {
                            if parent_slot == dest {
                                assert(old_state.parent(slot) == Some(dest));
                                assert(false);
                            }
                        }
                    }
                }
            }
        }
    }

    assert(depth_witness_valid_for(new_state, new_witness)) by {
        assert(new_state.maps_cover_dom());
        assert(new_witness.dom() =~= new_state.dom()) by {
            assert(new_witness.dom() =~= old_state.dom());
            assert(old_state.dom() =~= new_state.dom());
        }
        assert forall|slot: SlotPtr| #[trigger]
            new_state.dom().contains(slot) && new_state.parent(slot) is Some implies {
                let parent = new_state.parent(slot).unwrap();
                &&& new_state.dom().contains(parent)
                &&& new_witness.depth_of(parent) < new_witness.depth_of(slot)
            } by {
            if new_state.dom().contains(slot) && new_state.parent(slot) is Some {
                let parent = new_state.parent(slot).unwrap();
                if slot == dest {
                    assert(old_state.parent(src) == Some(parent));
                    assert(witness.depth_of(parent) < witness.depth_of(src));
                    assert(new_witness.depth_of(parent) == witness.depth_of(parent));
                    assert(new_witness.depth_of(slot) == witness.depth_of(src));
                } else if old_state.parent(slot) == Some(src) {
                    assert(parent == dest);
                    assert(old_state.dom().contains(slot));
                    assert(old_state.parent(slot) == Some(src));
                    assert(witness.depth_of(src) < witness.depth_of(slot));
                    assert(new_witness.depth_of(parent) == witness.depth_of(src));
                    assert(new_witness.depth_of(slot) == witness.depth_of(slot));
                } else {
                    assert(old_state.parent(slot) == Some(parent));
                    assert(witness.depth_of(parent) < witness.depth_of(slot));
                    assert(new_witness.depth_of(parent) == witness.depth_of(parent));
                    assert(new_witness.depth_of(slot) == witness.depth_of(slot));
                }
            }
        }
    }

    assert(no_mloop_wf_on(new_state));
    assert(structural_wf_on(new_state));
}

pub proof fn lemma_state_after_swap_preserves_structural_wf(
    old_state: CdtState,
    new_state: CdtState,
    slot1: SlotPtr,
    slot2: SlotPtr,
)
    requires
        structural_wf_on(old_state),
        old_state.dom() =~= new_state.dom(),
        old_state.dom().contains(slot1),
        old_state.dom().contains(slot2),
        new_state == old_state.state_after_swap(slot1, slot2),
    ensures
        structural_wf_on(new_state),
{
    let witness = choose|witness: CdtDepthWitness| depth_witness_valid_for(old_state, witness);
    let new_witness = CdtDepthWitness {
        depth: Map::new(
            |slot: SlotPtr| old_state.dom().contains(slot),
            |slot: SlotPtr| witness.depth_of(CdtState::swap_slot(slot, slot1, slot2)),
        ),
    };

    assert(old_state.maps_cover_dom());
    lemma_state_after_swap_maps_cover_dom(old_state, slot1, slot2);
    assert(new_state.maps_cover_dom());
    assert(cdt_parent_dom_wf_on(new_state));
    assert(is_original_dom_wf_on(new_state));

    assert(parent_graph_wf_on(new_state)) by {
        assert forall|slot: SlotPtr| #[trigger] new_state.dom().contains(slot) implies {
            let parent = new_state.parent(slot);
            parent is Some ==> {
                let parent_slot = parent.unwrap();
                &&& new_state.dom().contains(parent_slot)
                &&& parent_slot != slot
            }
        } by {
            if new_state.dom().contains(slot) {
                let parent = new_state.parent(slot);
                if parent is Some {
                    let parent_slot = parent.unwrap();
                    let old_slot = CdtState::swap_slot(slot, slot1, slot2);
                    let old_parent = old_state.parent(old_slot);
                    assert(old_parent == Some(CdtState::swap_slot(parent_slot, slot1, slot2)));
                    assert(old_state.dom().contains(old_slot));
                    assert(old_state.dom().contains(CdtState::swap_slot(parent_slot, slot1, slot2)));
                    assert(CdtState::swap_slot(parent_slot, slot1, slot2) != old_slot);
                    assert(parent_slot != slot) by {
                        if parent_slot == slot {
                            assert(CdtState::swap_slot(parent_slot, slot1, slot2)
                                == CdtState::swap_slot(slot, slot1, slot2));
                            assert(false);
                        }
                    }
                }
            }
        }
    }

    assert(depth_witness_valid_for(new_state, new_witness)) by {
        assert(new_state.maps_cover_dom());
        assert(new_witness.dom() =~= new_state.dom()) by {
            assert(new_witness.dom() =~= old_state.dom());
            assert(old_state.dom() =~= new_state.dom());
        }
        assert forall|slot: SlotPtr| #[trigger]
            new_state.dom().contains(slot) && new_state.parent(slot) is Some implies {
                let parent = new_state.parent(slot).unwrap();
                &&& new_state.dom().contains(parent)
                &&& new_witness.depth_of(parent) < new_witness.depth_of(slot)
            } by {
            if new_state.dom().contains(slot) && new_state.parent(slot) is Some {
                let parent = new_state.parent(slot).unwrap();
                let old_slot = CdtState::swap_slot(slot, slot1, slot2);
                let old_parent = CdtState::swap_slot(parent, slot1, slot2);
                assert(old_state.parent(old_slot) == Some(old_parent));
                assert(witness.depth_of(old_parent) < witness.depth_of(old_slot));
                assert(new_witness.depth_of(parent) == witness.depth_of(old_parent));
                assert(new_witness.depth_of(slot) == witness.depth_of(old_slot));
            }
        }
    }

    assert(no_mloop_wf_on(new_state));
    assert(structural_wf_on(new_state));
}

pub proof fn lemma_state_after_delete_preserves_structural_wf(
    old_state: CdtState,
    new_state: CdtState,
    deleted: SlotPtr,
)
    requires
        structural_wf_on(old_state),
        old_state.dom() =~= new_state.dom(),
        old_state.dom().contains(deleted),
        new_state == old_state.state_after_delete(deleted),
    ensures
        structural_wf_on(new_state),
{
    let witness = choose|witness: CdtDepthWitness| depth_witness_valid_for(old_state, witness);

    assert(old_state.maps_cover_dom());
    assert(cdt_parent_dom_wf_on(old_state));
    assert(is_original_dom_wf_on(old_state));
    assert(parent_graph_wf_on(old_state));
    assert(no_mloop_wf_on(old_state));

    lemma_state_after_delete_maps_cover_dom(old_state, deleted);
    assert(new_state.maps_cover_dom());
    assert(cdt_parent_dom_wf_on(new_state));
    assert(is_original_dom_wf_on(new_state));

    assert(parent_graph_wf_on(new_state)) by {
        assert forall|slot: SlotPtr| #[trigger] new_state.dom().contains(slot) implies {
            let parent = new_state.parent(slot);
            parent is Some ==> {
                let parent_slot = parent.unwrap();
                &&& new_state.dom().contains(parent_slot)
                &&& parent_slot != slot
            }
        } by {
            if new_state.dom().contains(slot) {
                let parent = new_state.parent(slot);
                if parent is Some {
                    let parent_slot = parent.unwrap();
                    assert(slot != deleted);
                    assert(old_state.deleted_parent_of(slot, deleted) == Some(parent_slot));
                    assert(old_state.parent(slot) == Some(parent_slot));
                    assert(old_state.dom().contains(slot));
                    assert(old_state.dom().contains(parent_slot));
                    assert(parent_slot != slot);
                }
            }
        }
    }

    assert(depth_witness_valid_for(new_state, witness)) by {
        assert(new_state.maps_cover_dom());
        assert(witness.dom() =~= new_state.dom()) by {
            assert(witness.dom() =~= old_state.dom());
            assert(old_state.dom() =~= new_state.dom());
        }
        assert forall|slot: SlotPtr| #[trigger]
            new_state.dom().contains(slot) && new_state.parent(slot) is Some implies {
                let parent = new_state.parent(slot).unwrap();
                &&& new_state.dom().contains(parent)
                &&& witness.depth_of(parent) < witness.depth_of(slot)
            } by {
            if new_state.dom().contains(slot) && new_state.parent(slot) is Some {
                let parent = new_state.parent(slot).unwrap();
                assert(slot != deleted);
                assert(old_state.deleted_parent_of(slot, deleted) == Some(parent));
                assert(old_state.parent(slot) == Some(parent));
                assert(old_state.dom().contains(slot));
                assert(old_state.dom().contains(parent));
                assert(witness.depth_of(parent) < witness.depth_of(slot));
            }
        }
    }

    assert(no_mloop_wf_on(new_state));
    assert(structural_wf_on(new_state));
}

pub proof fn lemma_state_after_delete_maps_cover_dom(state: CdtState, deleted: SlotPtr)
    requires
        state.maps_cover_dom(),
        state.dom().contains(deleted),
    ensures
        state.state_after_delete(deleted).maps_cover_dom(),
{
    let new_state = state.state_after_delete(deleted);
    assert(new_state.dom() == state.dom());
    assert(new_state.parent_of.dom() =~= state.dom()) by {
        assert forall|x: SlotPtr| #[trigger] new_state.parent_of.dom().contains(x)
            implies state.dom().contains(x) by {};
        assert forall|x: SlotPtr| #[trigger] state.dom().contains(x)
            implies new_state.parent_of.dom().contains(x) by {};
    }
    assert(new_state.is_original.dom() =~= state.dom()) by {
        assert(new_state.is_original.dom() =~= state.is_original.dom());
        assert(state.is_original.dom() =~= state.dom());
    }
}

}
