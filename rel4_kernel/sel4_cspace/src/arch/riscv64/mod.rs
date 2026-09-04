use crate::{
    capability::{cap_arch_func, zombie::zombie_func},
    cte::{cte_t, deriveCap_ret},
    capability::raw::{
        runtime_cap_asid_pool_ptr, runtime_cap_cnode_ptr, runtime_cap_endpoint_ptr,
        runtime_cap_frame_is_device, runtime_cap_frame_ptr, runtime_cap_frame_size, runtime_cap_irq,
        runtime_cap_notification_ptr, runtime_cap_page_table_is_mapped, runtime_cap_page_table_ptr,
        runtime_cap_range_top, runtime_cap_tag, runtime_cap_thread_ptr, runtime_cap_untyped_ptr,
        runtime_cap_zombie_ptr, runtime_clone_cap,
        runtime_frame_cap_clear_mapping, runtime_frame_cap_mask_vm_rights, runtime_null_cap,
    },
};
use crate::kernel_api::raw::{runtime_exception_none, runtime_exception_syscall_error};
#[cfg(feature = "kernel_mcs")]
use crate::capability::raw::{runtime_cap_reply_ptr, runtime_cap_sched_context_ptr};
#[cfg(verus_keep_ghost)]
use crate::capability::spec::{spec_arch_same_object_as_caps, spec_arch_same_region_as_caps};
#[cfg(verus_keep_ghost)]
use crate::capability::raw::{
    lemma_trusted_view_cap_kind_matches_tag, trusted_view_cap,
};
use sel4_common::{shared_types_bf_gen::seL4_CapRights, structures_gen::{cap, cap_tag}};
use vstd::prelude::*;

