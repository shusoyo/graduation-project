//! Capability helpers in a single Verus-oriented module.

pub mod raw;
pub mod spec;
pub mod zombie;

use crate::arch::{arch_same_object_as, arch_same_region_as};
#[cfg(verus_keep_ghost)]
use crate::capability::spec::{
    spec_is_cap_revocable, spec_is_physical_cap, spec_same_object_as_caps,
    spec_same_region_as_caps, spec_untyped_cap_contains_cap, CapKind,
};
use crate::capability::zombie::zombie_func;
#[cfg(feature = "kernel_mcs")]
use crate::capability::raw::runtime_cap_sched_context_size_bits;
#[cfg(verus_keep_ghost)]
use crate::capability::raw::{
    lemma_runtime_cap_tag_supported, lemma_trusted_view_cap_badge_shape,
    lemma_trusted_view_cap_kind_matches_tag, lemma_trusted_view_cap_no_object_for_control_tags,
    lemma_trusted_view_cap_region_matches_object, lemma_trusted_view_cap_untyped_bounds,
    spec_cap_removable, spec_zombie_number_cap, spec_zombie_ptr_cap, trusted_view_cap,
};
use crate::capability::raw::{
    runtime_cap_asid_pool_ptr, runtime_cap_cnode_ptr, runtime_cap_cnode_radix_bits,
    runtime_cap_cnode_size_bits, runtime_cap_endpoint_badge, runtime_cap_endpoint_ptr,
    runtime_cap_frame_ptr, runtime_cap_irq, runtime_cap_is_arch, runtime_cap_notification_badge,
    runtime_cap_notification_ptr, runtime_cap_page_table_ptr, runtime_cap_range_top, runtime_cap_reply_ptr,
    runtime_cap_tag, runtime_cap_thread_ptr, runtime_cap_untyped_block_size, runtime_cap_untyped_ptr,
    runtime_cap_zombie_number, runtime_cap_zombie_ptr, runtime_clone_cap, runtime_cnode_cap_set_guard, runtime_mask_bits,
    runtime_endpoint_cap_set_badge, runtime_notification_cap_set_badge, runtime_null_cap,
};
use sel4_common::sel4_config::*;
use sel4_common::structures_gen::{cap, cap_null_cap, cap_tag};
use vstd::prelude::*;

