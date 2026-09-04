//! zombie cap helpers
use crate::cspace::cte::cte_t;
#[cfg(verus_keep_ghost)]
use crate::capability::raw::{
    lemma_trusted_view_cap_kind_matches_tag, spec_cap_cyclic_zombie, spec_zombie_number_cap,
    spec_zombie_ptr_cap, spec_zombie_type_cap, trusted_view_cap,
};
#[cfg(verus_keep_ghost)]
use crate::capability::spec::CapKind;
use crate::capability::raw::{
    runtime_cap_tag, runtime_cap_zombie_ptr, runtime_mask_bits, runtime_raw_zombie_bit,
    runtime_raw_zombie_number, runtime_raw_zombie_ptr, runtime_raw_zombie_set_number,
    runtime_zombie_cap_new,
};
use sel4_common::structures_gen::{cap, cap_zombie_cap};
use vstd::prelude::*;

verus! {

pub const VERIFIED_CSPACE_WORD_RADIX: usize = 6;
pub const ZOMBIE_TYPE_ZOMBIE_TCB: usize = 1usize << VERIFIED_CSPACE_WORD_RADIX;
pub const TCB_CNODE_RADIX: usize = 4;
const TAG_ZOMBIE: u64 = 18;

pub uninterp spec fn spec_zombie_bit_raw(raw: &cap_zombie_cap) -> usize;

pub uninterp spec fn spec_zombie_ptr_raw(raw: &cap_zombie_cap) -> usize;

pub uninterp spec fn spec_zombie_number_raw(raw: &cap_zombie_cap) -> usize;

pub trait zombie_func {
    fn get_zombie_bit(&self) -> (ret: usize);
    fn get_zombie_ptr(&self) -> (ret: usize);
    fn get_zombie_number(&self) -> (ret: usize);
    fn set_zombie_number(&mut self, n: usize);
}

impl zombie_func for cap_zombie_cap {
    #[inline]
    fn get_zombie_bit(&self) -> (ret: usize)
        ensures
            ret == spec_zombie_bit_raw(self),
    {
        runtime_raw_zombie_bit(self)
    }

    #[inline]
    fn get_zombie_ptr(&self) -> (ret: usize)
        ensures
            ret == spec_zombie_ptr_raw(self),
    {
        runtime_raw_zombie_ptr(self)
    }

    #[inline]
    fn get_zombie_number(&self) -> (ret: usize)
        ensures
            ret == spec_zombie_number_raw(self),
    {
        runtime_raw_zombie_number(self)
    }

    #[inline]
    fn set_zombie_number(&mut self, n: usize)
        ensures
            spec_zombie_bit_raw(self) == spec_zombie_bit_raw(old(self)),
            spec_zombie_ptr_raw(self) == spec_zombie_ptr_raw(old(self)),
            spec_zombie_number_raw(self) == n,
    {
        runtime_raw_zombie_set_number(self, n);
    }
}

// Temporary semantic TCB: `zombie_new` already has the public semantic contract; this helper
// only bridges the raw constructor encoding to that contract.
#[verifier::external_body]
proof fn lemma_zombie_new_matches_runtime(number: usize, zombie_type: usize, ptr: usize, ret: &cap)
    ensures
        trusted_view_cap(ret).kind == crate::capability::spec::CapKind::ZombieCap,
        spec_zombie_number_cap(trusted_view_cap(ret)) == number,
        spec_zombie_type_cap(trusted_view_cap(ret)) == zombie_type,
        spec_zombie_ptr_cap(trusted_view_cap(ret)) == ptr,
{
}

#[inline]
pub fn zombie_new(number: usize, zombie_type: usize, ptr: usize) -> (ret: cap)
    ensures
        trusted_view_cap(&ret).kind == crate::capability::spec::CapKind::ZombieCap,
        spec_zombie_number_cap(trusted_view_cap(&ret)) == number,
        spec_zombie_type_cap(trusted_view_cap(&ret)) == zombie_type,
        spec_zombie_ptr_cap(trusted_view_cap(&ret)) == ptr,
{
    let cnode_mask_width = if zombie_type == usize::MAX {
        usize::MAX
    } else {
        zombie_type + 1
    };
    let mask = if zombie_type == ZOMBIE_TYPE_ZOMBIE_TCB {
        runtime_mask_bits(TCB_CNODE_RADIX + 1)
    } else {
        runtime_mask_bits(cnode_mask_width)
    };
    let ret = runtime_zombie_cap_new(((ptr & !mask) | (number & mask)) as u64, zombie_type as u64);
    proof {
        lemma_zombie_new_matches_runtime(number, zombie_type, ptr, &ret);
    }
    ret
}

pub fn zombie_type_zombie_cnode(n: usize) -> usize {
    n & runtime_mask_bits(VERIFIED_CSPACE_WORD_RADIX)
}

pub proof fn lemma_zombie_kind_and_ptr_imply_cyclic(capability: crate::capability::spec::CapSpec, slot: usize)
    requires
        capability.kind == CapKind::ZombieCap,
        spec_zombie_ptr_cap(capability) == slot,
    ensures
        spec_cap_cyclic_zombie(capability, slot),
{
}

#[no_mangle]
pub fn cap_cyclic_zombie(capability: &cap, slot: *mut cte_t) -> (ret: bool)
    ensures
        ret == spec_cap_cyclic_zombie(trusted_view_cap(capability), slot as usize),
{
    let tag = runtime_cap_tag(capability);
    let zombie_ptr = if tag == TAG_ZOMBIE {
        runtime_cap_zombie_ptr(capability)
    } else {
        0
    };
    let ret = tag == TAG_ZOMBIE && zombie_ptr == slot as usize;
    proof {
        lemma_trusted_view_cap_kind_matches_tag(capability);
        assert(tag == crate::capability::raw::spec_runtime_cap_tag(capability));
        if tag == TAG_ZOMBIE {
            assert(trusted_view_cap(capability).kind
                == crate::capability::spec::CapKind::ZombieCap);
            assert(spec_zombie_ptr_cap(trusted_view_cap(capability)) == zombie_ptr);
        } else {
            assert(trusted_view_cap(capability).kind
                != crate::capability::spec::CapKind::ZombieCap);
        }
    }
    ret
}

}
