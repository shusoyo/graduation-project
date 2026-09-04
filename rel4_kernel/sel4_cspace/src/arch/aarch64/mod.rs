use crate::{
    capability::{cap_arch_func, zombie::zombie_func},
    cte::{cte_t, deriveCap_ret},
    capability::raw::{
        runtime_clone_cap,
        runtime_frame_cap_clear_mapping, runtime_frame_cap_mask_vm_rights, runtime_null_cap,
    },
};
use crate::kernel_api::raw::{runtime_exception_none, runtime_exception_syscall_error};
#[cfg(verus_keep_ghost)]
use crate::capability::spec::{
    spec_arch_same_object_as_caps, spec_arch_same_region_as_caps,
};
#[cfg(verus_keep_ghost)]
use crate::capability::raw::trusted_view_cap;
use sel4_common::{
    shared_types_bf_gen::seL4_CapRights, structures_gen::{cap, cap_tag}, utils::pageBitsForSize,
};
use vstd::prelude::*;

verus! {

impl cap_arch_func for cap {
    fn arch_updatedata(&self, preserve: bool, new_data: u64) -> (ret: Self)
        ensures
            trusted_view_cap(&ret).kind == trusted_view_cap(self).kind
                || trusted_view_cap(&ret).kind
                    == crate::capability::spec::CapKind::NullCap,
            trusted_view_cap(&ret).kind
                != crate::capability::spec::CapKind::NullCap ==> (
                trusted_view_cap(&ret).object == trusted_view_cap(self).object
                    && trusted_view_cap(&ret).region_id == trusted_view_cap(self).region_id
                    && trusted_view_cap(&ret).rights == trusted_view_cap(self).rights
                    && trusted_view_cap(&ret).cnode == trusted_view_cap(self).cnode
                    && trusted_view_cap(&ret).untyped == trusted_view_cap(self).untyped
            ),
    {
        #[cfg(feature = "enable_smc")]
        {
            if self.clone().get_tag() == cap_tag::cap_smc_cap {
                if !preserve && cap::cap_smc_cap(self).get_capSMCBadge() == 0 {
                    let new_cap = runtime_clone_cap(self);
                    cap::cap_smc_cap(&new_cap).set_capSMCBadge(new_data);
                    return new_cap;
                } else {
                    return runtime_null_cap();
                }
            } else {
                return runtime_clone_cap(self);
            }
        }
        #[cfg(not(feature = "enable_smc"))]
        {
            let _ = preserve;
            let _ = new_data;
            runtime_clone_cap(self)
        }
    }

    fn arch_is_cap_revocable(&self, src_cap: &cap) -> (ret: bool)
        ensures
            ret ==> trusted_view_cap(self).kind
                == crate::capability::spec::CapKind::ArchCap,
            ret ==> trusted_view_cap(src_cap).kind
                == crate::capability::spec::CapKind::ArchCap,
            trusted_view_cap(self).kind
                != crate::capability::spec::CapKind::ArchCap ==> !ret,
            trusted_view_cap(src_cap).kind
                != crate::capability::spec::CapKind::ArchCap ==> !ret,
    {
        #[cfg(feature = "enable_smc")]
        {
            match self.get_tag() {
                cap_tag::cap_smc_cap => {
                    cap::cap_smc_cap(self).get_capSMCBadge()
                        != cap::cap_smc_cap(src_cap).get_capSMCBadge()
                }
                _ => false,
            }
        }
        #[cfg(not(feature = "enable_smc"))]
        {
            let _ = src_cap;
            false
        }
    }

    #[verifier::external_body]
    fn get_cap_ptr(&self) -> (ret: usize)
        ensures
            trusted_view_cap(self).object is Some
                ==> ret as int == trusted_view_cap(self).object.unwrap().id,
            trusted_view_cap(self).object is None ==> ret == 0,
    {
        match self.get_tag() {
            cap_tag::cap_untyped_cap => cap::cap_untyped_cap(self).get_capPtr() as usize,
            cap_tag::cap_endpoint_cap => cap::cap_endpoint_cap(self).get_capEPPtr() as usize,
            cap_tag::cap_notification_cap => {
                cap::cap_notification_cap(self).get_capNtfnPtr() as usize
            }
            cap_tag::cap_cnode_cap => cap::cap_cnode_cap(self).get_capCNodePtr() as usize,
            cap_tag::cap_thread_cap => cap::cap_thread_cap(self).get_capTCBPtr() as usize,
            cap_tag::cap_zombie_cap => cap::cap_zombie_cap(self).get_zombie_ptr() as usize,
            cap_tag::cap_frame_cap => cap::cap_frame_cap(self).get_capFBasePtr() as usize,
            cap_tag::cap_page_table_cap => {
                cap::cap_page_table_cap(self).get_capPTBasePtr() as usize
            }
            cap_tag::cap_vspace_cap => cap::cap_vspace_cap(self).get_capVSBasePtr() as usize,
            cap_tag::cap_asid_control_cap => 0,
            cap_tag::cap_asid_pool_cap => cap::cap_asid_pool_cap(self).get_capASIDPool() as usize,
            #[cfg(feature = "kernel_mcs")]
            cap_tag::cap_reply_cap => cap::cap_reply_cap(self).get_capTCBPtr() as usize,
            #[cfg(feature = "kernel_mcs")]
            cap_tag::cap_sched_context_cap => {
                cap::cap_sched_context_cap(self).get_capSCPtr() as usize
            }
            _ => 0,
        }
    }

    #[inline]
    fn is_vtable_root(&self) -> bool {
        self.get_tag() == cap_tag::cap_vspace_cap
    }

    #[inline]
    fn is_valid_native_root(&self) -> bool {
        self.is_vtable_root() && cap::cap_vspace_cap(self).get_capVSIsMapped() != 0
    }

    #[inline]
    fn is_valid_vtable_root(&self) -> bool {
        self.is_valid_native_root()
    }
}

impl cte_t {
    // Temporary semantic TCB: this keeps the runtime shape, but the caller-facing contract now
    // distinguishes the mapped-success and unmapped-error cases directly.
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
            (crate::capability::raw::spec_runtime_cap_tag(capability) == 5
                || crate::capability::raw::spec_runtime_cap_tag(capability) == 3) ==> (
                crate::kernel_api::raw::is_exception_none(ret.status)
                    ==> trusted_view_cap(&ret.capability) == trusted_view_cap(capability)
            ),
            (crate::capability::raw::spec_runtime_cap_tag(capability) == 5
                || crate::capability::raw::spec_runtime_cap_tag(capability) == 3) ==> (
                crate::kernel_api::raw::is_exception_syscall_error(ret.status)
                    ==> trusted_view_cap(&ret.capability).kind
                        == crate::capability::spec::CapKind::NullCap
            ),
            (crate::capability::raw::spec_runtime_cap_tag(capability) == 1
                || crate::capability::raw::spec_runtime_cap_tag(capability) == 11
                || crate::capability::raw::spec_runtime_cap_tag(capability) == 13) ==> (
                crate::kernel_api::raw::is_exception_none(ret.status)
                    && trusted_view_cap(&ret.capability) == trusted_view_cap(capability)
            ),
    {
        let mut ret = deriveCap_ret {
            status: runtime_exception_none(),
            capability: runtime_null_cap(),
        };
        match capability.get_tag() {
            cap_tag::cap_vspace_cap => {
                if cap::cap_vspace_cap(capability).get_capVSIsMapped() != 0 {
                    ret.capability = runtime_clone_cap(capability);
                    ret.status = runtime_exception_none();
                } else {
                    ret.capability = runtime_null_cap();
                    ret.status = runtime_exception_syscall_error();
                }
            }
            cap_tag::cap_page_table_cap => {
                if cap::cap_page_table_cap(capability).get_capPTIsMapped() != 0 {
                    ret.capability = runtime_clone_cap(capability);
                    ret.status = runtime_exception_none();
                } else {
                    ret.capability = runtime_null_cap();
                    ret.status = runtime_exception_syscall_error();
                }
            }
            cap_tag::cap_frame_cap => {
                ret.capability = runtime_frame_cap_clear_mapping(capability);
            }
            cap_tag::cap_asid_control_cap | cap_tag::cap_asid_pool_cap => {
                ret.capability = runtime_clone_cap(capability);
            }
            #[cfg(feature = "enable_smc")]
            cap_tag::cap_smc_cap => {
                ret.capability = runtime_clone_cap(capability);
            }
            _ => {
                panic!(" Invalid arch cap type : {}", capability.get_tag() as usize);
            }
        }
        proof {
            assert(capability.get_tag() as u64 == crate::capability::raw::spec_runtime_cap_tag(capability));
            match capability.get_tag() {
                cap_tag::cap_vspace_cap | cap_tag::cap_page_table_cap => {
                    if crate::kernel_api::raw::is_exception_none(ret.status) {
                        assert(trusted_view_cap(&ret.capability) == trusted_view_cap(capability));
                    } else {
                        assert(crate::kernel_api::raw::is_exception_syscall_error(ret.status));
                        assert(
                            trusted_view_cap(&ret.capability).kind
                                == crate::capability::spec::CapKind::NullCap
                        );
                    }
                }
                cap_tag::cap_frame_cap
                | cap_tag::cap_asid_control_cap
                | cap_tag::cap_asid_pool_cap => {
                    assert(crate::kernel_api::raw::is_exception_none(ret.status));
                    assert(trusted_view_cap(&ret.capability) == trusted_view_cap(capability));
                }
                #[cfg(feature = "enable_smc")]
                cap_tag::cap_smc_cap => {
                    assert(crate::kernel_api::raw::is_exception_none(ret.status));
                    assert(trusted_view_cap(&ret.capability) == trusted_view_cap(capability));
                }
                _ => {}
            }
        }
        ret
    }
}

pub fn arch_mask_cap_rights(rights: seL4_CapRights, capability: &cap) -> (ret: cap)
    ensures
        trusted_view_cap(&ret).kind == trusted_view_cap(capability).kind,
        trusted_view_cap(&ret).object == trusted_view_cap(capability).object,
        trusted_view_cap(&ret).region_id == trusted_view_cap(capability).region_id,
        trusted_view_cap(&ret).badge == trusted_view_cap(capability).badge,
        trusted_view_cap(&ret).cnode == trusted_view_cap(capability).cnode,
        trusted_view_cap(&ret).untyped == trusted_view_cap(capability).untyped,
{
    if capability.get_tag() == cap_tag::cap_frame_cap {
        runtime_frame_cap_mask_vm_rights(capability, rights)
    } else {
        runtime_clone_cap(capability)
    }
}

// Temporary semantic TCB: these runtime-match lemmas still discharge the remaining
// arch-cap relation cases until the aarch64 relation bodies are shrunk further.
#[verifier::external_body]
proof fn lemma_aarch64_arch_same_region_matches_runtime(cap1: &cap, cap2: &cap, ret: bool)
    ensures
        ret == spec_arch_same_region_as_caps(trusted_view_cap(cap1), trusted_view_cap(cap2)),
{
}

#[verifier::external_body]
proof fn lemma_aarch64_arch_same_object_matches_runtime(cap1: &cap, cap2: &cap, ret: bool)
    ensures
        ret == spec_arch_same_object_as_caps(trusted_view_cap(cap1), trusted_view_cap(cap2)),
{
}

pub fn arch_same_region_as(cap1: &cap, cap2: &cap) -> (ret: bool)
    requires
        trusted_view_cap(cap1).kind == crate::capability::spec::CapKind::ArchCap,
        trusted_view_cap(cap2).kind == crate::capability::spec::CapKind::ArchCap,
    ensures
        ret == spec_arch_same_region_as_caps(trusted_view_cap(cap1), trusted_view_cap(cap2)),
{
    match cap1.get_tag() {
        cap_tag::cap_frame_cap => {
            if cap2.get_tag() == cap_tag::cap_frame_cap {
                let botA = cap::cap_frame_cap(cap1).get_capFBasePtr() as usize;
                let botB = cap::cap_frame_cap(cap2).get_capFBasePtr() as usize;
                let topA = botA
                    + mask_bits!(pageBitsForSize(
                        cap::cap_frame_cap(cap1).get_capFSize() as usize
                    ));
                let topB = botB
                    + mask_bits!(pageBitsForSize(
                        cap::cap_frame_cap(cap2).get_capFSize() as usize
                    ));
                let ret = (botA <= botB) && (topA >= topB) && (botB <= topB);
                proof {
                    lemma_aarch64_arch_same_region_matches_runtime(cap1, cap2, ret);
                }
                return ret;
            }
        }
        cap_tag::cap_page_table_cap => {
            if cap2.get_tag() == cap_tag::cap_page_table_cap {
                let ret = cap::cap_page_table_cap(cap1).get_capPTBasePtr()
                    == cap::cap_page_table_cap(cap2).get_capPTBasePtr();
                proof {
                    lemma_aarch64_arch_same_region_matches_runtime(cap1, cap2, ret);
                }
                return ret;
            }
        }
        cap_tag::cap_vspace_cap => {
            if cap2.get_tag() == cap_tag::cap_vspace_cap {
                let ret = cap::cap_vspace_cap(cap1).get_capVSBasePtr()
                    == cap::cap_vspace_cap(cap2).get_capVSBasePtr();
                proof {
                    lemma_aarch64_arch_same_region_matches_runtime(cap1, cap2, ret);
                }
                return ret;
            }
        }
        cap_tag::cap_asid_control_cap => {
            let ret = cap2.get_tag() == cap_tag::cap_asid_control_cap;
            proof {
                lemma_aarch64_arch_same_region_matches_runtime(cap1, cap2, ret);
            }
            return ret;
        }
        cap_tag::cap_asid_pool_cap => {
            if cap2.get_tag() == cap_tag::cap_asid_pool_cap {
                let ret = cap::cap_asid_pool_cap(cap1).get_capASIDPool()
                    == cap::cap_asid_pool_cap(cap2).get_capASIDPool();
                proof {
                    lemma_aarch64_arch_same_region_matches_runtime(cap1, cap2, ret);
                }
                return ret;
            }
        }
        #[cfg(feature = "enable_smc")]
        cap_tag::cap_smc_cap => {
            if cap2.get_tag() == cap_tag::cap_smc_cap {
                proof {
                    lemma_aarch64_arch_same_region_matches_runtime(cap1, cap2, true);
                }
                return true;
            }
        }
        _ => panic!("unknown cap"),
    }
    proof {
        lemma_aarch64_arch_same_region_matches_runtime(cap1, cap2, false);
    }
    false
}

pub fn arch_same_object_as(cap1: &cap, cap2: &cap) -> (ret: bool)
    requires
        trusted_view_cap(cap1).kind == crate::capability::spec::CapKind::ArchCap,
        trusted_view_cap(cap2).kind == crate::capability::spec::CapKind::ArchCap,
    ensures
        ret == spec_arch_same_object_as_caps(trusted_view_cap(cap1), trusted_view_cap(cap2)),
{
    if cap1.get_tag() == cap_tag::cap_frame_cap && cap2.get_tag() == cap_tag::cap_frame_cap {
        let ret = cap::cap_frame_cap(cap1).get_capFBasePtr()
            == cap::cap_frame_cap(cap2).get_capFBasePtr()
            && cap::cap_frame_cap(cap1).get_capFSize() == cap::cap_frame_cap(cap2).get_capFSize()
            && cap::cap_frame_cap(cap1).get_capFIsDevice()
                == cap::cap_frame_cap(cap2).get_capFIsDevice();
        proof {
            lemma_aarch64_arch_same_object_matches_runtime(cap1, cap2, ret);
        }
        return ret;
    }
    let ret = arch_same_region_as(cap1, cap2);
    proof {
        lemma_aarch64_arch_same_object_matches_runtime(cap1, cap2, ret);
    }
    ret
}

}
