use vstd::prelude::*;

verus! {

pub type ObjId = nat;
pub type SlotId = nat;
pub type Irq = nat;

pub struct CapRights {
    pub allow_write: bool,
    pub allow_read: bool,
    pub allow_grant: bool,
    pub allow_grant_reply: bool,
}

pub enum ArchCapability {
    Placeholder,
}

pub enum Capability {
    NullCap,
    UntypedCap {
        is_device: bool,
        obj_ptr: ObjId,
        block_size: nat,
        free_index: nat,
    },
    EndpointCap {
        ep_ptr: ObjId,
        badge: nat,
        rights: CapRights,
    },
    NotificationCap {
        ntfn_ptr: ObjId,
        badge: nat,
        rights: CapRights,
    },
    ReplyCap {
        reply_ptr: ObjId,
        master: bool,
        rights: CapRights,
    },
    CNodeCap {
        cnode_ptr: SlotId,
        radix_bits: nat,
        guard: nat,
        guard_size: nat,
    },
    ThreadCap {
        tcb_ptr: ObjId,
    },
    DomainCap,
    IRQControlCap,
    IRQHandlerCap {
        irq: Irq,
    },
    ZombieCap {
        zombie_ptr: SlotId,
        zombie_number: nat,
        zombie_bits: nat,
    },
    ArchObjectCap {
        arch: ArchCapability,
    },
}

pub struct MDBNode {
    pub next: SlotId,
    pub revocable: bool,
    pub first_badged: bool,
    pub prev: SlotId,
}

pub struct CTE {
    pub capability: Capability,
    pub mdb_node: MDBNode,
}

pub struct ResolveAddressBitsResult {
    pub slot: SlotId,
    pub bits_remaining: nat,
    pub success: bool,
}

pub enum Exception {
    None,
    SyscallError,
    LookupFault,
}

pub struct DeriveCapResult {
    pub status: Exception,
    pub capability: Capability,
}

pub struct AbsState {
    pub ctes: Map<SlotId, CTE>,
}

pub open spec fn null_slot() -> SlotId {
    0
}

pub open spec fn no_rights() -> CapRights {
    CapRights {
        allow_write: false,
        allow_read: false,
        allow_grant: false,
        allow_grant_reply: false,
    }
}

pub open spec fn all_rights() -> CapRights {
    CapRights {
        allow_write: true,
        allow_read: true,
        allow_grant: true,
        allow_grant_reply: true,
    }
}

pub open spec fn null_cap() -> Capability {
    Capability::NullCap
}

pub open spec fn null_mdb_node() -> MDBNode {
    MDBNode {
        next: null_slot(),
        revocable: false,
        first_badged: false,
        prev: null_slot(),
    }
}

pub open spec fn null_cte() -> CTE {
    CTE {
        capability: null_cap(),
        mdb_node: null_mdb_node(),
    }
}

pub open spec fn empty_abs_state() -> AbsState {
    AbsState {
        ctes: Map::empty(),
    }
}

pub open spec fn pow2(bits: nat) -> nat
    decreases bits
{
    if bits == 0 { 1nat } else { 2nat * pow2((bits - 1) as nat) }
}

pub open spec fn get_cte(s: AbsState, slot: SlotId) -> CTE {
    if s.ctes.contains_key(slot) {
        s.ctes[slot]
    } else {
        null_cte()
    }
}

pub open spec fn get_cap(s: AbsState, slot: SlotId) -> Capability {
    get_cte(s, slot).capability
}

pub open spec fn slot_exists(s: AbsState, slot: SlotId) -> bool {
    s.ctes.contains_key(slot)
}

pub open spec fn slot_is_empty(s: AbsState, slot: SlotId) -> bool {
    &&& slot_exists(s, slot)
    &&& get_cap(s, slot) == Capability::NullCap
}

pub open spec fn slot_has_cap(s: AbsState, slot: SlotId) -> bool {
    &&& slot_exists(s, slot)
    &&& get_cap(s, slot) != Capability::NullCap
}

pub open spec fn slot_has_empty_mdb(s: AbsState, slot: SlotId) -> bool {
    &&& slot_exists(s, slot)
    &&& get_cte(s, slot).mdb_node == null_mdb_node()
}

pub open spec fn slot_is_free(s: AbsState, slot: SlotId) -> bool {
    &&& slot_is_empty(s, slot)
    &&& slot_has_empty_mdb(s, slot)
}

pub open spec fn cap_badge(cap: Capability) -> nat {
    match cap {
        Capability::EndpointCap { badge, .. } => badge,
        Capability::NotificationCap { badge, .. } => badge,
        _ => 0,
    }
}

pub open spec fn cap_ptr(cap: Capability) -> Option<nat> {
    match cap {
        Capability::UntypedCap { obj_ptr, .. } => Some(obj_ptr),
        Capability::EndpointCap { ep_ptr, .. } => Some(ep_ptr),
        Capability::NotificationCap { ntfn_ptr, .. } => Some(ntfn_ptr),
        Capability::ReplyCap { reply_ptr, .. } => Some(reply_ptr),
        Capability::CNodeCap { cnode_ptr, .. } => Some(cnode_ptr),
        Capability::ThreadCap { tcb_ptr } => Some(tcb_ptr),
        Capability::ZombieCap { zombie_ptr, .. } => Some(zombie_ptr),
        _ => None,
    }
}

pub open spec fn is_arch_cap(cap: Capability) -> bool {
    match cap {
        Capability::ArchObjectCap { .. } => true,
        _ => false,
    }
}

pub open spec fn is_untyped_cap(cap: Capability) -> bool {
    match cap {
        Capability::UntypedCap { .. } => true,
        _ => false,
    }
}

pub open spec fn is_reply_cap(cap: Capability) -> bool {
    match cap {
        Capability::ReplyCap { master, .. } => !master,
        _ => false,
    }
}

pub open spec fn is_zombie(cap: Capability) -> bool {
    match cap {
        Capability::ZombieCap { .. } => true,
        _ => false,
    }
}

pub open spec fn is_cnode_cap(cap: Capability) -> bool {
    match cap {
        Capability::CNodeCap { .. } => true,
        _ => false,
    }
}

pub open spec fn cap_size_bits(cap: Capability) -> nat {
    match cap {
        Capability::UntypedCap { block_size, .. } => block_size,
        Capability::EndpointCap { .. } => 1,
        Capability::NotificationCap { .. } => 1,
        Capability::CNodeCap { radix_bits, .. } => radix_bits,
        Capability::ReplyCap { .. } => 1,
        Capability::ThreadCap { .. } => 1,
        Capability::ZombieCap { .. } => 1,
        _ => 0,
    }
}

pub open spec fn is_physical_cap(cap: Capability) -> bool {
    match cap {
        Capability::NullCap => false,
        Capability::DomainCap => false,
        Capability::IRQControlCap => false,
        Capability::IRQHandlerCap { .. } => false,
        Capability::ArchObjectCap { .. } => false,
        _ =>
            match cap_ptr(cap) {
                Some(_) => true,
                None => false,
            },
    }
}

pub open spec fn same_region_as(lhs: Capability, rhs: Capability) -> bool {
    match lhs {
        Capability::UntypedCap { obj_ptr: lhs_ptr, block_size, .. } =>
            if is_physical_cap(rhs) {
                match cap_ptr(rhs) {
                    Some(rhs_ptr) => {
                        let lhs_top = lhs_ptr + pow2(block_size) - 1;
                        let rhs_top = rhs_ptr + pow2(cap_size_bits(rhs)) - 1;
                        &&& lhs_ptr <= rhs_ptr
                        &&& rhs_top <= lhs_top
                        &&& rhs_ptr <= rhs_top
                    }
                    None => false,
                }
            } else {
                false
            },
        Capability::EndpointCap { ep_ptr: lhs_ptr, .. } =>
            match rhs {
                Capability::EndpointCap { ep_ptr: rhs_ptr, .. } => lhs_ptr == rhs_ptr,
                _ => false,
            },
        Capability::NotificationCap { ntfn_ptr: lhs_ptr, .. } =>
            match rhs {
                Capability::NotificationCap { ntfn_ptr: rhs_ptr, .. } => lhs_ptr == rhs_ptr,
                _ => false,
            },
        Capability::CNodeCap { cnode_ptr: lhs_ptr, radix_bits: lhs_bits, .. } =>
            match rhs {
                Capability::CNodeCap { cnode_ptr: rhs_ptr, radix_bits: rhs_bits, .. } =>
                    lhs_ptr == rhs_ptr && lhs_bits == rhs_bits,
                _ => false,
            },
        Capability::ThreadCap { tcb_ptr: lhs_ptr } =>
            match rhs {
                Capability::ThreadCap { tcb_ptr: rhs_ptr } => lhs_ptr == rhs_ptr,
                _ => false,
            },
        Capability::ReplyCap { reply_ptr: lhs_ptr, .. } =>
            match rhs {
                Capability::ReplyCap { reply_ptr: rhs_ptr, .. } => lhs_ptr == rhs_ptr,
                _ => false,
            },
        Capability::DomainCap =>
            match rhs {
                Capability::DomainCap => true,
                _ => false,
            },
        Capability::IRQControlCap =>
            match rhs {
                Capability::IRQControlCap => true,
                Capability::IRQHandlerCap { .. } => true,
                _ => false,
            },
        Capability::IRQHandlerCap { irq: lhs_irq } =>
            match rhs {
                Capability::IRQHandlerCap { irq: rhs_irq } => lhs_irq == rhs_irq,
                _ => false,
            },
        Capability::ArchObjectCap { .. } =>
            match rhs {
                Capability::ArchObjectCap { .. } => true,
                _ => false,
            },
        _ => false,
    }
}

pub open spec fn same_object_as(lhs: Capability, rhs: Capability) -> bool {
    match lhs {
        Capability::UntypedCap { .. } => false,
        Capability::IRQControlCap => false,
        _ => same_region_as(lhs, rhs),
    }
}

pub open spec fn is_cap_revocable(new_cap: Capability, src_cap: Capability) -> bool {
    match new_cap {
        Capability::EndpointCap { .. } => cap_badge(new_cap) != cap_badge(src_cap),
        Capability::NotificationCap { .. } => cap_badge(new_cap) != cap_badge(src_cap),
        Capability::IRQHandlerCap { .. } =>
            match src_cap {
                Capability::IRQControlCap => true,
                _ => false,
            },
        Capability::UntypedCap { .. } => true,
        Capability::ArchObjectCap { .. } => false,
        _ => false,
    }
}

pub open spec fn is_mdb_parent_of(parent: CTE, next: CTE) -> bool {
    if !parent.mdb_node.revocable {
        false
    } else if !same_region_as(parent.capability, next.capability) {
        false
    } else {
        match parent.capability {
            Capability::EndpointCap { badge, .. } =>
                if badge == 0 {
                    true
                } else {
                    match next.capability {
                        Capability::EndpointCap { badge: next_badge, .. } =>
                            badge == next_badge && !next.mdb_node.first_badged,
                        _ => false,
                    }
                },
            Capability::NotificationCap { badge, .. } =>
                if badge == 0 {
                    true
                } else {
                    match next.capability {
                        Capability::NotificationCap { badge: next_badge, .. } =>
                            badge == next_badge && !next.mdb_node.first_badged,
                        _ => false,
                    }
                },
            _ => true,
        }
    }
}

pub open spec fn has_mdb_child(s: AbsState, slot: SlotId) -> bool {
    let cte = get_cte(s, slot);
    let next = cte.mdb_node.next;
    &&& next != null_slot()
    &&& slot_exists(s, next)
    &&& is_mdb_parent_of(cte, get_cte(s, next))
}

pub open spec fn is_final_cap(s: AbsState, slot: SlotId) -> bool {
    let cte = get_cte(s, slot);
    let mdb = cte.mdb_node;
    let prev_same =
        if mdb.prev == null_slot() {
            false
        } else {
            slot_exists(s, mdb.prev) && same_object_as(get_cap(s, mdb.prev), cte.capability)
        };
    if prev_same {
        false
    } else if mdb.next == null_slot() {
        true
    } else {
        slot_exists(s, mdb.next) && !same_object_as(cte.capability, get_cap(s, mdb.next))
    }
}

pub open spec fn is_long_running_delete(s: AbsState, slot: SlotId) -> bool {
    let cap = get_cap(s, slot);
    if cap == Capability::NullCap || !is_final_cap(s, slot) {
        false
    } else {
        match cap {
            Capability::ThreadCap { .. } => true,
            Capability::ZombieCap { .. } => true,
            Capability::CNodeCap { .. } => true,
            _ => false,
        }
    }
}

}
