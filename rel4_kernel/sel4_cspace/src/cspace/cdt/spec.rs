#[cfg(verus_keep_ghost)]
use crate::capability::spec::{spec_same_region_as_caps, CapKind, CapSpec};
use crate::cspace::cdt::state::CdtState;
use crate::cspace::types::SlotPtr;
use vstd::prelude::*;

verus! {

pub open spec fn spec_should_be_parent_of(
    parent_cap: CapSpec,
    parent_original: bool,
    child_cap: CapSpec,
    child_original: bool,
) -> bool {
    &&& parent_original
    &&& spec_same_region_as_caps(parent_cap, child_cap)
    &&& (if parent_cap.kind == CapKind::EndpointCap
        && parent_cap.badge is Some
        && parent_cap.badge.unwrap() != 0 {
        &&& child_cap.kind == CapKind::EndpointCap
        &&& child_cap.badge == parent_cap.badge
        &&& !child_original
    } else if parent_cap.kind == CapKind::NotificationCap
        && parent_cap.badge is Some
        && parent_cap.badge.unwrap() != 0 {
        &&& child_cap.kind == CapKind::NotificationCap
        &&& child_cap.badge == parent_cap.badge
        &&& !child_original
    } else {
        true
    })
}

pub open spec fn parent_slots_wf_on(state: CdtState, nonempty: Set<SlotPtr>) -> bool {
    forall|slot: SlotPtr| #![auto]
        state.dom().contains(slot) ==> {
            let parent = state.parent(slot);
            &&& (!nonempty.contains(slot) ==> {
                &&& parent is None
                &&& !state.original(slot)
            })
            &&& (parent is Some ==> {
                let parent_slot = parent.unwrap();
                &&& nonempty.contains(parent_slot)
                &&& parent_slot != slot
            })
        }
}

pub open spec fn parent_semantics_wf_on(state: CdtState, caps: Map<SlotPtr, CapSpec>) -> bool {
    &&& caps.dom() =~= state.dom()
    &&& forall|slot: SlotPtr| #![auto]
        state.dom().contains(slot) && state.parent(slot) is Some ==> {
            let parent = state.parent(slot).unwrap();
            &&& caps.dom().contains(parent)
            &&& spec_should_be_parent_of(
                caps[parent],
                state.original(parent),
                caps[slot],
                state.original(slot),
            )
        }
}

}
