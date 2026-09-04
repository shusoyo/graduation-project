use crate::cspace::manager::CSpaceManager;
#[cfg(verus_keep_ghost)]
use crate::capability::raw::{
    lemma_trusted_view_cap_kind_matches_tag, spec_cap_cyclic_zombie, spec_cap_removable,
    trusted_view_cap,
};
use crate::capability::raw::{
    runtime_cap_set_zombie_number, runtime_cap_tag, runtime_cap_zombie_number,
    runtime_cap_zombie_ptr, runtime_clone_cap, runtime_null_cap,
};
use crate::capability::zombie::cap_cyclic_zombie;
use crate::cspace::cte::cte_t;
#[cfg(verus_keep_ghost)]
use crate::cspace::cte::spec::{
    lemma_slot_cap_update_rel_implies_same_mdb_fields, same_mdb_fields, slot_cap_update_rel,
};
#[cfg(verus_keep_ghost)]
use crate::cspace::cte::raw::{
    lemma_cte_slot_view_at_ptr_matches_trusted_view, lemma_trusted_view_cte_cap_matches_cap_field,
    lemma_trusted_view_cte_matches_slot_perm_view_ref,
};
use crate::cspace::types::SlotPtr;
use crate::deps::{finalise_cap, post_cap_deletion, preemption_point};
#[cfg(verus_keep_ghost)]
use crate::deps::raw::{
    lemma_finalise_cap_non_immediate_nonremovable_projects_affected_cdt_parent_semantics,
    lemma_finalise_cap_non_immediate_nonremovable_projects_affected_incoming_edges,
    lemma_finalise_cap_non_immediate_nonremovable_projects_reduce_ready,
    lemma_finalise_cap_non_immediate_nonremovable_projects_reduce_target_admissible,
};
#[cfg(verus_keep_ghost)]
use crate::kernel_api::raw::{
    is_exception_none, lemma_exception_none_iff_spec_runtime_exception_none,
};
use crate::kernel_api::raw::{runtime_exception_none, runtime_status_is_none};
#[cfg(verus_keep_ghost)]
use crate::cspace::manager::proof as manager_proof;
use crate::structures::finaliseSlot_ret;
use core::ptr;
use sel4_common::structures_gen::{cap, cap_tag};
use sel4_common::utils::{convert_to_mut_type_ref, convert_to_type_ref};
use vstd::prelude::*;
use vstd::simple_pptr::{self, PPtr};

