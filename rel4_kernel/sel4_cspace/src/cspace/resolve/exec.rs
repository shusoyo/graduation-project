#[cfg(verus_keep_ghost)]
use crate::capability::spec::CapSpec;
#[cfg(verus_keep_ghost)]
use crate::cspace::resolve::{
    concrete_resolve_ret_refines_result,
    resolve_bits_after_root_step, resolve_cnode_level_bits, resolve_descend_case, resolve_exact_success_case,
    resolve_first_level_early_stop_case, resolve_root_step_next_cap, resolve_root_step_slot,
    resolve_address_bits_abstract,
    ResolveAddressBitsStatusSpec,
    resolve_guard_mismatch_result, resolve_guard_too_deep_result, resolve_invalid_root_result,
    resolve_level_invalid_result, lemma_resolve_address_bits_abstract_unfold_continue,
    lemma_resolve_address_bits_abstract_unfold_early_stop,
    lemma_resolve_address_bits_abstract_unfold_exact_success,
    lemma_resolve_address_bits_abstract_unfold_guard_mismatch,
    lemma_resolve_address_bits_abstract_unfold_guard_too_deep,
    lemma_resolve_address_bits_abstract_unfold_invalid_root,
    lemma_resolve_address_bits_abstract_unfold_level_invalid,
    resolve_guard_mismatch_case, resolve_guard_matches, resolve_guard_too_deep_case,
    resolve_first_level_continue_case, resolve_invalid_root_case, resolve_level_invalid,
    resolve_level_invalid_case,
};
use crate::structures::resolveAddressBits_ret_t;
use crate::capability::raw::runtime_clone_cap;
#[cfg(verus_keep_ghost)]
use crate::capability::raw::trusted_view_cap;
#[cfg(verus_keep_ghost)]
use crate::kernel_api::raw::{
    is_exception_lookup_fault, is_exception_none, lemma_exception_lookup_fault_not_none,
    lemma_exception_none_not_lookup_fault,
};
use crate::kernel_api::raw::{runtime_exception_lookup_fault, runtime_exception_none};
use crate::cspace::resolve::raw::{
    runtime_cnode_cap_guard,
    runtime_cnode_cap_guard_bits, runtime_cnode_cap_level_bits, runtime_cnode_cap_radix_bits,
    runtime_cnode_lookup_slot_from_cap, runtime_extract_bits_usize, runtime_slot_cap_clone,
    runtime_cap_is_cnode,
};
use core::intrinsics::unlikely;
use sel4_common::structures_gen::cap;
use vstd::prelude::*;

