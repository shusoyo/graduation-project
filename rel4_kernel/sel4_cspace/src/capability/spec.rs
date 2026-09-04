use vstd::prelude::*;

verus! {

pub ghost enum ObjectKind {
    Untyped,
    Endpoint,
    Notification,
    CNode,
    Thread,
    Reply,
    IRQ,
    Arch,
    Zombie,
    Opaque,
}

pub ghost enum CapKind {
    NullCap,
    UntypedCap,
    EndpointCap,
    NotificationCap,
    CNodeCap,
    ThreadCap,
    ReplyCap,
    IRQControlCap,
    IRQHandlerCap,
    ZombieCap,
    ArchCap,
    Other,
}

#[verifier::ext_equal]
pub ghost struct ObjectRef {
    pub id: int,
    pub kind: ObjectKind,
}

#[verifier::ext_equal]
pub ghost struct Rights {
    pub can_read: bool,
    pub can_write: bool,
    pub can_grant: bool,
    pub can_grant_reply: bool,
}

#[verifier::ext_equal]
pub ghost struct CNodeCapDataSpec {
    pub radix_bits: int,
    pub guard: int,
    pub guard_size: int,
}

#[verifier::ext_equal]
pub ghost struct UntypedCapDataSpec {
    pub block_size_bits: int,
    pub free_index: int,
    pub is_device: bool,
}

#[verifier::ext_equal]
pub ghost struct CapSpec {
    pub kind: CapKind,
    pub object: Option<ObjectRef>,
    pub region_id: Option<int>,
    pub rights: Rights,
    pub badge: Option<int>,
    pub cnode: Option<CNodeCapDataSpec>,
    pub untyped: Option<UntypedCapDataSpec>,
}

pub open spec fn spec_empty_rights() -> Rights {
    Rights {
        can_read: false,
        can_write: false,
        can_grant: false,
        can_grant_reply: false,
    }
}

pub open spec fn rights_subseteq(lhs: Rights, rhs: Rights) -> bool {
    &&& lhs.can_read ==> rhs.can_read
    &&& lhs.can_write ==> rhs.can_write
    &&& lhs.can_grant ==> rhs.can_grant
    &&& lhs.can_grant_reply ==> rhs.can_grant_reply
}

pub open spec fn spec_null_cap() -> CapSpec {
    CapSpec {
        kind: CapKind::NullCap,
        object: Option::None,
        region_id: Option::None,
        rights: spec_empty_rights(),
        badge: Option::None,
        cnode: Option::None,
        untyped: Option::None,
    }
}

pub open spec fn same_cap_except_untyped_free_index(old_cap: CapSpec, new_cap: CapSpec) -> bool {
    &&& new_cap.kind == old_cap.kind
    &&& new_cap.object == old_cap.object
    &&& new_cap.region_id == old_cap.region_id
    &&& new_cap.rights == old_cap.rights
    &&& new_cap.badge == old_cap.badge
    &&& new_cap.cnode == old_cap.cnode
    &&& old_cap.kind != CapKind::UntypedCap ==> new_cap == old_cap
    &&& old_cap.kind == CapKind::UntypedCap ==> {
        &&& new_cap.untyped is Some
        &&& old_cap.untyped is Some
        &&& new_cap.untyped.unwrap().block_size_bits
            == old_cap.untyped.unwrap().block_size_bits
        &&& new_cap.untyped.unwrap().is_device
            == old_cap.untyped.unwrap().is_device
    }
}

pub open spec fn cspace_endpoint_bits() -> int {
    4
}

pub open spec fn cspace_notification_bits() -> int {
    5
}

pub open spec fn cspace_slot_bits() -> int {
    5
}

pub open spec fn cspace_tcb_bits() -> int {
    10
}

pub open spec fn cspace_min_untyped_bits() -> int {
    4
}

pub open spec fn cspace_pow2(bits: nat) -> int
    decreases bits,
{
    if bits == 0 {
        1
    } else {
        2 * cspace_pow2((bits - 1) as nat)
    }
}

pub proof fn lemma_cspace_pow2_positive(bits: nat)
    ensures
        0 < cspace_pow2(bits),
    decreases bits,
{
    if bits != 0 {
        lemma_cspace_pow2_positive((bits - 1) as nat);
    }
}

pub open spec fn spec_extract_bits(value: int, start: int, width: int) -> int
    recommends
        0 <= value,
        0 <= start,
        0 <= width,
{
    (value / cspace_pow2(start as nat)) % cspace_pow2(width as nat)
}

pub proof fn lemma_extract_bits_range(value: int, start: int, width: int)
    requires
        0 <= value,
        0 <= start,
        0 <= width,
    ensures
        0 <= spec_extract_bits(value, start, width) < cspace_pow2(width as nat),
{
    lemma_cspace_pow2_positive(width as nat);
    vstd::arithmetic::div_mod::lemma_mod_bound(
        value / cspace_pow2(start as nat),
        cspace_pow2(width as nat),
    );
}

pub open spec fn spec_arch_is_physical_cap(cap: CapSpec) -> bool {
    cap.object is Some
}

pub open spec fn spec_arch_same_region_as_caps(lhs: CapSpec, rhs: CapSpec) -> bool {
    lhs.region_id is Some && lhs.region_id == rhs.region_id
}

pub open spec fn spec_arch_same_object_as_caps(lhs: CapSpec, rhs: CapSpec) -> bool {
    spec_same_object_ref(lhs, rhs)
}

pub open spec fn spec_is_physical_cap(cap: CapSpec) -> bool {
    match cap.kind {
        CapKind::NullCap
        | CapKind::IRQControlCap
        | CapKind::IRQHandlerCap
        | CapKind::ReplyCap => false,
        CapKind::ArchCap => spec_arch_is_physical_cap(cap),
        _ => true,
    }
}

pub open spec fn spec_same_object_ref(lhs: CapSpec, rhs: CapSpec) -> bool {
    lhs.object is Some && rhs.object is Some && lhs.object == rhs.object
}

pub open spec fn spec_cap_size_bits(cap: CapSpec) -> int {
    match cap.kind {
        CapKind::UntypedCap =>
            if cap.untyped is Some {
                cap.untyped.unwrap().block_size_bits
            } else {
                0
            },
        CapKind::EndpointCap => cspace_endpoint_bits(),
        CapKind::NotificationCap => cspace_notification_bits(),
        CapKind::CNodeCap =>
            if cap.cnode is Some {
                cap.cnode.unwrap().radix_bits + cspace_slot_bits()
            } else {
                0
            },
        CapKind::ThreadCap => cspace_tcb_bits(),
        _ => 0,
    }
}

pub open spec fn spec_cap_range_top(cap: CapSpec) -> int {
    if cap.object is Some {
        let base = cap.object.unwrap().id;
        let bits = spec_cap_size_bits(cap);
        if 0 <= bits {
            base + cspace_pow2(bits as nat) - 1
        } else {
            base - 1
        }
    } else {
        -1
    }
}

pub open spec fn spec_untyped_cap_contains_cap(lhs: CapSpec, rhs: CapSpec) -> bool {
    &&& lhs.kind == CapKind::UntypedCap
    &&& lhs.object is Some
    &&& lhs.untyped is Some
    &&& cspace_min_untyped_bits() <= lhs.untyped.unwrap().block_size_bits
    &&& spec_is_physical_cap(rhs)
    &&& rhs.object is Some
    &&& {
        let base = lhs.object.unwrap().id;
        let top = base + cspace_pow2(lhs.untyped.unwrap().block_size_bits as nat) - 1;
        let rhs_base = rhs.object.unwrap().id;
        let rhs_top = spec_cap_range_top(rhs);
        &&& base <= rhs_base
        &&& rhs_base <= rhs_top
        &&& rhs_top <= top
    }
}

pub open spec fn spec_same_region_as_caps(lhs: CapSpec, rhs: CapSpec) -> bool {
    match lhs.kind {
        CapKind::UntypedCap => spec_untyped_cap_contains_cap(lhs, rhs),
        CapKind::EndpointCap
        | CapKind::NotificationCap
        | CapKind::ThreadCap
        | CapKind::ReplyCap => lhs.kind == rhs.kind && spec_same_object_ref(lhs, rhs),
        CapKind::CNodeCap => {
            &&& rhs.kind == CapKind::CNodeCap
            &&& spec_same_object_ref(lhs, rhs)
            &&& lhs.cnode is Some
            &&& rhs.cnode is Some
            &&& lhs.cnode.unwrap().radix_bits == rhs.cnode.unwrap().radix_bits
        }
        CapKind::IRQControlCap => {
            rhs.kind == CapKind::IRQControlCap || rhs.kind == CapKind::IRQHandlerCap
        }
        CapKind::IRQHandlerCap => {
            rhs.kind == CapKind::IRQHandlerCap && spec_same_object_ref(lhs, rhs)
        }
        CapKind::ArchCap => {
            rhs.kind == CapKind::ArchCap && spec_arch_same_region_as_caps(lhs, rhs)
        }
        _ => false,
    }
}

pub open spec fn spec_same_object_as_caps(lhs: CapSpec, rhs: CapSpec) -> bool {
    if lhs.kind == CapKind::UntypedCap || lhs.kind == CapKind::IRQControlCap {
        false
    } else if lhs.kind == CapKind::ArchCap && rhs.kind == CapKind::ArchCap {
        spec_arch_same_object_as_caps(lhs, rhs)
    } else {
        spec_same_region_as_caps(lhs, rhs)
    }
}

pub proof fn lemma_same_region_after_untyped_full(old_cap: CapSpec, new_cap: CapSpec, rhs: CapSpec)
    requires
        old_cap.kind == new_cap.kind,
        old_cap.object == new_cap.object,
        old_cap.region_id == new_cap.region_id,
        old_cap.rights == new_cap.rights,
        old_cap.badge == new_cap.badge,
        old_cap.cnode == new_cap.cnode,
        old_cap.kind != CapKind::UntypedCap ==> new_cap == old_cap,
        old_cap.kind == CapKind::UntypedCap ==> {
            &&& old_cap.untyped is Some
            &&& new_cap.untyped is Some
            &&& old_cap.untyped.unwrap().block_size_bits == new_cap.untyped.unwrap().block_size_bits
            &&& old_cap.untyped.unwrap().is_device == new_cap.untyped.unwrap().is_device
        },
        spec_same_region_as_caps(old_cap, rhs),
    ensures
        spec_same_region_as_caps(new_cap, rhs),
{
    if old_cap.kind == CapKind::UntypedCap {
        assert(new_cap.kind == CapKind::UntypedCap);
        assert(new_cap.object is Some == old_cap.object is Some);
    } else {
        assert(new_cap == old_cap);
    }
}

pub proof fn lemma_same_region_preserved_when_rhs_same_except_untyped(
    lhs: CapSpec,
    old_rhs: CapSpec,
    new_rhs: CapSpec,
)
    requires
        same_cap_except_untyped_free_index(old_rhs, new_rhs),
        spec_same_region_as_caps(lhs, old_rhs),
    ensures
        spec_same_region_as_caps(lhs, new_rhs),
{
    match lhs.kind {
        CapKind::UntypedCap => {
            assert(old_rhs.object is Some);
            assert(new_rhs.object == old_rhs.object);
            assert(new_rhs.kind == old_rhs.kind);
            assert(spec_cap_range_top(new_rhs) == spec_cap_range_top(old_rhs));
        }
        CapKind::EndpointCap
        | CapKind::NotificationCap
        | CapKind::ThreadCap
        | CapKind::ReplyCap => {
            assert(old_rhs.kind == lhs.kind);
            assert(new_rhs.kind == old_rhs.kind);
            assert(new_rhs.object == old_rhs.object);
        }
        CapKind::CNodeCap => {
            assert(old_rhs.kind == CapKind::CNodeCap);
            assert(new_rhs.kind == CapKind::CNodeCap);
            assert(new_rhs.object == old_rhs.object);
            assert(old_rhs.cnode is Some);
            assert(new_rhs.cnode is Some);
            assert(new_rhs.cnode.unwrap().radix_bits == old_rhs.cnode.unwrap().radix_bits);
        }
        CapKind::IRQControlCap => {
            assert(old_rhs.kind == CapKind::IRQControlCap || old_rhs.kind == CapKind::IRQHandlerCap);
            assert(new_rhs.kind == old_rhs.kind);
        }
        CapKind::IRQHandlerCap => {
            assert(old_rhs.kind == CapKind::IRQHandlerCap);
            assert(new_rhs.kind == CapKind::IRQHandlerCap);
            assert(new_rhs.object == old_rhs.object);
        }
        CapKind::ArchCap => {
            assert(old_rhs.kind == CapKind::ArchCap);
            assert(new_rhs.kind == CapKind::ArchCap);
            assert(new_rhs.region_id == old_rhs.region_id);
        }
        CapKind::NullCap | CapKind::ZombieCap | CapKind::Other => {
            assert(false);
        }
    }
}

pub proof fn lemma_same_region_caps_non_null(lhs: CapSpec, rhs: CapSpec)
    requires
        spec_same_region_as_caps(lhs, rhs),
    ensures
        lhs.kind != CapKind::NullCap,
        rhs.kind != CapKind::NullCap,
{
    match lhs.kind {
        CapKind::UntypedCap => {
            assert(rhs.object is Some);
        }
        CapKind::EndpointCap
        | CapKind::NotificationCap
        | CapKind::ThreadCap
        | CapKind::ReplyCap => {
            assert(rhs.kind == lhs.kind);
        }
        CapKind::CNodeCap => {
            assert(rhs.kind == CapKind::CNodeCap);
        }
        CapKind::IRQControlCap => {
            assert(rhs.kind == CapKind::IRQControlCap || rhs.kind == CapKind::IRQHandlerCap);
        }
        CapKind::IRQHandlerCap => {
            assert(rhs.kind == CapKind::IRQHandlerCap);
        }
        CapKind::ArchCap => {
            assert(rhs.kind == CapKind::ArchCap);
        }
        CapKind::NullCap | CapKind::ZombieCap | CapKind::Other => {
            assert(false);
        }
    }
}

pub proof fn lemma_same_region_transitive_except_untyped_arch(
    lhs: CapSpec,
    mid: CapSpec,
    rhs: CapSpec,
)
    requires
        spec_same_region_as_caps(lhs, mid),
        spec_same_region_as_caps(mid, rhs),
        !(lhs.kind == CapKind::UntypedCap && mid.kind == CapKind::ArchCap),
    ensures
        spec_same_region_as_caps(lhs, rhs),
{
    match lhs.kind {
        CapKind::UntypedCap => {
            assert(mid.kind != CapKind::ArchCap);
            let lhs_base = lhs.object.unwrap().id;
            let lhs_top = lhs_base + cspace_pow2(lhs.untyped.unwrap().block_size_bits as nat) - 1;
            let mid_base = mid.object.unwrap().id;
            let mid_top = spec_cap_range_top(mid);
            if mid.kind == CapKind::UntypedCap {
                let rhs_base = rhs.object.unwrap().id;
                let rhs_top = spec_cap_range_top(rhs);
                assert(lhs_base <= mid_base);
                assert(mid_base <= rhs_base);
                assert(rhs_top <= spec_cap_range_top(mid));
                assert(spec_cap_range_top(mid) <= lhs_top);
                assert(lhs_base <= rhs_base);
                assert(rhs_top <= lhs_top);
            } else if mid.kind == CapKind::EndpointCap
                || mid.kind == CapKind::NotificationCap
                || mid.kind == CapKind::ThreadCap
                || mid.kind == CapKind::CNodeCap
                || mid.kind == CapKind::IRQHandlerCap
            {
                assert(rhs.kind == mid.kind);
                assert(rhs.object == mid.object);
                if mid.kind == CapKind::CNodeCap {
                    assert(rhs.cnode is Some);
                    assert(mid.cnode is Some);
                    assert(rhs.cnode.unwrap().radix_bits == mid.cnode.unwrap().radix_bits);
                }
                assert(spec_cap_range_top(rhs) == mid_top);
                assert(lhs_base <= mid_base);
                assert(mid_top <= lhs_top);
                assert(lhs_base <= rhs.object.unwrap().id);
                assert(spec_cap_range_top(rhs) <= lhs_top);
            } else {
                assert(false);
            }
        }
        CapKind::EndpointCap
        | CapKind::NotificationCap
        | CapKind::ThreadCap
        | CapKind::ReplyCap => {
            assert(mid.kind == lhs.kind);
            assert(rhs.kind == mid.kind);
            assert(lhs.object == mid.object);
            assert(mid.object == rhs.object);
            assert(lhs.object == rhs.object);
        }
        CapKind::CNodeCap => {
            assert(mid.kind == CapKind::CNodeCap);
            assert(rhs.kind == CapKind::CNodeCap);
            assert(lhs.object == mid.object);
            assert(mid.object == rhs.object);
            assert(lhs.cnode is Some);
            assert(mid.cnode is Some);
            assert(rhs.cnode is Some);
            assert(lhs.cnode.unwrap().radix_bits == mid.cnode.unwrap().radix_bits);
            assert(mid.cnode.unwrap().radix_bits == rhs.cnode.unwrap().radix_bits);
            assert(lhs.cnode.unwrap().radix_bits == rhs.cnode.unwrap().radix_bits);
        }
        CapKind::IRQControlCap => {
            if mid.kind == CapKind::IRQControlCap {
                assert(rhs.kind == CapKind::IRQControlCap || rhs.kind == CapKind::IRQHandlerCap);
            } else {
                assert(mid.kind == CapKind::IRQHandlerCap);
                assert(rhs.kind == CapKind::IRQHandlerCap);
            }
        }
        CapKind::IRQHandlerCap => {
            assert(mid.kind == CapKind::IRQHandlerCap);
            assert(rhs.kind == CapKind::IRQHandlerCap);
            assert(lhs.object == mid.object);
            assert(mid.object == rhs.object);
            assert(lhs.object == rhs.object);
        }
        CapKind::ArchCap => {
            assert(mid.kind == CapKind::ArchCap);
            assert(rhs.kind == CapKind::ArchCap);
            assert(lhs.region_id is Some);
            assert(lhs.region_id == mid.region_id);
            assert(mid.region_id == rhs.region_id);
            assert(lhs.region_id == rhs.region_id);
        }
        CapKind::NullCap | CapKind::ZombieCap | CapKind::Other => {
            assert(false);
        }
    }
}

pub open spec fn spec_is_cap_revocable(derived_cap: CapSpec, src_cap: CapSpec) -> bool {
    match derived_cap.kind {
        CapKind::EndpointCap | CapKind::NotificationCap => derived_cap.badge != src_cap.badge,
        CapKind::IRQHandlerCap => src_cap.kind == CapKind::IRQControlCap,
        CapKind::UntypedCap => true,
        _ => false,
    }
}

pub open spec fn spec_cap_badge_value(cap: CapSpec) -> int {
    if cap.badge is Some {
        cap.badge.unwrap()
    } else {
        0
    }
}

} // verus!
