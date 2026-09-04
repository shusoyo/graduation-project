use vstd::prelude::*;

use crate::invariants::*;
use crate::model::*;

verus! {

pub open spec fn ensure_no_children_pre(s: AbsState, slot: SlotId) -> bool {
    &&& wf_cspace(s)
    &&& slot_has_cap(s, slot)
}

pub open spec fn ensure_no_children_post(
    old: AbsState,
    new: AbsState,
    slot: SlotId,
    ok: bool,
) -> bool {
    &&& wf_cspace(new)
    &&& new == old
    &&& ok == !has_mdb_child(old, slot)
}

pub open spec fn derive_cap_pre(s: AbsState, slot: SlotId) -> bool {
    &&& wf_cspace(s)
    &&& slot_has_cap(s, slot)
}

pub open spec fn derive_cap_post(
    old: AbsState,
    new: AbsState,
    slot: SlotId,
    ret: DeriveCapResult,
) -> bool {
    &&& wf_cspace(new)
    &&& new == old
    &&& match get_cap(old, slot) {
            cap @ Capability::ArchObjectCap { .. } =>
                ret.status == Exception::None && ret.capability == cap,
            cap @ Capability::UntypedCap { .. } =>
                if has_mdb_child(old, slot) {
                    ret.status == Exception::SyscallError
                    && ret.capability == Capability::NullCap
                } else {
                    ret.status == Exception::None
                    && ret.capability == cap
                },
            Capability::ZombieCap { .. } =>
                ret.status == Exception::None && ret.capability == Capability::NullCap,
            Capability::ReplyCap { .. } =>
                ret.status == Exception::None && ret.capability == Capability::NullCap,
            Capability::IRQControlCap =>
                ret.status == Exception::None && ret.capability == Capability::NullCap,
            cap => ret.status == Exception::None && ret.capability == cap,
        }
}

pub open spec fn ensure_no_children_impl(s: AbsState, slot: SlotId) -> bool {
    !has_mdb_child(s, slot)
}

pub proof fn ensure_no_children_impl_correct(s: AbsState, slot: SlotId)
    requires
        ensure_no_children_pre(s, slot),
    ensures
        ensure_no_children_post(s, s, slot, ensure_no_children_impl(s, slot)),
{
}

pub open spec fn derive_cap_impl(s: AbsState, slot: SlotId) -> DeriveCapResult {
    let src_cap = get_cap(s, slot);
    if is_arch_cap(src_cap) {
        DeriveCapResult { status: Exception::None, capability: src_cap }
    } else {
        match src_cap {
            cap @ Capability::UntypedCap { .. } =>
                if ensure_no_children_impl(s, slot) {
                    DeriveCapResult { status: Exception::None, capability: cap }
                } else {
                    DeriveCapResult { status: Exception::SyscallError, capability: Capability::NullCap }
                },
            Capability::ZombieCap { .. } =>
                DeriveCapResult { status: Exception::None, capability: Capability::NullCap },
            Capability::ReplyCap { .. } =>
                DeriveCapResult { status: Exception::None, capability: Capability::NullCap },
            Capability::IRQControlCap =>
                DeriveCapResult { status: Exception::None, capability: Capability::NullCap },
            cap =>
                DeriveCapResult { status: Exception::None, capability: cap },
        }
    }
}

pub proof fn derive_cap_impl_correct(s: AbsState, slot: SlotId)
    requires
        derive_cap_pre(s, slot),
    ensures
        derive_cap_post(s, s, slot, derive_cap_impl(s, slot)),
{
}

pub open spec fn cte_insert_pre(
    s: AbsState,
    new_cap: Capability,
    src_slot: SlotId,
    dest_slot: SlotId,
) -> bool {
    &&& wf_cspace(s)
    &&& slot_has_cap(s, src_slot)
    &&& slot_is_free(s, dest_slot)
    &&& new_cap != Capability::NullCap
}

pub open spec fn cte_insert_post(
    old: AbsState,
    new: AbsState,
    new_cap: Capability,
    src_slot: SlotId,
    dest_slot: SlotId,
) -> bool {
    let src_cte = get_cte(old, src_slot);
    let next = src_cte.mdb_node.next;
    let revocable = is_cap_revocable(new_cap, src_cte.capability);
    &&& wf_cspace(new)
    &&& get_cap(new, dest_slot) == new_cap
    &&& get_cte(new, dest_slot).mdb_node.prev == src_slot
    &&& get_cte(new, dest_slot).mdb_node.next == next
    &&& get_cte(new, dest_slot).mdb_node.revocable == revocable
    &&& get_cte(new, dest_slot).mdb_node.first_badged == revocable
    &&& get_cte(new, src_slot).mdb_node.next == dest_slot
    &&& (next != null_slot() ==> get_cte(new, next).mdb_node.prev == dest_slot)
}

pub open spec fn insert_new_cap_pre(
    s: AbsState,
    parent: SlotId,
    slot: SlotId,
    capability: Capability,
) -> bool {
    &&& wf_cspace(s)
    &&& slot_has_cap(s, parent)
    &&& slot_is_free(s, slot)
    &&& capability != Capability::NullCap
}

pub open spec fn insert_new_cap_post(
    old: AbsState,
    new: AbsState,
    parent: SlotId,
    slot: SlotId,
    capability: Capability,
) -> bool {
    let next = get_cte(old, parent).mdb_node.next;
    &&& wf_cspace(new)
    &&& get_cap(new, slot) == capability
    &&& get_cte(new, slot).mdb_node.next == next
    &&& get_cte(new, slot).mdb_node.revocable
    &&& get_cte(new, slot).mdb_node.first_badged
    &&& get_cte(new, slot).mdb_node.prev == parent
    &&& get_cte(new, parent).mdb_node.next == slot
    &&& (next != null_slot() ==> get_cte(new, next).mdb_node.prev == slot)
}

pub open spec fn cte_move_pre(
    s: AbsState,
    new_cap: Capability,
    src_slot: SlotId,
    dest_slot: SlotId,
) -> bool {
    &&& wf_cspace(s)
    &&& slot_has_cap(s, src_slot)
    &&& slot_is_free(s, dest_slot)
    &&& new_cap != Capability::NullCap
}

pub open spec fn cte_move_post(
    old: AbsState,
    new: AbsState,
    new_cap: Capability,
    src_slot: SlotId,
    dest_slot: SlotId,
) -> bool {
    let mdb = get_cte(old, src_slot).mdb_node;
    &&& wf_cspace(new)
    &&& get_cap(new, dest_slot) == new_cap
    &&& get_cte(new, dest_slot).mdb_node == mdb
    &&& get_cte(new, src_slot) == null_cte()
    &&& (mdb.prev != null_slot() ==> get_cte(new, mdb.prev).mdb_node.next == dest_slot)
    &&& (mdb.next != null_slot() ==> get_cte(new, mdb.next).mdb_node.prev == dest_slot)
}

pub open spec fn cte_swap_pre(
    s: AbsState,
    cap1: Capability,
    slot1: SlotId,
    cap2: Capability,
    slot2: SlotId,
) -> bool {
    &&& wf_cspace(s)
    &&& slot_has_cap(s, slot1)
    &&& slot_has_cap(s, slot2)
    &&& slot1 != slot2
    &&& cap1 != Capability::NullCap
    &&& cap2 != Capability::NullCap
}

pub open spec fn cte_swap_post(
    old: AbsState,
    new: AbsState,
    cap1: Capability,
    slot1: SlotId,
    cap2: Capability,
    slot2: SlotId,
) -> bool {
    let mdb1 = get_cte(old, slot1).mdb_node;
    let mdb2 = get_cte(old, slot2).mdb_node;
    &&& wf_cspace(new)
    &&& get_cap(new, slot1) == cap2
    &&& get_cap(new, slot2) == cap1
    &&& get_cte(new, slot1).mdb_node == mdb2
    &&& get_cte(new, slot2).mdb_node == mdb1
}

pub open spec fn delete_all_pre(s: AbsState, slot: SlotId) -> bool {
    &&& wf_cspace(s)
    &&& slot_exists(s, slot)
}

pub open spec fn delete_all_post(old: AbsState, new: AbsState, slot: SlotId) -> bool {
    &&& wf_cspace(new)
    &&& slot_exists(old, slot)
    &&& get_cte(new, slot) == null_cte()
}

pub open spec fn revoke_pre(s: AbsState, slot: SlotId) -> bool {
    &&& wf_cspace(s)
    &&& slot_has_cap(s, slot)
}

pub open spec fn revoke_post(old: AbsState, new: AbsState, slot: SlotId) -> bool {
    let next = get_cte(old, slot).mdb_node.next;
    &&& wf_cspace(new)
    &&& slot_has_cap(old, slot)
    &&& (next != null_slot() && slot_exists(old, next) && is_mdb_parent_of(get_cte(old, slot), get_cte(old, next))
            ==> get_cap(new, next) == Capability::NullCap)
}

pub open spec fn resolve_address_bits_pre(
    s: AbsState,
    node_cap: Capability,
    cap_ptr: nat,
    n_bits: nat,
) -> bool {
    &&& wf_cspace(s)
    &&& n_bits > 0
    &&& cap_ptr >= 0
    &&& is_cnode_cap(node_cap)
}

pub open spec fn resolve_address_bits_post(
    old: AbsState,
    new: AbsState,
    node_cap: Capability,
    cap_ptr: nat,
    n_bits: nat,
    ret: ResolveAddressBitsResult,
) -> bool {
    &&& wf_cspace(new)
    &&& new == old
    &&& ret.bits_remaining <= n_bits
    &&& ret.success ==> slot_exists(old, ret.slot)
}

}
