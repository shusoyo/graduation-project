use crate::cspace::cte::cte_t;
#[cfg(verus_keep_ghost)]
use crate::cspace::cte::spec::{
    same_entry_except_untyped_free_index, same_mdb_fields, slot_cap_update_rel,
};
use crate::cspace::types::SlotPtr;
use sel4_common::structures_gen::{cap, cap_tag};
use sel4_common::utils::{convert_to_mut_type_ref, max_free_index};
use vstd::prelude::*;
use vstd::simple_pptr;

verus! {

#[verifier::external_body]
pub fn write_slot_cap_only_tracked(
    slot: SlotPtr,
    Tracked(slot_perm): Tracked<&mut simple_pptr::PointsTo<cte_t>>,
    capability: &cap,
)
    requires
        old(slot_perm).is_init(),
        old(slot_perm).addr() == slot,
    ensures
        slot_perm.is_init(),
        slot_perm.addr() == slot,
        slot_cap_update_rel(
            crate::cspace::cte::raw::trusted_slot_perm_view(*old(slot_perm)),
            crate::cspace::cte::raw::trusted_slot_perm_view(*slot_perm),
            crate::capability::raw::trusted_view_cap(capability),
        ),
        same_mdb_fields(
            crate::cspace::cte::raw::trusted_slot_perm_view(*old(slot_perm)),
            crate::cspace::cte::raw::trusted_slot_perm_view(*slot_perm),
        ),
        crate::cspace::cte::raw::trusted_slot_perm_view(*slot_perm).cap
            == crate::capability::raw::trusted_view_cap(capability),
{
    convert_to_mut_type_ref::<cte_t>(slot).capability = capability.clone();
}

#[verifier::external_body]
pub fn set_untyped_cap_as_full_tracked(
    slot: SlotPtr,
    Tracked(slot_perm): Tracked<&mut simple_pptr::PointsTo<cte_t>>,
    src_cap: &cap,
    new_cap: &cap,
)
    requires
        old(slot_perm).is_init(),
        old(slot_perm).addr() == slot,
    ensures
        slot_perm.is_init(),
        slot_perm.addr() == slot,
        same_entry_except_untyped_free_index(
            crate::cspace::cte::raw::trusted_slot_perm_view(*old(slot_perm)),
            crate::cspace::cte::raw::trusted_slot_perm_view(*slot_perm),
        ),
{
    if src_cap.get_tag() == cap_tag::cap_untyped_cap
        && new_cap.get_tag() == cap_tag::cap_untyped_cap
    {
        let slot_ref = convert_to_mut_type_ref::<cte_t>(slot);
        if cap::cap_untyped_cap(src_cap).get_capPtr()
            == cap::cap_untyped_cap(new_cap).get_capPtr()
            && cap::cap_untyped_cap(src_cap).get_capBlockSize()
                == cap::cap_untyped_cap(new_cap).get_capBlockSize()
        {
            cap::cap_untyped_cap(&slot_ref.capability).set_capFreeIndex(max_free_index(
                cap::cap_untyped_cap(src_cap).get_capBlockSize() as usize,
            ) as u64);
        }
    }
}

}
