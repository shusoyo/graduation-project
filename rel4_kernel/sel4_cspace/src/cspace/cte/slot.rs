#[cfg(verus_keep_ghost)]
use crate::capability::raw::{
    lemma_trusted_view_cap_badge_shape, lemma_trusted_view_cap_kind_matches_tag,
};
#[cfg(verus_keep_ghost)]
use crate::capability::raw::trusted_view_cap;
use crate::capability::raw::{
    runtime_cap_endpoint_badge, runtime_cap_is_arch, runtime_cap_notification_badge,
    runtime_cap_tag, runtime_clone_cap, runtime_null_cap,
};
#[cfg(verus_keep_ghost)]
use crate::capability::spec::{spec_same_object_as_caps, CapKind};
use crate::capability::{same_object_as, same_region_as};
#[cfg(verus_keep_ghost)]
use crate::cspace::cte::raw::{
    lemma_cte_slot_view_at_ptr_matches_trusted_view, lemma_trusted_view_cte_cap_matches_cap_field,
};
use crate::cspace::cte::raw::runtime_slot_ref_at;
#[cfg(verus_keep_ghost)]
use crate::cspace::cte::spec::{
    cte_offset_slot_call_pre, cte_slot_ptr, cte_slot_view_at, spec_mdb_parent_of_caps,
    spec_slot_derive_cap_expected_cap, spec_slot_derive_cap_returns_syscall_error,
    spec_slot_ensure_no_children_blocks, spec_slot_is_final_cap_at,
    spec_slot_is_long_running_delete_at, spec_slot_mdb_parent_of,
};
use crate::cspace::cte::types::{cte_t, deriveCap_ret};
use crate::cspace::mdb::MdbTable;
#[cfg(verus_keep_ghost)]
use crate::kernel_api::raw::{
    is_exception_none, is_exception_syscall_error, lemma_exception_none_iff_spec_runtime_exception_none,
    lemma_exception_syscall_error_not_none,
};
use crate::kernel_api::raw::{
    runtime_exception_none, runtime_exception_syscall_error, runtime_status_is_none,
};
use sel4_common::structures::exception_t;
use sel4_common::structures_gen::{cap, mdb_node};
use sel4_common::utils::convert_to_mut_type_ref;
use vstd::prelude::*;

