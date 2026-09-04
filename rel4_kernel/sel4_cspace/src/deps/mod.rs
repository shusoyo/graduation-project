pub mod raw;

use crate::structures::FinaliseCapRet;
use sel4_common::{structures::exception_t, structures_gen::cap};
use vstd::prelude::*;

extern "C" {
    #[link_name = "finalise_cap"]
    fn finalise_cap_raw(capability: &cap, _final: bool, _exposed: bool) -> FinaliseCapRet;

    #[link_name = "post_cap_deletion"]
    fn post_cap_deletion_raw(capability: &cap);

    #[link_name = "preemption_point"]
    fn preemption_point_raw() -> exception_t;
}

verus! {

// Temporary semantic TCB: delete-side reasoning consumes the abstract
// `finalise_cap_contract(...)` rather than the raw extern behavior directly.
#[verifier::external_body]
pub exec fn finalise_cap(capability: &cap, is_final: bool, exposed: bool) -> (ret: FinaliseCapRet)
    ensures
        crate::capability::raw::trusted_view_cap(&ret.remainder)
            == crate::deps::raw::finalise_cap_contract(
                crate::capability::raw::trusted_view_cap(capability),
                is_final,
                exposed,
            ).0,
        crate::capability::raw::trusted_view_cap(&ret.cleanupInfo)
            == crate::deps::raw::finalise_cap_contract(
                crate::capability::raw::trusted_view_cap(capability),
                is_final,
                exposed,
            ).1,
{
    unsafe { finalise_cap_raw(capability, is_final, exposed) }
}

// Raw dependency bridge: callers should consume the manager-level wrapper
// contract (`post_cap_deletion_bridge`) rather than depending on this raw hook.
#[verifier::external_body]
pub exec fn post_cap_deletion(capability: &cap)
{
    unsafe { post_cap_deletion_raw(capability) }
}

// Raw dependency bridge: callers should consume the manager-level wrapper
// contract (`preemption_point_bridge`) rather than depending on this raw hook.
#[verifier::external_body]
pub exec fn preemption_point() -> (ret: exception_t)
{
    unsafe { preemption_point_raw() }
}

}