verus! {

const TAG_NULL: u64 = 0;
const TAG_FRAME: u64 = 1;
const TAG_UNTYPED: u64 = 2;
const TAG_PAGE_TABLE: u64 = 3;
const TAG_ENDPOINT: u64 = 4;
const ENDPOINT_BITS: usize = 4;
const TAG_NOTIFICATION: u64 = 6;
const NOTIFICATION_BITS: usize = 5;
const TAG_REPLY: u64 = 8;
const TAG_CNODE: u64 = 10;
const SLOT_BITS: usize = 5;
const TAG_ASID_CONTROL: u64 = 11;
const TAG_THREAD: u64 = 12;
const TCB_BITS: usize = 10;
const TAG_ASID_POOL: u64 = 13;
const TAG_IRQ_CONTROL: u64 = 14;
const TAG_IRQ_HANDLER: u64 = 16;
const TAG_ZOMBIE: u64 = 18;
const TAG_DOMAIN: u64 = 20;
const PAGE_TABLE_BITS: usize = 12;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct CNodeCapData {
    pub words: [usize; 1],
}

impl CNodeCapData {
    #[inline]
    pub fn new(data: usize) -> Self {
        CNodeCapData { words: [data] }
    }

    #[inline]
    pub fn get_guard(&self) -> usize {
        (self.words[0] & !0x3fusize) >> 6
    }

    #[inline]
    pub fn get_guard_size(&self) -> usize {
        self.words[0] & 0x3fusize
    }
}

pub trait cap_func {
    fn update_data(&self, preserve: bool, new_data: u64) -> Self
        where Self: Sized;
    fn get_cap_size_bits(&self) -> usize;
    fn get_cap_is_physical(&self) -> bool;
    fn is_arch_cap(&self) -> bool;
}

pub trait cap_arch_func {
    fn arch_updatedata(&self, preserve: bool, new_data: u64) -> Self
        where Self: Sized;
    fn arch_is_cap_revocable(&self, src_cap: &cap) -> bool;
    fn get_cap_ptr(&self) -> usize;
    fn is_vtable_root(&self) -> bool;
    fn is_valid_native_root(&self) -> bool;
    fn is_valid_vtable_root(&self) -> bool;
}

impl cap_func for cap {
    fn update_data(&self, preserve: bool, new_data: u64) -> (ret: Self)
        ensures
            trusted_view_cap(&ret).kind == trusted_view_cap(self).kind
                || trusted_view_cap(&ret).kind == CapKind::NullCap,
            trusted_view_cap(&ret).kind != CapKind::NullCap ==> (
                trusted_view_cap(&ret).object == trusted_view_cap(self).object
                    && trusted_view_cap(&ret).region_id == trusted_view_cap(self).region_id
                    && trusted_view_cap(&ret).rights == trusted_view_cap(self).rights
            ),
    {
        let is_arch = self.is_arch_cap();
        let tag = if is_arch { 0 } else { runtime_cap_tag(self) };
        let endpoint_badge = if !is_arch && tag == TAG_ENDPOINT {
            runtime_cap_endpoint_badge(self)
        } else {
            0
        };
        let notification_badge = if !is_arch && tag == TAG_NOTIFICATION {
            runtime_cap_notification_badge(self)
        } else {
            0
        };
        let ret = if is_arch {
            self.arch_updatedata(preserve, new_data)
        } else {
            match tag {
                TAG_ENDPOINT => {
                    if !preserve && endpoint_badge == 0 {
                        runtime_endpoint_cap_set_badge(self, new_data)
                    } else {
                        runtime_null_cap()
                    }
                }
                TAG_NOTIFICATION => {
                    if !preserve && notification_badge == 0 {
                        runtime_notification_cap_set_badge(self, new_data)
                    } else {
                        runtime_null_cap()
                    }
                }
                TAG_CNODE => {
                    let w = CNodeCapData::new(new_data as usize);
                    let guard_size = w.get_guard_size();
                    let radix = runtime_cap_cnode_radix_bits(self);
                    if guard_size >= 64usize {
                        runtime_null_cap()
                    } else if radix > 64usize {
                        runtime_null_cap()
                    } else if radix > 64usize - guard_size {
                        runtime_null_cap()
                    } else {
                        let guard = w.get_guard() & runtime_mask_bits(guard_size);
                        runtime_cnode_cap_set_guard(self, guard as u64, guard_size as u64)
                    }
                }
                _ => runtime_clone_cap(self),
            }
        };
        proof {
            lemma_trusted_view_cap_kind_matches_tag(self);
            if is_arch {
                assert(trusted_view_cap(&ret).kind == trusted_view_cap(self).kind
                    || trusted_view_cap(&ret).kind == CapKind::NullCap);
                if trusted_view_cap(&ret).kind != CapKind::NullCap {
                    assert(trusted_view_cap(&ret).object == trusted_view_cap(self).object);
                    assert(trusted_view_cap(&ret).region_id == trusted_view_cap(self).region_id);
                    assert(trusted_view_cap(&ret).rights == trusted_view_cap(self).rights);
                }
            } else if tag == TAG_ENDPOINT {
                assert(trusted_view_cap(self).kind == CapKind::EndpointCap);
                if !preserve && endpoint_badge == 0 {
                    assert(trusted_view_cap(&ret).kind == CapKind::EndpointCap);
                    assert(trusted_view_cap(&ret).object == trusted_view_cap(self).object);
                    assert(trusted_view_cap(&ret).region_id == trusted_view_cap(self).region_id);
                    assert(trusted_view_cap(&ret).rights == trusted_view_cap(self).rights);
                } else {
                    assert(trusted_view_cap(&ret).kind == CapKind::NullCap);
                }
            } else if tag == TAG_NOTIFICATION {
                assert(trusted_view_cap(self).kind == CapKind::NotificationCap);
                if !preserve && notification_badge == 0 {
                    assert(trusted_view_cap(&ret).kind == CapKind::NotificationCap);
                    assert(trusted_view_cap(&ret).object == trusted_view_cap(self).object);
                    assert(trusted_view_cap(&ret).region_id == trusted_view_cap(self).region_id);
                    assert(trusted_view_cap(&ret).rights == trusted_view_cap(self).rights);
                } else {
                    assert(trusted_view_cap(&ret).kind == CapKind::NullCap);
                }
            } else if tag == TAG_CNODE {
                assert(trusted_view_cap(self).kind == CapKind::CNodeCap);
                if trusted_view_cap(&ret).kind != CapKind::NullCap {
                    assert(trusted_view_cap(&ret).kind == CapKind::CNodeCap);
                    assert(trusted_view_cap(&ret).object == trusted_view_cap(self).object);
                    assert(trusted_view_cap(&ret).region_id == trusted_view_cap(self).region_id);
                    assert(trusted_view_cap(&ret).rights == trusted_view_cap(self).rights);
                }
            } else {
                assert(trusted_view_cap(&ret) == trusted_view_cap(self));
            }
        }
        ret
    }

    fn get_cap_size_bits(&self) -> (ret: usize)
        ensures
            trusted_view_cap(self).kind != CapKind::ArchCap
                && trusted_view_cap(self).kind != CapKind::ReplyCap
                ==> ret as int == crate::capability::spec::spec_cap_size_bits(
                trusted_view_cap(self),
            ),
    {
        let tag = runtime_cap_tag(self);
        let ret = match tag {
            TAG_UNTYPED => runtime_cap_untyped_block_size(self),
            TAG_ENDPOINT => ENDPOINT_BITS,
            TAG_NOTIFICATION => NOTIFICATION_BITS,
            TAG_CNODE => runtime_cap_cnode_size_bits(self),
            TAG_THREAD => TCB_BITS,
            TAG_PAGE_TABLE => PAGE_TABLE_BITS,
            #[cfg(feature = "kernel_mcs")]
            TAG_REPLY => 5,
            #[cfg(not(feature = "kernel_mcs"))]
            TAG_REPLY => 0,
            #[cfg(feature = "kernel_mcs")]
            15 => runtime_cap_sched_context_size_bits(self),
            _ => 0,
        };
        proof {
            lemma_trusted_view_cap_kind_matches_tag(self);
            if trusted_view_cap(self).kind != CapKind::ArchCap
                && trusted_view_cap(self).kind != CapKind::ReplyCap
            {
                if tag == TAG_UNTYPED {
                    assert(trusted_view_cap(self).kind == CapKind::UntypedCap);
                    assert(ret as int
                        == crate::capability::spec::spec_cap_size_bits(
                        trusted_view_cap(self),
                    ));
                } else if tag == TAG_ENDPOINT {
                    assert(trusted_view_cap(self).kind == CapKind::EndpointCap);
                    assert(ret as int
                        == crate::capability::spec::spec_cap_size_bits(
                        trusted_view_cap(self),
                    ));
                } else if tag == TAG_NOTIFICATION {
                    assert(trusted_view_cap(self).kind == CapKind::NotificationCap);
                    assert(ret as int
                        == crate::capability::spec::spec_cap_size_bits(
                        trusted_view_cap(self),
                    ));
                } else if tag == TAG_CNODE {
                    assert(trusted_view_cap(self).kind == CapKind::CNodeCap);
                    assert(ret as int
                        == crate::capability::spec::spec_cap_size_bits(
                        trusted_view_cap(self),
                    ));
                } else if tag == TAG_THREAD {
                    assert(trusted_view_cap(self).kind == CapKind::ThreadCap);
                    assert(ret as int
                        == crate::capability::spec::spec_cap_size_bits(
                        trusted_view_cap(self),
                    ));
                } else {
                    assert(trusted_view_cap(self).kind != CapKind::UntypedCap);
                    assert(trusted_view_cap(self).kind != CapKind::EndpointCap);
                    assert(trusted_view_cap(self).kind != CapKind::NotificationCap);
                    assert(trusted_view_cap(self).kind != CapKind::CNodeCap);
                    assert(trusted_view_cap(self).kind != CapKind::ThreadCap);
                    assert(ret as int
                        == crate::capability::spec::spec_cap_size_bits(
                        trusted_view_cap(self),
                    ));
                }
            }
        }
        ret
    }

    fn get_cap_is_physical(&self) -> (ret: bool)
        ensures
            trusted_view_cap(self).kind == CapKind::UntypedCap ==> ret,
            trusted_view_cap(self).kind == CapKind::EndpointCap ==> ret,
            trusted_view_cap(self).kind == CapKind::NotificationCap ==> ret,
            trusted_view_cap(self).kind == CapKind::CNodeCap ==> ret,
            trusted_view_cap(self).kind == CapKind::ThreadCap ==> ret,
            trusted_view_cap(self).kind == CapKind::ZombieCap ==> ret,
            crate::capability::raw::spec_runtime_cap_tag(self) == 1 ==> ret,
            crate::capability::raw::spec_runtime_cap_tag(self) == 3 ==> ret,
            crate::capability::raw::spec_runtime_cap_tag(self) == 13 ==> ret,
            trusted_view_cap(self).kind == CapKind::NullCap ==> !ret,
            trusted_view_cap(self).kind == CapKind::IRQControlCap ==> !ret,
            trusted_view_cap(self).kind == CapKind::IRQHandlerCap ==> !ret,
            trusted_view_cap(self).kind == CapKind::Other ==> !ret,
    {
        let tag = runtime_cap_tag(self);
        #[cfg(target_arch = "aarch64")]
        if tag == 5 {
            let _ = crate::capability::raw::runtime_cap_vspace_ptr(self);
            proof {
                lemma_trusted_view_cap_kind_matches_tag(self);
                assert(trusted_view_cap(self).kind == CapKind::ArchCap);
                assert(spec_is_physical_cap(trusted_view_cap(self)));
            }
            return true;
        }
        let arch_ptr = if tag == TAG_FRAME {
            crate::capability::raw::runtime_cap_frame_ptr(self)
        } else if tag == TAG_PAGE_TABLE {
            crate::capability::raw::runtime_cap_page_table_ptr(self)
        } else if tag == TAG_ASID_POOL {
            crate::capability::raw::runtime_cap_asid_pool_ptr(self)
        } else {
            0
        };
        #[cfg(not(feature = "kernel_mcs"))]
        let ret = matches!(
            tag,
            TAG_UNTYPED
                | TAG_ENDPOINT
                | TAG_NOTIFICATION
                | TAG_CNODE
                | TAG_FRAME
                | TAG_ASID_POOL
                | TAG_PAGE_TABLE
                | TAG_ZOMBIE
                | TAG_THREAD
        );
        #[cfg(feature = "kernel_mcs")]
        let ret = matches!(
            tag,
            TAG_UNTYPED
                | TAG_ENDPOINT
                | TAG_NOTIFICATION
                | TAG_CNODE
                | TAG_FRAME
                | TAG_ASID_POOL
                | TAG_PAGE_TABLE
                | TAG_ZOMBIE
                | TAG_THREAD
                | 15
                | TAG_REPLY
        );
        proof {
            lemma_trusted_view_cap_kind_matches_tag(self);
            if tag == TAG_UNTYPED {
                assert(trusted_view_cap(self).kind == CapKind::UntypedCap);
                assert(ret);
            } else if tag == TAG_ENDPOINT {
                assert(trusted_view_cap(self).kind == CapKind::EndpointCap);
                assert(ret);
            } else if tag == TAG_NOTIFICATION {
                assert(trusted_view_cap(self).kind == CapKind::NotificationCap);
                assert(ret);
            } else if tag == TAG_CNODE {
                assert(trusted_view_cap(self).kind == CapKind::CNodeCap);
                assert(ret);
            } else if tag == TAG_THREAD {
                assert(trusted_view_cap(self).kind == CapKind::ThreadCap);
                assert(ret);
            } else if tag == TAG_ZOMBIE {
                assert(trusted_view_cap(self).kind == CapKind::ZombieCap);
                assert(ret);
            } else if tag == TAG_FRAME {
                assert(trusted_view_cap(self).kind == CapKind::ArchCap);
                assert(arch_ptr as int == trusted_view_cap(self).object.unwrap().id);
                assert(ret);
            } else if tag == TAG_PAGE_TABLE {
                assert(trusted_view_cap(self).kind == CapKind::ArchCap);
                assert(arch_ptr as int == trusted_view_cap(self).object.unwrap().id);
                assert(ret);
            } else if tag == TAG_ASID_POOL {
                assert(trusted_view_cap(self).kind == CapKind::ArchCap);
                assert(arch_ptr as int == trusted_view_cap(self).object.unwrap().id);
                assert(ret);
            } else if tag == TAG_NULL || tag == TAG_IRQ_CONTROL || tag == 20 {
                crate::capability::raw::lemma_trusted_view_cap_no_object_for_control_tags(self);
                if tag == TAG_NULL {
                    assert(trusted_view_cap(self).kind == CapKind::NullCap);
                } else if tag == TAG_IRQ_CONTROL {
                    assert(trusted_view_cap(self).kind == CapKind::IRQControlCap);
                } else {
                    crate::capability::raw::lemma_runtime_cap_tag_supported(self);
                    assert(trusted_view_cap(self).kind != CapKind::NullCap);
                    assert(trusted_view_cap(self).kind != CapKind::UntypedCap);
                    assert(trusted_view_cap(self).kind != CapKind::EndpointCap);
                    assert(trusted_view_cap(self).kind != CapKind::NotificationCap);
                    assert(trusted_view_cap(self).kind != CapKind::CNodeCap);
                    assert(trusted_view_cap(self).kind != CapKind::ThreadCap);
                    assert(trusted_view_cap(self).kind != CapKind::ReplyCap);
                    assert(trusted_view_cap(self).kind != CapKind::IRQControlCap);
                    assert(trusted_view_cap(self).kind != CapKind::IRQHandlerCap);
                    assert(trusted_view_cap(self).kind != CapKind::ZombieCap);
                    assert(trusted_view_cap(self).kind != CapKind::ArchCap);
                    assert(trusted_view_cap(self).kind == CapKind::Other);
                }
                assert(!ret);
            } else if tag == TAG_IRQ_HANDLER {
                assert(trusted_view_cap(self).kind == CapKind::IRQHandlerCap);
                assert(!ret);
            } else if tag == TAG_ASID_CONTROL {
                crate::capability::raw::lemma_trusted_view_cap_no_object_for_control_tags(self);
                assert(trusted_view_cap(self).kind == CapKind::ArchCap);
                assert(!ret);
            }
        }
        ret
    }

    fn is_arch_cap(&self) -> (ret: bool)
        ensures
            ret == (trusted_view_cap(self).kind == CapKind::ArchCap),
    {
        runtime_cap_is_arch(self)
    }
}

pub fn same_region_as(cap1: &cap, cap2: &cap) -> (ret: bool)
    ensures
        ret == spec_same_region_as_caps(trusted_view_cap(cap1), trusted_view_cap(cap2)),
{
    let cap1_tag = runtime_cap_tag(cap1);
    let cap2_tag = runtime_cap_tag(cap2);
    let lhs_is_arch = cap1_tag % 2 == 1;
    let rhs_is_arch = cap2_tag % 2 == 1;
    let rhs_is_untyped = cap2_tag == TAG_UNTYPED;
    let rhs_is_endpoint = cap2_tag == TAG_ENDPOINT;
    let rhs_is_notification = cap2_tag == TAG_NOTIFICATION;
    let rhs_is_cnode = cap2_tag == TAG_CNODE;
    let rhs_is_reply = cap2_tag == TAG_REPLY;
    let rhs_is_thread = cap2_tag == TAG_THREAD;
    let rhs_is_irq_control = cap2_tag == TAG_IRQ_CONTROL;
    let rhs_is_irq_handler = cap2_tag == TAG_IRQ_HANDLER;
    let rhs_is_zombie = cap2_tag == TAG_ZOMBIE;
    let rhs_is_frame = cap2_tag == TAG_FRAME;
    let rhs_is_page_table = cap2_tag == TAG_PAGE_TABLE;
    let rhs_is_asid_pool = cap2_tag == TAG_ASID_POOL;
    let rhs_is_physical = rhs_is_untyped
        || rhs_is_endpoint
        || rhs_is_notification
        || rhs_is_cnode
        || rhs_is_thread
        || rhs_is_zombie
        || rhs_is_frame
        || rhs_is_page_table
        || rhs_is_asid_pool;
    let cap1_untyped_ptr = if cap1_tag == TAG_UNTYPED { runtime_cap_untyped_ptr(cap1) } else { 0 };
    let cap1_untyped_block_size =
        if cap1_tag == TAG_UNTYPED { runtime_cap_untyped_block_size(cap1) } else { 0 };
    let cap1_endpoint_ptr = if cap1_tag == TAG_ENDPOINT { runtime_cap_endpoint_ptr(cap1) } else { 0 };
    let cap1_notification_ptr =
        if cap1_tag == TAG_NOTIFICATION { runtime_cap_notification_ptr(cap1) } else { 0 };
    let cap1_cnode_ptr = if cap1_tag == TAG_CNODE { runtime_cap_cnode_ptr(cap1) } else { 0 };
    let cap1_cnode_radix = if cap1_tag == TAG_CNODE { runtime_cap_cnode_radix_bits(cap1) } else { 0 };
    let cap1_reply_ptr = if cap1_tag == TAG_REPLY { runtime_cap_reply_ptr(cap1) } else { 0 };
    let cap1_thread_ptr = if cap1_tag == TAG_THREAD { runtime_cap_thread_ptr(cap1) } else { 0 };
    let cap1_irq = if cap1_tag == TAG_IRQ_HANDLER { runtime_cap_irq(cap1) } else { 0 };
    let cap1_top = if cap1_tag == TAG_UNTYPED { runtime_cap_range_top(cap1) } else { 0 };
    let cap2_untyped_ptr = if rhs_is_untyped { runtime_cap_untyped_ptr(cap2) } else { 0 };
    let cap2_endpoint_ptr = if rhs_is_endpoint { runtime_cap_endpoint_ptr(cap2) } else { 0 };
    let cap2_notification_ptr =
        if rhs_is_notification { runtime_cap_notification_ptr(cap2) } else { 0 };
    let cap2_cnode_ptr = if rhs_is_cnode { runtime_cap_cnode_ptr(cap2) } else { 0 };
    let cap2_cnode_radix = if rhs_is_cnode { runtime_cap_cnode_radix_bits(cap2) } else { 0 };
    let cap2_reply_ptr = if rhs_is_reply { runtime_cap_reply_ptr(cap2) } else { 0 };
    let cap2_thread_ptr = if rhs_is_thread { runtime_cap_thread_ptr(cap2) } else { 0 };
    let cap2_irq = if rhs_is_irq_handler { runtime_cap_irq(cap2) } else { 0 };
    let cap2_zombie_ptr = if rhs_is_zombie { runtime_cap_zombie_ptr(cap2) } else { 0 };
    let cap2_frame_ptr = if rhs_is_frame { runtime_cap_frame_ptr(cap2) } else { 0 };
    let cap2_page_table_ptr =
        if rhs_is_page_table { runtime_cap_page_table_ptr(cap2) } else { 0 };
    let cap2_asid_pool_ptr = if rhs_is_asid_pool { runtime_cap_asid_pool_ptr(cap2) } else { 0 };
    let cap2_untyped_block_size =
        if rhs_is_untyped { runtime_cap_untyped_block_size(cap2) } else { 0 };
    let cap2_physical_base = if rhs_is_untyped {
        cap2_untyped_ptr
    } else if rhs_is_endpoint {
        cap2_endpoint_ptr
    } else if rhs_is_notification {
        cap2_notification_ptr
    } else if rhs_is_cnode {
        cap2_cnode_ptr
    } else if rhs_is_thread {
        cap2_thread_ptr
    } else if rhs_is_zombie {
        cap2_zombie_ptr
    } else if rhs_is_frame {
        cap2_frame_ptr
    } else if rhs_is_page_table {
        cap2_page_table_ptr
    } else if rhs_is_asid_pool {
        cap2_asid_pool_ptr
    } else {
        0
    };
    let cap2_top = if rhs_is_physical { runtime_cap_range_top(cap2) } else { 0 };

    let ret = if lhs_is_arch {
        if rhs_is_arch {
            proof {
                lemma_trusted_view_cap_kind_matches_tag(cap1);
                lemma_trusted_view_cap_kind_matches_tag(cap2);
                assert(trusted_view_cap(cap1).kind == CapKind::ArchCap);
                assert(trusted_view_cap(cap2).kind == CapKind::ArchCap);
            }
            arch_same_region_as(cap1, cap2)
        } else {
            false
        }
    } else {
        match cap1_tag {
            TAG_UNTYPED => {
                if rhs_is_physical {
                    (cap1_untyped_ptr <= cap2_physical_base)
                        && (cap2_physical_base <= cap2_top)
                        && (cap2_top <= cap1_top)
                } else {
                    false
                }
            }
            TAG_ENDPOINT => {
                cap2_tag == TAG_ENDPOINT
                    && runtime_cap_endpoint_ptr(cap1) == runtime_cap_endpoint_ptr(cap2)
            }
            TAG_NOTIFICATION => {
                cap2_tag == TAG_NOTIFICATION
                    && runtime_cap_notification_ptr(cap1) == runtime_cap_notification_ptr(cap2)
            }
            TAG_CNODE => {
                cap2_tag == TAG_CNODE
                    && runtime_cap_cnode_ptr(cap1) == runtime_cap_cnode_ptr(cap2)
                    && runtime_cap_cnode_radix_bits(cap1) == runtime_cap_cnode_radix_bits(cap2)
            }
            TAG_REPLY => {
                cap2_tag == TAG_REPLY
                    && runtime_cap_reply_ptr(cap1) == runtime_cap_reply_ptr(cap2)
            }
            TAG_THREAD => {
                cap2_tag == TAG_THREAD
                    && runtime_cap_thread_ptr(cap1) == runtime_cap_thread_ptr(cap2)
            }
            TAG_IRQ_CONTROL => {
                cap2_tag == TAG_IRQ_CONTROL || cap2_tag == TAG_IRQ_HANDLER
            }
            TAG_IRQ_HANDLER => {
                cap2_tag == TAG_IRQ_HANDLER
                    && runtime_cap_irq(cap1) == runtime_cap_irq(cap2)
            }
            TAG_DOMAIN => false,
            _ => false,
        }
    };

    proof {
        lemma_trusted_view_cap_kind_matches_tag(cap1);
        lemma_trusted_view_cap_kind_matches_tag(cap2);
        lemma_runtime_cap_tag_supported(cap2);
        let lhs = trusted_view_cap(cap1);
        let rhs = trusted_view_cap(cap2);
        assert(cap1_tag == crate::capability::raw::spec_runtime_cap_tag(cap1));
        assert(cap2_tag == crate::capability::raw::spec_runtime_cap_tag(cap2));
        assert(lhs_is_arch == (lhs.kind == CapKind::ArchCap));
        assert(rhs_is_arch == (rhs.kind == CapKind::ArchCap));
        assert(rhs_is_untyped == (rhs.kind == CapKind::UntypedCap));
        assert(rhs_is_endpoint == (rhs.kind == CapKind::EndpointCap));
        assert(rhs_is_notification == (rhs.kind == CapKind::NotificationCap));
        assert(rhs_is_cnode == (rhs.kind == CapKind::CNodeCap));
        assert(rhs_is_reply == (rhs.kind == CapKind::ReplyCap));
        assert(rhs_is_thread == (rhs.kind == CapKind::ThreadCap));
        assert(rhs_is_irq_control == (rhs.kind == CapKind::IRQControlCap));
        assert(rhs_is_irq_handler == (rhs.kind == CapKind::IRQHandlerCap));
        assert(rhs_is_zombie == (rhs.kind == CapKind::ZombieCap));
        if lhs_is_arch {
            if rhs_is_arch {
                assert(lhs.kind == CapKind::ArchCap);
                assert(rhs.kind == CapKind::ArchCap);
                assert(ret == spec_same_region_as_caps(lhs, rhs));
            } else {
                assert(lhs.kind == CapKind::ArchCap);
                assert(rhs.kind != CapKind::ArchCap);
                assert(spec_same_region_as_caps(lhs, rhs) == false);
            }
        } else if cap1_tag == TAG_UNTYPED {
            lemma_trusted_view_cap_region_matches_object(cap1);
            lemma_trusted_view_cap_untyped_bounds(cap1);
            assert(lhs.kind == CapKind::UntypedCap);
            assert(lhs.object == Some(crate::capability::spec::ObjectRef {
                id: cap1_untyped_ptr as int,
                kind: crate::capability::spec::ObjectKind::Untyped,
            }));
            assert(lhs.untyped is Some);
            assert(lhs.untyped.unwrap().block_size_bits == cap1_untyped_block_size as int);
            assert(spec_same_region_as_caps(lhs, rhs) == spec_untyped_cap_contains_cap(lhs, rhs));
            if rhs_is_physical {
                if rhs_is_untyped {
                    lemma_trusted_view_cap_untyped_bounds(cap2);
                    assert(rhs.object == Some(crate::capability::spec::ObjectRef {
                        id: cap2_untyped_ptr as int,
                        kind: crate::capability::spec::ObjectKind::Untyped,
                    }));
                    assert(rhs.untyped is Some);
                    assert(rhs.untyped.unwrap().block_size_bits == cap2_untyped_block_size as int);
                    assert(spec_is_physical_cap(rhs));
                } else if rhs_is_endpoint {
                    assert(rhs.object == Some(crate::capability::spec::ObjectRef {
                        id: cap2_endpoint_ptr as int,
                        kind: crate::capability::spec::ObjectKind::Endpoint,
                    }));
                    assert(spec_is_physical_cap(rhs));
                } else if rhs_is_notification {
                    assert(rhs.object == Some(crate::capability::spec::ObjectRef {
                        id: cap2_notification_ptr as int,
                        kind: crate::capability::spec::ObjectKind::Notification,
                    }));
                    assert(spec_is_physical_cap(rhs));
                } else if rhs_is_cnode {
                    assert(rhs.object == Some(crate::capability::spec::ObjectRef {
                        id: cap2_cnode_ptr as int,
                        kind: crate::capability::spec::ObjectKind::CNode,
                    }));
                    assert(rhs.cnode is Some);
                    assert(rhs.cnode.unwrap().radix_bits == cap2_cnode_radix as int);
                    assert(spec_is_physical_cap(rhs));
                } else if rhs_is_thread {
                    assert(rhs.object == Some(crate::capability::spec::ObjectRef {
                        id: cap2_thread_ptr as int,
                        kind: crate::capability::spec::ObjectKind::Thread,
                    }));
                    assert(spec_is_physical_cap(rhs));
                } else if rhs_is_zombie {
                    assert(rhs.object == Some(crate::capability::spec::ObjectRef {
                        id: cap2_zombie_ptr as int,
                        kind: crate::capability::spec::ObjectKind::Zombie,
                    }));
                    assert(spec_is_physical_cap(rhs));
                } else if rhs_is_frame {
                    assert(rhs.kind == CapKind::ArchCap);
                    assert(rhs.object == Some(crate::capability::spec::ObjectRef {
                        id: cap2_frame_ptr as int,
                        kind: crate::capability::spec::ObjectKind::Arch,
                    }));
                    assert(spec_is_physical_cap(rhs));
                } else if rhs_is_page_table {
                    assert(rhs.kind == CapKind::ArchCap);
                    assert(rhs.object == Some(crate::capability::spec::ObjectRef {
                        id: cap2_page_table_ptr as int,
                        kind: crate::capability::spec::ObjectKind::Arch,
                    }));
                    assert(spec_is_physical_cap(rhs));
                } else {
                    assert(rhs_is_asid_pool);
                    assert(rhs.kind == CapKind::ArchCap);
                    assert(rhs.object == Some(crate::capability::spec::ObjectRef {
                        id: cap2_asid_pool_ptr as int,
                        kind: crate::capability::spec::ObjectKind::Arch,
                    }));
                    assert(spec_is_physical_cap(rhs));
                }
                assert(rhs.object is Some);
                assert(cap1_top as int == crate::capability::spec::spec_cap_range_top(lhs));
                assert(cap2_top as int == crate::capability::spec::spec_cap_range_top(rhs));
                assert(crate::capability::spec::cspace_min_untyped_bits()
                    <= lhs.untyped.unwrap().block_size_bits);
                assert(lhs.object.unwrap().id == cap1_untyped_ptr as int);
                assert(rhs.object.unwrap().id == cap2_physical_base as int);
                assert(spec_untyped_cap_contains_cap(lhs, rhs) == ret);
            } else {
                if cap2_tag == TAG_NULL || cap2_tag == TAG_ASID_CONTROL || cap2_tag == TAG_IRQ_CONTROL || cap2_tag == TAG_DOMAIN {
                    lemma_trusted_view_cap_no_object_for_control_tags(cap2);
                    assert(rhs.object is None);
                } else if rhs_is_reply {
                    assert(rhs.kind == CapKind::ReplyCap);
                    assert(spec_is_physical_cap(rhs) == false);
                } else if rhs_is_irq_handler {
                    assert(rhs.kind == CapKind::IRQHandlerCap);
                    assert(spec_is_physical_cap(rhs) == false);
                } else {
                    assert(
                        cap2_tag == TAG_NULL || cap2_tag == TAG_ASID_CONTROL || cap2_tag == TAG_IRQ_CONTROL
                            || cap2_tag == TAG_DOMAIN || cap2_tag == TAG_REPLY
                            || cap2_tag == TAG_IRQ_HANDLER
                    );
                    assert(rhs.kind != CapKind::UntypedCap);
                    assert(rhs.kind != CapKind::EndpointCap);
                    assert(rhs.kind != CapKind::NotificationCap);
                    assert(rhs.kind != CapKind::CNodeCap);
                    assert(rhs.kind != CapKind::ThreadCap);
                    assert(rhs.kind != CapKind::ZombieCap);
                    assert(rhs.kind != CapKind::ReplyCap);
                    assert(rhs.kind != CapKind::IRQHandlerCap);
                    assert(rhs.kind != CapKind::IRQControlCap || cap2_tag == TAG_IRQ_CONTROL);
                    assert(rhs.object is None || !spec_is_physical_cap(rhs));
                }
                assert(spec_untyped_cap_contains_cap(lhs, rhs) == false);
            }
            assert(ret == spec_same_region_as_caps(lhs, rhs));
        } else {
            if cap1_tag == TAG_ENDPOINT
                || cap1_tag == TAG_NOTIFICATION
                || cap1_tag == TAG_CNODE
                || cap1_tag == TAG_REPLY
                || cap1_tag == TAG_THREAD
                || cap1_tag == TAG_IRQ_HANDLER {
                lemma_trusted_view_cap_region_matches_object(cap1);
                lemma_trusted_view_cap_region_matches_object(cap2);
            }
            if cap1_tag == TAG_ENDPOINT {
                assert(lhs.kind == CapKind::EndpointCap);
                assert(lhs.object == Some(crate::capability::spec::ObjectRef {
                    id: cap1_endpoint_ptr as int,
                    kind: crate::capability::spec::ObjectKind::Endpoint,
                }));
                if rhs_is_endpoint {
                    assert(rhs.object == Some(crate::capability::spec::ObjectRef {
                        id: cap2_endpoint_ptr as int,
                        kind: crate::capability::spec::ObjectKind::Endpoint,
                    }));
                }
            } else if cap1_tag == TAG_NOTIFICATION {
                assert(lhs.kind == CapKind::NotificationCap);
                assert(lhs.object == Some(crate::capability::spec::ObjectRef {
                    id: cap1_notification_ptr as int,
                    kind: crate::capability::spec::ObjectKind::Notification,
                }));
                if rhs_is_notification {
                    assert(rhs.object == Some(crate::capability::spec::ObjectRef {
                        id: cap2_notification_ptr as int,
                        kind: crate::capability::spec::ObjectKind::Notification,
                    }));
                }
            } else if cap1_tag == TAG_CNODE {
                assert(lhs.kind == CapKind::CNodeCap);
                assert(lhs.object == Some(crate::capability::spec::ObjectRef {
                    id: cap1_cnode_ptr as int,
                    kind: crate::capability::spec::ObjectKind::CNode,
                }));
                assert(lhs.cnode is Some);
                assert(lhs.cnode.unwrap().radix_bits == cap1_cnode_radix as int);
                if rhs_is_cnode {
                    assert(rhs.object == Some(crate::capability::spec::ObjectRef {
                        id: cap2_cnode_ptr as int,
                        kind: crate::capability::spec::ObjectKind::CNode,
                    }));
                    assert(rhs.cnode is Some);
                    assert(rhs.cnode.unwrap().radix_bits == cap2_cnode_radix as int);
                }
            } else if cap1_tag == TAG_REPLY {
                assert(lhs.kind == CapKind::ReplyCap);
                assert(lhs.object == Some(crate::capability::spec::ObjectRef {
                    id: cap1_reply_ptr as int,
                    kind: crate::capability::spec::ObjectKind::Reply,
                }));
                if rhs_is_reply {
                    assert(rhs.object == Some(crate::capability::spec::ObjectRef {
                        id: cap2_reply_ptr as int,
                        kind: crate::capability::spec::ObjectKind::Reply,
                    }));
                }
            } else if cap1_tag == TAG_THREAD {
                assert(lhs.kind == CapKind::ThreadCap);
                assert(lhs.object == Some(crate::capability::spec::ObjectRef {
                    id: cap1_thread_ptr as int,
                    kind: crate::capability::spec::ObjectKind::Thread,
                }));
                if rhs_is_thread {
                    assert(rhs.object == Some(crate::capability::spec::ObjectRef {
                        id: cap2_thread_ptr as int,
                        kind: crate::capability::spec::ObjectKind::Thread,
                    }));
                }
            } else if cap1_tag == TAG_IRQ_CONTROL {
                assert(lhs.kind == CapKind::IRQControlCap);
            } else if cap1_tag == TAG_IRQ_HANDLER {
                assert(lhs.kind == CapKind::IRQHandlerCap);
                assert(lhs.object == Some(crate::capability::spec::ObjectRef {
                    id: cap1_irq as int,
                    kind: crate::capability::spec::ObjectKind::IRQ,
                }));
                if rhs_is_irq_handler {
                    assert(rhs.object == Some(crate::capability::spec::ObjectRef {
                        id: cap2_irq as int,
                        kind: crate::capability::spec::ObjectKind::IRQ,
                    }));
                }
            } else {
                assert(lhs.kind != CapKind::UntypedCap);
                assert(lhs.kind != CapKind::EndpointCap);
                assert(lhs.kind != CapKind::NotificationCap);
                assert(lhs.kind != CapKind::CNodeCap);
                assert(lhs.kind != CapKind::ReplyCap);
                assert(lhs.kind != CapKind::ThreadCap);
                assert(lhs.kind != CapKind::IRQControlCap);
                assert(lhs.kind != CapKind::IRQHandlerCap);
                assert(lhs.kind != CapKind::ArchCap);
            }
            assert(ret == spec_same_region_as_caps(lhs, rhs));
        }
    }
    ret
}

pub fn same_object_as(cap1: &cap, cap2: &cap) -> (ret: bool)
    ensures
        ret == spec_same_object_as_caps(trusted_view_cap(cap1), trusted_view_cap(cap2)),
{
    let cap1_tag = runtime_cap_tag(cap1);
    let lhs_is_arch = cap1_tag % 2 == 1;
    let rhs_is_arch = runtime_cap_tag(cap2) % 2 == 1;
    if cap1_tag == TAG_UNTYPED {
        proof {
            lemma_trusted_view_cap_kind_matches_tag(cap1);
            assert(trusted_view_cap(cap1).kind == CapKind::UntypedCap);
            assert(spec_same_object_as_caps(trusted_view_cap(cap1), trusted_view_cap(cap2)) == false);
        }
        return false;
    }
    if cap1_tag == TAG_IRQ_CONTROL {
        proof {
            lemma_trusted_view_cap_kind_matches_tag(cap1);
            assert(trusted_view_cap(cap1).kind == CapKind::IRQControlCap);
            assert(spec_same_object_as_caps(trusted_view_cap(cap1), trusted_view_cap(cap2)) == false);
        }
        return false;
    }

    let ret = if lhs_is_arch && rhs_is_arch {
        proof {
            lemma_trusted_view_cap_kind_matches_tag(cap1);
            lemma_trusted_view_cap_kind_matches_tag(cap2);
            assert(trusted_view_cap(cap1).kind == CapKind::ArchCap);
            assert(trusted_view_cap(cap2).kind == CapKind::ArchCap);
        }
        arch_same_object_as(cap1, cap2)
    } else {
        same_region_as(cap1, cap2)
    };

    proof {
        lemma_trusted_view_cap_kind_matches_tag(cap1);
        lemma_trusted_view_cap_kind_matches_tag(cap2);
        let lhs = trusted_view_cap(cap1);
        let rhs = trusted_view_cap(cap2);
        assert(lhs_is_arch == (lhs.kind == CapKind::ArchCap));
        assert(rhs_is_arch == (rhs.kind == CapKind::ArchCap));
        if lhs_is_arch && rhs_is_arch {
            assert(lhs.kind == CapKind::ArchCap);
            assert(rhs.kind == CapKind::ArchCap);
            assert(spec_same_object_as_caps(lhs, rhs)
                == crate::capability::spec::spec_arch_same_object_as_caps(lhs, rhs));
        } else {
            assert(spec_same_object_as_caps(lhs, rhs) == spec_same_region_as_caps(lhs, rhs));
        }
        assert(ret == spec_same_object_as_caps(lhs, rhs));
    }
    ret
}

pub fn is_cap_revocable(derived_cap: &cap, src_cap: &cap) -> (ret: bool)
    ensures
        ret == spec_is_cap_revocable(trusted_view_cap(derived_cap), trusted_view_cap(src_cap)),
{
    let derived_tag = runtime_cap_tag(derived_cap);
    let src_tag = runtime_cap_tag(src_cap);
    let derived_is_arch = derived_tag % 2 == 1;
    let derived_endpoint_badge =
        if derived_tag == TAG_ENDPOINT { runtime_cap_endpoint_badge(derived_cap) } else { 0 };
    let derived_notification_badge =
        if derived_tag == TAG_NOTIFICATION { runtime_cap_notification_badge(derived_cap) } else { 0 };
    let src_endpoint_badge = if src_tag == TAG_ENDPOINT { runtime_cap_endpoint_badge(src_cap) } else { 0 };
    let src_notification_badge =
        if src_tag == TAG_NOTIFICATION { runtime_cap_notification_badge(src_cap) } else { 0 };
    if derived_is_arch {
        #[cfg(target_arch = "riscv64")]
        let ret = false;
        #[cfg(target_arch = "aarch64")]
        let ret = derived_cap.arch_is_cap_revocable(src_cap);
        proof {
            lemma_trusted_view_cap_kind_matches_tag(derived_cap);
            assert(derived_tag == crate::capability::raw::spec_runtime_cap_tag(derived_cap));
            assert(derived_is_arch == (trusted_view_cap(derived_cap).kind == CapKind::ArchCap));
            assert(trusted_view_cap(derived_cap).kind == CapKind::ArchCap);
            assert(spec_is_cap_revocable(trusted_view_cap(derived_cap), trusted_view_cap(src_cap)) == false);
        }
        return ret;
    }

    let ret = match derived_tag {
        TAG_ENDPOINT => {
            if src_tag == TAG_ENDPOINT {
                runtime_cap_endpoint_badge(derived_cap) != runtime_cap_endpoint_badge(src_cap)
            } else if src_tag == TAG_NOTIFICATION {
                runtime_cap_endpoint_badge(derived_cap) != runtime_cap_notification_badge(src_cap)
            } else {
                true
            }
        }
        TAG_NOTIFICATION => {
            if src_tag == TAG_NOTIFICATION {
                runtime_cap_notification_badge(derived_cap) != runtime_cap_notification_badge(src_cap)
            } else if src_tag == TAG_ENDPOINT {
                runtime_cap_notification_badge(derived_cap) != runtime_cap_endpoint_badge(src_cap)
            } else {
                true
            }
        }
        TAG_IRQ_HANDLER => src_tag == TAG_IRQ_CONTROL,
        TAG_UNTYPED => true,
        _ => false,
    };

    proof {
        lemma_trusted_view_cap_kind_matches_tag(derived_cap);
        lemma_trusted_view_cap_kind_matches_tag(src_cap);
        lemma_trusted_view_cap_badge_shape(derived_cap);
        lemma_trusted_view_cap_badge_shape(src_cap);
        let derived = trusted_view_cap(derived_cap);
        let src = trusted_view_cap(src_cap);
        assert(derived_tag == crate::capability::raw::spec_runtime_cap_tag(derived_cap));
        assert(src_tag == crate::capability::raw::spec_runtime_cap_tag(src_cap));
        if derived_tag == TAG_ENDPOINT {
            assert(derived.kind == CapKind::EndpointCap);
            assert(derived.badge == Some(derived_endpoint_badge as int));
            if src_tag == TAG_ENDPOINT {
                assert(src.kind == CapKind::EndpointCap);
                assert(src.badge == Some(src_endpoint_badge as int));
            } else if src_tag == TAG_NOTIFICATION {
                assert(src.kind == CapKind::NotificationCap);
                assert(src.badge == Some(src_notification_badge as int));
            } else {
                assert(src.kind != CapKind::EndpointCap);
                assert(src.kind != CapKind::NotificationCap);
                assert(src.badge is None);
            }
        } else if derived_tag == TAG_NOTIFICATION {
            assert(derived.kind == CapKind::NotificationCap);
            assert(derived.badge == Some(derived_notification_badge as int));
            if src_tag == TAG_NOTIFICATION {
                assert(src.kind == CapKind::NotificationCap);
                assert(src.badge == Some(src_notification_badge as int));
            } else if src_tag == TAG_ENDPOINT {
                assert(src.kind == CapKind::EndpointCap);
                assert(src.badge == Some(src_endpoint_badge as int));
            } else {
                assert(src.kind != CapKind::EndpointCap);
                assert(src.kind != CapKind::NotificationCap);
                assert(src.badge is None);
            }
        } else if derived_tag == TAG_IRQ_HANDLER {
            assert(derived.kind == CapKind::IRQHandlerCap);
            assert((src_tag == TAG_IRQ_CONTROL) == (src.kind == CapKind::IRQControlCap));
        } else if derived_tag == TAG_UNTYPED {
            assert(derived.kind == CapKind::UntypedCap);
        } else {
            assert(derived.kind != CapKind::EndpointCap);
            assert(derived.kind != CapKind::NotificationCap);
            assert(derived.kind != CapKind::IRQHandlerCap);
            assert(derived.kind != CapKind::UntypedCap);
        }
        assert(ret == spec_is_cap_revocable(derived, src));
    }
    ret
}

pub fn cap_removable(capability: &cap, slot: usize) -> (ret: bool)
    ensures
        ret == spec_cap_removable(trusted_view_cap(capability), slot),
{
    let tag = runtime_cap_tag(capability);
    let zombie_number = if tag == TAG_ZOMBIE {
        runtime_cap_zombie_number(capability)
    } else {
        0
    };
    let zombie_ptr = if tag == TAG_ZOMBIE {
        runtime_cap_zombie_ptr(capability)
    } else {
        0
    };
    let ret = if tag == TAG_NULL {
        true
    } else if tag == TAG_ZOMBIE {
        zombie_number == 0 || (zombie_number == 1 && slot == zombie_ptr)
    } else {
        false
    };

    proof {
        lemma_trusted_view_cap_kind_matches_tag(capability);
        let capability_view = trusted_view_cap(capability);
        assert(tag == crate::capability::raw::spec_runtime_cap_tag(capability));
        if tag == TAG_NULL {
            assert(capability_view.kind == CapKind::NullCap);
        } else if tag == TAG_ZOMBIE {
            assert(capability_view.kind == CapKind::ZombieCap);
            assert(spec_zombie_number_cap(capability_view) == zombie_number);
            assert(spec_zombie_ptr_cap(capability_view) == zombie_ptr);
        } else {
            assert(capability_view.kind != CapKind::NullCap);
            assert(capability_view.kind != CapKind::ZombieCap);
        }
        assert(ret == spec_cap_removable(capability_view, slot));
    }

    ret
}

}