verus! {

const TAG_NULL: u64 = 0;
const TAG_UNTYPED: u64 = 2;
const TAG_ENDPOINT: u64 = 4;
const TAG_NOTIFICATION: u64 = 6;
const TAG_CNODE: u64 = 10;
const TAG_THREAD: u64 = 12;
const TAG_IRQ_CONTROL: u64 = 14;
const TAG_ZOMBIE: u64 = 18;

impl cte_t {
    #[inline]
    #[verifier::external_body]
    pub fn get_ptr(&self) -> (ret: usize)
        ensures
            ret == cte_slot_ptr(self),
    {
        self as *const cte_t as usize
    }

    #[inline]
    #[verifier::external_body]
    #[cfg(verus_keep_ghost)]
    pub fn get_offset_slot(&self, index: usize) -> (ret: *mut Self)
        requires
            cte_offset_slot_call_pre(cte_slot_ptr(self), index),
        ensures
            ret as usize == cte_slot_ptr(self) + core::mem::size_of::<cte_t>() * index,
    {
        convert_to_mut_type_ref::<Self>(self.get_ptr() + core::mem::size_of::<cte_t>() * index)
            as *mut Self
    }

    #[inline]
    #[verifier::external_body]
    #[cfg(not(verus_keep_ghost))]
    pub fn get_offset_slot(&self, index: usize) -> (ret: &'static mut Self)
        requires
            cte_offset_slot_call_pre(cte_slot_ptr(self), index),
        ensures
            cte_slot_ptr(ret) == cte_slot_ptr(self) + core::mem::size_of::<cte_t>() * index,
    {
        convert_to_mut_type_ref::<Self>(self.get_ptr() + core::mem::size_of::<cte_t>() * index)
    }

    #[inline]
    #[verifier::external_body]
    fn empty_mdb_node() -> mdb_node {
        mdb_node::new(0, 0, 0, 0)
    }

    #[inline]
    pub(crate) fn is_mdb_parent_of_contents(&self, next_cap: &cap, next_first_badged: bool) -> (ret: bool)
        ensures
            ret == spec_mdb_parent_of_caps(
                cte_slot_view_at(cte_slot_ptr(self)).cap,
                cte_slot_view_at(cte_slot_ptr(self)).mdb_revocable,
                trusted_view_cap(next_cap),
                next_first_badged,
            ),
    {
        let self_revocable = MdbTable::runtime_revocable_of_ref(self);
        let same_region = same_region_as(&self.capability, next_cap);
        let self_tag = runtime_cap_tag(&self.capability);
        let next_tag = runtime_cap_tag(next_cap);
        let ret = if !self_revocable {
            false
        } else if !same_region {
            false
        } else {
            match self_tag {
                TAG_ENDPOINT => {
                    let badge = runtime_cap_endpoint_badge(&self.capability);
                    badge == 0
                        || (next_tag == TAG_ENDPOINT
                            && badge == runtime_cap_endpoint_badge(next_cap)
                            && !next_first_badged)
                }
                TAG_NOTIFICATION => {
                    let badge = runtime_cap_notification_badge(&self.capability);
                    badge == 0
                        || (next_tag == TAG_NOTIFICATION
                            && badge == runtime_cap_notification_badge(next_cap)
                            && !next_first_badged)
                }
                #[cfg(feature = "enable_smc")]
                cap_tag::cap_smc_cap => {
                    let badge = cap::cap_smc_cap(&self.capability).get_capSMCBadge();
                    badge == 0 || (badge == cap::cap_smc_cap(next_cap).get_capSMCBadge() && !next_first_badged)
                }
                _ => true,
            }
        };
        proof {
            lemma_trusted_view_cte_cap_matches_cap_field(self);
            lemma_cte_slot_view_at_ptr_matches_trusted_view(self);
            lemma_trusted_view_cap_kind_matches_tag(&self.capability);
            lemma_trusted_view_cap_kind_matches_tag(next_cap);
            if self_tag == TAG_ENDPOINT || self_tag == TAG_NOTIFICATION {
                lemma_trusted_view_cap_badge_shape(&self.capability);
                lemma_trusted_view_cap_badge_shape(next_cap);
            }
            if !self_revocable {
                assert(!cte_slot_view_at(cte_slot_ptr(self)).mdb_revocable);
            }
            if self_tag == TAG_ENDPOINT {
                assert(trusted_view_cap(&self.capability).kind == CapKind::EndpointCap);
                assert(cte_slot_view_at(cte_slot_ptr(self)).cap.kind == CapKind::EndpointCap);
                assert(trusted_view_cap(&self.capability).badge is Some);
                assert(next_tag == TAG_ENDPOINT ==> trusted_view_cap(next_cap).kind == CapKind::EndpointCap);
                if next_tag == TAG_ENDPOINT {
                    assert(trusted_view_cap(next_cap).kind == CapKind::EndpointCap);
                    assert(trusted_view_cap(next_cap).badge is Some);
                }
            } else if self_tag == TAG_NOTIFICATION {
                assert(trusted_view_cap(&self.capability).kind == CapKind::NotificationCap);
                assert(cte_slot_view_at(cte_slot_ptr(self)).cap.kind == CapKind::NotificationCap);
                assert(trusted_view_cap(&self.capability).badge is Some);
                assert(next_tag == TAG_NOTIFICATION
                    ==> trusted_view_cap(next_cap).kind == CapKind::NotificationCap);
                if next_tag == TAG_NOTIFICATION {
                    assert(trusted_view_cap(next_cap).kind == CapKind::NotificationCap);
                    assert(trusted_view_cap(next_cap).badge is Some);
                }
            }
            assert(ret == spec_mdb_parent_of_caps(
                cte_slot_view_at(cte_slot_ptr(self)).cap,
                cte_slot_view_at(cte_slot_ptr(self)).mdb_revocable,
                trusted_view_cap(next_cap),
                next_first_badged,
            ));
        }
        ret
    }

    #[inline]
    pub(crate) fn is_mdb_parent_of(&self, next: &Self) -> (ret: bool)
        ensures
            ret == spec_slot_mdb_parent_of(
                cte_slot_view_at(cte_slot_ptr(self)),
                cte_slot_view_at(cte_slot_ptr(next)),
            ),
    {
        let next_first_badged = MdbTable::runtime_first_badged_of_ref(next);
        let ret = self.is_mdb_parent_of_contents(&next.capability, next_first_badged);
        proof {
            lemma_trusted_view_cte_cap_matches_cap_field(next);
            lemma_cte_slot_view_at_ptr_matches_trusted_view(next);
            assert(spec_slot_mdb_parent_of(
                cte_slot_view_at(cte_slot_ptr(self)),
                cte_slot_view_at(cte_slot_ptr(next)),
            ) == spec_mdb_parent_of_caps(
                cte_slot_view_at(cte_slot_ptr(self)).cap,
                cte_slot_view_at(cte_slot_ptr(self)).mdb_revocable,
                cte_slot_view_at(cte_slot_ptr(next)).cap,
                cte_slot_view_at(cte_slot_ptr(next)).mdb_first_badged,
            ));
        }
        ret
    }

    #[inline]
    pub fn derive_cap(&self, capability: &cap) -> (ret: deriveCap_ret)
        ensures
            is_exception_none(ret.status) || is_exception_syscall_error(ret.status),
            spec_slot_derive_cap_returns_syscall_error(
                cte_slot_view_at(cte_slot_ptr(self)),
                trusted_view_cap(capability),
            ) ==> is_exception_syscall_error(ret.status),
            spec_slot_derive_cap_returns_syscall_error(
                cte_slot_view_at(cte_slot_ptr(self)),
                trusted_view_cap(capability),
            ) ==> trusted_view_cap(&ret.capability).kind == CapKind::NullCap,
            trusted_view_cap(capability).kind != CapKind::ArchCap ==> {
                trusted_view_cap(&ret.capability) == spec_slot_derive_cap_expected_cap(
                    cte_slot_view_at(cte_slot_ptr(self)),
                    trusted_view_cap(capability),
                )
            },
            trusted_view_cap(capability).kind == CapKind::ArchCap
                && trusted_view_cap(&ret.capability).kind != CapKind::NullCap ==> {
                trusted_view_cap(&ret.capability) == trusted_view_cap(capability)
            },
    {
        if runtime_cap_is_arch(capability) {
            return self.arch_derive_cap(capability);
        }

        let mut ret = deriveCap_ret {
            status: runtime_exception_none(),
            capability: runtime_null_cap(),
        };
        let capability_tag = runtime_cap_tag(capability);
        match capability_tag {
            TAG_ZOMBIE => {
                ret.capability = runtime_null_cap();
            }
            TAG_UNTYPED => {
                ret.status = self.ensure_no_children();
                if runtime_status_is_none(ret.status) {
                    ret.capability = runtime_clone_cap(capability);
                } else {
                    ret.capability = runtime_null_cap();
                }
            }
            #[cfg(not(feature = "kernel_mcs"))]
            8 => {
                ret.capability = runtime_null_cap();
            }
            TAG_IRQ_CONTROL => {
                ret.capability = runtime_null_cap();
            }
            _ => {
                ret.capability = runtime_clone_cap(capability);
            }
        }
        let ret_status_is_none = runtime_status_is_none(ret.status);
        proof {
            lemma_trusted_view_cap_kind_matches_tag(capability);
            lemma_cte_slot_view_at_ptr_matches_trusted_view(self);
            if capability_tag == TAG_UNTYPED {
                assert(trusted_view_cap(capability).kind == CapKind::UntypedCap);
                if ret_status_is_none {
                    assert(trusted_view_cap(&ret.capability) == trusted_view_cap(capability));
                    assert(!spec_slot_ensure_no_children_blocks(cte_slot_view_at(cte_slot_ptr(self)))) by {
                        if spec_slot_ensure_no_children_blocks(cte_slot_view_at(cte_slot_ptr(self))) {
                            assert(is_exception_syscall_error(ret.status));
                            lemma_exception_syscall_error_not_none(ret.status);
                            lemma_exception_none_iff_spec_runtime_exception_none(ret.status);
                            assert(ret.status == crate::kernel_api::raw::spec_runtime_exception_none());
                            assert(false);
                        }
                    }
                } else {
                    assert(trusted_view_cap(&ret.capability).kind == CapKind::NullCap);
                    assert(spec_slot_ensure_no_children_blocks(cte_slot_view_at(cte_slot_ptr(self)))) by {
                        if !spec_slot_ensure_no_children_blocks(cte_slot_view_at(cte_slot_ptr(self))) {
                            assert(is_exception_none(ret.status));
                            lemma_exception_none_iff_spec_runtime_exception_none(ret.status);
                            assert(ret.status == crate::kernel_api::raw::spec_runtime_exception_none());
                            assert(false);
                        }
                    }
                }
            }
        }
        ret
    }

    #[inline]
    pub fn ensure_no_children(&self) -> (ret: exception_t)
        ensures
            spec_slot_ensure_no_children_blocks(cte_slot_view_at(cte_slot_ptr(self)))
                ==> is_exception_syscall_error(ret),
            !spec_slot_ensure_no_children_blocks(cte_slot_view_at(cte_slot_ptr(self)))
                ==> is_exception_none(ret),
    {
        let next = MdbTable::runtime_next_of_ref(self);
        if next != 0 {
            let next_slot = runtime_slot_ref_at(next);
            if self.is_mdb_parent_of(next_slot) {
                proof {
                    lemma_cte_slot_view_at_ptr_matches_trusted_view(self);
                    assert(cte_slot_view_at(cte_slot_ptr(self)).mdb_next is Some);
                    assert(cte_slot_view_at(cte_slot_ptr(self)).mdb_next.unwrap() == next);
                    assert(spec_slot_ensure_no_children_blocks(cte_slot_view_at(cte_slot_ptr(self))));
                }
                runtime_exception_syscall_error()
            } else {
                proof {
                    lemma_cte_slot_view_at_ptr_matches_trusted_view(self);
                    assert(cte_slot_view_at(cte_slot_ptr(self)).mdb_next is Some);
                    assert(cte_slot_view_at(cte_slot_ptr(self)).mdb_next.unwrap() == next);
                    assert(!spec_slot_ensure_no_children_blocks(cte_slot_view_at(cte_slot_ptr(self))));
                }
                runtime_exception_none()
            }
        } else {
            proof {
                lemma_cte_slot_view_at_ptr_matches_trusted_view(self);
                assert(cte_slot_view_at(cte_slot_ptr(self)).mdb_next is None);
                assert(!spec_slot_ensure_no_children_blocks(cte_slot_view_at(cte_slot_ptr(self))));
            }
            runtime_exception_none()
        }
    }

    #[inline]
    pub fn is_final_cap(&self) -> (ret: bool)
        ensures
            ret == spec_slot_is_final_cap_at(cte_slot_ptr(self)),
    {
        proof {
            lemma_cte_slot_view_at_ptr_matches_trusted_view(self);
            lemma_trusted_view_cte_cap_matches_cap_field(self);
        }
        let prev_raw = MdbTable::runtime_prev_of_ref(self);
        if prev_raw != 0 {
            let prev = runtime_slot_ref_at(prev_raw);
            proof {
                lemma_trusted_view_cte_cap_matches_cap_field(prev);
                assert(cte_slot_view_at(cte_slot_ptr(self)).mdb_prev is Some);
                assert(cte_slot_view_at(cte_slot_ptr(self)).mdb_prev.unwrap() == prev_raw);
            }
            let same_as_prev = same_object_as(&prev.capability, &self.capability);
            if same_as_prev {
                proof {
                    assert(spec_same_object_as_caps(
                        cte_slot_view_at(prev_raw).cap,
                        cte_slot_view_at(cte_slot_ptr(self)).cap,
                    ));
                    assert(false == spec_slot_is_final_cap_at(cte_slot_ptr(self)));
                }
                false
            } else {
                proof {
                    assert(!spec_same_object_as_caps(
                        cte_slot_view_at(prev_raw).cap,
                        cte_slot_view_at(cte_slot_ptr(self)).cap,
                    ));
                }
                let next_raw = MdbTable::runtime_next_of_ref(self);
                if next_raw == 0 {
                    proof {
                        assert(cte_slot_view_at(cte_slot_ptr(self)).mdb_next is None);
                        assert(true == spec_slot_is_final_cap_at(cte_slot_ptr(self)));
                    }
                    true
                } else {
                    let next = runtime_slot_ref_at(next_raw);
                    proof {
                        lemma_trusted_view_cte_cap_matches_cap_field(next);
                        assert(cte_slot_view_at(cte_slot_ptr(self)).mdb_next is Some);
                        assert(cte_slot_view_at(cte_slot_ptr(self)).mdb_next.unwrap() == next_raw);
                    }
                    let ret = !same_object_as(&self.capability, &next.capability);
                    proof {
                        assert(
                            ret == !spec_same_object_as_caps(
                                cte_slot_view_at(cte_slot_ptr(self)).cap,
                                cte_slot_view_at(next_raw).cap,
                            )
                        );
                        assert(ret == spec_slot_is_final_cap_at(cte_slot_ptr(self)));
                    }
                    ret
                }
            }
        } else {
            proof {
                assert(cte_slot_view_at(cte_slot_ptr(self)).mdb_prev is None);
            }
            let next_raw = MdbTable::runtime_next_of_ref(self);
            if next_raw == 0 {
                proof {
                    assert(cte_slot_view_at(cte_slot_ptr(self)).mdb_next is None);
                    assert(true == spec_slot_is_final_cap_at(cte_slot_ptr(self)));
                }
                true
            } else {
                let next = runtime_slot_ref_at(next_raw);
                proof {
                    lemma_trusted_view_cte_cap_matches_cap_field(next);
                    assert(cte_slot_view_at(cte_slot_ptr(self)).mdb_next is Some);
                    assert(cte_slot_view_at(cte_slot_ptr(self)).mdb_next.unwrap() == next_raw);
                }
                let ret = !same_object_as(&self.capability, &next.capability);
                proof {
                    assert(
                        ret == !spec_same_object_as_caps(
                            cte_slot_view_at(cte_slot_ptr(self)).cap,
                            cte_slot_view_at(next_raw).cap,
                        )
                    );
                    assert(ret == spec_slot_is_final_cap_at(cte_slot_ptr(self)));
                }
                ret
            }
        }
    }

    #[inline]
    pub fn is_long_running_delete(&self) -> (ret: bool)
        ensures
            ret == spec_slot_is_long_running_delete_at(cte_slot_ptr(self)),
    {
        let tag = runtime_cap_tag(&self.capability);
        let final_cap = self.is_final_cap();
        let ret = if tag == TAG_NULL || !final_cap {
            false
        } else {
            tag == TAG_THREAD || tag == TAG_ZOMBIE || tag == TAG_CNODE
        };
        proof {
            lemma_cte_slot_view_at_ptr_matches_trusted_view(self);
            lemma_trusted_view_cte_cap_matches_cap_field(self);
            lemma_trusted_view_cap_kind_matches_tag(&self.capability);
            assert(ret == spec_slot_is_long_running_delete_at(cte_slot_ptr(self)));
        }
        ret
    }

    #[inline]
    #[verifier::external_body]
    pub fn delete_all(&mut self, exposed: bool) -> (ret: exception_t) {
        crate::cspace::kernel::delete_all(self, exposed)
    }

    #[inline]
    #[verifier::external_body]
    pub fn delete_one(&mut self) {
        crate::cspace::kernel::delete_one(self)
    }

    #[inline]
    #[verifier::external_body]
    pub fn revoke(&mut self) -> (ret: exception_t) {
        crate::cspace::kernel::revoke(self)
    }
}

}
