pub mod exec;
pub mod raw;

pub use exec::resolve_address_bits;

#[cfg(verus_keep_ghost)]
use crate::capability::spec::spec_extract_bits;
use crate::capability::spec::{CapKind, CapSpec};
use crate::cspace::types::SlotPtr;
use vstd::prelude::*;

verus! {

#[cfg(verus_keep_ghost)]
use crate::cspace::cte::spec::cte_slot_view_at;
#[cfg(verus_keep_ghost)]
use crate::structures::resolveAddressBits_ret_t;
#[cfg(verus_keep_ghost)]
use crate::cspace::resolve::raw::trusted_cnode_lookup_slot_ptr;
#[cfg(verus_keep_ghost)]
use crate::kernel_api::raw::{is_exception_lookup_fault, is_exception_none};

pub ghost enum ResolveAddressBitsStatusSpec {
    Success,
    LookupFault,
}

pub ghost enum ResolveAddressBitsFaultSpec {
    InvalidRoot,
    GuardTooDeep {
        bits_left: int,
        guard_bits: int,
    },
    GuardMismatch {
        bits_left: int,
        guard_found: int,
        guard_expected: int,
        guard_bits: int,
    },
    MalformedLevel {
        bits_left: int,
    },
    LevelTooDeep {
        bits_left: int,
        level_bits: int,
    },
}

#[verifier::ext_equal]
pub ghost struct ResolveAddressBitsResultSpec {
    pub status: ResolveAddressBitsStatusSpec,
    pub slot: Option<SlotPtr>,
    pub bits_remaining: int,
    pub fault: Option<ResolveAddressBitsFaultSpec>,
}

pub open spec fn resolve_cnode_level_bits(cap: CapSpec) -> int
    recommends
        cap.kind == CapKind::CNodeCap,
        cap.cnode is Some,
{
    cap.cnode.unwrap().guard_size + cap.cnode.unwrap().radix_bits
}

pub open spec fn resolve_guard_too_deep(cap: CapSpec, bits: int) -> bool
    recommends
        cap.kind == CapKind::CNodeCap,
        cap.cnode is Some,
{
    bits < cap.cnode.unwrap().guard_size
}

pub open spec fn resolve_guard_value(cap: CapSpec, cap_ptr: int, bits: int) -> int
    recommends
        cap.kind == CapKind::CNodeCap,
        cap.cnode is Some,
        0 <= cap_ptr,
        0 <= bits,
        !resolve_guard_too_deep(cap, bits),
{
    let guard_bits = cap.cnode.unwrap().guard_size;
    spec_extract_bits(cap_ptr, bits - guard_bits, guard_bits)
}

pub open spec fn resolve_guard_matches(cap: CapSpec, cap_ptr: int, bits: int) -> bool
    recommends
        cap.kind == CapKind::CNodeCap,
        cap.cnode is Some,
        0 <= cap_ptr,
        0 <= bits,
{
    !resolve_guard_too_deep(cap, bits)
        && resolve_guard_value(cap, cap_ptr, bits) == cap.cnode.unwrap().guard
}

pub open spec fn resolve_level_invalid(cap: CapSpec, bits: int) -> bool
    recommends
        cap.kind == CapKind::CNodeCap,
        cap.cnode is Some,
{
    let level_bits = resolve_cnode_level_bits(cap);
    level_bits == 0 || bits < level_bits
}

pub open spec fn resolve_step_offset(cap: CapSpec, cap_ptr: int, bits: int) -> int
    recommends
        cap.kind == CapKind::CNodeCap,
        cap.cnode is Some,
        0 <= cap_ptr,
        0 <= bits,
        resolve_guard_matches(cap, cap_ptr, bits),
        !resolve_level_invalid(cap, bits),
{
    spec_extract_bits(
        cap_ptr,
        bits - resolve_cnode_level_bits(cap),
        cap.cnode.unwrap().radix_bits,
    )
}

pub open spec fn resolve_invalid_root_case(root_cap: CapSpec) -> bool {
    root_cap.kind != CapKind::CNodeCap
}

pub open spec fn resolve_guard_too_deep_case(root_cap: CapSpec, bits: int) -> bool {
    root_cap.kind == CapKind::CNodeCap
        && root_cap.cnode is Some
        && resolve_guard_too_deep(root_cap, bits)
}

pub open spec fn resolve_guard_mismatch_case(root_cap: CapSpec, cap_ptr: int, bits: int) -> bool {
    0 <= cap_ptr
        && 0 <= bits
        && root_cap.kind == CapKind::CNodeCap
        && root_cap.cnode is Some
        && !resolve_guard_too_deep(root_cap, bits)
        && !resolve_guard_matches(root_cap, cap_ptr, bits)
}

pub open spec fn resolve_level_invalid_case(root_cap: CapSpec, cap_ptr: int, bits: int) -> bool {
    0 <= cap_ptr
        && 0 <= bits
        && root_cap.kind == CapKind::CNodeCap
        && root_cap.cnode is Some
        && resolve_guard_matches(root_cap, cap_ptr, bits)
        && resolve_level_invalid(root_cap, bits)
}

pub open spec fn resolve_step_offset_case(root_cap: CapSpec, cap_ptr: int, bits: int) -> bool {
    0 <= cap_ptr
        && 0 <= bits
        && root_cap.kind == CapKind::CNodeCap
        && root_cap.cnode is Some
        && 0 <= root_cap.cnode.unwrap().guard_size
        && 0 <= root_cap.cnode.unwrap().radix_bits
        && resolve_guard_matches(root_cap, cap_ptr, bits)
        && !resolve_level_invalid(root_cap, bits)
}

pub open spec fn resolve_exact_success_case(root_cap: CapSpec, cap_ptr: int, bits: int) -> bool {
    resolve_step_offset_case(root_cap, cap_ptr, bits) && bits == resolve_cnode_level_bits(root_cap)
}

pub open spec fn resolve_descend_case(root_cap: CapSpec, cap_ptr: int, bits: int) -> bool {
    resolve_step_offset_case(root_cap, cap_ptr, bits) && resolve_cnode_level_bits(root_cap) < bits
}

pub open spec fn resolve_bits_after_root_step(root_cap: CapSpec, bits: int) -> int
    recommends
        root_cap.kind == CapKind::CNodeCap,
        root_cap.cnode is Some,
        !resolve_level_invalid(root_cap, bits),
{
    bits - resolve_cnode_level_bits(root_cap)
}

pub open spec fn resolve_root_step_slot(root_cap: CapSpec, cap_ptr: int, bits: int) -> SlotPtr
    recommends
        resolve_step_offset_case(root_cap, cap_ptr, bits),
{
    resolve_root_step_slot_from_offset(root_cap, resolve_step_offset(root_cap, cap_ptr, bits))
}

pub open spec fn resolve_root_step_slot_from_offset(root_cap: CapSpec, offset: int) -> SlotPtr {
    trusted_cnode_lookup_slot_ptr(root_cap, offset).addr()
}

pub open spec fn resolve_root_step_next_cap(root_cap: CapSpec, cap_ptr: int, bits: int) -> CapSpec
    recommends
        resolve_step_offset_case(root_cap, cap_ptr, bits),
{
    resolve_root_step_next_cap_from_offset(root_cap, resolve_step_offset(root_cap, cap_ptr, bits))
}

pub open spec fn resolve_root_step_next_cap_from_offset(root_cap: CapSpec, offset: int) -> CapSpec {
    cte_slot_view_at(resolve_root_step_slot_from_offset(root_cap, offset)).cap
}

pub open spec fn resolve_first_level_early_stop_case(
    root_cap: CapSpec,
    cap_ptr: int,
    bits: int,
) -> bool {
    resolve_descend_case(root_cap, cap_ptr, bits)
        && resolve_root_step_next_cap(root_cap, cap_ptr, bits).kind != CapKind::CNodeCap
}

pub open spec fn resolve_first_level_continue_case(
    root_cap: CapSpec,
    cap_ptr: int,
    bits: int,
) -> bool {
    resolve_descend_case(root_cap, cap_ptr, bits)
        && resolve_root_step_next_cap(root_cap, cap_ptr, bits).kind == CapKind::CNodeCap
}

pub open spec fn resolve_address_bits_abstract(
    root_cap: CapSpec,
    cap_ptr: int,
    bits: int,
) -> ResolveAddressBitsResultSpec
    recommends
        0 <= cap_ptr,
        0 <= bits,
    decreases bits,
{
    if !(root_cap.kind == CapKind::CNodeCap && root_cap.cnode is Some) {
        resolve_invalid_root_result(bits)
    } else {
        let guard_bits = root_cap.cnode.unwrap().guard_size;
        let radix_bits = root_cap.cnode.unwrap().radix_bits;
        if guard_bits < 0 || radix_bits < 0 {
            resolve_address_bits_fault_result(
                bits,
                ResolveAddressBitsFaultSpec::MalformedLevel { bits_left: bits },
            )
        } else if bits < guard_bits {
            resolve_address_bits_fault_result(
                bits,
                ResolveAddressBitsFaultSpec::GuardTooDeep { bits_left: bits, guard_bits },
            )
        } else {
            let guard = spec_extract_bits(cap_ptr, bits - guard_bits, guard_bits);
            let expected_guard = root_cap.cnode.unwrap().guard;
            let level_bits = guard_bits + radix_bits;
            if guard != expected_guard {
                resolve_address_bits_fault_result(
                    bits,
                    ResolveAddressBitsFaultSpec::GuardMismatch {
                        bits_left: bits,
                        guard_found: guard,
                        guard_expected: expected_guard,
                        guard_bits,
                    },
                )
            } else if level_bits == 0 {
                resolve_address_bits_fault_result(
                    bits,
                    ResolveAddressBitsFaultSpec::MalformedLevel { bits_left: bits },
                )
            } else if bits < level_bits {
                resolve_address_bits_fault_result(
                    bits,
                    ResolveAddressBitsFaultSpec::LevelTooDeep { bits_left: bits, level_bits },
                )
            } else {
                let offset = spec_extract_bits(cap_ptr, bits - level_bits, radix_bits);
                let slot = resolve_root_step_slot_from_offset(root_cap, offset);
                let next_cap = resolve_root_step_next_cap_from_offset(root_cap, offset);
                let bits_after_step = bits - level_bits;
                if bits == level_bits {
                    resolve_address_bits_success_result(slot, 0)
                } else if next_cap.kind != CapKind::CNodeCap {
                    resolve_address_bits_success_result(slot, bits_after_step)
                } else {
                    resolve_address_bits_abstract(next_cap, cap_ptr, bits_after_step)
                }
            }
        }
    }
}

pub open spec fn resolve_address_bits_fault_result(
    bits_remaining: int,
    fault: ResolveAddressBitsFaultSpec,
) -> ResolveAddressBitsResultSpec {
    ResolveAddressBitsResultSpec {
        status: ResolveAddressBitsStatusSpec::LookupFault,
        slot: None,
        bits_remaining,
        fault: Some(fault),
    }
}

pub open spec fn resolve_address_bits_success_result(
    slot: SlotPtr,
    bits_remaining: int,
) -> ResolveAddressBitsResultSpec {
    ResolveAddressBitsResultSpec {
        status: ResolveAddressBitsStatusSpec::Success,
        slot: Some(slot),
        bits_remaining,
        fault: None,
    }
}

pub open spec fn resolve_exact_success_result(
    root_cap: CapSpec,
    cap_ptr: int,
    bits: int,
) -> ResolveAddressBitsResultSpec
    recommends
        resolve_exact_success_case(root_cap, cap_ptr, bits),
{
    resolve_address_bits_success_result(resolve_root_step_slot(root_cap, cap_ptr, bits), 0)
}

pub open spec fn resolve_first_level_early_stop_result(
    root_cap: CapSpec,
    cap_ptr: int,
    bits: int,
) -> ResolveAddressBitsResultSpec
    recommends
        resolve_first_level_early_stop_case(root_cap, cap_ptr, bits),
{
    resolve_address_bits_success_result(
        resolve_root_step_slot(root_cap, cap_ptr, bits),
        resolve_bits_after_root_step(root_cap, bits),
    )
}

pub open spec fn resolve_invalid_root_result(bits: int) -> ResolveAddressBitsResultSpec {
    resolve_address_bits_fault_result(bits, ResolveAddressBitsFaultSpec::InvalidRoot)
}

pub open spec fn resolve_guard_too_deep_result(
    root_cap: CapSpec,
    bits: int,
) -> ResolveAddressBitsResultSpec
    recommends
        resolve_guard_too_deep_case(root_cap, bits),
{
    resolve_address_bits_fault_result(
        bits,
        ResolveAddressBitsFaultSpec::GuardTooDeep {
            bits_left: bits,
            guard_bits: root_cap.cnode.unwrap().guard_size,
        },
    )
}

pub open spec fn resolve_guard_mismatch_result(
    root_cap: CapSpec,
    cap_ptr: int,
    bits: int,
) -> ResolveAddressBitsResultSpec
    recommends
        resolve_guard_mismatch_case(root_cap, cap_ptr, bits),
{
    resolve_address_bits_fault_result(
        bits,
        ResolveAddressBitsFaultSpec::GuardMismatch {
            bits_left: bits,
            guard_found: resolve_guard_value(root_cap, cap_ptr, bits),
            guard_expected: root_cap.cnode.unwrap().guard,
            guard_bits: root_cap.cnode.unwrap().guard_size,
        },
    )
}

pub open spec fn resolve_level_invalid_result(
    root_cap: CapSpec,
    bits: int,
) -> ResolveAddressBitsResultSpec
    recommends
        root_cap.kind == CapKind::CNodeCap,
        root_cap.cnode is Some,
        resolve_level_invalid(root_cap, bits),
{
    if resolve_cnode_level_bits(root_cap) == 0 {
        resolve_address_bits_fault_result(
            bits,
            ResolveAddressBitsFaultSpec::MalformedLevel { bits_left: bits },
        )
    } else {
        resolve_address_bits_fault_result(
            bits,
            ResolveAddressBitsFaultSpec::LevelTooDeep {
                bits_left: bits,
                level_bits: resolve_cnode_level_bits(root_cap),
            },
        )
    }
}

pub proof fn lemma_resolve_address_bits_abstract_unfold_invalid_root(
    root_cap: CapSpec,
    cap_ptr: int,
    bits: int,
)
    requires
        0 <= cap_ptr,
        0 <= bits,
        resolve_invalid_root_case(root_cap),
    ensures
        resolve_address_bits_abstract(root_cap, cap_ptr, bits) == resolve_invalid_root_result(bits),
{
}

pub proof fn lemma_resolve_address_bits_abstract_unfold_guard_too_deep(
    root_cap: CapSpec,
    cap_ptr: int,
    bits: int,
)
    requires
        0 <= cap_ptr,
        0 <= bits,
        resolve_guard_too_deep_case(root_cap, bits),
        0 <= root_cap.cnode.unwrap().guard_size,
        0 <= root_cap.cnode.unwrap().radix_bits,
    ensures
        resolve_address_bits_abstract(root_cap, cap_ptr, bits)
            == resolve_guard_too_deep_result(root_cap, bits),
{
}

pub proof fn lemma_resolve_address_bits_abstract_unfold_guard_mismatch(
    root_cap: CapSpec,
    cap_ptr: int,
    bits: int,
)
    requires
        0 <= cap_ptr,
        0 <= bits,
        resolve_guard_mismatch_case(root_cap, cap_ptr, bits),
        0 <= root_cap.cnode.unwrap().guard_size,
        0 <= root_cap.cnode.unwrap().radix_bits,
    ensures
        resolve_address_bits_abstract(root_cap, cap_ptr, bits)
            == resolve_guard_mismatch_result(root_cap, cap_ptr, bits),
{
}

pub proof fn lemma_resolve_address_bits_abstract_unfold_level_invalid(
    root_cap: CapSpec,
    cap_ptr: int,
    bits: int,
)
    requires
        0 <= cap_ptr,
        0 <= bits,
        resolve_level_invalid_case(root_cap, cap_ptr, bits),
        0 <= root_cap.cnode.unwrap().guard_size,
        0 <= root_cap.cnode.unwrap().radix_bits,
    ensures
        resolve_address_bits_abstract(root_cap, cap_ptr, bits)
            == resolve_level_invalid_result(root_cap, bits),
{
}

pub proof fn lemma_resolve_address_bits_abstract_unfold_exact_success(
    root_cap: CapSpec,
    cap_ptr: int,
    bits: int,
)
    requires
        0 <= cap_ptr,
        0 <= bits,
        resolve_exact_success_case(root_cap, cap_ptr, bits),
    ensures
        resolve_address_bits_abstract(root_cap, cap_ptr, bits)
            == resolve_exact_success_result(root_cap, cap_ptr, bits),
{
}

pub proof fn lemma_resolve_address_bits_abstract_unfold_early_stop(
    root_cap: CapSpec,
    cap_ptr: int,
    bits: int,
)
    requires
        0 <= cap_ptr,
        0 <= bits,
        resolve_first_level_early_stop_case(root_cap, cap_ptr, bits),
    ensures
        resolve_address_bits_abstract(root_cap, cap_ptr, bits)
            == resolve_first_level_early_stop_result(root_cap, cap_ptr, bits),
{
}

pub proof fn lemma_resolve_address_bits_abstract_unfold_continue(
    root_cap: CapSpec,
    cap_ptr: int,
    bits: int,
)
    requires
        0 <= cap_ptr,
        0 <= bits,
        resolve_first_level_continue_case(root_cap, cap_ptr, bits),
    ensures
        resolve_address_bits_abstract(root_cap, cap_ptr, bits)
            == resolve_address_bits_abstract(
                resolve_root_step_next_cap(root_cap, cap_ptr, bits),
                cap_ptr,
                resolve_bits_after_root_step(root_cap, bits),
            ),
{
}

pub open spec fn concrete_resolve_ret_refines_result(
    ret: resolveAddressBits_ret_t,
    result: ResolveAddressBitsResultSpec,
) -> bool {
    &&& ret.bitsRemaining as int == result.bits_remaining
    &&& (result.status == ResolveAddressBitsStatusSpec::Success ==> is_exception_none(ret.status))
    &&& (is_exception_none(ret.status) ==> result.status == ResolveAddressBitsStatusSpec::Success)
    &&& (result.status == ResolveAddressBitsStatusSpec::LookupFault
        ==> is_exception_lookup_fault(ret.status))
    &&& (is_exception_lookup_fault(ret.status)
        ==> result.status == ResolveAddressBitsStatusSpec::LookupFault)
    &&& (result.slot is Some ==> ret.slot as usize == result.slot.unwrap())
    &&& (result.slot is None ==> ret.slot as usize == 0)
}

} // verus!
