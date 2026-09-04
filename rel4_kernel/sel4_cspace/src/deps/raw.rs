#[cfg(verus_keep_ghost)]
use crate::capability::raw::{
    spec_cap_removable, spec_same_zombie_shape, spec_zombie_number_cap, spec_zombie_ptr_cap,
    spec_zombie_type_cap, trusted_zombie_end_slot_of_cap,
};
use crate::capability::spec::{CapKind, CapSpec};
#[cfg(verus_keep_ghost)]
use crate::cspace::manager::CSpaceManager;
use crate::cspace::types::SlotPtr;
#[cfg(verus_keep_ghost)]
use sel4_common::structures::exception_t;
use vstd::prelude::*;

verus! {

pub uninterp spec fn finalise_cap_contract(
    capability: CapSpec,
    is_final: bool,
    exposed: bool,
) -> (CapSpec, CapSpec);

pub open spec fn finalise_cap_non_immediate_reduce_ready(
    capability: CapSpec,
    is_final: bool,
    slot: SlotPtr,
) -> bool {
    let remainder = finalise_cap_contract(capability, is_final, false).0;
    &&& remainder.kind == CapKind::ZombieCap
    &&& 0 < spec_zombie_number_cap(remainder)
    &&& spec_zombie_ptr_cap(remainder) != slot
}

pub open spec fn finalise_cap_reduce_target_admissible(
    mgr: CSpaceManager,
    slot: SlotPtr,
    immediate: bool,
) -> bool
    recommends
        mgr.slot_dom().contains(slot),
        !mgr.slot_is_empty(slot),
{
    let remainder = finalise_cap_contract(
        mgr.get_cap(slot),
        mgr.spec_is_final_cap_at(slot),
        false,
    ).0;
    &&& immediate ==> mgr.slot_dom().contains(trusted_zombie_end_slot_of_cap(remainder))
    &&& !immediate ==> {
        let target = spec_zombie_ptr_cap(remainder);
        &&& mgr.slot_dom().contains(target)
        &&& !mgr.slot_is_empty(target)
    }
}

pub uninterp spec fn post_cap_deletion_preserves_visible_cspace(
    old: CSpaceManager,
    new: CSpaceManager,
    cleanup: CapSpec,
) -> bool;

pub uninterp spec fn preemption_point_preserves_manager(
    old: CSpaceManager,
    new: CSpaceManager,
    status: exception_t,
) -> bool;

pub open spec fn reduce_zombie_immediate_foreign_end_context(
    old_mgr: CSpaceManager,
    new_mgr: CSpaceManager,
    slot: SlotPtr,
) -> bool {
    &&& old_mgr.slot_dom().contains(slot)
    &&& old_mgr.get_cap(slot).kind == CapKind::ZombieCap
    &&& !spec_cap_removable(old_mgr.get_cap(slot), slot)
    &&& old_mgr.slot_dom().contains(trusted_zombie_end_slot_of_cap(old_mgr.get_cap(slot)))
    &&& trusted_zombie_end_slot_of_cap(old_mgr.get_cap(slot)) != slot
    &&& old_mgr.delete_all_success_slot_shape_post(
        &new_mgr,
        trusted_zombie_end_slot_of_cap(old_mgr.get_cap(slot)),
        false,
    )
}

// Current minimal semantic TCB for the immediate foreign-end branch of
// `reduce_zombie`. After `delete_all(end_slot, false)`, the verified manager proof
// already narrows the remaining black-box behavior to the two reference facts below:
// a changed non-zombie result at the original slot is impossible, and a changed zombie
// result must now point back to the original slot.
#[verifier::external_body]
pub proof fn lemma_reduce_zombie_immediate_foreign_end_delete_all_nonnull_nonzombie_impossible(
    old_mgr: CSpaceManager,
    new_mgr: CSpaceManager,
    slot: SlotPtr,
)
    requires
        reduce_zombie_immediate_foreign_end_context(old_mgr, new_mgr, slot),
        new_mgr.get_cap(slot).kind != CapKind::NullCap,
        new_mgr.get_cap(slot).kind != CapKind::ZombieCap,
    ensures
        false,
{
}

#[verifier::external_body]
pub proof fn lemma_reduce_zombie_immediate_foreign_end_delete_all_changed_zombie_projects_self_ptr(
    old_mgr: CSpaceManager,
    new_mgr: CSpaceManager,
    slot: SlotPtr,
)
    requires
        reduce_zombie_immediate_foreign_end_context(old_mgr, new_mgr, slot),
        new_mgr.get_cap(slot).kind == CapKind::ZombieCap,
        !spec_same_zombie_shape(new_mgr.get_cap(slot), old_mgr.get_cap(slot)),
    ensures
        spec_zombie_ptr_cap(new_mgr.get_cap(slot)) == slot,
{
}

#[verifier::external_body]
pub proof fn lemma_finalise_cap_immediate_projects_removable(
    capability: CapSpec,
    is_final: bool,
    slot: SlotPtr,
)
    ensures
        spec_cap_removable(finalise_cap_contract(capability, is_final, true).0, slot),
{
}

#[verifier::external_body]
pub proof fn lemma_finalise_cap_immediate_projects_cleanup_null(
    capability: CapSpec,
    is_final: bool,
)
    ensures
        finalise_cap_contract(capability, is_final, true).1.kind == CapKind::NullCap,
{
}

#[verifier::external_body]
pub proof fn lemma_finalise_cap_projects_delete_reply_arch_orphan_safe(
    mgr: CSpaceManager,
    slot: SlotPtr,
    exposed: bool,
)
    requires
        mgr.wf(),
        mgr.slot_dom().contains(slot),
        !mgr.slot_is_empty(slot),
        spec_cap_removable(
            finalise_cap_contract(mgr.get_cap(slot), mgr.spec_is_final_cap_at(slot), exposed).0,
            slot,
        ),
    ensures
        mgr.delete_reply_arch_orphan_safe(slot),
{
}

#[verifier::external_body]
pub proof fn lemma_finalise_cap_projects_delete_old_next_incoming_edges_admissible(
    mgr: CSpaceManager,
    slot: SlotPtr,
    exposed: bool,
)
    requires
        mgr.wf(),
        mgr.slot_dom().contains(slot),
        !mgr.slot_is_empty(slot),
        spec_cap_removable(
            finalise_cap_contract(mgr.get_cap(slot), mgr.spec_is_final_cap_at(slot), exposed).0,
            slot,
        ),
    ensures
        mgr.delete_old_next_incoming_edges_admissible(slot),
{
}

#[verifier::external_body]
pub proof fn lemma_finalise_cap_non_immediate_nonremovable_projects_reduce_ready(
    capability: CapSpec,
    is_final: bool,
    slot: SlotPtr,
)
    requires
        !spec_cap_removable(finalise_cap_contract(capability, is_final, false).0, slot),
    ensures
        finalise_cap_non_immediate_reduce_ready(capability, is_final, slot),
{
}

#[verifier::external_body]
pub proof fn lemma_finalise_cap_non_immediate_nonremovable_projects_reduce_target_admissible(
    mgr: CSpaceManager,
    slot: SlotPtr,
    immediate: bool,
)
    requires
        mgr.slot_dom().contains(slot),
        !mgr.slot_is_empty(slot),
        !spec_cap_removable(
            finalise_cap_contract(mgr.get_cap(slot), mgr.spec_is_final_cap_at(slot), false).0,
            slot,
        ),
    ensures
        finalise_cap_reduce_target_admissible(mgr, slot, immediate),
{
}

#[verifier::external_body]
pub proof fn lemma_finalise_cap_non_immediate_nonremovable_projects_affected_incoming_edges(
    old_mgr: CSpaceManager,
    new_mgr: CSpaceManager,
    slot: SlotPtr,
)
    requires
        old_mgr.wf(),
        old_mgr.slot_dom().contains(slot),
        !old_mgr.slot_is_empty(slot),
        old_mgr.finalise_slot_cap_write_rel(
            &new_mgr,
            slot,
            finalise_cap_contract(
                old_mgr.get_cap(slot),
                old_mgr.spec_is_final_cap_at(slot),
                false,
            ).0,
        ),
        !spec_cap_removable(
            finalise_cap_contract(
                old_mgr.get_cap(slot),
                old_mgr.spec_is_final_cap_at(slot),
                false,
            ).0,
            slot,
        ),
    ensures
        old_mgr.finalise_slot_cap_write_affected_incoming_edges_ok(&new_mgr, slot),
{
}

#[verifier::external_body]
pub proof fn lemma_finalise_cap_non_immediate_nonremovable_projects_affected_cdt_parent_semantics(
    old_mgr: CSpaceManager,
    new_mgr: CSpaceManager,
    slot: SlotPtr,
)
    requires
        old_mgr.wf(),
        old_mgr.slot_dom().contains(slot),
        !old_mgr.slot_is_empty(slot),
        old_mgr.finalise_slot_cap_write_rel(
            &new_mgr,
            slot,
            finalise_cap_contract(
                old_mgr.get_cap(slot),
                old_mgr.spec_is_final_cap_at(slot),
                false,
            ).0,
        ),
        !spec_cap_removable(
            finalise_cap_contract(
                old_mgr.get_cap(slot),
                old_mgr.spec_is_final_cap_at(slot),
                false,
            ).0,
            slot,
        ),
    ensures
        old_mgr.finalise_slot_cap_write_affected_cdt_parent_semantics_ok(&new_mgr, slot),
{
}

}
