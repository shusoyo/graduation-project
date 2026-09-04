use vstd::prelude::*;

use sel4_cspace_specs::model::*;
use sel4_cspace_specs::primitives::*;

verus! {

pub open spec fn derive_cap_proof_obligation(s: AbsState, slot: SlotId) -> bool {
    derive_cap_pre(s, slot)
        ==> derive_cap_post(s, s, slot, derive_cap_impl(s, slot))
}

pub proof fn lemma_derive_cap_spec_sound(s: AbsState, slot: SlotId)
    ensures
        derive_cap_proof_obligation(s, slot),
{
    if derive_cap_pre(s, slot) {
        derive_cap_impl_correct(s, slot);
    }
}

}