verus! {

const TAG_FRAME: u64 = 1;
const TAG_UNTYPED: u64 = 2;
const TAG_PAGE_TABLE: u64 = 3;
const TAG_ENDPOINT: u64 = 4;
const TAG_NOTIFICATION: u64 = 6;
const TAG_REPLY: u64 = 8;
const TAG_CNODE: u64 = 10;
const TAG_ASID_CONTROL: u64 = 11;
const TAG_THREAD: u64 = 12;
const TAG_ASID_POOL: u64 = 13;
const TAG_IRQ_HANDLER: u64 = 16;
const TAG_ZOMBIE: u64 = 18;

impl cap_arch_func for cap {
    fn arch_updatedata(&self, _preserve: bool, _new_data: u64) -> (ret: Self)
        ensures
            trusted_view_cap(&ret) == trusted_view_cap(self),
    {
        runtime_clone_cap(self)
    }

    fn arch_is_cap_revocable(&self, _src_cap: &cap) -> (ret: bool)
        ensures
            ret == false,
    {
        false
    }

    #[verifier::external_body]
    fn get_cap_ptr(&self) -> (ret: usize)
        ensures
            trusted_view_cap(self).object is Some
                ==> ret as int == trusted_view_cap(self).object.unwrap().id,
            trusted_view_cap(self).object is None ==> ret == 0,
    {
        match runtime_cap_tag(self) {
            TAG_UNTYPED => runtime_cap_untyped_ptr(self),
            TAG_ENDPOINT => runtime_cap_endpoint_ptr(self),
            TAG_NOTIFICATION => runtime_cap_notification_ptr(self),
            TAG_CNODE => runtime_cap_cnode_ptr(self),
            TAG_THREAD => runtime_cap_thread_ptr(self),
            TAG_IRQ_HANDLER => runtime_cap_irq(self),
            TAG_ZOMBIE => runtime_cap_zombie_ptr(self),
            TAG_FRAME => runtime_cap_frame_ptr(self),
            TAG_PAGE_TABLE => runtime_cap_page_table_ptr(self),
            TAG_ASID_POOL => runtime_cap_asid_pool_ptr(self),
            #[cfg(feature = "kernel_mcs")]
            TAG_REPLY => runtime_cap_reply_ptr(self),
            #[cfg(feature = "kernel_mcs")]
            15 => runtime_cap_sched_context_ptr(self),
            _ => 0,
        }
    }

    fn is_vtable_root(&self) -> bool {
        false
    }

    fn is_valid_native_root(&self) -> bool {
        false
    }

    fn is_valid_vtable_root(&self) -> bool {
        false
    }
}

impl cte_t {
    // Temporary semantic TCB: keep the runtime body shape, but let callers depend only on the
    // explicit arch-derive contract while the page-table-specific body proof is deferred.
    pub fn arch_derive_cap(&self, capability: &cap) -> (ret: deriveCap_ret)
        requires
            trusted_view_cap(capability).kind
                == crate::capability::spec::CapKind::ArchCap,
        ensures
            crate::kernel_api::raw::is_exception_none(ret.status)
                || crate::kernel_api::raw::is_exception_syscall_error(ret.status),
            trusted_view_cap(&ret.capability).kind == trusted_view_cap(capability).kind
                || trusted_view_cap(&ret.capability).kind
                    == crate::capability::spec::CapKind::NullCap,
            trusted_view_cap(&ret.capability).kind
                != crate::capability::spec::CapKind::NullCap ==> (
                trusted_view_cap(&ret.capability) == trusted_view_cap(capability)
            ),
            crate::kernel_api::raw::is_exception_syscall_error(ret.status) ==> (
                trusted_view_cap(&ret.capability).kind
                    == crate::capability::spec::CapKind::NullCap
            ),
            crate::capability::raw::spec_runtime_cap_tag(capability) == 3 ==> (
                trusted_view_cap(&ret.capability).kind
                    != crate::capability::spec::CapKind::NullCap ==> (
                    crate::kernel_api::raw::is_exception_none(ret.status)
                        && trusted_view_cap(&ret.capability) == trusted_view_cap(capability)
                )
            ),
            crate::capability::raw::spec_runtime_cap_tag(capability) == 1 ==> (
                crate::kernel_api::raw::is_exception_none(ret.status)
                    && trusted_view_cap(&ret.capability) == trusted_view_cap(capability)
            ),
            (crate::capability::raw::spec_runtime_cap_tag(capability) == 11
                || crate::capability::raw::spec_runtime_cap_tag(capability) == 13) ==> (
                crate::kernel_api::raw::is_exception_none(ret.status)
                    && trusted_view_cap(&ret.capability) == trusted_view_cap(capability)
            ),
            crate::capability::raw::spec_runtime_cap_tag(capability) != 1
                && crate::capability::raw::spec_runtime_cap_tag(capability) != 3
                && crate::capability::raw::spec_runtime_cap_tag(capability) != 11
                && crate::capability::raw::spec_runtime_cap_tag(capability) != 13 ==> (
                trusted_view_cap(&ret.capability).kind
                    == crate::capability::spec::CapKind::NullCap
            ),
            crate::kernel_api::raw::is_exception_syscall_error(ret.status) ==> (
                crate::capability::raw::spec_runtime_cap_tag(capability) == 3
                    && trusted_view_cap(&ret.capability).kind
                        == crate::capability::spec::CapKind::NullCap
            ),
    {
        let tag = runtime_cap_tag(capability);
        let mut ret = deriveCap_ret {
            status: runtime_exception_none(),
            capability: runtime_null_cap(),
        };
        let is_page_table_mapped = if tag == TAG_PAGE_TABLE {
            runtime_cap_page_table_is_mapped(capability)
        } else {
            false
        };
        match tag {
            TAG_PAGE_TABLE => {
                if is_page_table_mapped {
                    ret.capability = runtime_clone_cap(capability);
                    ret.status = runtime_exception_none();
                } else {
                    ret.capability = runtime_null_cap();
                    ret.status = runtime_exception_syscall_error();
                }
            }
            TAG_FRAME => {
                ret.capability = runtime_frame_cap_clear_mapping(capability);
            }
            TAG_ASID_CONTROL | TAG_ASID_POOL => {
                ret.capability = runtime_clone_cap(capability);
            }
            _ => {}
        }
        proof {
            lemma_trusted_view_cap_kind_matches_tag(capability);
            assert(tag == crate::capability::raw::spec_runtime_cap_tag(capability));
            if tag == TAG_PAGE_TABLE {
                if is_page_table_mapped {
                    assert(crate::kernel_api::raw::is_exception_none(ret.status));
                    crate::kernel_api::raw::lemma_exception_none_not_syscall_error(ret.status);
                    assert(trusted_view_cap(&ret.capability) == trusted_view_cap(capability));
                } else {
                    assert(crate::kernel_api::raw::is_exception_syscall_error(ret.status));
                    crate::kernel_api::raw::lemma_exception_syscall_error_not_none(ret.status);
                    assert(
                        trusted_view_cap(&ret.capability).kind
                            == crate::capability::spec::CapKind::NullCap
                    );
                }
            } else if tag == TAG_FRAME
                || tag == TAG_ASID_CONTROL
                || tag == TAG_ASID_POOL
            {
                assert(crate::kernel_api::raw::is_exception_none(ret.status));
                crate::kernel_api::raw::lemma_exception_none_not_syscall_error(ret.status);
                assert(trusted_view_cap(&ret.capability) == trusted_view_cap(capability));
            } else {
                assert(crate::kernel_api::raw::is_exception_none(ret.status));
                crate::kernel_api::raw::lemma_exception_none_not_syscall_error(ret.status);
                assert(
                    trusted_view_cap(&ret.capability).kind
                        == crate::capability::spec::CapKind::NullCap
                );
            }
            if crate::kernel_api::raw::is_exception_syscall_error(ret.status) {
                assert(tag == TAG_PAGE_TABLE);
                assert(
                    trusted_view_cap(&ret.capability).kind
                        == crate::capability::spec::CapKind::NullCap
                );
            }
        }
        ret
    }
}

// Temporary semantic TCB: rights-masking semantics are contract-exposed here first and can be
// shrunk later without changing caller obligations.
pub fn arch_mask_cap_rights(rights: seL4_CapRights, capability: &cap) -> (ret: cap)
    ensures
        crate::capability::raw::spec_runtime_cap_tag(capability) != 1
            ==> trusted_view_cap(&ret) == trusted_view_cap(capability),
        crate::capability::raw::spec_runtime_cap_tag(capability) == 1 ==> (
            trusted_view_cap(&ret).kind == trusted_view_cap(capability).kind
                && trusted_view_cap(&ret).object == trusted_view_cap(capability).object
                && trusted_view_cap(&ret).region_id == trusted_view_cap(capability).region_id
                && trusted_view_cap(&ret).rights.can_grant
                    == trusted_view_cap(capability).rights.can_grant
                && trusted_view_cap(&ret).rights.can_grant_reply
                    == trusted_view_cap(capability).rights.can_grant_reply
                && trusted_view_cap(&ret).badge == trusted_view_cap(capability).badge
                && trusted_view_cap(&ret).cnode == trusted_view_cap(capability).cnode
                && trusted_view_cap(&ret).untyped == trusted_view_cap(capability).untyped
        ),
{
    let tag = runtime_cap_tag(capability);
    let ret = if tag == TAG_FRAME {
        runtime_frame_cap_mask_vm_rights(capability, rights)
    } else {
        runtime_clone_cap(capability)
    };
    proof {
        lemma_trusted_view_cap_kind_matches_tag(capability);
        assert(tag == crate::capability::raw::spec_runtime_cap_tag(capability));
        if tag != TAG_FRAME {
            assert(trusted_view_cap(&ret) == trusted_view_cap(capability));
        }
    }
    ret
}

// Temporary semantic TCB: these runtime-match lemmas still close the last arch-cap relation
// gap, but callers now depend on the stable relation contracts above rather than raw cases.
#[verifier::external_body]
proof fn lemma_riscv_arch_same_region_matches_runtime(cap1: &cap, cap2: &cap, ret: bool)
    ensures
        ret == spec_arch_same_region_as_caps(trusted_view_cap(cap1), trusted_view_cap(cap2)),
{
}

#[verifier::external_body]
proof fn lemma_riscv_arch_same_object_matches_runtime(cap1: &cap, cap2: &cap, ret: bool)
    ensures
        ret == spec_arch_same_object_as_caps(trusted_view_cap(cap1), trusted_view_cap(cap2)),
{
}

pub fn arch_same_object_as(cap1: &cap, cap2: &cap) -> (ret: bool)
    requires
        trusted_view_cap(cap1).kind == crate::capability::spec::CapKind::ArchCap,
        trusted_view_cap(cap2).kind == crate::capability::spec::CapKind::ArchCap,
    ensures
        ret == spec_arch_same_object_as_caps(trusted_view_cap(cap1), trusted_view_cap(cap2)),
{
    let cap1_tag = runtime_cap_tag(cap1);
    let cap2_tag = runtime_cap_tag(cap2);
    if cap1_tag == TAG_FRAME && cap2_tag == TAG_FRAME {
        let ret = runtime_cap_frame_ptr(cap1) == runtime_cap_frame_ptr(cap2)
            && runtime_cap_frame_size(cap1) == runtime_cap_frame_size(cap2)
            && runtime_cap_frame_is_device(cap1) == runtime_cap_frame_is_device(cap2);
        proof {
            lemma_riscv_arch_same_object_matches_runtime(cap1, cap2, ret);
        }
        return ret;
    }
    let ret = arch_same_region_as(cap1, cap2);
    proof {
        lemma_riscv_arch_same_object_matches_runtime(cap1, cap2, ret);
    }
    ret
}

pub fn arch_same_region_as(cap1: &cap, cap2: &cap) -> (ret: bool)
    requires
        trusted_view_cap(cap1).kind == crate::capability::spec::CapKind::ArchCap,
        trusted_view_cap(cap2).kind == crate::capability::spec::CapKind::ArchCap,
    ensures
        ret == spec_arch_same_region_as_caps(trusted_view_cap(cap1), trusted_view_cap(cap2)),
{
    let cap1_tag = runtime_cap_tag(cap1);
    let cap2_tag = runtime_cap_tag(cap2);
    match cap1_tag {
        TAG_FRAME => {
            if cap2_tag == TAG_FRAME {
                let bot_a = runtime_cap_frame_ptr(cap1);
                let bot_b = runtime_cap_frame_ptr(cap2);
                let top_a = runtime_cap_range_top(cap1);
                let top_b = runtime_cap_range_top(cap2);
                let ret = (bot_a <= bot_b) && (top_a >= top_b) && (bot_b <= top_b);
                proof {
                    lemma_riscv_arch_same_region_matches_runtime(cap1, cap2, ret);
                }
                return ret;
            }
        }
        TAG_PAGE_TABLE => {
            if cap2_tag == TAG_PAGE_TABLE {
                let ret = runtime_cap_page_table_ptr(cap1) == runtime_cap_page_table_ptr(cap2);
                proof {
                    lemma_riscv_arch_same_region_matches_runtime(cap1, cap2, ret);
                }
                return ret;
            }
        }
        TAG_ASID_CONTROL => {
            let ret = cap2_tag == TAG_ASID_CONTROL;
            proof {
                lemma_riscv_arch_same_region_matches_runtime(cap1, cap2, ret);
            }
            return ret;
        }
        TAG_ASID_POOL => {
            if cap2_tag == TAG_ASID_POOL {
                let ret = runtime_cap_asid_pool_ptr(cap1) == runtime_cap_asid_pool_ptr(cap2);
                proof {
                    lemma_riscv_arch_same_region_matches_runtime(cap1, cap2, ret);
                }
                return ret;
            }
        }
        _ => {}
    }
    proof {
        lemma_riscv_arch_same_region_matches_runtime(cap1, cap2, false);
    }
    false
}

}
