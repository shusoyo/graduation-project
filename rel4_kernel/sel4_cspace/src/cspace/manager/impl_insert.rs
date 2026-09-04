use crate::cspace::manager::CSpaceManager;
use crate::capability::is_cap_revocable;
#[cfg(verus_keep_ghost)]
use crate::capability::spec::spec_is_cap_revocable;
#[cfg(verus_keep_ghost)]
use crate::capability::raw::trusted_view_cap;
use crate::capability::raw::runtime_clone_cap;
use crate::cspace::types::SlotPtr;
#[cfg(verus_keep_ghost)]
use crate::cspace::manager::proof as manager_proof;
#[cfg(verus_keep_ghost)]
use crate::cspace::cdt::proof as cdt_proof;
#[cfg(verus_keep_ghost)]
use crate::cspace::cte::raw::{lemma_trusted_view_cte_cap_matches_cap_field, trusted_view_cte};
use sel4_common::structures_gen::cap;
use vstd::prelude::*;
verus! {

impl CSpaceManager {
    pub fn cte_insert(&mut self, new_cap: &cap, src_slot: SlotPtr, dest_slot: SlotPtr)
        requires
            old(self).wf(),
            old(self).slot_dom().contains(src_slot),
            old(self).slot_dom().contains(dest_slot),
            old(self).slot_is_empty(dest_slot),
            !old(self).slot_is_empty(src_slot),
            src_slot != dest_slot,
            old(self).cte_insert_semantic_admissible(src_slot, dest_slot, trusted_view_cap(new_cap)),
        ensures
            self.wf(),
            old(self).cte_insert_rel(
                self,
                src_slot,
                dest_slot,
                trusted_view_cap(new_cap),
            ),
    {
        let src_ref = self.get_slot(src_slot);
        let dest_ref = self.get_slot(dest_slot);
        let src_cap = runtime_clone_cap(&src_ref.capability);
        let old_next = self.mdb.runtime_next(src_slot);
        let old_next_opt = if old_next == 0 { None } else { Some(old_next) };
        let new_cap_is_revocable = is_cap_revocable(new_cap, &src_cap);
        let Ghost(old_mgr) = Ghost(*self);
        let Ghost(old_cdt) = Ghost(old_mgr.cdt@);
        let Ghost(dest_original) = Ghost(old_mgr.insert_dest_original_of(
            src_slot,
            trusted_view_cap(new_cap),
        ));
        let Ghost(src_parent) = Ghost(old_mgr.insert_src_parent_of(
            src_slot,
            trusted_view_cap(new_cap),
        ));
        let Ghost(new_cdt) = Ghost(old_mgr.cdt@.state_after_cap_insert(
            src_slot,
            dest_slot,
            src_parent,
            dest_original,
        ));

        Self::assert_slot_empty_runtime(dest_ref);

        let Tracked(mut src_perm) = self.mdb.take_entry_perm(src_slot);
        let Tracked(mut dest_perm) = self.mdb.take_entry_perm(dest_slot);
        crate::cspace::cte::payload::set_untyped_cap_as_full_tracked(
            src_slot,
            Tracked(&mut src_perm),
            &src_cap,
            new_cap,
        );
        crate::cspace::cte::payload::write_slot_cap_only_tracked(
            dest_slot,
            Tracked(&mut dest_perm),
            new_cap,
        );
        let Ghost(src_entry_after_payload) = Ghost(
            crate::cspace::cte::raw::trusted_slot_perm_view(src_perm),
        );
        let Ghost(dest_entry_after_payload) = Ghost(
            crate::cspace::cte::raw::trusted_slot_perm_view(dest_perm),
        );
        self.mdb.put_entry_perm(src_slot, Tracked(src_perm));
        self.mdb.put_entry_perm(dest_slot, Tracked(dest_perm));
        let Ghost(pre_mdb_insert_mgr) = Ghost(*self);
        proof {
            assert(self.mdb.entry_view(src_slot) == src_entry_after_payload);
            assert(self.mdb.entry_view(dest_slot) == dest_entry_after_payload);
            old_mgr.mdb.lemma_two_modified_slots_same_mdb_fields_preserve_structural_wf(
                &self.mdb,
                src_slot,
                dest_slot,
            );
            assert(self.mdb.slot_is_detached(dest_slot));
            assert(crate::capability::spec::same_cap_except_untyped_free_index(
                old_mgr.get_cap(src_slot),
                self.get_cap(src_slot),
            ));
            manager_proof::lemma_manager_mdb_live_slots_match_nonempty_from_manager(old_mgr);
            manager_proof::lemma_manager_slot_is_empty_matches_mdb(old_mgr, dest_slot);
            assert(!old_mgr.mdb.live_slots@.contains(dest_slot));
        }

        self.mdb.insert_node_after(
            src_slot,
            dest_slot,
            old_next_opt,
            new_cap_is_revocable,
            new_cap_is_revocable,
        );
        self.cdt = Ghost(new_cdt);
        proof {
            lemma_trusted_view_cte_cap_matches_cap_field(src_ref);
            assert(new_cap_is_revocable == spec_is_cap_revocable(
                trusted_view_cap(new_cap),
                trusted_view_cap(&src_cap),
            ));
            assert(trusted_view_cap(&src_cap) == old_mgr.get_cap(src_slot));
            assert(new_cap_is_revocable == spec_is_cap_revocable(
                trusted_view_cap(new_cap),
                old_mgr.get_cap(src_slot),
            ));
            assert(pre_mdb_insert_mgr.get_cap(src_slot) == self.get_cap(src_slot));
            assert(crate::capability::spec::same_cap_except_untyped_free_index(
                old_mgr.get_cap(src_slot),
                pre_mdb_insert_mgr.get_cap(src_slot),
            ));
            assert(crate::capability::spec::same_cap_except_untyped_free_index(
                old_mgr.get_cap(src_slot),
                self.get_cap(src_slot),
            ));
            assert(old_next_opt == old_mgr.get_next(src_slot));
            old_mgr.mdb.lemma_same_links_then_insert_between_rel(
                &pre_mdb_insert_mgr.mdb,
                &self.mdb,
                src_slot,
                dest_slot,
                old_next_opt,
                dest_original,
                dest_original,
            );
            assert forall|other: SlotPtr| #[trigger]
                old_mgr.slot_dom().contains(other) && other != src_slot && other != dest_slot
                    implies self.get_cap(other) == old_mgr.get_cap(other) by {
                if old_mgr.slot_dom().contains(other) && other != src_slot && other != dest_slot {
                    assert(pre_mdb_insert_mgr.get_cap(other) == old_mgr.get_cap(other));
                    assert(self.get_cap(other) == pre_mdb_insert_mgr.get_cap(other));
                }
            };

            cdt_proof::lemma_parent_slots_wf_implies_empty_slot_is_parentless(
                old_mgr.cdt@,
                old_mgr.mdb.nonempty_slots(),
                dest_slot,
            );

            cdt_proof::lemma_state_after_cap_insert_preserves_structural_wf(
                old_cdt,
                new_cdt,
                src_slot,
                dest_slot,
                src_parent,
                dest_original,
            );
            manager_proof::lemma_insert_preserves_manager_semantics_wf(
                old_mgr,
                *self,
                src_slot,
                dest_slot,
            );
        }

    }

    pub fn insert_new_cap(&mut self, parent: SlotPtr, slot: SlotPtr, capability: &cap)
        requires
            old(self).wf(),
            old(self).slot_dom().contains(parent),
            old(self).slot_dom().contains(slot),
            old(self).slot_is_empty(slot),
            !old(self).slot_is_empty(parent),
            parent != slot,
            old(self).insert_new_cap_semantic_admissible(parent, slot, trusted_view_cap(capability)),
        ensures
            self.wf(),
            old(self).insert_new_cap_rel(
                self,
                parent,
                slot,
                trusted_view_cap(capability),
            ),
    {
        let parent_ref = self.get_slot(parent);
        let next = self.mdb.runtime_next(parent);
        let next_opt = if next == 0 { None } else { Some(next) };
        let parent_cap = runtime_clone_cap(&parent_ref.capability);
        let Ghost(old_mgr) = Ghost(*self);
        let Ghost(old_cdt) = Ghost(old_mgr.cdt@);
        let Ghost(new_cdt) = Ghost(old_mgr.cdt@.state_after_insert_new_cap(parent, slot));

        let Tracked(mut slot_perm) = self.mdb.take_entry_perm(slot);
        crate::cspace::cte::payload::write_slot_cap_only_tracked(
            slot,
            Tracked(&mut slot_perm),
            capability,
        );
        let Ghost(slot_entry_after_payload) = Ghost(
            crate::cspace::cte::raw::trusted_slot_perm_view(slot_perm),
        );
        self.mdb.put_entry_perm(slot, Tracked(slot_perm));
        let Ghost(pre_mdb_insert_mgr) = Ghost(*self);
        proof {
            assert(self.mdb.entry_view(slot) == slot_entry_after_payload);
            old_mgr.mdb.lemma_one_modified_slot_same_mdb_fields_preserve_structural_wf(
                &self.mdb,
                slot,
            );
            assert(self.mdb.slot_is_detached(slot));
            manager_proof::lemma_manager_mdb_live_slots_match_nonempty_from_manager(old_mgr);
            manager_proof::lemma_manager_slot_is_empty_matches_mdb(old_mgr, slot);
            assert(!old_mgr.mdb.live_slots@.contains(slot));
        }

        self.mdb.insert_node_after(
            parent,
            slot,
            next_opt,
            true,
            true,
        );
        self.cdt = Ghost(new_cdt);
        proof {
            lemma_trusted_view_cte_cap_matches_cap_field(parent_ref);
            assert(old_mgr.insert_new_cap_rel(
                self,
                parent,
                slot,
                trusted_view_cap(capability),
            ));
            old_mgr.mdb.lemma_same_links_then_insert_between_rel(
                &pre_mdb_insert_mgr.mdb,
                &self.mdb,
                parent,
                slot,
                next_opt,
                true,
                true,
            );
            assert forall|other: SlotPtr| #[trigger]
                old_mgr.slot_dom().contains(other) && other != slot
                    implies self.get_cap(other) == old_mgr.get_cap(other) by {
                if old_mgr.slot_dom().contains(other) && other != slot {
                    assert(self.get_cap(other) == old_mgr.get_cap(other));
                }
            };

            cdt_proof::lemma_parent_slots_wf_implies_empty_slot_is_parentless(
                old_mgr.cdt@,
                old_mgr.mdb.nonempty_slots(),
                slot,
            );

            cdt_proof::lemma_state_after_insert_new_cap_preserves_structural_wf(
                old_cdt,
                new_cdt,
                parent,
                slot,
            );
            manager_proof::lemma_insert_new_cap_preserves_manager_semantics_wf(
                old_mgr,
                *self,
                parent,
                slot,
            );
        }
    }
}

} // verus!
