use crate::cspace::cte::cte_t;
#[cfg(verus_keep_ghost)]
use crate::capability::raw::trusted_view_cap;
#[cfg(verus_keep_ghost)]
use crate::capability::spec::{CapKind, CapSpec, spec_extract_bits};
use sel4_common::structures_gen::{cap, cap_tag};
use vstd::prelude::*;
#[cfg(verus_keep_ghost)]
use vstd::simple_pptr;

verus! {

pub uninterp spec fn trusted_cnode_lookup_slot_ptr(
    node_cap: CapSpec,
    offset: int,
) -> simple_pptr::PPtr<cte_t>;

    #[verifier::external_body]
    pub fn runtime_cap_is_cnode(raw: &cap) -> (ret: bool)
    ensures
        ret == (trusted_view_cap(raw).kind == CapKind::CNodeCap),
{
    raw.get_tag() == cap_tag::cap_cnode_cap
}

#[verifier::external_body]
pub fn runtime_cnode_lookup_slot_from_cap(
    node_cap: &cap,
    offset: usize,
) -> (ret: *mut cte_t)
    ensures
        ret as usize == trusted_cnode_lookup_slot_ptr(trusted_view_cap(node_cap), offset as int).addr(),
{
    unsafe { (cap::cap_cnode_cap(node_cap).get_capCNodePtr() as *mut cte_t).add(offset) }
}

#[verifier::external_body]
pub fn runtime_slot_cap_clone(slot: *mut cte_t) -> (ret: cap)
    ensures
        trusted_view_cap(&ret) == crate::cspace::cte::spec::cte_slot_view_at(slot as usize).cap,
{
    unsafe { (*slot).capability.clone() }
}

    #[verifier::external_body]
    pub fn runtime_cnode_cap_radix_bits(raw: &cap) -> (ret: usize)
    ensures
        trusted_view_cap(raw).kind == CapKind::CNodeCap
            ==> trusted_view_cap(raw).cnode is Some
                && ret as int == trusted_view_cap(raw).cnode.unwrap().radix_bits,
{
    cap::cap_cnode_cap(raw).get_capCNodeRadix() as usize
}

    #[verifier::external_body]
    pub fn runtime_cnode_cap_guard_bits(raw: &cap) -> (ret: usize)
    ensures
        trusted_view_cap(raw).kind == CapKind::CNodeCap
            ==> trusted_view_cap(raw).cnode is Some
                && ret as int == trusted_view_cap(raw).cnode.unwrap().guard_size,
{
    cap::cap_cnode_cap(raw).get_capCNodeGuardSize() as usize
}

    #[verifier::external_body]
    pub fn runtime_cnode_cap_guard(raw: &cap) -> (ret: usize)
    ensures
        trusted_view_cap(raw).kind == CapKind::CNodeCap
            ==> trusted_view_cap(raw).cnode is Some
                && ret as int == trusted_view_cap(raw).cnode.unwrap().guard,
{
    cap::cap_cnode_cap(raw).get_capCNodeGuard() as usize
}

    #[verifier::external_body]
    pub fn runtime_cnode_cap_level_bits(raw: &cap) -> (ret: usize)
    ensures
        trusted_view_cap(raw).kind == CapKind::CNodeCap
            ==> trusted_view_cap(raw).cnode is Some
                && ret as int
                    == trusted_view_cap(raw).cnode.unwrap().radix_bits
                        + trusted_view_cap(raw).cnode.unwrap().guard_size,
{
    let cnode_cap = cap::cap_cnode_cap(raw);
    cnode_cap.get_capCNodeRadix() as usize + cnode_cap.get_capCNodeGuardSize() as usize
}

    #[verifier::external_body]
    pub fn runtime_extract_bits_usize(
        value: usize,
        start: usize,
        width: usize,
    ) -> (ret: usize)
    ensures
        ret as int == spec_extract_bits(value as int, start as int, width as int),
{
    (value >> start) & mask_bits!(width)
}

}
