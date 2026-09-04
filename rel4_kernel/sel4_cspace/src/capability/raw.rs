use crate::capability::zombie::zombie_func;
#[cfg(verus_keep_ghost)]
use crate::capability::spec::{
    cspace_slot_bits, spec_cap_range_top, spec_is_physical_cap, spec_null_cap, CNodeCapDataSpec,
    CapKind, CapSpec, ObjectKind, ObjectRef,
};
#[cfg(verus_keep_ghost)]
use crate::cspace::types::SlotPtr;
use sel4_common::shared_types_bf_gen::seL4_CapRights;
use sel4_common::structures_gen::{cap, cap_null_cap, cap_zombie_cap};
use vstd::prelude::*;

verus! {

const TAG_NULL: u64 = 0;
const TAG_UNTYPED: u64 = 2;
const TAG_ENDPOINT: u64 = 4;
const TAG_NOTIFICATION: u64 = 6;
const TAG_REPLY: u64 = 8;
const TAG_CNODE: u64 = 10;
const TAG_THREAD: u64 = 12;
const TAG_IRQ_CONTROL: u64 = 14;
const TAG_IRQ_HANDLER: u64 = 16;
const TAG_ZOMBIE: u64 = 18;

#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExCap(cap);

#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExCapNullCap(cap_null_cap);

#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExCapZombieCap(cap_zombie_cap);

#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExCapRights(seL4_CapRights);

pub uninterp spec fn trusted_view_cap(raw: &cap) -> CapSpec;

pub uninterp spec fn spec_runtime_cap_tag(raw: &cap) -> u64;

pub uninterp spec fn spec_runtime_null_cap_tag() -> u64;

pub uninterp spec fn spec_zombie_type_cap(capability: CapSpec) -> usize;

pub uninterp spec fn spec_zombie_ptr_cap(capability: CapSpec) -> SlotPtr;

pub uninterp spec fn spec_zombie_number_cap(capability: CapSpec) -> usize;

pub open spec fn spec_cap_cyclic_zombie(capability: CapSpec, slot: SlotPtr) -> bool {
    capability.kind == CapKind::ZombieCap && spec_zombie_ptr_cap(capability) == slot
}

pub open spec fn spec_same_zombie_shape(lhs: CapSpec, rhs: CapSpec) -> bool {
    &&& spec_zombie_ptr_cap(lhs) == spec_zombie_ptr_cap(rhs)
    &&& spec_zombie_number_cap(lhs) == spec_zombie_number_cap(rhs)
    &&& spec_zombie_type_cap(lhs) == spec_zombie_type_cap(rhs)
}

pub open spec fn trusted_zombie_end_slot_of_cap(capability: CapSpec) -> SlotPtr {
    let n = spec_zombie_number_cap(capability);
    let ptr = spec_zombie_ptr_cap(capability);
    if n == 0 {
        ptr
    } else {
        (((ptr as int) + ((n - 1) as int) * (core::mem::size_of::<crate::cspace::cte::cte_t>() as int))
            as usize)
    }
}

pub open spec fn spec_cap_removable(capability: CapSpec, slot: SlotPtr) -> bool {
    capability.kind == CapKind::NullCap || (capability.kind == CapKind::ZombieCap
        && (spec_zombie_number_cap(capability) == 0 || (spec_zombie_number_cap(capability) == 1
            && spec_zombie_ptr_cap(capability) == slot)))
}

pub open spec fn spec_decremented_same_zombie(new_cap: CapSpec, old_cap: CapSpec) -> bool {
    &&& new_cap.kind == CapKind::ZombieCap
    &&& old_cap.kind == CapKind::ZombieCap
    &&& spec_zombie_ptr_cap(new_cap) == spec_zombie_ptr_cap(old_cap)
    &&& spec_zombie_type_cap(new_cap) == spec_zombie_type_cap(old_cap)
    &&& spec_zombie_number_cap(new_cap) + 1 == spec_zombie_number_cap(old_cap)
}

pub open spec fn spec_reduce_zombie_immediate_slot_result(
    old_cap: CapSpec,
    new_cap: CapSpec,
    slot_empty: bool,
    slot: SlotPtr,
) -> bool {
    slot_empty || spec_decremented_same_zombie(new_cap, old_cap)
        || spec_cap_cyclic_zombie(new_cap, slot)
}

#[verifier::external_body]
pub fn runtime_null_cap() -> (ret: cap)
    ensures
        trusted_view_cap(&ret) == spec_null_cap(),
        trusted_view_cap(&ret).kind == crate::capability::spec::CapKind::NullCap,
{
    cap_null_cap::new().unsplay()
}

#[verifier::external_body]
pub fn runtime_clone_cap(raw: &cap) -> (ret: cap)
    ensures
        trusted_view_cap(&ret) == trusted_view_cap(raw),
{
    raw.clone()
}

#[verifier::external_body]
pub fn runtime_cap_tag(raw: &cap) -> (ret: u64)
    ensures
        ret == spec_runtime_cap_tag(raw),
        ret == spec_runtime_null_cap_tag() <==> trusted_view_cap(raw).kind
            == crate::capability::spec::CapKind::NullCap,
{
    raw.get_tag()
}

#[verifier::external_body]
pub fn runtime_cap_is_arch(raw: &cap) -> (ret: bool)
    ensures
        ret == (trusted_view_cap(raw).kind == CapKind::ArchCap),
{
    raw.get_tag() % 2 == 1
}

#[verifier::external_body]
pub fn runtime_cap_is_physical(raw: &cap) -> (ret: bool)
    ensures
        ret == spec_is_physical_cap(trusted_view_cap(raw)),
{
    match raw.get_tag() {
        2 | 4 | 6 | 10 | 12 | 18 | 1 | 3 | 13 => true,
        _ => false,
    }
}

#[verifier::external_body]
pub fn runtime_null_cap_tag() -> (ret: u64)
    ensures
        ret == spec_runtime_null_cap_tag(),
{
    cap_null_cap::new().unsplay().get_tag()
}

#[verifier::external_body]
pub fn runtime_mask_bits(width: usize) -> (ret: usize) {
    if width == 0usize {
        0usize
    } else if width < 64usize {
        (1usize << width) - 1usize
    } else {
        usize::MAX
    }
}

#[verifier::external_body]
pub fn runtime_zombie_cap_new(raw_id: u64, zombie_type: u64) -> (ret: cap) {
    cap_zombie_cap::new(raw_id, zombie_type).unsplay()
}

#[verifier::external_body]
pub fn runtime_raw_zombie_bit(raw: &cap_zombie_cap) -> (ret: usize)
    ensures
        ret == crate::capability::zombie::spec_zombie_bit_raw(raw),
{
    let zombie_type = raw.get_capZombieType() as usize;
    if zombie_type == crate::capability::zombie::ZOMBIE_TYPE_ZOMBIE_TCB {
        crate::capability::zombie::TCB_CNODE_RADIX
    } else {
        zombie_type & mask_bits!(crate::capability::zombie::VERIFIED_CSPACE_WORD_RADIX)
    }
}

#[verifier::external_body]
pub fn runtime_raw_zombie_ptr(raw: &cap_zombie_cap) -> (ret: usize)
    ensures
        ret == crate::capability::zombie::spec_zombie_ptr_raw(raw),
{
    let radix = runtime_raw_zombie_bit(raw);
    raw.get_capZombieID() as usize & !mask_bits!(radix + 1)
}

#[verifier::external_body]
pub fn runtime_raw_zombie_number(raw: &cap_zombie_cap) -> (ret: usize)
    ensures
        ret == crate::capability::zombie::spec_zombie_number_raw(raw),
{
    let radix = runtime_raw_zombie_bit(raw);
    raw.get_capZombieID() as usize & mask_bits!(radix + 1)
}

#[verifier::external_body]
pub fn runtime_raw_zombie_set_number(raw: &mut cap_zombie_cap, n: usize)
    ensures
        crate::capability::zombie::spec_zombie_bit_raw(raw)
            == crate::capability::zombie::spec_zombie_bit_raw(old(raw)),
        crate::capability::zombie::spec_zombie_ptr_raw(raw)
            == crate::capability::zombie::spec_zombie_ptr_raw(old(raw)),
        crate::capability::zombie::spec_zombie_number_raw(raw) == n,
{
    let radix = runtime_raw_zombie_bit(raw);
    let ptr = raw.get_capZombieID() as usize & !mask_bits!(radix + 1);
    raw.set_capZombieID((ptr | (n & mask_bits!(radix + 1))) as u64);
}

#[verifier::external_body]
pub proof fn lemma_trusted_view_cap_kind_matches_tag(raw: &cap)
    ensures
        spec_runtime_cap_tag(raw) == 0 <==> trusted_view_cap(raw).kind == CapKind::NullCap,
        spec_runtime_cap_tag(raw) == 2 <==> trusted_view_cap(raw).kind == CapKind::UntypedCap,
        spec_runtime_cap_tag(raw) == 4 <==> trusted_view_cap(raw).kind == CapKind::EndpointCap,
        spec_runtime_cap_tag(raw) == 6 <==> trusted_view_cap(raw).kind == CapKind::NotificationCap,
        spec_runtime_cap_tag(raw) == 10 <==> trusted_view_cap(raw).kind == CapKind::CNodeCap,
        spec_runtime_cap_tag(raw) == 12 <==> trusted_view_cap(raw).kind == CapKind::ThreadCap,
        spec_runtime_cap_tag(raw) == 8 <==> trusted_view_cap(raw).kind == CapKind::ReplyCap,
        spec_runtime_cap_tag(raw) == 16 <==> trusted_view_cap(raw).kind == CapKind::IRQHandlerCap,
        spec_runtime_cap_tag(raw) == 18 <==> trusted_view_cap(raw).kind == CapKind::ZombieCap,
        spec_runtime_cap_tag(raw) == 14 <==> trusted_view_cap(raw).kind == CapKind::IRQControlCap,
        spec_runtime_cap_tag(raw) % 2 == 1 <==> trusted_view_cap(raw).kind == CapKind::ArchCap,
{
}

#[verifier::external_body]
pub proof fn lemma_trusted_view_cap_region_matches_object(raw: &cap)
    ensures
        trusted_view_cap(raw).object is Some ==> trusted_view_cap(raw).region_id
            == Some(trusted_view_cap(raw).object.unwrap().id),
{
}

#[verifier::external_body]
pub proof fn lemma_trusted_view_cap_badge_shape(raw: &cap)
    ensures
        trusted_view_cap(raw).kind == CapKind::EndpointCap ==> trusted_view_cap(raw).badge is Some,
        trusted_view_cap(raw).kind == CapKind::NotificationCap ==> trusted_view_cap(raw).badge is Some,
        trusted_view_cap(raw).kind != CapKind::EndpointCap
            && trusted_view_cap(raw).kind != CapKind::NotificationCap ==> trusted_view_cap(raw).badge is None,
{
}

#[verifier::external_body]
pub proof fn lemma_runtime_cap_tag_supported(raw: &cap)
    ensures
        spec_runtime_cap_tag(raw) == 0 || spec_runtime_cap_tag(raw) == 1 || spec_runtime_cap_tag(raw) == 2
            || spec_runtime_cap_tag(raw) == 3 || spec_runtime_cap_tag(raw) == 4
            || spec_runtime_cap_tag(raw) == 6 || spec_runtime_cap_tag(raw) == 8
            || spec_runtime_cap_tag(raw) == 10 || spec_runtime_cap_tag(raw) == 11
            || spec_runtime_cap_tag(raw) == 12 || spec_runtime_cap_tag(raw) == 13
            || spec_runtime_cap_tag(raw) == 14 || spec_runtime_cap_tag(raw) == 16
            || spec_runtime_cap_tag(raw) == 18 || spec_runtime_cap_tag(raw) == 20,
{
}

#[verifier::external_body]
pub proof fn lemma_trusted_view_cap_untyped_bounds(raw: &cap)
    ensures
        trusted_view_cap(raw).kind == CapKind::UntypedCap ==> {
            &&& trusted_view_cap(raw).untyped is Some
            &&& 4 <= trusted_view_cap(raw).untyped.unwrap().block_size_bits
        },
{
}

#[verifier::external_body]
pub proof fn lemma_trusted_view_cap_no_object_for_control_tags(raw: &cap)
    ensures
        spec_runtime_cap_tag(raw) == 0 ==> trusted_view_cap(raw).object is None,
        spec_runtime_cap_tag(raw) == 11 ==> trusted_view_cap(raw).object is None,
        spec_runtime_cap_tag(raw) == 14 ==> trusted_view_cap(raw).object is None,
        spec_runtime_cap_tag(raw) == 20 ==> trusted_view_cap(raw).object is None,
{
}

#[verifier::external_body]
pub fn runtime_cap_untyped_ptr(raw: &cap) -> (ret: usize)
    ensures
        trusted_view_cap(raw).kind == CapKind::UntypedCap ==> trusted_view_cap(raw).object == Some(
            ObjectRef { id: ret as int, kind: ObjectKind::Untyped },
        ),
{
    cap::cap_untyped_cap(raw).get_capPtr() as usize
}

#[verifier::external_body]
pub fn runtime_cap_untyped_block_size(raw: &cap) -> (ret: usize)
    ensures
        trusted_view_cap(raw).kind == CapKind::UntypedCap ==> {
            &&& trusted_view_cap(raw).untyped is Some
            &&& trusted_view_cap(raw).untyped.unwrap().block_size_bits == ret as int
        },
{
    cap::cap_untyped_cap(raw).get_capBlockSize() as usize
}

#[verifier::external_body]
pub fn runtime_cap_endpoint_ptr(raw: &cap) -> (ret: usize)
    ensures
        trusted_view_cap(raw).kind == CapKind::EndpointCap ==> trusted_view_cap(raw).object == Some(
            ObjectRef { id: ret as int, kind: ObjectKind::Endpoint },
        ),
{
    cap::cap_endpoint_cap(raw).get_capEPPtr() as usize
}

#[verifier::external_body]
pub fn runtime_cap_endpoint_badge(raw: &cap) -> (ret: u64)
    ensures
        trusted_view_cap(raw).kind == CapKind::EndpointCap ==> trusted_view_cap(raw).badge
            == Some(ret as int),
{
    cap::cap_endpoint_cap(raw).get_capEPBadge()
}

#[verifier::external_body]
pub fn runtime_endpoint_cap_set_badge(raw: &cap, badge: u64) -> (ret: cap)
    ensures
        trusted_view_cap(raw).kind == CapKind::EndpointCap ==> {
            &&& trusted_view_cap(&ret).kind == CapKind::EndpointCap
            &&& trusted_view_cap(&ret).object == trusted_view_cap(raw).object
            &&& trusted_view_cap(&ret).region_id == trusted_view_cap(raw).region_id
            &&& trusted_view_cap(&ret).rights == trusted_view_cap(raw).rights
            &&& trusted_view_cap(&ret).badge == Some(badge as int)
            &&& trusted_view_cap(&ret).cnode == trusted_view_cap(raw).cnode
            &&& trusted_view_cap(&ret).untyped == trusted_view_cap(raw).untyped
        },
{
    let new_cap = runtime_clone_cap(raw);
    cap::cap_endpoint_cap(&new_cap).set_capEPBadge(badge);
    new_cap
}

#[verifier::external_body]
pub fn runtime_cap_notification_ptr(raw: &cap) -> (ret: usize)
    ensures
        trusted_view_cap(raw).kind == CapKind::NotificationCap ==> trusted_view_cap(raw).object
            == Some(ObjectRef { id: ret as int, kind: ObjectKind::Notification }),
{
    cap::cap_notification_cap(raw).get_capNtfnPtr() as usize
}

#[verifier::external_body]
pub fn runtime_cap_notification_badge(raw: &cap) -> (ret: u64)
    ensures
        trusted_view_cap(raw).kind == CapKind::NotificationCap ==> trusted_view_cap(raw).badge
            == Some(ret as int),
{
    cap::cap_notification_cap(raw).get_capNtfnBadge()
}

#[verifier::external_body]
pub fn runtime_notification_cap_set_badge(raw: &cap, badge: u64) -> (ret: cap)
    ensures
        trusted_view_cap(raw).kind == CapKind::NotificationCap ==> {
            &&& trusted_view_cap(&ret).kind == CapKind::NotificationCap
            &&& trusted_view_cap(&ret).object == trusted_view_cap(raw).object
            &&& trusted_view_cap(&ret).region_id == trusted_view_cap(raw).region_id
            &&& trusted_view_cap(&ret).rights == trusted_view_cap(raw).rights
            &&& trusted_view_cap(&ret).badge == Some(badge as int)
            &&& trusted_view_cap(&ret).cnode == trusted_view_cap(raw).cnode
            &&& trusted_view_cap(&ret).untyped == trusted_view_cap(raw).untyped
        },
{
    let new_cap = runtime_clone_cap(raw);
    cap::cap_notification_cap(&new_cap).set_capNtfnBadge(badge);
    new_cap
}

#[verifier::external_body]
pub fn runtime_cap_cnode_ptr(raw: &cap) -> (ret: usize)
    ensures
        trusted_view_cap(raw).kind == CapKind::CNodeCap ==> trusted_view_cap(raw).object == Some(
            ObjectRef { id: ret as int, kind: ObjectKind::CNode },
        ),
{
    cap::cap_cnode_cap(raw).get_capCNodePtr() as usize
}

#[verifier::external_body]
pub fn runtime_cap_cnode_radix_bits(raw: &cap) -> (ret: usize)
    ensures
        trusted_view_cap(raw).kind == CapKind::CNodeCap ==> trusted_view_cap(raw).cnode is Some
            && trusted_view_cap(raw).cnode.unwrap().radix_bits == ret as int,
{
    cap::cap_cnode_cap(raw).get_capCNodeRadix() as usize
}

#[verifier::external_body]
pub fn runtime_cap_cnode_size_bits(raw: &cap) -> (ret: usize)
    ensures
        trusted_view_cap(raw).kind == CapKind::CNodeCap ==> {
            &&& trusted_view_cap(raw).cnode is Some
            &&& ret as int
                == trusted_view_cap(raw).cnode.unwrap().radix_bits + cspace_slot_bits()
        },
{
    cap::cap_cnode_cap(raw).get_capCNodeRadix() as usize + 5
}

#[verifier::external_body]
pub fn runtime_cnode_cap_set_guard(raw: &cap, guard: u64, guard_size: u64) -> (ret: cap)
    ensures
        trusted_view_cap(raw).kind == CapKind::CNodeCap ==> {
            &&& trusted_view_cap(&ret).kind == CapKind::CNodeCap
            &&& trusted_view_cap(&ret).object == trusted_view_cap(raw).object
            &&& trusted_view_cap(&ret).region_id == trusted_view_cap(raw).region_id
            &&& trusted_view_cap(&ret).rights == trusted_view_cap(raw).rights
            &&& trusted_view_cap(&ret).badge == trusted_view_cap(raw).badge
            &&& trusted_view_cap(&ret).untyped == trusted_view_cap(raw).untyped
            &&& trusted_view_cap(&ret).cnode is Some
            &&& trusted_view_cap(&ret).cnode.unwrap().guard == guard as int
            &&& trusted_view_cap(&ret).cnode.unwrap().guard_size == guard_size as int
        },
{
    let new_cap = runtime_clone_cap(raw);
    cap::cap_cnode_cap(&new_cap).set_capCNodeGuard(guard);
    cap::cap_cnode_cap(&new_cap).set_capCNodeGuardSize(guard_size);
    new_cap
}

#[verifier::external_body]
pub fn runtime_cap_thread_ptr(raw: &cap) -> (ret: usize)
    ensures
        trusted_view_cap(raw).kind == CapKind::ThreadCap ==> trusted_view_cap(raw).object == Some(
            ObjectRef { id: ret as int, kind: ObjectKind::Thread },
        ),
{
    cap::cap_thread_cap(raw).get_capTCBPtr() as usize
}

#[verifier::external_body]
pub fn runtime_cap_reply_ptr(raw: &cap) -> (ret: usize)
    ensures
        trusted_view_cap(raw).kind == CapKind::ReplyCap ==> trusted_view_cap(raw).object == Some(
            ObjectRef { id: ret as int, kind: ObjectKind::Reply },
        ),
{
    cap::cap_reply_cap(raw).get_capTCBPtr() as usize
}

#[verifier::external_body]
pub fn runtime_cap_irq(raw: &cap) -> (ret: usize)
    ensures
        trusted_view_cap(raw).kind == CapKind::IRQHandlerCap ==> trusted_view_cap(raw).object
            == Some(ObjectRef { id: ret as int, kind: ObjectKind::IRQ }),
{
    cap::cap_irq_handler_cap(raw).get_capIRQ() as usize
}

#[verifier::external_body]
pub fn runtime_cap_zombie_ptr(raw: &cap) -> (ret: usize)
    ensures
        trusted_view_cap(raw).kind == CapKind::ZombieCap ==> trusted_view_cap(raw).object == Some(
            ObjectRef { id: ret as int, kind: ObjectKind::Zombie },
        ),
        trusted_view_cap(raw).kind == CapKind::ZombieCap ==> spec_zombie_ptr_cap(
            trusted_view_cap(raw),
        ) == ret,
{
    runtime_raw_zombie_ptr(&cap::cap_zombie_cap(raw))
}

#[verifier::external_body]
pub fn runtime_cap_zombie_number(raw: &cap) -> (ret: usize)
    ensures
        trusted_view_cap(raw).kind == CapKind::ZombieCap ==> spec_zombie_number_cap(
            trusted_view_cap(raw),
        ) == ret,
{
    runtime_raw_zombie_number(&cap::cap_zombie_cap(raw))
}

#[verifier::external_body]
pub fn runtime_cap_zombie_type(raw: &cap) -> (ret: usize)
    ensures
        trusted_view_cap(raw).kind == CapKind::ZombieCap ==> spec_zombie_type_cap(
            trusted_view_cap(raw),
        ) == ret,
{
    cap::cap_zombie_cap(raw).get_capZombieType() as usize
}

    #[verifier::external_body]
    pub fn runtime_cap_zombie_end_slot(raw: &cap) -> (ret: usize)
        ensures
            trusted_view_cap(raw).kind == CapKind::ZombieCap ==> ret
                == trusted_zombie_end_slot_of_cap(trusted_view_cap(raw)),
    {
        let n = runtime_cap_zombie_number(raw);
        let ptr = runtime_cap_zombie_ptr(raw);
        if n == 0 {
        ptr
    } else {
        ptr + (n - 1) * core::mem::size_of::<crate::cspace::cte::cte_t>()
    }
}

#[verifier::external_body]
pub fn runtime_cap_set_zombie_number(raw: &cap, zombie_number: usize) -> (ret: cap)
    ensures
        trusted_view_cap(&ret).kind == trusted_view_cap(raw).kind,
        trusted_view_cap(&ret).object == trusted_view_cap(raw).object,
        trusted_view_cap(&ret).region_id == trusted_view_cap(raw).region_id,
        trusted_view_cap(&ret).rights == trusted_view_cap(raw).rights,
        trusted_view_cap(&ret).badge == trusted_view_cap(raw).badge,
        trusted_view_cap(&ret).cnode == trusted_view_cap(raw).cnode,
        trusted_view_cap(&ret).untyped == trusted_view_cap(raw).untyped,
        trusted_view_cap(raw).kind == CapKind::ZombieCap ==> spec_zombie_ptr_cap(trusted_view_cap(
            &ret,
        )) == spec_zombie_ptr_cap(trusted_view_cap(raw)),
        trusted_view_cap(raw).kind == CapKind::ZombieCap ==> spec_zombie_type_cap(
            trusted_view_cap(&ret),
        ) == spec_zombie_type_cap(trusted_view_cap(raw)),
        trusted_view_cap(raw).kind == CapKind::ZombieCap ==> spec_zombie_number_cap(
            trusted_view_cap(&ret),
        ) == zombie_number,
{
    let new_cap = runtime_clone_cap(raw);
    cap::cap_zombie_cap(&new_cap).set_zombie_number(zombie_number);
    new_cap
}

#[verifier::external_body]
pub fn runtime_cap_frame_ptr(raw: &cap) -> (ret: usize)
    ensures
        trusted_view_cap(raw).kind == CapKind::ArchCap ==> trusted_view_cap(raw).object == Some(
            ObjectRef { id: ret as int, kind: ObjectKind::Arch },
        ),
{
    cap::cap_frame_cap(raw).get_capFBasePtr() as usize
}

#[verifier::external_body]
pub fn runtime_cap_frame_size(raw: &cap) -> (ret: usize) {
    cap::cap_frame_cap(raw).get_capFSize() as usize
}

#[verifier::external_body]
pub fn runtime_cap_frame_is_device(raw: &cap) -> (ret: bool) {
    cap::cap_frame_cap(raw).get_capFIsDevice() == 0
}

#[verifier::external_body]
pub fn runtime_cap_frame_vm_rights(raw: &cap) -> (ret: usize) {
    cap::cap_frame_cap(raw).get_capFVMRights() as usize
}

#[verifier::external_body]
pub fn runtime_frame_cap_clear_mapping(raw: &cap) -> (ret: cap)
    ensures
        trusted_view_cap(&ret) == trusted_view_cap(raw),
{
    let new_cap = runtime_clone_cap(raw);
    cap::cap_frame_cap(&new_cap).set_capFMappedAddress(0);
    cap::cap_frame_cap(&new_cap).set_capFMappedASID(0);
    new_cap
}

#[verifier::external_body]
pub fn runtime_frame_cap_set_vm_rights(raw: &cap, vm_rights: u64) -> (ret: cap) {
    let new_cap = runtime_clone_cap(raw);
    cap::cap_frame_cap(&new_cap).set_capFVMRights(vm_rights);
    new_cap
}

#[verifier::external_body]
pub fn runtime_frame_cap_mask_vm_rights(raw: &cap, rights: seL4_CapRights) -> (ret: cap)
    ensures
        trusted_view_cap(&ret).kind == trusted_view_cap(raw).kind,
        trusted_view_cap(&ret).object == trusted_view_cap(raw).object,
        trusted_view_cap(&ret).region_id == trusted_view_cap(raw).region_id,
        trusted_view_cap(&ret).rights.can_grant == trusted_view_cap(raw).rights.can_grant,
        trusted_view_cap(&ret).rights.can_grant_reply
            == trusted_view_cap(raw).rights.can_grant_reply,
        trusted_view_cap(&ret).badge == trusted_view_cap(raw).badge,
        trusted_view_cap(&ret).cnode == trusted_view_cap(raw).cnode,
        trusted_view_cap(&ret).untyped == trusted_view_cap(raw).untyped,
{
    let mut vm_rights =
        sel4_common::vm_rights::vm_rights_from_word(cap::cap_frame_cap(raw).get_capFVMRights() as usize);
    vm_rights = sel4_common::arch::maskVMRights(vm_rights, rights);
    let new_cap = runtime_clone_cap(raw);
    cap::cap_frame_cap(&new_cap).set_capFVMRights(vm_rights as u64);
    new_cap
}

#[verifier::external_body]
pub fn runtime_cap_page_table_ptr(raw: &cap) -> (ret: usize)
    ensures
        trusted_view_cap(raw).kind == CapKind::ArchCap ==> trusted_view_cap(raw).object == Some(
            ObjectRef { id: ret as int, kind: ObjectKind::Arch },
        ),
{
    cap::cap_page_table_cap(raw).get_capPTBasePtr() as usize
}

#[verifier::external_body]
pub fn runtime_cap_page_table_is_mapped(raw: &cap) -> (ret: bool) {
    cap::cap_page_table_cap(raw).get_capPTIsMapped() != 0
}

#[cfg(target_arch = "aarch64")]
#[verifier::external_body]
pub fn runtime_cap_vspace_ptr(raw: &cap) -> (ret: usize)
    ensures
        trusted_view_cap(raw).kind == CapKind::ArchCap ==> trusted_view_cap(raw).object == Some(
            ObjectRef { id: ret as int, kind: ObjectKind::Arch },
        ),
{
    cap::cap_vspace_cap(raw).get_capVSBasePtr() as usize
}

#[verifier::external_body]
pub fn runtime_cap_asid_pool_ptr(raw: &cap) -> (ret: usize)
    ensures
        trusted_view_cap(raw).kind == CapKind::ArchCap ==> trusted_view_cap(raw).object == Some(
            ObjectRef { id: ret as int, kind: ObjectKind::Arch },
        ),
{
    cap::cap_asid_pool_cap(raw).get_capASIDPool() as usize
}

#[verifier::external_body]
pub fn runtime_cap_range_top(raw: &cap) -> (ret: usize)
    ensures
        trusted_view_cap(raw).object is Some ==> ret as int == spec_cap_range_top(trusted_view_cap(raw)),
{
    match raw.get_tag() {
        0 => 0,
        2 => {
            let base = cap::cap_untyped_cap(raw).get_capPtr() as usize;
            base + mask_bits!(cap::cap_untyped_cap(raw).get_capBlockSize() as usize)
        }
        4 => cap::cap_endpoint_cap(raw).get_capEPPtr() as usize + mask_bits!(4),
        6 => cap::cap_notification_cap(raw).get_capNtfnPtr() as usize + mask_bits!(5),
        8 => cap::cap_reply_cap(raw).get_capTCBPtr() as usize,
        10 => {
            let base = cap::cap_cnode_cap(raw).get_capCNodePtr() as usize;
            base + mask_bits!(cap::cap_cnode_cap(raw).get_capCNodeRadix() as usize + 5)
        }
        12 => cap::cap_thread_cap(raw).get_capTCBPtr() as usize + mask_bits!(10),
        16 => cap::cap_irq_handler_cap(raw).get_capIRQ() as usize,
        18 => runtime_cap_zombie_ptr(raw),
        1 => {
            let base = cap::cap_frame_cap(raw).get_capFBasePtr() as usize;
            base
                + mask_bits!(sel4_common::utils::pageBitsForSize(
                    cap::cap_frame_cap(raw).get_capFSize() as usize,
                ))
        }
        3 => cap::cap_page_table_cap(raw).get_capPTBasePtr() as usize + mask_bits!(12),
        13 => cap::cap_asid_pool_cap(raw).get_capASIDPool() as usize,
        _ => 0,
    }
}

#[cfg(feature = "kernel_mcs")]
#[verifier::external_body]
pub fn runtime_cap_sched_context_ptr(raw: &cap) -> (ret: usize) {
    cap::cap_sched_context_cap(raw).get_capSCPtr() as usize
}

#[cfg(feature = "kernel_mcs")]
#[verifier::external_body]
pub fn runtime_cap_sched_context_size_bits(raw: &cap) -> (ret: usize) {
    cap::cap_sched_context_cap(raw).get_capSCSizeBits() as usize
}

}
