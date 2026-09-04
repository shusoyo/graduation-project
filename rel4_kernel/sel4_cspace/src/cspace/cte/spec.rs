use vstd::prelude::*;

verus! {

use crate::capability::spec::CapSpec;
#[cfg(verus_keep_ghost)]
use crate::capability::spec::{
    lemma_same_region_preserved_when_rhs_same_except_untyped,
    spec_cap_badge_value, spec_same_object_as_caps, spec_same_region_as_caps,
    spec_untyped_cap_contains_cap, CapKind,
};
#[cfg(verus_keep_ghost)]
use crate::capability::spec::same_cap_except_untyped_free_index;

#[verifier::ext_equal]
pub ghost struct SlotEntrySpec {
    pub cap: CapSpec,
    pub mdb_prev: Option<usize>,
    pub mdb_next: Option<usize>,
    pub mdb_revocable: bool,
    pub mdb_first_badged: bool,
}

pub open spec fn spec_empty_slot_entry() -> SlotEntrySpec {
    SlotEntrySpec {
        cap: CapSpec {
            kind: crate::capability::spec::CapKind::NullCap,
            object: Option::None,
            region_id: Option::None,
            rights: crate::capability::spec::Rights {
                can_read: false,
                can_write: false,
                can_grant: false,
                can_grant_reply: false,
            },
            badge: Option::None,
            cnode: Option::None,
            untyped: Option::None,
        },
        mdb_prev: Option::None,
        mdb_next: Option::None,
        mdb_revocable: false,
        mdb_first_badged: false,
    }
}

pub open spec fn spec_slot_entry_with_cap(entry: SlotEntrySpec, cap: CapSpec) -> SlotEntrySpec {
    SlotEntrySpec {
        cap,
        mdb_prev: entry.mdb_prev,
        mdb_next: entry.mdb_next,
        mdb_revocable: entry.mdb_revocable,
        mdb_first_badged: entry.mdb_first_badged,
    }
}

pub open spec fn slot_cap_update_rel(
    old_entry: SlotEntrySpec,
    new_entry: SlotEntrySpec,
    new_cap: CapSpec,
) -> bool {
    new_entry == spec_slot_entry_with_cap(old_entry, new_cap)
}

pub open spec fn slot_cleared_rel(old_entry: SlotEntrySpec, new_entry: SlotEntrySpec) -> bool {
    slot_cap_update_rel(old_entry, new_entry, spec_empty_slot_entry().cap)
}

pub open spec fn same_mdb_fields(old_entry: SlotEntrySpec, new_entry: SlotEntrySpec) -> bool {
    &&& new_entry.mdb_prev == old_entry.mdb_prev
    &&& new_entry.mdb_next == old_entry.mdb_next
    &&& new_entry.mdb_revocable == old_entry.mdb_revocable
    &&& new_entry.mdb_first_badged == old_entry.mdb_first_badged
}

pub proof fn lemma_slot_cap_update_rel_implies_same_mdb_fields(
    old_entry: SlotEntrySpec,
    new_entry: SlotEntrySpec,
    new_cap: CapSpec,
)
    requires
        slot_cap_update_rel(old_entry, new_entry, new_cap),
    ensures
        same_mdb_fields(old_entry, new_entry),
        new_entry.cap == new_cap,
{
}

pub proof fn lemma_slot_cleared_rel_implies_same_mdb_fields(
    old_entry: SlotEntrySpec,
    new_entry: SlotEntrySpec,
)
    requires
        slot_cleared_rel(old_entry, new_entry),
    ensures
        same_mdb_fields(old_entry, new_entry),
        new_entry.cap == spec_empty_slot_entry().cap,
{
    lemma_slot_cap_update_rel_implies_same_mdb_fields(
        old_entry,
        new_entry,
        spec_empty_slot_entry().cap,
    );
}

pub open spec fn same_entry_except_untyped_free_index(
    old_entry: SlotEntrySpec,
    new_entry: SlotEntrySpec,
) -> bool {
    &&& same_mdb_fields(old_entry, new_entry)
    &&& same_cap_except_untyped_free_index(old_entry.cap, new_entry.cap)
}

pub open spec fn spec_slot_entry_with_prev(
    entry: SlotEntrySpec,
    prev: Option<usize>,
) -> SlotEntrySpec {
    SlotEntrySpec {
        cap: entry.cap,
        mdb_prev: prev,
        mdb_next: entry.mdb_next,
        mdb_revocable: entry.mdb_revocable,
        mdb_first_badged: entry.mdb_first_badged,
    }
}

pub open spec fn spec_slot_entry_with_next(
    entry: SlotEntrySpec,
    next: Option<usize>,
) -> SlotEntrySpec {
    SlotEntrySpec {
        cap: entry.cap,
        mdb_prev: entry.mdb_prev,
        mdb_next: next,
        mdb_revocable: entry.mdb_revocable,
        mdb_first_badged: entry.mdb_first_badged,
    }
}

pub open spec fn spec_slot_entry_with_revocable(
    entry: SlotEntrySpec,
    revocable: bool,
) -> SlotEntrySpec {
    SlotEntrySpec {
        cap: entry.cap,
        mdb_prev: entry.mdb_prev,
        mdb_next: entry.mdb_next,
        mdb_revocable: revocable,
        mdb_first_badged: entry.mdb_first_badged,
    }
}

pub open spec fn spec_slot_entry_with_first_badged(
    entry: SlotEntrySpec,
    first_badged: bool,
) -> SlotEntrySpec {
    SlotEntrySpec {
        cap: entry.cap,
        mdb_prev: entry.mdb_prev,
        mdb_next: entry.mdb_next,
        mdb_revocable: entry.mdb_revocable,
        mdb_first_badged: first_badged,
    }
}

pub open spec fn spec_slot_entry_with_contents(
    cap: CapSpec,
    prev: Option<usize>,
    next: Option<usize>,
    revocable: bool,
    first_badged: bool,
) -> SlotEntrySpec {
    SlotEntrySpec {
        cap,
        mdb_prev: prev,
        mdb_next: next,
        mdb_revocable: revocable,
        mdb_first_badged: first_badged,
    }
}

#[cfg(verus_keep_ghost)]
pub open spec fn spec_badge_chain_allows(parent_cap: CapSpec, child_entry: SlotEntrySpec) -> bool {
    if parent_cap.kind == CapKind::EndpointCap
        && parent_cap.badge is Some
        && parent_cap.badge.unwrap() != 0 {
        &&& child_entry.cap.kind == CapKind::EndpointCap
        &&& child_entry.cap.badge == parent_cap.badge
        &&& !child_entry.mdb_first_badged
    } else if parent_cap.kind == CapKind::NotificationCap
        && parent_cap.badge is Some
        && parent_cap.badge.unwrap() != 0 {
        &&& child_entry.cap.kind == CapKind::NotificationCap
        &&& child_entry.cap.badge == parent_cap.badge
        &&& !child_entry.mdb_first_badged
    } else {
        true
    }
}

#[cfg(verus_keep_ghost)]
pub proof fn lemma_badge_value_match_implies_badge_chain(
    parent_cap: CapSpec,
    child_entry: SlotEntrySpec,
)
    requires
        parent_cap.kind == CapKind::EndpointCap && parent_cap.badge is Some
            && parent_cap.badge.unwrap() != 0 ==> {
            &&& child_entry.cap.kind == CapKind::EndpointCap
            &&& child_entry.cap.badge == parent_cap.badge
            &&& !child_entry.mdb_first_badged
        },
        parent_cap.kind == CapKind::NotificationCap && parent_cap.badge is Some
            && parent_cap.badge.unwrap() != 0 ==> {
            &&& child_entry.cap.kind == CapKind::NotificationCap
            &&& child_entry.cap.badge == parent_cap.badge
            &&& !child_entry.mdb_first_badged
        },
    ensures
        spec_badge_chain_allows(parent_cap, child_entry),
{
    if parent_cap.kind == CapKind::EndpointCap {
        if parent_cap.badge is Some && parent_cap.badge.unwrap() != 0 {
            assert(child_entry.cap.kind == CapKind::EndpointCap);
            assert(child_entry.cap.badge == parent_cap.badge);
            assert(!child_entry.mdb_first_badged);
        }
    } else if parent_cap.kind == CapKind::NotificationCap {
        if parent_cap.badge is Some && parent_cap.badge.unwrap() != 0 {
            assert(child_entry.cap.kind == CapKind::NotificationCap);
            assert(child_entry.cap.badge == parent_cap.badge);
            assert(!child_entry.mdb_first_badged);
        }
    }
}

#[cfg(verus_keep_ghost)]
pub proof fn lemma_badge_chain_composes_through_middle(
    parent_cap: CapSpec,
    middle_entry: SlotEntrySpec,
    child_entry: SlotEntrySpec,
)
    requires
        spec_badge_chain_allows(parent_cap, middle_entry),
        spec_badge_chain_allows(middle_entry.cap, child_entry),
        !middle_entry.mdb_first_badged,
    ensures
        spec_badge_chain_allows(parent_cap, child_entry),
{
    if parent_cap.kind == CapKind::EndpointCap {
        if parent_cap.badge is Some && parent_cap.badge.unwrap() != 0 {
            assert(middle_entry.cap.kind == CapKind::EndpointCap);
            assert(middle_entry.cap.badge == parent_cap.badge);
            assert(!middle_entry.mdb_first_badged);
            assert(middle_entry.cap.badge is Some);
            assert(middle_entry.cap.badge.unwrap() != 0);
            assert(child_entry.cap.kind == CapKind::EndpointCap);
            assert(child_entry.cap.badge == middle_entry.cap.badge);
            assert(!child_entry.mdb_first_badged);
            assert(child_entry.cap.badge == parent_cap.badge);
        }
    } else if parent_cap.kind == CapKind::NotificationCap {
        if parent_cap.badge is Some && parent_cap.badge.unwrap() != 0 {
            assert(middle_entry.cap.kind == CapKind::NotificationCap);
            assert(middle_entry.cap.badge == parent_cap.badge);
            assert(!middle_entry.mdb_first_badged);
            assert(middle_entry.cap.badge is Some);
            assert(middle_entry.cap.badge.unwrap() != 0);
            assert(child_entry.cap.kind == CapKind::NotificationCap);
            assert(child_entry.cap.badge == middle_entry.cap.badge);
            assert(!child_entry.mdb_first_badged);
            assert(child_entry.cap.badge == parent_cap.badge);
        }
    }
}

#[cfg(verus_keep_ghost)]
pub open spec fn spec_arch_mdb_parent_ok(parent_cap: CapSpec, child_cap: CapSpec) -> bool {
    if child_cap.kind == CapKind::ArchCap {
        parent_cap.kind == CapKind::UntypedCap || parent_cap.kind == CapKind::ArchCap
    } else if parent_cap.kind == CapKind::ArchCap {
        false
    } else {
        true
    }
}

#[cfg(verus_keep_ghost)]
pub open spec fn spec_mdb_parent_of_caps(
    parent_cap: CapSpec,
    parent_revocable: bool,
    child_cap: CapSpec,
    child_first_badged: bool,
) -> bool {
    &&& parent_revocable
    &&& spec_same_region_as_caps(parent_cap, child_cap)
    &&& spec_arch_mdb_parent_ok(parent_cap, child_cap)
    &&& (if parent_cap.kind == CapKind::EndpointCap
        && parent_cap.badge is Some
        && parent_cap.badge.unwrap() != 0 {
        &&& child_cap.kind == CapKind::EndpointCap
        &&& child_cap.badge == parent_cap.badge
        &&& !child_first_badged
    } else if parent_cap.kind == CapKind::NotificationCap
        && parent_cap.badge is Some
        && parent_cap.badge.unwrap() != 0 {
        &&& child_cap.kind == CapKind::NotificationCap
        &&& child_cap.badge == parent_cap.badge
        &&& !child_first_badged
    } else {
        true
    })
}

#[cfg(verus_keep_ghost)]
pub open spec fn spec_slot_mdb_parent_of(parent: SlotEntrySpec, child: SlotEntrySpec) -> bool {
    &&& parent.mdb_revocable
    &&& spec_same_region_as_caps(parent.cap, child.cap)
    &&& spec_arch_mdb_parent_ok(parent.cap, child.cap)
    &&& spec_badge_chain_allows(parent.cap, child)
}

#[cfg(verus_keep_ghost)]
pub open spec fn spec_incoming_parent_edge_ok(
    parent_cap: Option<CapSpec>,
    child_entry: SlotEntrySpec,
) -> bool {
    if child_entry.mdb_prev is Some {
        &&& parent_cap is Some
        &&& (!child_entry.mdb_revocable
            || spec_same_region_as_caps(parent_cap.unwrap(), child_entry.cap))
    } else {
        !child_entry.mdb_revocable
    }
}

#[cfg(verus_keep_ghost)]
pub open spec fn spec_incoming_badge_edge_ok(
    parent_cap: Option<CapSpec>,
    child_entry: SlotEntrySpec,
) -> bool {
    if child_entry.mdb_prev is Some {
        &&& parent_cap is Some
        &&& (!child_entry.mdb_revocable
            || spec_badge_chain_allows(parent_cap.unwrap(), child_entry))
    } else {
        true
    }
}

#[cfg(verus_keep_ghost)]
pub open spec fn spec_incoming_untyped_edge_ok(
    parent_cap: Option<CapSpec>,
    child_entry: SlotEntrySpec,
) -> bool {
    if child_entry.mdb_prev is Some {
        &&& parent_cap is Some
        &&& (!(child_entry.mdb_revocable && parent_cap.unwrap().kind == CapKind::UntypedCap)
            || spec_untyped_cap_contains_cap(parent_cap.unwrap(), child_entry.cap))
    } else {
        true
    }
}

#[cfg(verus_keep_ghost)]
pub proof fn lemma_incoming_parent_edge_ok_preserved_by_relevant_fields(
    old_parent_cap: Option<CapSpec>,
    new_parent_cap: Option<CapSpec>,
    old_entry: SlotEntrySpec,
    new_entry: SlotEntrySpec,
)
    requires
        new_parent_cap == old_parent_cap,
        new_entry.cap == old_entry.cap,
        new_entry.mdb_prev == old_entry.mdb_prev,
        new_entry.mdb_revocable == old_entry.mdb_revocable,
    ensures
        spec_incoming_parent_edge_ok(new_parent_cap, new_entry)
            == spec_incoming_parent_edge_ok(old_parent_cap, old_entry),
{
}

#[cfg(verus_keep_ghost)]
pub proof fn lemma_incoming_parent_edge_ok_preserved_when_parent_cap_same(
    old_parent_cap: Option<CapSpec>,
    new_parent_cap: Option<CapSpec>,
    old_entry: SlotEntrySpec,
    new_entry: SlotEntrySpec,
)
    requires
        old_entry.mdb_prev is Some,
        new_entry.mdb_prev is Some,
        new_parent_cap == old_parent_cap,
        new_entry.cap == old_entry.cap,
        new_entry.mdb_revocable == old_entry.mdb_revocable,
    ensures
        spec_incoming_parent_edge_ok(new_parent_cap, new_entry)
            == spec_incoming_parent_edge_ok(old_parent_cap, old_entry),
{
}

#[cfg(verus_keep_ghost)]
pub proof fn lemma_incoming_badge_edge_ok_preserved_by_relevant_fields(
    old_parent_cap: Option<CapSpec>,
    new_parent_cap: Option<CapSpec>,
    old_entry: SlotEntrySpec,
    new_entry: SlotEntrySpec,
)
    requires
        new_parent_cap == old_parent_cap,
        new_entry.cap == old_entry.cap,
        new_entry.mdb_prev == old_entry.mdb_prev,
        new_entry.mdb_revocable == old_entry.mdb_revocable,
        new_entry.mdb_first_badged == old_entry.mdb_first_badged,
    ensures
        spec_incoming_badge_edge_ok(new_parent_cap, new_entry)
            == spec_incoming_badge_edge_ok(old_parent_cap, old_entry),
{
}

#[cfg(verus_keep_ghost)]
pub proof fn lemma_incoming_badge_edge_ok_preserved_when_parent_cap_same(
    old_parent_cap: Option<CapSpec>,
    new_parent_cap: Option<CapSpec>,
    old_entry: SlotEntrySpec,
    new_entry: SlotEntrySpec,
)
    requires
        old_entry.mdb_prev is Some,
        new_entry.mdb_prev is Some,
        new_parent_cap == old_parent_cap,
        new_entry.cap == old_entry.cap,
        new_entry.mdb_revocable == old_entry.mdb_revocable,
        new_entry.mdb_first_badged == old_entry.mdb_first_badged,
    ensures
        spec_incoming_badge_edge_ok(new_parent_cap, new_entry)
            == spec_incoming_badge_edge_ok(old_parent_cap, old_entry),
{
}

#[cfg(verus_keep_ghost)]
pub proof fn lemma_incoming_untyped_edge_ok_preserved_by_relevant_fields(
    old_parent_cap: Option<CapSpec>,
    new_parent_cap: Option<CapSpec>,
    old_entry: SlotEntrySpec,
    new_entry: SlotEntrySpec,
)
    requires
        new_parent_cap == old_parent_cap,
        new_entry.cap == old_entry.cap,
        new_entry.mdb_prev == old_entry.mdb_prev,
        new_entry.mdb_revocable == old_entry.mdb_revocable,
    ensures
        spec_incoming_untyped_edge_ok(new_parent_cap, new_entry)
            == spec_incoming_untyped_edge_ok(old_parent_cap, old_entry),
{
}

#[cfg(verus_keep_ghost)]
pub proof fn lemma_incoming_untyped_edge_ok_preserved_when_parent_cap_same(
    old_parent_cap: Option<CapSpec>,
    new_parent_cap: Option<CapSpec>,
    old_entry: SlotEntrySpec,
    new_entry: SlotEntrySpec,
)
    requires
        old_entry.mdb_prev is Some,
        new_entry.mdb_prev is Some,
        new_parent_cap == old_parent_cap,
        new_entry.cap == old_entry.cap,
        new_entry.mdb_revocable == old_entry.mdb_revocable,
    ensures
        spec_incoming_untyped_edge_ok(new_parent_cap, new_entry)
            == spec_incoming_untyped_edge_ok(old_parent_cap, old_entry),
{
}

#[cfg(verus_keep_ghost)]
pub proof fn lemma_incoming_parent_edge_ok_preserved_when_child_cap_same_except_untyped(
    parent_cap: Option<CapSpec>,
    old_entry: SlotEntrySpec,
    new_entry: SlotEntrySpec,
)
    requires
        old_entry.mdb_prev == new_entry.mdb_prev,
        old_entry.mdb_revocable == new_entry.mdb_revocable,
        same_cap_except_untyped_free_index(old_entry.cap, new_entry.cap),
        spec_incoming_parent_edge_ok(parent_cap, old_entry),
    ensures
        spec_incoming_parent_edge_ok(parent_cap, new_entry),
{
    if old_entry.mdb_prev is Some {
        assert(parent_cap is Some);
        if old_entry.mdb_revocable {
            lemma_same_region_preserved_when_rhs_same_except_untyped(
                parent_cap.unwrap(),
                old_entry.cap,
                new_entry.cap,
            );
        }
    }
}

#[cfg(verus_keep_ghost)]
pub proof fn lemma_incoming_badge_edge_ok_preserved_when_child_cap_same_except_untyped(
    parent_cap: Option<CapSpec>,
    old_entry: SlotEntrySpec,
    new_entry: SlotEntrySpec,
)
    requires
        old_entry.mdb_prev == new_entry.mdb_prev,
        old_entry.mdb_revocable == new_entry.mdb_revocable,
        old_entry.mdb_first_badged == new_entry.mdb_first_badged,
        same_cap_except_untyped_free_index(old_entry.cap, new_entry.cap),
        spec_incoming_badge_edge_ok(parent_cap, old_entry),
    ensures
        spec_incoming_badge_edge_ok(parent_cap, new_entry),
{
    if old_entry.mdb_prev is Some {
        assert(parent_cap is Some);
        assert(new_entry.cap.kind == old_entry.cap.kind);
        assert(new_entry.cap.badge == old_entry.cap.badge);
    }
}

#[cfg(verus_keep_ghost)]
pub proof fn lemma_incoming_untyped_edge_ok_preserved_when_child_cap_same_except_untyped(
    parent_cap: Option<CapSpec>,
    old_entry: SlotEntrySpec,
    new_entry: SlotEntrySpec,
)
    requires
        old_entry.mdb_prev == new_entry.mdb_prev,
        old_entry.mdb_revocable == new_entry.mdb_revocable,
        same_cap_except_untyped_free_index(old_entry.cap, new_entry.cap),
        spec_incoming_untyped_edge_ok(parent_cap, old_entry),
    ensures
        spec_incoming_untyped_edge_ok(parent_cap, new_entry),
{
    if old_entry.mdb_prev is Some {
        assert(parent_cap is Some);
        if old_entry.mdb_revocable && parent_cap.unwrap().kind == CapKind::UntypedCap {
            lemma_same_region_preserved_when_rhs_same_except_untyped(
                parent_cap.unwrap(),
                old_entry.cap,
                new_entry.cap,
            );
        }
    }
}

#[cfg(verus_keep_ghost)]
pub open spec fn spec_slot_ensure_no_children_blocks(entry: SlotEntrySpec) -> bool {
    entry.mdb_next is Some
        && spec_slot_mdb_parent_of(entry, cte_slot_view_at(entry.mdb_next.unwrap()))
}

#[cfg(verus_keep_ghost)]
pub open spec fn spec_slot_is_final_cap_at(slot: usize) -> bool {
    let entry = cte_slot_view_at(slot);
    if entry.mdb_prev is Some
        && spec_same_object_as_caps(cte_slot_view_at(entry.mdb_prev.unwrap()).cap, entry.cap)
    {
        false
    } else if entry.mdb_next is None {
        true
    } else {
        !spec_same_object_as_caps(entry.cap, cte_slot_view_at(entry.mdb_next.unwrap()).cap)
    }
}

#[cfg(verus_keep_ghost)]
pub open spec fn spec_cap_is_long_running_delete_target(capability: CapSpec) -> bool {
    capability.kind == CapKind::ThreadCap
        || capability.kind == CapKind::ZombieCap
        || capability.kind == CapKind::CNodeCap
}

#[cfg(verus_keep_ghost)]
pub open spec fn spec_slot_is_long_running_delete_at(slot: usize) -> bool {
    let entry = cte_slot_view_at(slot);
    entry.cap.kind != CapKind::NullCap
        && spec_slot_is_final_cap_at(slot)
        && spec_cap_is_long_running_delete_target(entry.cap)
}

#[cfg(verus_keep_ghost)]
pub open spec fn spec_slot_derive_cap_returns_syscall_error(
    entry: SlotEntrySpec,
    capability: CapSpec,
) -> bool {
    capability.kind == CapKind::UntypedCap && spec_slot_ensure_no_children_blocks(entry)
}

#[cfg(verus_keep_ghost)]
pub open spec fn spec_slot_derive_cap_expected_cap(
    entry: SlotEntrySpec,
    capability: CapSpec,
) -> CapSpec {
    match capability.kind {
        CapKind::ZombieCap => spec_empty_slot_entry().cap,
        CapKind::UntypedCap => {
            if spec_slot_ensure_no_children_blocks(entry) {
                spec_empty_slot_entry().cap
            } else {
                capability
            }
        },
        #[cfg(not(feature = "kernel_mcs"))]
        CapKind::ReplyCap => spec_empty_slot_entry().cap,
        CapKind::IRQControlCap => spec_empty_slot_entry().cap,
        _ => capability,
    }
}

pub uninterp spec fn cte_slot_ptr(slot: &crate::cspace::cte::types::cte_t) -> usize;

pub uninterp spec fn cte_offset_slot_call_pre(base: usize, index: usize) -> bool;

pub uninterp spec fn cte_slot_view_at(slot: usize) -> SlotEntrySpec;

}
