//! Compatibility surface for legacy `crate::cte::*` paths.

pub use crate::cspace::cte::types::{cte_t, deriveCap_ret};
use crate::structures::resolveAddressBits_ret_t;
use sel4_common::structures_gen::cap;
use vstd::prelude::*;

verus! {

#[inline]
#[verifier::external_body]
pub fn cte_insert(new_cap: &cap, src_slot: &mut cte_t, dest_slot: &mut cte_t) {
    crate::cspace::kernel::cte_insert(new_cap, src_slot, dest_slot)
}

#[inline]
#[verifier::external_body]
pub fn insert_new_cap(parent: &mut cte_t, slot: &mut cte_t, capability: &cap) {
    crate::cspace::kernel::insert_new_cap(parent, slot, capability)
}

#[inline]
#[verifier::external_body]
pub fn cte_move(new_cap: &cap, src_slot: &mut cte_t, dest_slot: &mut cte_t) {
    crate::cspace::kernel::cte_move(new_cap, src_slot, dest_slot)
}

#[inline]
#[verifier::external_body]
pub fn cte_swap(cap1: &cap, slot1: &mut cte_t, cap2: &cap, slot2: &mut cte_t) {
    crate::cspace::kernel::cte_swap(cap1, slot1, cap2, slot2)
}

#[inline]
pub fn resolve_address_bits(
    node_cap: &cap,
    cap_ptr: usize,
    n_bits: usize,
) -> resolveAddressBits_ret_t {
    crate::cspace::resolve::resolve_address_bits(node_cap, cap_ptr, n_bits)
}

}
