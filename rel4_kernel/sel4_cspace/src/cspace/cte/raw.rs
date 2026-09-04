#[cfg(verus_keep_ghost)]
use crate::capability::raw::trusted_view_cap;
use crate::cspace::cte::cte_t;
#[cfg(verus_keep_ghost)]
use crate::cspace::cte::spec::{cte_slot_ptr, cte_slot_view_at, SlotEntrySpec};
use sel4_common::utils::convert_to_type_ref;
use vstd::prelude::*;
#[cfg(verus_keep_ghost)]
use vstd::simple_pptr;

verus! {

pub uninterp spec fn trusted_view_cte(raw: &cte_t) -> SlotEntrySpec;

pub uninterp spec fn trusted_slot_perm_view(
    perm: simple_pptr::PointsTo<cte_t>,
) -> SlotEntrySpec;

#[verifier::external_body]
pub proof fn lemma_trusted_view_cte_matches_slot_perm_view(
    raw: &cte_t,
    perm: simple_pptr::PointsTo<cte_t>,
)
    requires
        perm.is_init(),
        &perm.value() == raw,
    ensures
        trusted_view_cte(raw) == trusted_slot_perm_view(perm),
{
}

#[verifier::external_body]
pub proof fn lemma_trusted_view_cte_matches_slot_perm_view_ref(
    raw: &cte_t,
    perm: &simple_pptr::PointsTo<cte_t>,
)
    requires
        perm.is_init(),
        &perm.value() == raw,
    ensures
        trusted_view_cte(raw) == trusted_slot_perm_view(*perm),
{
}

#[verifier::external_body]
pub proof fn lemma_trusted_view_cte_cap_matches_cap_field(raw: &cte_t)
    ensures
        trusted_view_cte(raw).cap == trusted_view_cap(&raw.capability),
{
}

#[cfg(verus_keep_ghost)]
#[verifier::external_body]
pub proof fn lemma_cte_slot_view_at_ptr_matches_trusted_view(raw: &cte_t)
    ensures
        cte_slot_view_at(cte_slot_ptr(raw)) == trusted_view_cte(raw),
{
}

#[verifier::external_body]
pub fn runtime_slot_ref_at(slot: usize) -> (ret: &'static cte_t)
    requires
        slot != 0,
    ensures
        cte_slot_ptr(ret) == slot,
        cte_slot_view_at(slot) == trusted_view_cte(ret),
{
    convert_to_type_ref::<cte_t>(slot)
}

}
