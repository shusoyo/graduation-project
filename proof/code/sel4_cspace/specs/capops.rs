use vstd::prelude::*;

use crate::invariants::*;
use crate::model::*;

verus! {

pub enum CSpaceOpError {
    IllegalOperation,
    DeleteFirst,
    FailedOnDeriveCap,
}

pub open spec fn cspace_copy_pre(
    s: AbsState,
    src_slot: SlotId,
    dest_slot: SlotId,
    derived_cap: Capability,
) -> bool {
    &&& wf_cspace(s)
    &&& slot_has_cap(s, src_slot)
    &&& slot_is_free(s, dest_slot)
    &&& derived_cap != Capability::NullCap
}

pub open spec fn cspace_copy_post(
    old: AbsState,
    new: AbsState,
    src_slot: SlotId,
    dest_slot: SlotId,
    result: Result<(), CSpaceOpError>,
    derived_cap: Capability,
) -> bool {
    match result {
        Ok(()) => {
            let src_cte = get_cte(old, src_slot);
            let next = src_cte.mdb_node.next;
            let revocable = is_cap_revocable(derived_cap, src_cte.capability);
            &&& wf_cspace(new)
            &&& get_cap(new, dest_slot) == derived_cap
            &&& get_cte(new, dest_slot).mdb_node.prev == src_slot
            &&& get_cte(new, dest_slot).mdb_node.next == next
            &&& get_cte(new, dest_slot).mdb_node.revocable == revocable
            &&& get_cte(new, dest_slot).mdb_node.first_badged == revocable
        }
        Err(_) => new == old,
    }
}

pub open spec fn cspace_mint_pre(
    s: AbsState,
    src_slot: SlotId,
    dest_slot: SlotId,
    minted_cap: Capability,
) -> bool {
    &&& wf_cspace(s)
    &&& slot_has_cap(s, src_slot)
    &&& slot_is_free(s, dest_slot)
    &&& minted_cap != Capability::NullCap
}

pub open spec fn cspace_mint_post(
    old: AbsState,
    new: AbsState,
    src_slot: SlotId,
    dest_slot: SlotId,
    result: Result<(), CSpaceOpError>,
    minted_cap: Capability,
) -> bool {
    cspace_copy_post(old, new, src_slot, dest_slot, result, minted_cap)
}

pub open spec fn cspace_mutate_pre(
    s: AbsState,
    src_slot: SlotId,
    dest_slot: SlotId,
    new_cap: Capability,
) -> bool {
    &&& wf_cspace(s)
    &&& slot_has_cap(s, src_slot)
    &&& slot_is_free(s, dest_slot)
    &&& new_cap != Capability::NullCap
}

pub open spec fn cspace_mutate_post(
    old: AbsState,
    new: AbsState,
    src_slot: SlotId,
    dest_slot: SlotId,
    result: Result<(), CSpaceOpError>,
    new_cap: Capability,
) -> bool {
    match result {
        Ok(()) => {
            let mdb = get_cte(old, src_slot).mdb_node;
            &&& wf_cspace(new)
            &&& get_cap(new, dest_slot) == new_cap
            &&& get_cte(new, dest_slot).mdb_node == mdb
            &&& get_cte(new, src_slot) == null_cte()
        }
        Err(_) => new == old,
    }
}

pub open spec fn cspace_rotate_pre(
    s: AbsState,
    slot1: SlotId,
    slot2: SlotId,
    slot3: SlotId,
    new_src_cap: Capability,
    new_pivot_cap: Capability,
) -> bool {
    &&& wf_cspace(s)
    &&& slot_has_cap(s, slot1)
    &&& slot_has_cap(s, slot2)
    &&& new_src_cap != Capability::NullCap
    &&& new_pivot_cap != Capability::NullCap
    &&& (slot1 == slot3 || slot_is_free(s, slot3))
}

pub open spec fn cspace_rotate_post(
    old: AbsState,
    new: AbsState,
    slot1: SlotId,
    slot2: SlotId,
    slot3: SlotId,
    result: Result<(), CSpaceOpError>,
    new_src_cap: Capability,
    new_pivot_cap: Capability,
) -> bool {
    match result {
        Ok(()) =>
            if slot1 == slot3 {
                let mdb1 = get_cte(old, slot1).mdb_node;
                let mdb2 = get_cte(old, slot2).mdb_node;
                &&& wf_cspace(new)
                &&& get_cap(new, slot1) == new_pivot_cap
                &&& get_cap(new, slot2) == new_src_cap
                &&& get_cte(new, slot1).mdb_node == mdb2
                &&& get_cte(new, slot2).mdb_node == mdb1
            } else {
                &&& wf_cspace(new)
                &&& get_cap(new, slot3) == new_pivot_cap
                &&& get_cap(new, slot2) == new_src_cap
            },
        Err(_) => new == old,
    }
}

pub open spec fn cspace_move_pre(s: AbsState, src_slot: SlotId, dest_slot: SlotId) -> bool {
    &&& wf_cspace(s)
    &&& slot_has_cap(s, src_slot)
    &&& slot_is_free(s, dest_slot)
}

pub open spec fn cspace_move_post(
    old: AbsState,
    new: AbsState,
    src_slot: SlotId,
    dest_slot: SlotId,
    result: Result<(), CSpaceOpError>,
) -> bool {
    match result {
        Ok(()) => {
            let moved_cap = get_cap(old, src_slot);
            let mdb = get_cte(old, src_slot).mdb_node;
            &&& wf_cspace(new)
            &&& get_cap(new, dest_slot) == moved_cap
            &&& get_cte(new, dest_slot).mdb_node == mdb
            &&& get_cte(new, src_slot) == null_cte()
        }
        Err(_) => new == old,
    }
}

pub open spec fn cspace_revoke_pre(s: AbsState, slot: SlotId) -> bool {
    &&& wf_cspace(s)
    &&& slot_has_cap(s, slot)
}

pub open spec fn cspace_revoke_post(old: AbsState, new: AbsState, slot: SlotId) -> bool {
    let next = get_cte(old, slot).mdb_node.next;
    &&& wf_cspace(new)
    &&& (next != null_slot() && slot_exists(old, next) && is_mdb_parent_of(get_cte(old, slot), get_cte(old, next))
            ==> get_cap(new, next) == Capability::NullCap)
}

pub open spec fn cspace_delete_pre(s: AbsState, slot: SlotId) -> bool {
    &&& wf_cspace(s)
    &&& slot_exists(s, slot)
}

pub open spec fn cspace_delete_post(old: AbsState, new: AbsState, slot: SlotId) -> bool {
    &&& wf_cspace(new)
    &&& slot_exists(old, slot)
    &&& get_cte(new, slot) == null_cte()
}

}
