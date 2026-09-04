use crate::cspace::manager::CSpaceManager;
#[cfg(verus_keep_ghost)]
use crate::capability::spec::CapKind;
#[cfg(verus_keep_ghost)]
use crate::capability::raw::trusted_view_cap;
use crate::capability::raw::runtime_null_cap;
use crate::cspace::types::SlotPtr;
#[cfg(verus_keep_ghost)]
use crate::cspace::cdt::proof as cdt_proof;
#[cfg(verus_keep_ghost)]
use crate::cspace::manager::proof as manager_proof;
use sel4_common::structures_gen::cap;
use vstd::prelude::*;
verus! {

impl CSpaceManager {
    pub fn cte_move(&mut self, new_cap: &cap, src_slot: SlotPtr, dest_slot: SlotPtr)
        requires
            old(self).wf(),
            old(self).slot_dom().contains(src_slot),
            old(self).slot_dom().contains(dest_slot),
            !old(self).slot_is_empty(src_slot),
            old(self).slot_is_empty(dest_slot),
            old(self).get_cap(src_slot) == trusted_view_cap(new_cap),
            src_slot != dest_slot,
        ensures
            self.wf(),
            old(self).cte_move_rel(self, src_slot, dest_slot, trusted_view_cap(new_cap)),
    {
        let dest_ref = self.get_slot(dest_slot);
        let Ghost(old_mgr) = Ghost(*self);
        let Ghost(old_cdt) = Ghost(old_mgr.cdt@);

        Self::assert_slot_empty_runtime(dest_ref);

        let Tracked(mut src_perm) = self.mdb.take_entry_perm(src_slot);
        let Tracked(mut dest_perm) = self.mdb.take_entry_perm(dest_slot);

        crate::cspace::cte::payload::write_slot_cap_only_tracked(dest_slot, Tracked(&mut dest_perm), new_cap);
        crate::cspace::cte::payload::write_slot_cap_only_tracked(src_slot, Tracked(&mut src_perm), &runtime_null_cap());

        let Ghost(src_entry_after_payload) = Ghost(
            crate::cspace::cte::raw::trusted_slot_perm_view(src_perm),
        );
        let Ghost(dest_entry_after_payload) = Ghost(
            crate::cspace::cte::raw::trusted_slot_perm_view(dest_perm),
        );
        self.mdb.put_entry_perm(src_slot, Tracked(src_perm));
        self.mdb.put_entry_perm(dest_slot, Tracked(dest_perm));
        let Ghost(pre_mdb_move_mgr) = Ghost(*self);
        proof {
            assert(self.mdb.entry_view(src_slot) == src_entry_after_payload);
            assert(self.mdb.entry_view(dest_slot) == dest_entry_after_payload);
            old_mgr.mdb.lemma_two_modified_slots_same_mdb_fields_preserve_structural_wf(
                &self.mdb,
                src_slot,
                dest_slot,
            );
            manager_proof::lemma_manager_mdb_live_slots_match_nonempty_from_manager(old_mgr);
            manager_proof::lemma_manager_slot_is_empty_matches_mdb(old_mgr, dest_slot);
            assert(!old_mgr.mdb.live_slots@.contains(dest_slot));
        }

        self.mdb.move_node(src_slot, dest_slot);

        self.zombie_slots = Ghost(if trusted_view_cap(new_cap).kind == CapKind::ZombieCap {
            self.zombie_slots@.remove(src_slot).remove(dest_slot).insert(dest_slot)
        } else {
            self.zombie_slots@.remove(src_slot).remove(dest_slot)
        });
        self.cdt = Ghost(old_mgr.cdt@.state_after_move(src_slot, dest_slot));
        proof {
            let Ghost(new_cdt) = Ghost(self.cdt@);
            assert forall|other: SlotPtr| #[trigger]
                old_mgr.slot_dom().contains(other) && other != src_slot && other != dest_slot
                    implies self.get_cap(other) == old_mgr.get_cap(other) by {
                if old_mgr.slot_dom().contains(other) && other != src_slot && other != dest_slot {
                    assert(pre_mdb_move_mgr.get_cap(other) == old_mgr.get_cap(other));
                    assert(self.get_cap(other) == pre_mdb_move_mgr.get_cap(other));
                }
            };
            assert(old_mgr.cte_move_rel(
                self,
                src_slot,
                dest_slot,
                old_mgr.get_cap(src_slot),
            ));
            old_mgr.mdb.lemma_same_links_then_move_slot_rel(
                &pre_mdb_move_mgr.mdb,
                &self.mdb,
                src_slot,
                dest_slot,
            );
            cdt_proof::lemma_state_after_move_preserves_structural_wf(
                old_cdt,
                new_cdt,
                src_slot,
                dest_slot,
            );
            manager_proof::lemma_move_preserves_manager_semantics_wf(
                old_mgr,
                *self,
                src_slot,
                dest_slot,
            );
        }

    }
}

} // verus!