verus! {

const TAG_NULL: u64 = 0;
const TAG_ZOMBIE: u64 = 18;

impl CSpaceManager {
    #[inline]
    #[verifier::external_body]
    pub(crate) fn slot_ref(&self, slot: SlotPtr) -> &'static cte_t {
        convert_to_type_ref::<cte_t>(slot)
    }

    #[inline]
    #[verifier::external_body]
    pub(crate) fn slot_mut(&mut self, slot: SlotPtr) -> *mut cte_t {
        convert_to_mut_type_ref::<cte_t>(slot) as *mut cte_t
    }

    #[inline]
    #[verifier::external_body]
    pub(crate) fn slot_as_mut_ptr(slot: SlotPtr) -> (ret: *mut cte_t)
        ensures
            ret as usize == slot,
    {
        slot as *mut cte_t
    }

    #[inline]
    #[verifier::external_body]
    pub(crate) fn assert_slot_empty_runtime(slot_ref: &cte_t) {
        assert_eq!(slot_ref.capability.get_tag(), cap_tag::cap_null_cap);
    }

    #[verifier::exec_allows_no_decreases_clause]
    pub(crate) unsafe fn finalise_slot(
        &mut self,
        slot: SlotPtr,
        immediate: bool,
    ) -> (ret: finaliseSlot_ret)
        requires
            old(self).wf(),
            old(self).slot_dom().contains(slot),
        ensures
            self.wf(),
            self.slot_dom() =~= old(self).slot_dom(),
            crate::kernel_api::raw::is_exception_none(ret.status) && ret.success ==> self.finalise_slot_success_rel(
                slot,
                trusted_view_cap(&ret.cleanupInfo),
            ),
            crate::kernel_api::raw::is_exception_none(ret.status) && !ret.success ==> self.finalise_slot_cyclic_failure_rel(
                slot,
                immediate,
            ),
            !crate::kernel_api::raw::is_exception_none(ret.status) ==> {
                &&& !ret.success
                &&& trusted_view_cap(&ret.cleanupInfo) == crate::capability::spec::spec_null_cap()
            },
    {
        let mut ret = finaliseSlot_ret {
            status: runtime_exception_none(),
            success: true,
            cleanupInfo: runtime_null_cap(),
        };
        loop
            invariant
                self.wf(),
                self.slot_dom() =~= old(self).slot_dom(),
                self.slot_dom().contains(slot),
                is_exception_none(ret.status),
                ret.success,
                trusted_view_cap(&ret.cleanupInfo) == crate::capability::spec::spec_null_cap(),
        {
            let slot_ref = self.get_slot(slot);
            let slot_tag = runtime_cap_tag(&slot_ref.capability);
            let null_tag = TAG_NULL;
            if slot_tag == null_tag {
                proof {
                    lemma_trusted_view_cte_cap_matches_cap_field(slot_ref);
                    lemma_trusted_view_cap_kind_matches_tag(&slot_ref.capability);
                    assert(self.get_cap(slot) == trusted_view_cap(&slot_ref.capability));
                    assert(self.slot_is_empty(slot));
                    assert(trusted_view_cap(&ret.cleanupInfo)
                        == crate::capability::spec::spec_null_cap());
                    assert(self.finalise_slot_success_rel(
                        slot,
                        trusted_view_cap(&ret.cleanupInfo),
                    ));
                }
                return ret;
            }

            let Ghost(loop_mgr) = Ghost(*self);
            let current_cap = runtime_clone_cap(&slot_ref.capability);
            let final_cap = slot_ref.is_final_cap();
            let fc_ret = finalise_cap(&current_cap, final_cap, false);
            if crate::capability::cap_removable(&fc_ret.remainder, slot) {
                proof {
                    lemma_trusted_view_cte_cap_matches_cap_field(slot_ref);
                    lemma_trusted_view_cap_kind_matches_tag(&slot_ref.capability);
                    lemma_cte_slot_view_at_ptr_matches_trusted_view(slot_ref);
                    loop_mgr.lemma_spec_get_slot_ptr_matches_slot(slot);
                    loop_mgr.lemma_spec_is_final_cap_at_matches_cte(slot);
                    assert(slot_ref == loop_mgr.spec_get_slot(slot));
                    assert(crate::cspace::cte::spec::cte_slot_ptr(slot_ref) == slot);
                    assert(final_cap == crate::cspace::cte::spec::spec_slot_is_final_cap_at(slot));
                    assert(self.get_cap(slot) == trusted_view_cap(&slot_ref.capability));
                    assert(trusted_view_cap(&current_cap) == self.get_cap(slot));
                    assert(final_cap == loop_mgr.spec_is_final_cap_at(slot));
                    assert(self.finalise_slot_success_rel(
                        slot,
                        trusted_view_cap(&fc_ret.cleanupInfo),
                    ));
                }
                ret.status = runtime_exception_none();
                ret.success = true;
                ret.cleanupInfo = fc_ret.cleanupInfo;
                return ret;
            }

            proof {
                lemma_trusted_view_cte_cap_matches_cap_field(slot_ref);
                lemma_trusted_view_cap_kind_matches_tag(&slot_ref.capability);
                lemma_cte_slot_view_at_ptr_matches_trusted_view(slot_ref);
                loop_mgr.lemma_spec_get_slot_ptr_matches_slot(slot);
                loop_mgr.lemma_spec_is_final_cap_at_matches_cte(slot);
                assert(slot_ref == loop_mgr.spec_get_slot(slot));
                assert(crate::cspace::cte::spec::cte_slot_ptr(slot_ref) == slot);
                assert(final_cap == crate::cspace::cte::spec::spec_slot_is_final_cap_at(slot));
                assert(self.get_cap(slot) == trusted_view_cap(&slot_ref.capability));
                assert(trusted_view_cap(&current_cap) == self.get_cap(slot));
                assert(final_cap == loop_mgr.spec_is_final_cap_at(slot));
                assert(!self.slot_is_empty(slot));
                lemma_finalise_cap_non_immediate_nonremovable_projects_reduce_ready(
                    self.get_cap(slot),
                    final_cap,
                    slot,
                );
                lemma_finalise_cap_non_immediate_nonremovable_projects_reduce_target_admissible(
                    *self,
                    slot,
                    immediate,
                );
            }
            self.finalise_slot_write_remainder_bridge(slot, &fc_ret.remainder);
            proof {
                assert(self.get_cap(slot) == trusted_view_cap(&fc_ret.remainder));
                assert(!spec_cap_removable(self.get_cap(slot), slot));
            }
            let cyclic = cap_cyclic_zombie(
                &fc_ret.remainder,
                Self::slot_as_mut_ptr(slot),
            );
            if !immediate && cyclic {
                proof {
                    assert(loop_mgr.finalise_slot_cap_write_rel(
                        self,
                        slot,
                        trusted_view_cap(&fc_ret.remainder),
                    ));
                    assert(self.get_cap(slot) == trusted_view_cap(&fc_ret.remainder));
                    assert(self.get_cap(slot).kind == crate::capability::spec::CapKind::ZombieCap);
                    assert(spec_cap_cyclic_zombie(self.get_cap(slot), slot));
                    assert(!self.slot_is_empty(slot));
                    assert(self.finalise_slot_cyclic_failure_rel(slot, immediate));
                }
                ret.status = runtime_exception_none();
                ret.success = false;
                ret.cleanupInfo = fc_ret.cleanupInfo;
                return ret;
            }

            let status = self.reduce_zombie(slot, immediate);
            let status_is_none = runtime_status_is_none(status);
            if !status_is_none {
                proof {
                    assert(!is_exception_none(status)) by {
                        if is_exception_none(status) {
                            lemma_exception_none_iff_spec_runtime_exception_none(status);
                            assert(status == crate::kernel_api::raw::spec_runtime_exception_none());
                            assert(status_is_none);
                            assert(false);
                        }
                    }
                }
                ret.status = status;
                ret.success = false;
                ret.cleanupInfo = runtime_null_cap();
                return ret;
            }

            let status = self.preemption_point_bridge();
            let status_is_none = runtime_status_is_none(status);
            if !status_is_none {
                proof {
                    assert(!is_exception_none(status)) by {
                        if is_exception_none(status) {
                            lemma_exception_none_iff_spec_runtime_exception_none(status);
                            assert(status == crate::kernel_api::raw::spec_runtime_exception_none());
                            assert(status_is_none);
                            assert(false);
                        }
                    }
                }
                ret.status = status;
                ret.success = false;
                ret.cleanupInfo = runtime_null_cap();
                return ret;
            }
            proof {
                assert(self.wf()) by {
                    assert(self.wf() == loop_mgr.wf());
                }
                assert(self.slot_dom() =~= loop_mgr.slot_dom());
                assert(self.slot_dom().contains(slot));
            }
        }
    }

    pub(crate) fn finalise_slot_write_remainder_bridge(&mut self, slot: SlotPtr, capability: &cap)
        requires
            old(self).wf(),
            old(self).slot_dom().contains(slot),
            !old(self).slot_is_empty(slot),
            trusted_view_cap(capability) == crate::deps::raw::finalise_cap_contract(
                old(self).get_cap(slot),
                old(self).spec_is_final_cap_at(slot),
                false,
            ).0,
            !spec_cap_removable(trusted_view_cap(capability), slot),
            trusted_view_cap(capability).kind == crate::capability::spec::CapKind::ZombieCap,
            0 < crate::capability::raw::spec_zombie_number_cap(trusted_view_cap(capability)),
        ensures
            self.wf(),
            old(self).finalise_slot_cap_write_rel(self, slot, trusted_view_cap(capability)),
    {
        let Ghost(old_mgr) = Ghost(*self);
        let Tracked(mut slot_perm) = self.mdb.take_entry_perm(slot);
        let Ghost(slot_entry_before_payload) = Ghost(
            crate::cspace::cte::raw::trusted_slot_perm_view(slot_perm),
        );
        crate::cspace::cte::payload::write_slot_cap_only_tracked(
            slot,
            Tracked(&mut slot_perm),
            capability,
        );
        let Ghost(slot_entry_after_payload) = Ghost(
            crate::cspace::cte::raw::trusted_slot_perm_view(slot_perm),
        );
        self.mdb.put_entry_perm(slot, Tracked(slot_perm));
        proof {
            assert(slot_entry_before_payload == old_mgr.get_slot_view(slot));
            assert(self.mdb.entry_view(slot) == slot_entry_after_payload);
            assert(slot_cap_update_rel(
                slot_entry_before_payload,
                slot_entry_after_payload,
                trusted_view_cap(capability),
            ));
            lemma_slot_cap_update_rel_implies_same_mdb_fields(
                slot_entry_before_payload,
                slot_entry_after_payload,
                trusted_view_cap(capability),
            );
            assert(old_mgr.mdb.entries_unchanged_except(&self.mdb, set![slot]));
            old_mgr.mdb.lemma_one_modified_slot_same_mdb_fields_preserve_structural_wf(
                &self.mdb,
                slot,
            );
            assert(old_mgr.finalise_slot_cap_write_rel(self, slot, trusted_view_cap(capability))) by {
                assert(self.slot_dom() =~= old_mgr.slot_dom());
                assert(self.cdt@ == old_mgr.cdt@);
                assert(self.zombie_slots@ =~= old_mgr.zombie_slots@);
                assert(self.mdb.order@ =~= old_mgr.mdb.order@);
                assert(self.mdb.live_slots@ =~= old_mgr.mdb.live_slots@);
                assert(self.get_cap(slot) == trusted_view_cap(capability));
            }
            manager_proof::lemma_finalise_slot_cap_write_preserves_easy_wf_components(
                old_mgr,
                *self,
                slot,
                trusted_view_cap(capability),
            );
            lemma_finalise_cap_non_immediate_nonremovable_projects_affected_incoming_edges(
                old_mgr,
                *self,
                slot,
            );
            lemma_finalise_cap_non_immediate_nonremovable_projects_affected_cdt_parent_semantics(
                old_mgr,
                *self,
                slot,
            );
            manager_proof::lemma_finalise_slot_cap_write_preserves_hard_wf_from_affected(
                old_mgr,
                *self,
                slot,
                trusted_view_cap(capability),
            );
            assert(self.wf());
        }
    }

    // Temporary semantic TCB: this bridge packages the raw preemption hook as a
    // manager-preservation contract so delete-side callers only depend on the
    // explicit frame/preservation post, not on the raw extern directly.
    #[verifier::external_body]
    pub(crate) fn preemption_point_bridge(&mut self) -> (ret: sel4_common::structures::exception_t)
        ensures
            crate::deps::raw::preemption_point_preserves_manager(*old(self), *self, ret),
            self.wf() == old(self).wf(),
            self.mdb.entries@ == old(self).mdb.entries@,
            self.mdb.order@ =~= old(self).mdb.order@,
            self.mdb.live_slots@ =~= old(self).mdb.live_slots@,
            self.cdt@ == old(self).cdt@,
            self.zombie_slots@ =~= old(self).zombie_slots@,
    {
        preemption_point()
    }

    // Temporary semantic TCB: this bridge packages the raw cleanup hook as a
    // visible-C-space preservation contract so delete-side callers can keep the
    // hook outside their local semantic reasoning.
    #[verifier::external_body]
    pub(crate) fn post_cap_deletion_bridge(&mut self, cleanup_info: &cap)
        ensures
            crate::deps::raw::post_cap_deletion_preserves_visible_cspace(
                *old(self),
                *self,
                trusted_view_cap(cleanup_info),
            ),
            self.wf() == old(self).wf(),
            self.mdb.entries@ == old(self).mdb.entries@,
            self.mdb.order@ =~= old(self).mdb.order@,
            self.mdb.live_slots@ =~= old(self).mdb.live_slots@,
            self.cdt@ == old(self).cdt@,
            self.zombie_slots@ =~= old(self).zombie_slots@,
    {
        post_cap_deletion(cleanup_info);
    }

    #[cfg(target_arch = "riscv64")]
    #[inline]
    #[verifier::external_body]
    pub(crate) fn get_volatile_value(&self, slot: SlotPtr) -> (ret: usize)
        requires
            self.slot_dom().contains(slot),
        ensures
            self.get_next(slot) is Some ==> ret == self.get_next(slot).unwrap(),
            self.get_next(slot) is None ==> ret == 0,
            ret != 0 ==> self.get_next(slot) == Some(ret),
            ret == 0 ==> self.get_next(slot) is None,
    {
        unsafe {
            let raw_value = ptr::read_volatile((slot + 24) as *const usize);
            let mut value = ((raw_value >> 2) & mask_bits!(37)) << 2;
            if (value & (1usize << 38)) != 0 {
                value |= 0xffffff8000000000;
            }
            value
        }
    }

    #[cfg(target_arch = "aarch64")]
    #[inline]
    #[verifier::external_body]
    pub(crate) fn get_volatile_value(&self, slot: SlotPtr) -> (ret: usize)
        requires
            self.slot_dom().contains(slot),
        ensures
            self.get_next(slot) is Some ==> ret == self.get_next(slot).unwrap(),
            self.get_next(slot) is None ==> ret == 0,
            ret != 0 ==> self.get_next(slot) == Some(ret),
            ret == 0 ==> self.get_next(slot) is None,
    {
        unsafe {
            let raw_value = ptr::read_volatile((slot + 24) as *const usize);
            let mut value = ((raw_value >> 2) & mask_bits!(46)) << 2;
            if (value & (1usize << 46)) != 0 {
                #[cfg(not(feature = "hypervisor"))]
                {
                    value |= 0xffffff8000000000;
                }
                #[cfg(feature = "hypervisor")]
                {
                    value |= 0x8000000000;
                }
            }
            value
        }
    }
    pub fn borrow_slot_with_perm<'a>(
        slot: SlotPtr,
        Tracked(slot_perm): Tracked<&'a simple_pptr::PointsTo<cte_t>>,
    ) -> (ret: &'a cte_t)
        requires
            slot_perm.is_init(),
            slot_perm.addr() == slot,
        ensures
            ret == &slot_perm.value(),
    {
        let slot_ref: &cte_t = PPtr::<cte_t>::from_usize(slot).borrow(Tracked(slot_perm));
        slot_ref
    }

    pub(crate) fn set_slot_zombie_number_runtime(&mut self, slot: SlotPtr, zombie_number: usize)
        requires
            old(self).wf(),
            old(self).slot_dom().contains(slot),
            old(self).get_cap(slot).kind == crate::capability::spec::CapKind::ZombieCap,
        ensures
            self.wf(),
            old(self).zombie_slot_number_update_rel(self, slot, zombie_number),
    {
        let Ghost(old_mgr) = Ghost(*self);
        let Tracked(mut slot_perm) = self.mdb.take_entry_perm(slot);
        let Ghost(slot_entry_before_payload) = Ghost(
            crate::cspace::cte::raw::trusted_slot_perm_view(slot_perm),
        );
        let slot_ref = CSpaceManager::borrow_slot_with_perm(slot, Tracked(&slot_perm));
        let updated_cap = runtime_cap_set_zombie_number(&slot_ref.capability, zombie_number);
        proof {
            lemma_trusted_view_cte_matches_slot_perm_view_ref(slot_ref, &slot_perm);
            lemma_trusted_view_cte_cap_matches_cap_field(slot_ref);
            assert(crate::cspace::cte::raw::trusted_view_cte(slot_ref) == slot_entry_before_payload);
            assert(trusted_view_cap(&slot_ref.capability) == old_mgr.get_cap(slot));
            assert(trusted_view_cap(&updated_cap).kind == old_mgr.get_cap(slot).kind);
            assert(trusted_view_cap(&updated_cap).object == old_mgr.get_cap(slot).object);
            assert(trusted_view_cap(&updated_cap).region_id == old_mgr.get_cap(slot).region_id);
            assert(trusted_view_cap(&updated_cap).rights == old_mgr.get_cap(slot).rights);
            assert(trusted_view_cap(&updated_cap).badge == old_mgr.get_cap(slot).badge);
            assert(trusted_view_cap(&updated_cap).cnode == old_mgr.get_cap(slot).cnode);
            assert(trusted_view_cap(&updated_cap).untyped == old_mgr.get_cap(slot).untyped);
            assert(crate::capability::raw::spec_zombie_ptr_cap(trusted_view_cap(&updated_cap))
                == crate::capability::raw::spec_zombie_ptr_cap(old_mgr.get_cap(slot)));
            assert(crate::capability::raw::spec_zombie_type_cap(trusted_view_cap(&updated_cap))
                == crate::capability::raw::spec_zombie_type_cap(old_mgr.get_cap(slot)));
            assert(crate::capability::raw::spec_zombie_number_cap(trusted_view_cap(&updated_cap))
                == zombie_number);
        }
        crate::cspace::cte::payload::write_slot_cap_only_tracked(
            slot,
            Tracked(&mut slot_perm),
            &updated_cap,
        );
        let Ghost(slot_entry_after_payload) = Ghost(
            crate::cspace::cte::raw::trusted_slot_perm_view(slot_perm),
        );
        self.mdb.put_entry_perm(slot, Tracked(slot_perm));
        proof {
            assert(slot_entry_before_payload == old_mgr.get_slot_view(slot));
            assert(self.mdb.entry_view(slot) == slot_entry_after_payload);
            assert(slot_cap_update_rel(
                slot_entry_before_payload,
                slot_entry_after_payload,
                trusted_view_cap(&updated_cap),
            ));
            lemma_slot_cap_update_rel_implies_same_mdb_fields(
                slot_entry_before_payload,
                slot_entry_after_payload,
                trusted_view_cap(&updated_cap),
            );
            assert(old_mgr.mdb.entries_unchanged_except(&self.mdb, set![slot]));
            old_mgr.mdb.lemma_one_modified_slot_same_mdb_fields_preserve_structural_wf(
                &self.mdb,
                slot,
            );
            assert(old_mgr.zombie_slot_number_update_rel(self, slot, zombie_number)) by {
                assert(self.slot_dom() =~= old_mgr.slot_dom());
                assert(self.cdt@ == old_mgr.cdt@);
                assert(self.zombie_slots@ =~= old_mgr.zombie_slots@);
                assert(self.mdb.order@ =~= old_mgr.mdb.order@);
                assert(self.mdb.live_slots@ =~= old_mgr.mdb.live_slots@);
                assert(same_mdb_fields(old_mgr.get_slot_view(slot), self.get_slot_view(slot)));
                assert(self.get_cap(slot) == trusted_view_cap(&updated_cap));
            }
            manager_proof::lemma_zombie_slot_number_update_preserves_easy_wf_components(
                old_mgr,
                *self,
                slot,
                zombie_number,
            );
            manager_proof::lemma_zombie_slot_number_update_preserves_hard_wf(
                old_mgr,
                *self,
                slot,
                zombie_number,
            );
            assert(self.wf());
        }
    }
}

} // verus!