verus! {

proof fn lemma_root_guard_too_deep_contradiction(
    root_cap: CapSpec,
    n_bits: int,
    remaining: int,
    guard_bits: usize,
)
    requires
        resolve_guard_too_deep_case(root_cap, n_bits),
        remaining == n_bits,
        guard_bits as int == root_cap.cnode.unwrap().guard_size,
        guard_bits <= remaining,
    ensures
        false,
{
    assert(root_cap.cnode.unwrap().guard_size > n_bits);
    assert(guard_bits as int > remaining);
    assert(false);
}

proof fn lemma_root_guard_mismatch_contradiction(
    root_cap: CapSpec,
    cap_ptr: int,
    n_bits: int,
    resolve_guard: bool,
)
    requires
        resolve_guard_mismatch_case(root_cap, cap_ptr, n_bits),
        resolve_guard,
        resolve_guard == resolve_guard_matches(root_cap, cap_ptr, n_bits),
    ensures
        false,
{
    assert(!resolve_guard_matches(root_cap, cap_ptr, n_bits));
    assert(false);
}

proof fn lemma_root_level_invalid_contradiction(
    root_cap: CapSpec,
    cap_ptr: int,
    n_bits: int,
    remaining: int,
    level_bits: usize,
)
    requires
        resolve_level_invalid_case(root_cap, cap_ptr, n_bits),
        remaining == n_bits,
        level_bits as int == root_cap.cnode.unwrap().guard_size + root_cap.cnode.unwrap().radix_bits,
        !(level_bits == 0 || level_bits > remaining),
    ensures
        false,
{
    assert(resolve_level_invalid(root_cap, n_bits));
    assert(level_bits == 0 || level_bits > remaining);
    assert(false);
}

proof fn lemma_root_descend_not_first_round_fault(
    root_cap: CapSpec,
    cap_ptr: int,
    n_bits: int,
    remaining: int,
)
    requires
        resolve_descend_case(root_cap, cap_ptr, n_bits),
        remaining == n_bits,
    ensures
        !resolve_guard_too_deep_case(root_cap, n_bits),
        !resolve_guard_mismatch_case(root_cap, cap_ptr, n_bits),
        !resolve_level_invalid_case(root_cap, cap_ptr, n_bits),
        !resolve_exact_success_case(root_cap, cap_ptr, n_bits),
        0 < n_bits,
{
    assert(0 <= resolve_cnode_level_bits(root_cap));
    assert(resolve_cnode_level_bits(root_cap) < n_bits);
    assert(0 < n_bits);
    assert(!resolve_level_invalid_case(root_cap, cap_ptr, n_bits));
    assert(resolve_guard_matches(root_cap, cap_ptr, n_bits));
    assert(!resolve_guard_too_deep_case(root_cap, n_bits));
    assert(!resolve_guard_mismatch_case(root_cap, cap_ptr, n_bits));
    assert(!resolve_exact_success_case(root_cap, cap_ptr, n_bits));
}

proof fn lemma_root_exact_success_not_nonexact_branch(
    root_cap: CapSpec,
    cap_ptr: int,
    n_bits: int,
    remaining: int,
    level_bits: usize,
)
    requires
        resolve_exact_success_case(root_cap, cap_ptr, n_bits),
        remaining == n_bits,
        level_bits as int == resolve_cnode_level_bits(root_cap),
        remaining != level_bits,
    ensures
        false,
{
    assert(n_bits == resolve_cnode_level_bits(root_cap));
    assert(level_bits as int == n_bits);
    assert(level_bits as int == remaining);
    assert(false);
}

proof fn lemma_current_refinement_lifts_to_root(
    root_cap: CapSpec,
    current_cap: CapSpec,
    cap_ptr: int,
    n_bits: int,
    remaining: int,
    ret: resolveAddressBits_ret_t,
)
    requires
        0 <= cap_ptr,
        0 <= n_bits,
        0 <= remaining,
        resolve_address_bits_abstract(root_cap, cap_ptr, n_bits)
            == resolve_address_bits_abstract(current_cap, cap_ptr, remaining),
        concrete_resolve_ret_refines_result(
            ret,
            resolve_address_bits_abstract(current_cap, cap_ptr, remaining),
        ),
    ensures
        concrete_resolve_ret_refines_result(
            ret,
            resolve_address_bits_abstract(root_cap, cap_ptr, n_bits),
        ),
{
    assert(resolve_address_bits_abstract(root_cap, cap_ptr, n_bits)
        == resolve_address_bits_abstract(current_cap, cap_ptr, remaining));
}

proof fn lemma_exact_success_current_refines(
    current_cap: CapSpec,
    cap_ptr: int,
    remaining: int,
    ret: resolveAddressBits_ret_t,
)
    requires
        0 <= cap_ptr,
        0 <= remaining,
        resolve_exact_success_case(current_cap, cap_ptr, remaining),
        is_exception_none(ret.status),
        ret.slot as usize == resolve_root_step_slot(current_cap, cap_ptr, remaining),
        ret.bitsRemaining == 0,
    ensures
        concrete_resolve_ret_refines_result(
            ret,
            resolve_address_bits_abstract(current_cap, cap_ptr, remaining),
        ),
{
    lemma_resolve_address_bits_abstract_unfold_exact_success(current_cap, cap_ptr, remaining);
    assert(resolve_address_bits_abstract(current_cap, cap_ptr, remaining).status
        == ResolveAddressBitsStatusSpec::Success);
    assert(resolve_address_bits_abstract(current_cap, cap_ptr, remaining).slot is Some);
    assert(resolve_address_bits_abstract(current_cap, cap_ptr, remaining).slot.unwrap()
        == resolve_root_step_slot(current_cap, cap_ptr, remaining));
    assert(resolve_address_bits_abstract(current_cap, cap_ptr, remaining).bits_remaining == 0);
    lemma_exception_none_not_lookup_fault(ret.status);
    assert(concrete_resolve_ret_refines_result(
        ret,
        resolve_address_bits_abstract(current_cap, cap_ptr, remaining),
    ));
}

proof fn lemma_early_stop_current_refines(
    current_cap: CapSpec,
    cap_ptr: int,
    prev_remaining: int,
    ret: resolveAddressBits_ret_t,
)
    requires
        0 <= cap_ptr,
        0 <= prev_remaining,
        resolve_first_level_early_stop_case(current_cap, cap_ptr, prev_remaining),
        is_exception_none(ret.status),
        ret.slot as usize == resolve_root_step_slot(current_cap, cap_ptr, prev_remaining),
        ret.bitsRemaining as int == resolve_bits_after_root_step(current_cap, prev_remaining),
    ensures
        concrete_resolve_ret_refines_result(
            ret,
            resolve_address_bits_abstract(current_cap, cap_ptr, prev_remaining),
        ),
{
    lemma_resolve_address_bits_abstract_unfold_early_stop(current_cap, cap_ptr, prev_remaining);
    assert(resolve_address_bits_abstract(current_cap, cap_ptr, prev_remaining).status
        == ResolveAddressBitsStatusSpec::Success);
    assert(resolve_address_bits_abstract(current_cap, cap_ptr, prev_remaining).slot is Some);
    assert(resolve_address_bits_abstract(current_cap, cap_ptr, prev_remaining).slot.unwrap()
        == resolve_root_step_slot(current_cap, cap_ptr, prev_remaining));
    assert(resolve_address_bits_abstract(current_cap, cap_ptr, prev_remaining).bits_remaining
        == resolve_bits_after_root_step(current_cap, prev_remaining));
    lemma_exception_none_not_lookup_fault(ret.status);
    assert(concrete_resolve_ret_refines_result(
        ret,
        resolve_address_bits_abstract(current_cap, cap_ptr, prev_remaining),
    ));
}

pub fn resolve_address_bits(
    node_cap: &cap,
    cap_ptr: usize,
    n_bits: usize,
) -> (ret: resolveAddressBits_ret_t)
    ensures
        ret.bitsRemaining <= n_bits,
        concrete_resolve_ret_refines_result(
            ret,
            resolve_address_bits_abstract(
                trusted_view_cap(node_cap),
                cap_ptr as int,
                n_bits as int,
            ),
        ),
        resolve_invalid_root_case(trusted_view_cap(node_cap))
            ==> is_exception_lookup_fault(ret.status)
                && ret.bitsRemaining == n_bits
                && concrete_resolve_ret_refines_result(
                    ret,
                    resolve_address_bits_abstract(
                        trusted_view_cap(node_cap),
                        cap_ptr as int,
                        n_bits as int,
                    ),
                ),
        resolve_guard_too_deep_case(trusted_view_cap(node_cap), n_bits as int)
            ==> is_exception_lookup_fault(ret.status)
                && ret.bitsRemaining == n_bits
                && concrete_resolve_ret_refines_result(
                    ret,
                    resolve_address_bits_abstract(
                        trusted_view_cap(node_cap),
                        cap_ptr as int,
                        n_bits as int,
                    ),
                ),
        resolve_guard_mismatch_case(trusted_view_cap(node_cap), cap_ptr as int, n_bits as int)
            ==> is_exception_lookup_fault(ret.status)
                && ret.bitsRemaining == n_bits
                && concrete_resolve_ret_refines_result(
                    ret,
                    resolve_address_bits_abstract(
                        trusted_view_cap(node_cap),
                        cap_ptr as int,
                        n_bits as int,
                    ),
                ),
        resolve_level_invalid_case(trusted_view_cap(node_cap), cap_ptr as int, n_bits as int)
            ==> is_exception_lookup_fault(ret.status)
                && ret.bitsRemaining == n_bits
                && concrete_resolve_ret_refines_result(
                    ret,
                    resolve_address_bits_abstract(
                        trusted_view_cap(node_cap),
                        cap_ptr as int,
                        n_bits as int,
                    ),
                ),
        resolve_exact_success_case(trusted_view_cap(node_cap), cap_ptr as int, n_bits as int)
            ==> ret.bitsRemaining == 0
                && is_exception_none(ret.status)
                && ret.slot as usize
                    == resolve_root_step_slot(trusted_view_cap(node_cap), cap_ptr as int, n_bits as int)
                && concrete_resolve_ret_refines_result(
                    ret,
                    resolve_address_bits_abstract(
                        trusted_view_cap(node_cap),
                        cap_ptr as int,
                        n_bits as int,
                    ),
                ),
        resolve_first_level_early_stop_case(trusted_view_cap(node_cap), cap_ptr as int, n_bits as int)
            ==> is_exception_none(ret.status)
                && ret.slot as usize
                    == resolve_root_step_slot(trusted_view_cap(node_cap), cap_ptr as int, n_bits as int)
                && ret.bitsRemaining
                    == resolve_bits_after_root_step(trusted_view_cap(node_cap), n_bits as int)
                && concrete_resolve_ret_refines_result(
                    ret,
                    resolve_address_bits_abstract(
                        trusted_view_cap(node_cap),
                        cap_ptr as int,
                        n_bits as int,
                    ),
                ),
        resolve_descend_case(trusted_view_cap(node_cap), cap_ptr as int, n_bits as int)
            ==> ret.bitsRemaining < n_bits
                && ret.bitsRemaining
                    <= resolve_bits_after_root_step(trusted_view_cap(node_cap), n_bits as int),
{
    let mut ret = resolveAddressBits_ret_t::default();
    ret.status = runtime_exception_none();
    let mut remaining = n_bits;
    ret.bitsRemaining = remaining;

    if unlikely(!runtime_cap_is_cnode(node_cap)) {
        ret.status = runtime_exception_lookup_fault();
        ret.slot = core::ptr::null_mut();
        proof {
            lemma_resolve_address_bits_abstract_unfold_invalid_root(
                trusted_view_cap(node_cap),
                cap_ptr as int,
                n_bits as int,
            );
            assert(resolve_address_bits_abstract(
                trusted_view_cap(node_cap),
                cap_ptr as int,
                n_bits as int,
            ) == resolve_invalid_root_result(n_bits as int));
            lemma_exception_lookup_fault_not_none(ret.status);
            assert(concrete_resolve_ret_refines_result(
                ret,
                resolve_address_bits_abstract(
                    trusted_view_cap(node_cap),
                    cap_ptr as int,
                    n_bits as int,
                ),
            ));
        }
        return ret;
    }

    let mut current = runtime_clone_cap(node_cap);

    proof {
        assert(is_exception_none(ret.status));
        assert(trusted_view_cap(node_cap).kind
            == crate::capability::spec::CapKind::CNodeCap);
        assert(trusted_view_cap(&current) == trusted_view_cap(node_cap));
        assert(trusted_view_cap(&current).kind
            == crate::capability::spec::CapKind::CNodeCap);
        assert(!resolve_invalid_root_case(trusted_view_cap(node_cap)));
    }

    loop
        invariant
            remaining <= n_bits,
            ret.bitsRemaining <= n_bits,
            is_exception_none(ret.status),
            !resolve_invalid_root_case(trusted_view_cap(node_cap)),
            resolve_address_bits_abstract(
                trusted_view_cap(node_cap),
                cap_ptr as int,
                n_bits as int,
            ) == resolve_address_bits_abstract(
                trusted_view_cap(&current),
                cap_ptr as int,
                remaining as int,
            ),
            resolve_guard_too_deep_case(trusted_view_cap(node_cap), n_bits as int)
                ==> remaining == n_bits && trusted_view_cap(&current) == trusted_view_cap(node_cap),
            resolve_guard_mismatch_case(trusted_view_cap(node_cap), cap_ptr as int, n_bits as int)
                ==> remaining == n_bits && trusted_view_cap(&current) == trusted_view_cap(node_cap),
            resolve_level_invalid_case(trusted_view_cap(node_cap), cap_ptr as int, n_bits as int)
                ==> remaining == n_bits && trusted_view_cap(&current) == trusted_view_cap(node_cap),
            resolve_exact_success_case(trusted_view_cap(node_cap), cap_ptr as int, n_bits as int)
                ==> remaining == n_bits && trusted_view_cap(&current) == trusted_view_cap(node_cap),
            resolve_first_level_early_stop_case(trusted_view_cap(node_cap), cap_ptr as int, n_bits as int)
                ==> remaining == n_bits && trusted_view_cap(&current) == trusted_view_cap(node_cap),
            resolve_descend_case(trusted_view_cap(node_cap), cap_ptr as int, n_bits as int)
                ==> remaining <= resolve_bits_after_root_step(trusted_view_cap(node_cap), n_bits as int)
                    || (remaining == n_bits
                        && trusted_view_cap(&current) == trusted_view_cap(node_cap)),
            trusted_view_cap(node_cap).kind == crate::capability::spec::CapKind::CNodeCap,
            trusted_view_cap(&current).kind == crate::capability::spec::CapKind::CNodeCap,
        decreases remaining,
    {
        let radix_bits = runtime_cnode_cap_radix_bits(&current);
        let guard_bits = runtime_cnode_cap_guard_bits(&current);
        let level_bits = runtime_cnode_cap_level_bits(&current);
        let cap_guard = runtime_cnode_cap_guard(&current);

        if unlikely(guard_bits > remaining) {
            proof {
                if resolve_guard_too_deep_case(trusted_view_cap(node_cap), n_bits as int) {
                    assert(remaining == n_bits);
                }
                if resolve_guard_mismatch_case(
                    trusted_view_cap(node_cap),
                    cap_ptr as int,
                    n_bits as int,
                ) {
                    assert(remaining == n_bits);
                    lemma_root_guard_too_deep_contradiction(
                        trusted_view_cap(node_cap),
                        n_bits as int,
                        remaining as int,
                        guard_bits,
                    );
                }
                if resolve_level_invalid_case(
                    trusted_view_cap(node_cap),
                    cap_ptr as int,
                    n_bits as int,
                ) {
                    assert(remaining == n_bits);
                    lemma_root_guard_too_deep_contradiction(
                        trusted_view_cap(node_cap),
                        n_bits as int,
                        remaining as int,
                        guard_bits,
                    );
                }
                if resolve_exact_success_case(
                    trusted_view_cap(node_cap),
                    cap_ptr as int,
                    n_bits as int,
                ) {
                    assert(remaining == n_bits);
                    lemma_root_guard_too_deep_contradiction(
                        trusted_view_cap(node_cap),
                        n_bits as int,
                        remaining as int,
                        guard_bits,
                    );
                }
                if resolve_descend_case(
                    trusted_view_cap(node_cap),
                    cap_ptr as int,
                    n_bits as int,
                ) {
                    if remaining == n_bits {
                        lemma_root_descend_not_first_round_fault(
                            trusted_view_cap(node_cap),
                            cap_ptr as int,
                            n_bits as int,
                            remaining as int,
                        );
                        assert(false);
                    }
                }
                if resolve_first_level_early_stop_case(
                    trusted_view_cap(node_cap),
                    cap_ptr as int,
                    n_bits as int,
                ) {
                    assert(remaining == n_bits);
                    lemma_root_descend_not_first_round_fault(
                        trusted_view_cap(node_cap),
                        cap_ptr as int,
                        n_bits as int,
                        remaining as int,
                    );
                    assert(false);
                }
            }
            ret.status = runtime_exception_lookup_fault();
            ret.slot = core::ptr::null_mut();
            ret.bitsRemaining = remaining;
            proof {
                assert(resolve_guard_too_deep_case(
                    trusted_view_cap(&current),
                    remaining as int,
                ));
                lemma_resolve_address_bits_abstract_unfold_guard_too_deep(
                    trusted_view_cap(&current),
                    cap_ptr as int,
                    remaining as int,
                );
                lemma_exception_lookup_fault_not_none(ret.status);
                assert(concrete_resolve_ret_refines_result(
                    ret,
                    resolve_address_bits_abstract(
                        trusted_view_cap(&current),
                        cap_ptr as int,
                        remaining as int,
                    ),
                ));
                assert(concrete_resolve_ret_refines_result(
                    ret,
                    resolve_address_bits_abstract(
                        trusted_view_cap(node_cap),
                        cap_ptr as int,
                        n_bits as int,
                    ),
                ));
                if resolve_guard_too_deep_case(trusted_view_cap(node_cap), n_bits as int) {
                    lemma_resolve_address_bits_abstract_unfold_guard_too_deep(
                        trusted_view_cap(node_cap),
                        cap_ptr as int,
                        n_bits as int,
                    );
                }
            }
            return ret;
        }

        let guard = runtime_extract_bits_usize(cap_ptr, remaining - guard_bits, guard_bits);
        if unlikely(guard != cap_guard) {
            proof {
                if resolve_guard_too_deep_case(trusted_view_cap(node_cap), n_bits as int) {
                    assert(remaining == n_bits);
                    lemma_root_guard_too_deep_contradiction(
                        trusted_view_cap(node_cap),
                        n_bits as int,
                        remaining as int,
                        guard_bits,
                    );
                }
                if resolve_guard_mismatch_case(
                    trusted_view_cap(node_cap),
                    cap_ptr as int,
                    n_bits as int,
                ) {
                    assert(remaining == n_bits);
                }
                if resolve_level_invalid_case(
                    trusted_view_cap(node_cap),
                    cap_ptr as int,
                    n_bits as int,
                ) {
                    assert(remaining == n_bits);
                    lemma_root_guard_mismatch_contradiction(
                        trusted_view_cap(node_cap),
                        cap_ptr as int,
                        n_bits as int,
                        resolve_guard_matches(
                            trusted_view_cap(node_cap),
                            cap_ptr as int,
                            n_bits as int,
                        ),
                    );
                }
                if resolve_exact_success_case(
                    trusted_view_cap(node_cap),
                    cap_ptr as int,
                    n_bits as int,
                ) {
                    assert(remaining == n_bits);
                    lemma_root_guard_mismatch_contradiction(
                        trusted_view_cap(node_cap),
                        cap_ptr as int,
                        n_bits as int,
                        resolve_guard_matches(
                            trusted_view_cap(node_cap),
                            cap_ptr as int,
                            n_bits as int,
                        ),
                    );
                }
                if resolve_descend_case(
                    trusted_view_cap(node_cap),
                    cap_ptr as int,
                    n_bits as int,
                ) {
                    if remaining == n_bits {
                        lemma_root_descend_not_first_round_fault(
                            trusted_view_cap(node_cap),
                            cap_ptr as int,
                            n_bits as int,
                            remaining as int,
                        );
                        assert(false);
                    }
                }
                if resolve_first_level_early_stop_case(
                    trusted_view_cap(node_cap),
                    cap_ptr as int,
                    n_bits as int,
                ) {
                    assert(remaining == n_bits);
                    lemma_root_descend_not_first_round_fault(
                        trusted_view_cap(node_cap),
                        cap_ptr as int,
                        n_bits as int,
                        remaining as int,
                    );
                    assert(false);
                }
            }
            ret.status = runtime_exception_lookup_fault();
            ret.slot = core::ptr::null_mut();
            ret.bitsRemaining = remaining;
            proof {
                assert(resolve_guard_mismatch_case(
                    trusted_view_cap(&current),
                    cap_ptr as int,
                    remaining as int,
                ));
                lemma_resolve_address_bits_abstract_unfold_guard_mismatch(
                    trusted_view_cap(&current),
                    cap_ptr as int,
                    remaining as int,
                );
                lemma_exception_lookup_fault_not_none(ret.status);
                assert(concrete_resolve_ret_refines_result(
                    ret,
                    resolve_address_bits_abstract(
                        trusted_view_cap(&current),
                        cap_ptr as int,
                        remaining as int,
                    ),
                ));
                assert(concrete_resolve_ret_refines_result(
                    ret,
                    resolve_address_bits_abstract(
                        trusted_view_cap(node_cap),
                        cap_ptr as int,
                        n_bits as int,
                    ),
                ));
                if resolve_guard_mismatch_case(
                    trusted_view_cap(node_cap),
                    cap_ptr as int,
                    n_bits as int,
                ) {
                    lemma_resolve_address_bits_abstract_unfold_guard_mismatch(
                        trusted_view_cap(node_cap),
                        cap_ptr as int,
                        n_bits as int,
                    );
                }
            }
            return ret;
        }

        if unlikely(level_bits == 0 || level_bits > remaining) {
            proof {
                if resolve_guard_too_deep_case(trusted_view_cap(node_cap), n_bits as int) {
                    assert(remaining == n_bits);
                    lemma_root_guard_too_deep_contradiction(
                        trusted_view_cap(node_cap),
                        n_bits as int,
                        remaining as int,
                        guard_bits,
                    );
                }
                if resolve_guard_mismatch_case(
                    trusted_view_cap(node_cap),
                    cap_ptr as int,
                    n_bits as int,
                ) {
                    assert(remaining == n_bits);
                    lemma_root_guard_mismatch_contradiction(
                        trusted_view_cap(node_cap),
                        cap_ptr as int,
                        n_bits as int,
                        resolve_guard_matches(
                            trusted_view_cap(node_cap),
                            cap_ptr as int,
                            n_bits as int,
                        ),
                    );
                }
                if resolve_level_invalid_case(
                    trusted_view_cap(node_cap),
                    cap_ptr as int,
                    n_bits as int,
                ) {
                    assert(remaining == n_bits);
                }
                if resolve_exact_success_case(
                    trusted_view_cap(node_cap),
                    cap_ptr as int,
                    n_bits as int,
                ) {
                    assert(remaining == n_bits);
                    lemma_root_level_invalid_contradiction(
                        trusted_view_cap(node_cap),
                        cap_ptr as int,
                        n_bits as int,
                        remaining as int,
                        level_bits,
                    );
                }
                if resolve_descend_case(
                    trusted_view_cap(node_cap),
                    cap_ptr as int,
                    n_bits as int,
                ) {
                    if remaining == n_bits {
                        lemma_root_descend_not_first_round_fault(
                            trusted_view_cap(node_cap),
                            cap_ptr as int,
                            n_bits as int,
                            remaining as int,
                        );
                        assert(false);
                    }
                }
                if resolve_first_level_early_stop_case(
                    trusted_view_cap(node_cap),
                    cap_ptr as int,
                    n_bits as int,
                ) {
                    assert(remaining == n_bits);
                    lemma_root_descend_not_first_round_fault(
                        trusted_view_cap(node_cap),
                        cap_ptr as int,
                        n_bits as int,
                        remaining as int,
                    );
                    assert(false);
                }
            }
            ret.status = runtime_exception_lookup_fault();
            ret.slot = core::ptr::null_mut();
            ret.bitsRemaining = remaining;
            proof {
                assert(resolve_level_invalid_case(
                    trusted_view_cap(&current),
                    cap_ptr as int,
                    remaining as int,
                ));
                lemma_resolve_address_bits_abstract_unfold_level_invalid(
                    trusted_view_cap(&current),
                    cap_ptr as int,
                    remaining as int,
                );
                lemma_exception_lookup_fault_not_none(ret.status);
                assert(concrete_resolve_ret_refines_result(
                    ret,
                    resolve_address_bits_abstract(
                        trusted_view_cap(&current),
                        cap_ptr as int,
                        remaining as int,
                    ),
                ));
                assert(concrete_resolve_ret_refines_result(
                    ret,
                    resolve_address_bits_abstract(
                        trusted_view_cap(node_cap),
                        cap_ptr as int,
                        n_bits as int,
                    ),
                ));
                if resolve_level_invalid_case(
                    trusted_view_cap(node_cap),
                    cap_ptr as int,
                    n_bits as int,
                ) {
                    lemma_resolve_address_bits_abstract_unfold_level_invalid(
                        trusted_view_cap(node_cap),
                        cap_ptr as int,
                        n_bits as int,
                    );
                }
            }
            return ret;
        }

        let offset = runtime_extract_bits_usize(cap_ptr, remaining - level_bits, radix_bits);
        let slot = runtime_cnode_lookup_slot_from_cap(&current, offset);

        if remaining == level_bits {
            proof {
                if resolve_guard_too_deep_case(trusted_view_cap(node_cap), n_bits as int) {
                    assert(remaining == n_bits);
                    lemma_root_guard_too_deep_contradiction(
                        trusted_view_cap(node_cap),
                        n_bits as int,
                        remaining as int,
                        guard_bits,
                    );
                }
                if resolve_guard_mismatch_case(
                    trusted_view_cap(node_cap),
                    cap_ptr as int,
                    n_bits as int,
                ) {
                    assert(remaining == n_bits);
                    lemma_root_guard_mismatch_contradiction(
                        trusted_view_cap(node_cap),
                        cap_ptr as int,
                        n_bits as int,
                        resolve_guard_matches(
                            trusted_view_cap(node_cap),
                            cap_ptr as int,
                            n_bits as int,
                        ),
                    );
                }
                if resolve_level_invalid_case(
                    trusted_view_cap(node_cap),
                    cap_ptr as int,
                    n_bits as int,
                ) {
                    assert(remaining == n_bits);
                    lemma_root_level_invalid_contradiction(
                        trusted_view_cap(node_cap),
                        cap_ptr as int,
                        n_bits as int,
                        remaining as int,
                        level_bits,
                    );
                }
                if resolve_exact_success_case(
                    trusted_view_cap(node_cap),
                    cap_ptr as int,
                    n_bits as int,
                ) {
                    assert(remaining == n_bits);
                    assert(is_exception_none(ret.status));
                    lemma_exception_none_not_lookup_fault(ret.status);
                }
                if resolve_descend_case(
                    trusted_view_cap(node_cap),
                    cap_ptr as int,
                    n_bits as int,
                ) {
                    if remaining == n_bits {
                        lemma_root_descend_not_first_round_fault(
                            trusted_view_cap(node_cap),
                            cap_ptr as int,
                            n_bits as int,
                            remaining as int,
                        );
                        assert(false);
                    } else {
                        assert(remaining < n_bits);
                        assert(0 < n_bits);
                    }
                }
                if resolve_first_level_early_stop_case(
                    trusted_view_cap(node_cap),
                    cap_ptr as int,
                    n_bits as int,
                ) {
                    assert(remaining == n_bits);
                    lemma_root_descend_not_first_round_fault(
                        trusted_view_cap(node_cap),
                        cap_ptr as int,
                        n_bits as int,
                        remaining as int,
                    );
                    assert(false);
                }
            }
            ret.slot = slot;
            ret.bitsRemaining = 0;
            proof {
                lemma_exact_success_current_refines(
                    trusted_view_cap(&current),
                    cap_ptr as int,
                    remaining as int,
                    ret,
                );
                lemma_current_refinement_lifts_to_root(
                    trusted_view_cap(node_cap),
                    trusted_view_cap(&current),
                    cap_ptr as int,
                    n_bits as int,
                    remaining as int,
                    ret,
                );
            }
            return ret;
        }

        let prev_remaining = remaining;
        remaining = remaining - level_bits;
        let next_cap = runtime_slot_cap_clone(slot);

        if unlikely(!runtime_cap_is_cnode(&next_cap)) {
            proof {
                if resolve_guard_too_deep_case(trusted_view_cap(node_cap), n_bits as int) {
                    assert(prev_remaining == n_bits);
                    lemma_root_guard_too_deep_contradiction(
                        trusted_view_cap(node_cap),
                        n_bits as int,
                        prev_remaining as int,
                        guard_bits,
                    );
                }
                if resolve_guard_mismatch_case(
                    trusted_view_cap(node_cap),
                    cap_ptr as int,
                    n_bits as int,
                ) {
                    assert(prev_remaining == n_bits);
                    lemma_root_guard_mismatch_contradiction(
                        trusted_view_cap(node_cap),
                        cap_ptr as int,
                        n_bits as int,
                        resolve_guard_matches(
                            trusted_view_cap(node_cap),
                            cap_ptr as int,
                            n_bits as int,
                        ),
                    );
                }
                if resolve_level_invalid_case(
                    trusted_view_cap(node_cap),
                    cap_ptr as int,
                    n_bits as int,
                ) {
                    assert(prev_remaining == n_bits);
                    lemma_root_level_invalid_contradiction(
                        trusted_view_cap(node_cap),
                        cap_ptr as int,
                        n_bits as int,
                        prev_remaining as int,
                        level_bits,
                    );
                }
                if resolve_exact_success_case(
                    trusted_view_cap(node_cap),
                    cap_ptr as int,
                    n_bits as int,
                ) {
                    assert(prev_remaining == n_bits);
                    lemma_root_exact_success_not_nonexact_branch(
                        trusted_view_cap(node_cap),
                        cap_ptr as int,
                        n_bits as int,
                        prev_remaining as int,
                        level_bits,
                    );
                }
                if resolve_descend_case(
                    trusted_view_cap(node_cap),
                    cap_ptr as int,
                    n_bits as int,
                ) {
                    if prev_remaining == n_bits {
                        lemma_root_descend_not_first_round_fault(
                            trusted_view_cap(node_cap),
                            cap_ptr as int,
                            n_bits as int,
                            prev_remaining as int,
                        );
                        assert(0 < n_bits);
                        assert(level_bits != 0);
                        assert(remaining == prev_remaining - level_bits);
                        assert(remaining < prev_remaining);
                    }
                    assert(remaining < n_bits);
                }
                if resolve_first_level_early_stop_case(
                    trusted_view_cap(node_cap),
                    cap_ptr as int,
                    n_bits as int,
                ) {
                    assert(prev_remaining == n_bits);
                    assert(trusted_view_cap(&current) == trusted_view_cap(node_cap));
                    assert(slot as usize
                        == resolve_root_step_slot(
                            trusted_view_cap(node_cap),
                            cap_ptr as int,
                            n_bits as int,
                        ));
                    assert(trusted_view_cap(&next_cap) == resolve_root_step_next_cap(
                        trusted_view_cap(node_cap),
                        cap_ptr as int,
                        n_bits as int,
                    ));
                    assert(remaining
                        == resolve_bits_after_root_step(
                            trusted_view_cap(node_cap),
                            n_bits as int,
                        ));
                }
            }
            ret.slot = slot;
            ret.bitsRemaining = remaining;
            proof {
                lemma_early_stop_current_refines(
                    trusted_view_cap(&current),
                    cap_ptr as int,
                    prev_remaining as int,
                    ret,
                );
                lemma_current_refinement_lifts_to_root(
                    trusted_view_cap(node_cap),
                    trusted_view_cap(&current),
                    cap_ptr as int,
                    n_bits as int,
                    prev_remaining as int,
                    ret,
                );
                if resolve_first_level_early_stop_case(
                    trusted_view_cap(node_cap),
                    cap_ptr as int,
                    n_bits as int,
                ) {
                    lemma_resolve_address_bits_abstract_unfold_early_stop(
                        trusted_view_cap(node_cap),
                        cap_ptr as int,
                        n_bits as int,
                    );
                    assert(resolve_address_bits_abstract(
                        trusted_view_cap(node_cap),
                        cap_ptr as int,
                        n_bits as int,
                    ) == crate::cspace::resolve::resolve_first_level_early_stop_result(
                        trusted_view_cap(node_cap),
                        cap_ptr as int,
                        n_bits as int,
                    ));
                    assert(resolve_address_bits_abstract(
                        trusted_view_cap(node_cap),
                        cap_ptr as int,
                        n_bits as int,
                    ).status
                        == ResolveAddressBitsStatusSpec::Success);
                    assert(resolve_address_bits_abstract(
                        trusted_view_cap(node_cap),
                        cap_ptr as int,
                        n_bits as int,
                    ).slot is Some);
                    assert(resolve_address_bits_abstract(
                        trusted_view_cap(node_cap),
                        cap_ptr as int,
                        n_bits as int,
                    ).slot.unwrap()
                        == resolve_root_step_slot(
                            trusted_view_cap(node_cap),
                            cap_ptr as int,
                            n_bits as int,
                        ));
                    assert(resolve_address_bits_abstract(
                        trusted_view_cap(node_cap),
                        cap_ptr as int,
                        n_bits as int,
                    ).bits_remaining
                        == resolve_bits_after_root_step(
                            trusted_view_cap(node_cap),
                            n_bits as int,
                        ));
                    assert(is_exception_none(ret.status));
                    assert(ret.slot as usize
                        == resolve_root_step_slot(
                            trusted_view_cap(node_cap),
                            cap_ptr as int,
                            n_bits as int,
                        ));
                    assert(ret.bitsRemaining as int
                        == resolve_bits_after_root_step(
                            trusted_view_cap(node_cap),
                            n_bits as int,
                        ));
                    lemma_exception_none_not_lookup_fault(ret.status);
                    assert(!is_exception_lookup_fault(ret.status));
                    assert(concrete_resolve_ret_refines_result(
                        ret,
                        resolve_address_bits_abstract(
                            trusted_view_cap(node_cap),
                            cap_ptr as int,
                            n_bits as int,
                        ),
                    ));
                }
            }
            return ret;
        }

        proof {
            assert(level_bits != 0);
            assert(remaining == prev_remaining - level_bits);
            assert(remaining < prev_remaining);
            assert(remaining <= n_bits);
            assert(trusted_view_cap(&next_cap).kind
                == crate::capability::spec::CapKind::CNodeCap);
            if resolve_guard_too_deep_case(trusted_view_cap(node_cap), n_bits as int) {
                assert(prev_remaining == n_bits);
                lemma_root_guard_too_deep_contradiction(
                    trusted_view_cap(node_cap),
                    n_bits as int,
                    prev_remaining as int,
                    guard_bits,
                );
            }
            if resolve_guard_mismatch_case(
                trusted_view_cap(node_cap),
                cap_ptr as int,
                n_bits as int,
            ) {
                assert(prev_remaining == n_bits);
                lemma_root_guard_mismatch_contradiction(
                    trusted_view_cap(node_cap),
                    cap_ptr as int,
                    n_bits as int,
                    resolve_guard_matches(
                        trusted_view_cap(node_cap),
                        cap_ptr as int,
                        n_bits as int,
                    ),
                );
            }
            if resolve_level_invalid_case(
                trusted_view_cap(node_cap),
                cap_ptr as int,
                n_bits as int,
            ) {
                assert(prev_remaining == n_bits);
                lemma_root_level_invalid_contradiction(
                    trusted_view_cap(node_cap),
                    cap_ptr as int,
                    n_bits as int,
                    prev_remaining as int,
                    level_bits,
                );
            }
            if resolve_exact_success_case(
                trusted_view_cap(node_cap),
                cap_ptr as int,
                n_bits as int,
            ) {
                assert(prev_remaining == n_bits);
                lemma_root_exact_success_not_nonexact_branch(
                    trusted_view_cap(node_cap),
                    cap_ptr as int,
                    n_bits as int,
                    prev_remaining as int,
                    level_bits,
                );
            }
            if resolve_descend_case(
                trusted_view_cap(node_cap),
                cap_ptr as int,
                n_bits as int,
            ) {
                if prev_remaining == n_bits {
                    lemma_root_descend_not_first_round_fault(
                        trusted_view_cap(node_cap),
                        cap_ptr as int,
                        n_bits as int,
                        prev_remaining as int,
                    );
                    assert(0 < n_bits);
                }
                assert(remaining < n_bits);
            }
            assert(resolve_first_level_continue_case(
                trusted_view_cap(&current),
                cap_ptr as int,
                prev_remaining as int,
            ));
            lemma_resolve_address_bits_abstract_unfold_continue(
                trusted_view_cap(&current),
                cap_ptr as int,
                prev_remaining as int,
            );
            assert(resolve_address_bits_abstract(
                trusted_view_cap(node_cap),
                cap_ptr as int,
                n_bits as int,
            ) == resolve_address_bits_abstract(
                trusted_view_cap(&next_cap),
                cap_ptr as int,
                remaining as int,
            ));
            if resolve_first_level_early_stop_case(
                trusted_view_cap(node_cap),
                cap_ptr as int,
                n_bits as int,
            ) {
                assert(prev_remaining == n_bits);
                assert(trusted_view_cap(&current) == trusted_view_cap(node_cap));
                assert(slot as usize
                    == resolve_root_step_slot(
                        trusted_view_cap(node_cap),
                        cap_ptr as int,
                        n_bits as int,
                    ));
                assert(trusted_view_cap(&next_cap) == resolve_root_step_next_cap(
                    trusted_view_cap(node_cap),
                    cap_ptr as int,
                    n_bits as int,
                ));
                assert(trusted_view_cap(&next_cap).kind
                    != crate::capability::spec::CapKind::CNodeCap);
                assert(trusted_view_cap(&next_cap).kind
                    == crate::capability::spec::CapKind::CNodeCap);
                assert(false);
            }
        }

        current = next_cap;
    }
}

}
