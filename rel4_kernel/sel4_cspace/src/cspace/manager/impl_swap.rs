use crate::cspace::manager::CSpaceManager;
#[cfg(verus_keep_ghost)]
use crate::capability::raw::trusted_view_cap;
#[cfg(verus_keep_ghost)]
use crate::capability::spec::CapKind;
#[cfg(verus_keep_ghost)]
use crate::cspace::cdt::proof as cdt_proof;
#[cfg(verus_keep_ghost)]
use crate::cspace::manager::proof as manager_proof;
use crate::cspace::types::SlotPtr;
use sel4_common::structures_gen::cap;
use vstd::prelude::*;

verus! {


impl CSpaceManager {
    pub fn cte_swap(&mut self, cap1: &cap, slot1: SlotPtr, cap2: &cap, slot2: SlotPtr)
        requires
            old(self).wf(),
            old(self).slot_dom().contains(slot1),
            old(self).slot_dom().contains(slot2),
            !old(self).slot_is_empty(slot1),
            !old(self).slot_is_empty(slot2),
            old(self).get_cap(slot1) == trusted_view_cap(cap1),
            old(self).get_cap(slot2) == trusted_view_cap(cap2),
            slot1 != slot2,
        ensures
            self.wf(),
            old(self).cte_swap_rel(
                self,
                slot1,
                trusted_view_cap(cap1),
                slot2,
                trusted_view_cap(cap2),
            ),
    {
        let Ghost(old_mgr) = Ghost(*self);
        let Ghost(old_cdt) = Ghost(old_mgr.cdt@);

        let Tracked(mut slot1_perm) = self.mdb.take_entry_perm(slot1);
        let Tracked(mut slot2_perm) = self.mdb.take_entry_perm(slot2);

        crate::cspace::cte::payload::write_slot_cap_only_tracked(slot1, Tracked(&mut slot1_perm), cap2);
        crate::cspace::cte::payload::write_slot_cap_only_tracked(slot2, Tracked(&mut slot2_perm), cap1);

        let Ghost(slot1_entry_after_payload) = Ghost(
            crate::cspace::cte::raw::trusted_slot_perm_view(slot1_perm),
        );
        let Ghost(slot2_entry_after_payload) = Ghost(
            crate::cspace::cte::raw::trusted_slot_perm_view(slot2_perm),
        );
        self.mdb.put_entry_perm(slot1, Tracked(slot1_perm));
        self.mdb.put_entry_perm(slot2, Tracked(slot2_perm));
        let Ghost(pre_mdb_swap_mgr) = Ghost(*self);
        proof {
            assert(self.mdb.entry_view(slot1) == slot1_entry_after_payload);
            assert(self.mdb.entry_view(slot2) == slot2_entry_after_payload);
            old_mgr.mdb.lemma_two_modified_slots_same_mdb_fields_preserve_structural_wf(
                &self.mdb,
                slot1,
                slot2,
            );
        }

        self.mdb.swap_nodes(slot1, slot2);

        let Ghost(swap_zombie_base) = Ghost(self.zombie_slots@.remove(slot1).remove(slot2));
        let Ghost(swap_zombie_with_slot1) = Ghost(
            if trusted_view_cap(cap2).kind == CapKind::ZombieCap {
                swap_zombie_base.insert(slot1)
            } else {
                swap_zombie_base
            },
        );
        self.zombie_slots = Ghost(if trusted_view_cap(cap1).kind == CapKind::ZombieCap {
            swap_zombie_with_slot1.insert(slot2)
        } else {
            swap_zombie_with_slot1
        });
        self.cdt = Ghost(old_mgr.cdt@.state_after_swap(slot1, slot2));
        proof {
            let Ghost(new_cdt) = Ghost(self.cdt@);
            assert forall|other: SlotPtr| #[trigger]
                old_mgr.slot_dom().contains(other) && other != slot1 && other != slot2
                    implies self.get_cap(other) == old_mgr.get_cap(other) by {
                if old_mgr.slot_dom().contains(other) && other != slot1 && other != slot2 {
                    assert(pre_mdb_swap_mgr.get_cap(other) == old_mgr.get_cap(other));
                    assert(self.get_cap(other) == pre_mdb_swap_mgr.get_cap(other));
                }
            };
            assert(old_mgr.cte_swap_rel(
                self,
                slot1,
                old_mgr.get_cap(slot1),
                slot2,
                old_mgr.get_cap(slot2),
            ));
            old_mgr.mdb.lemma_same_links_then_swap_slots_rel(
                &pre_mdb_swap_mgr.mdb,
                &self.mdb,
                slot1,
                slot2,
            );
            cdt_proof::lemma_state_after_swap_preserves_structural_wf(
                old_cdt,
                new_cdt,
                slot1,
                slot2,
            );
            manager_proof::lemma_swap_preserves_manager_semantics_wf(
                old_mgr,
                *self,
                slot1,
                slot2,
            );
        }
    }
}

} // verus!
