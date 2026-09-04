use crate::cspace::cte::cte_t;
#[cfg(verus_keep_ghost)]
use crate::cspace::cte::raw::trusted_view_cte;
#[cfg(verus_keep_ghost)]
use crate::cspace::cte::spec::{
    spec_slot_entry_with_contents, spec_slot_entry_with_next, spec_slot_entry_with_prev,
};
use crate::cspace::types::SlotPtr;
use sel4_common::structures_gen::mdb_node;
use sel4_common::utils::convert_to_mut_type_ref;
use vstd::prelude::*;
use vstd::simple_pptr;

verus! {

#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExMdbNode(mdb_node);

#[verifier::external_body]
pub fn runtime_slot_mdb_next(raw: &cte_t) -> (ret: usize)
    ensures
        trusted_view_cte(raw).mdb_next is Some ==> ret == trusted_view_cte(raw).mdb_next.unwrap(),
        trusted_view_cte(raw).mdb_next is None ==> ret == 0,
        ret == 0 ==> trusted_view_cte(raw).mdb_next is None,
        ret != 0 ==> trusted_view_cte(raw).mdb_next == Some(ret),
{
    raw.cteMDBNode.get_mdbNext() as usize
}

#[verifier::external_body]
pub fn runtime_slot_mdb_prev(raw: &cte_t) -> (ret: usize)
    ensures
        trusted_view_cte(raw).mdb_prev is Some ==> ret == trusted_view_cte(raw).mdb_prev.unwrap(),
        trusted_view_cte(raw).mdb_prev is None ==> ret == 0,
        ret == 0 ==> trusted_view_cte(raw).mdb_prev is None,
        ret != 0 ==> trusted_view_cte(raw).mdb_prev == Some(ret),
{
    raw.cteMDBNode.get_mdbPrev() as usize
}

#[verifier::external_body]
pub fn runtime_slot_mdb_revocable(raw: &cte_t) -> (ret: bool)
    ensures
        ret == trusted_view_cte(raw).mdb_revocable,
{
    raw.cteMDBNode.get_mdbRevocable() != 0
}

#[verifier::external_body]
pub fn runtime_slot_mdb_first_badged(raw: &cte_t) -> (ret: bool)
    ensures
        ret == trusted_view_cte(raw).mdb_first_badged,
{
    raw.cteMDBNode.get_mdbFirstBadged() != 0
}

#[verifier::external_body]
pub fn runtime_slot_mdb_first_badged_ptr(slot: *mut cte_t) -> (ret: bool)
    ensures
        ret == crate::cspace::cte::spec::cte_slot_view_at(slot as usize).mdb_first_badged,
{
    unsafe { (*slot).cteMDBNode.get_mdbFirstBadged() != 0 }
}

#[verifier::external_body]
pub fn write_mdb_node_tracked(
    slot: SlotPtr,
    Tracked(slot_perm): Tracked<&mut simple_pptr::PointsTo<cte_t>>,
    next: Option<SlotPtr>,
    revocable: bool,
    first_badged: bool,
    prev: Option<SlotPtr>,
)
    requires
        old(slot_perm).is_init(),
        old(slot_perm).addr() == slot,
    ensures
        slot_perm.is_init(),
        slot_perm.addr() == slot,
        crate::cspace::cte::raw::trusted_slot_perm_view(*slot_perm)
            == spec_slot_entry_with_contents(
                crate::cspace::cte::raw::trusted_slot_perm_view(*old(slot_perm)).cap,
                prev,
                next,
                revocable,
                first_badged,
            ),
{
    let next_raw = match next {
        Some(v) => v as u64,
        None => 0,
    };
    let prev_raw = match prev {
        Some(v) => v as u64,
        None => 0,
    };
    convert_to_mut_type_ref::<cte_t>(slot).cteMDBNode = mdb_node::new(
        next_raw,
        revocable as u64,
        first_badged as u64,
        prev_raw,
    );
}

#[verifier::external_body]
pub fn write_mdb_next_tracked(
    slot: SlotPtr,
    Tracked(slot_perm): Tracked<&mut simple_pptr::PointsTo<cte_t>>,
    next: Option<SlotPtr>,
)
    requires
        old(slot_perm).is_init(),
        old(slot_perm).addr() == slot,
    ensures
        slot_perm.is_init(),
        slot_perm.addr() == slot,
        crate::cspace::cte::raw::trusted_slot_perm_view(*slot_perm)
            == spec_slot_entry_with_next(
                crate::cspace::cte::raw::trusted_slot_perm_view(*old(slot_perm)),
                next,
            ),
{
    let next_raw = match next {
        Some(v) => v as u64,
        None => 0,
    };
    convert_to_mut_type_ref::<cte_t>(slot).cteMDBNode.set_mdbNext(next_raw);
}

#[verifier::external_body]
pub fn write_mdb_prev_tracked(
    slot: SlotPtr,
    Tracked(slot_perm): Tracked<&mut simple_pptr::PointsTo<cte_t>>,
    prev: Option<SlotPtr>,
)
    requires
        old(slot_perm).is_init(),
        old(slot_perm).addr() == slot,
    ensures
        slot_perm.is_init(),
        slot_perm.addr() == slot,
        crate::cspace::cte::raw::trusted_slot_perm_view(*slot_perm)
            == spec_slot_entry_with_prev(
                crate::cspace::cte::raw::trusted_slot_perm_view(*old(slot_perm)),
                prev,
            ),
{
    let prev_raw = match prev {
        Some(v) => v as u64,
        None => 0,
    };
    convert_to_mut_type_ref::<cte_t>(slot).cteMDBNode.set_mdbPrev(prev_raw);
}

}
