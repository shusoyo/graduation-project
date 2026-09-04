use crate::cspace::cte::cte_t;
#[cfg(verus_keep_ghost)]
use crate::cspace::cte::spec::{
    spec_empty_slot_entry, spec_incoming_badge_edge_ok, spec_incoming_parent_edge_ok,
    spec_incoming_untyped_edge_ok, SlotEntrySpec,
};
#[cfg(verus_keep_ghost)]
use crate::cspace::cte::spec::same_mdb_fields;
use crate::cspace::types::SlotPtr;
use super::math;
use super::raw;
use sel4_common::structures_gen::mdb_node;
use sel4_common::utils::convert_to_mut_type_ref;
use vstd::prelude::*;
use vstd::simple_pptr::{self, PPtr};
mod insert;
mod move_op;
mod remove;
mod swap;

verus! {

pub open spec fn order_after_insert_between(
    old_order: Seq<SlotPtr>,
    src: SlotPtr,
    dest: SlotPtr,
) -> Seq<SlotPtr> {
    old_order.insert(old_order.index_of(src) + 1, dest)
}

pub open spec fn order_after_move_slot(
    old_order: Seq<SlotPtr>,
    src: SlotPtr,
    dest: SlotPtr,
) -> Seq<SlotPtr> {
    old_order.update(old_order.index_of(src), dest)
}

pub open spec fn order_after_swap_slots(
    old_order: Seq<SlotPtr>,
    slot1: SlotPtr,
    slot2: SlotPtr,
) -> Seq<SlotPtr> {
    old_order.update(old_order.index_of(slot1), slot2).update(old_order.index_of(slot2), slot1)
}

pub open spec fn order_after_remove_slot(
    old_order: Seq<SlotPtr>,
    slot: SlotPtr,
) -> Seq<SlotPtr> {
    old_order.remove(old_order.index_of(slot))
}

pub struct MdbTable {
    pub entries: Tracked<Map<SlotPtr, simple_pptr::PointsTo<cte_t>>>,
    pub order: Ghost<Seq<SlotPtr>>,
    pub live_slots: Ghost<Set<SlotPtr>>,
}

impl MdbTable {
    pub proof fn lemma_order_after_insert_between_shape(
        &self,
        src: SlotPtr,
        dest: SlotPtr,
    )
        requires
            self.summary_wf(),
            self.live_slots@.contains(src),
            !self.live_slots@.contains(dest),
        ensures
            self.order_after_insert_between(src, dest).len() == self.order@.len() + 1,
            self.order_after_insert_between(src, dest)[self.order@.index_of(src)] == src,
            self.order_after_insert_between(src, dest)[self.order@.index_of(src) + 1] == dest,
            forall|i: int| 0 <= i < self.order@.index_of(src) ==> #[trigger]
                self.order_after_insert_between(src, dest)[i] == self.order@[i],
            forall|i: int| self.order@.index_of(src) + 1 <= i < self.order@.len() ==> #[trigger]
                self.order_after_insert_between(src, dest)[i + 1] == self.order@[i],
    {
        let idx = self.order@.index_of(src);
        self.order@.index_of_first_ensures(src);
        assert(self.order@.contains(src));
        assert(0 <= idx < self.order@.len());
        assert(self.order@[idx] == src);
        math::lemma_seq_insert_shift(self.order@, idx + 1, dest);
    }

    pub proof fn lemma_order_after_insert_between_summary(
        &self,
        src: SlotPtr,
        dest: SlotPtr,
    )
        requires
            self.summary_wf(),
            self.live_slots@.contains(src),
            !self.live_slots@.contains(dest),
        ensures
            self.order_after_insert_between(src, dest).no_duplicates(),
            self.order_after_insert_between(src, dest).to_set()
                =~= self.live_slots@.insert(dest),
    {
        let idx = self.order@.index_of(src);
        self.lemma_order_after_insert_between_shape(src, dest);
        self.order@.index_of_first_ensures(src);
        assert(!self.order@.contains(dest));
        assert(self.order@.insert(idx + 1, dest).to_set() =~= self.order@.to_set().insert(dest)) by {
            broadcast use vstd::seq_lib::group_seq_properties;
            broadcast use vstd::seq_lib::to_multiset_insert;
            let left = self.order@.subrange(0, idx + 1);
            let right = self.order@.subrange(idx + 1, self.order@.len() as int);
            assert(self.order@ =~= left + right);
            assert(self.order@.insert(idx + 1, dest) =~= left + seq![dest] + right);
            assert((left + seq![dest]).to_set() =~= left.to_set().insert(dest));
            assert((left + seq![dest] + right).to_set() =~= (left + seq![dest]).to_set() + right.to_set());
            assert((left + right).to_set() =~= left.to_set() + right.to_set());
        }
        assert(self.order_after_insert_between(src, dest).to_set() =~= self.live_slots@.insert(dest));
        self.order_after_insert_between(src, dest).unique_seq_to_set();
        assert(self.order_after_insert_between(src, dest).to_set().len()
            == self.order_after_insert_between(src, dest).len());
        assert(self.live_slots@.len() == self.order@.len()) by {
            self.order@.unique_seq_to_set();
        }
        assert(self.order_after_insert_between(src, dest).no_duplicates()) by {
            self.order_after_insert_between(src, dest).lemma_no_dup_set_cardinality();
        }
    }

    pub proof fn lemma_order_after_remove_slot_shape(
        &self,
        slot: SlotPtr,
    )
        requires
            self.summary_wf(),
            self.live_slots@.contains(slot),
        ensures
            self.order_after_remove_slot(slot).len() == self.order@.len() - 1,
            forall|i: int| 0 <= i < self.order@.index_of(slot) ==> #[trigger]
                self.order_after_remove_slot(slot)[i] == self.order@[i],
            forall|i: int| self.order@.index_of(slot) <= i < self.order_after_remove_slot(slot).len() ==> #[trigger]
                self.order_after_remove_slot(slot)[i] == self.order@[i + 1],
    {
        let idx = self.order@.index_of(slot);
        self.order@.index_of_first_ensures(slot);
        assert(self.order@.contains(slot));
        assert(0 <= idx < self.order@.len());
        math::lemma_seq_remove_shift(self.order@, idx);
    }

    pub proof fn lemma_order_after_remove_slot_summary(
        &self,
        slot: SlotPtr,
    )
        requires
            self.summary_wf(),
            self.live_slots@.contains(slot),
        ensures
            self.order_after_remove_slot(slot).no_duplicates(),
            self.order_after_remove_slot(slot).to_set() =~= self.live_slots@.remove(slot),
    {
        let idx = self.order@.index_of(slot);
        self.lemma_order_after_remove_slot_shape(slot);
        self.order@.index_of_first_ensures(slot);
        assert(self.order@.contains(slot));
        assert(0 <= idx < self.order@.len());
        assert(self.order@[idx] == slot);
        assert(self.order_after_remove_slot(slot).to_set() =~= self.order@.to_set().remove(slot)) by {
            broadcast use vstd::set::group_set_axioms;
            assert forall|x: SlotPtr| #[trigger] self.order_after_remove_slot(slot).to_set().contains(x)
                <==> self.order@.to_set().remove(slot).contains(x) by {
                if self.order_after_remove_slot(slot).to_set().contains(x) {
                    if x == slot {
                        assert(false);
                    }
                }
                if self.order@.to_set().remove(slot).contains(x) {
                    if idx == self.order@.len() - 1 {
                        assert(self.order_after_remove_slot(slot)[self.order@.index_of(x)] == x);
                    } else if self.order@.index_of(x) < idx {
                        assert(self.order_after_remove_slot(slot)[self.order@.index_of(x)] == x);
                    } else {
                        assert(self.order_after_remove_slot(slot)[self.order@.index_of(x) - 1] == x);
                    }
                }
            }
        }
        assert(self.order_after_remove_slot(slot).to_set() =~= self.live_slots@.remove(slot));
        self.order@.unique_seq_to_set();
        assert(self.live_slots@.remove(slot).len() == self.live_slots@.len() - 1) by {
            broadcast use vstd::set::axiom_set_remove_len;
        }
        assert(self.live_slots@.len() == self.order@.len());
        assert(self.order_after_remove_slot(slot).no_duplicates()) by {
            self.order_after_remove_slot(slot).lemma_no_dup_set_cardinality();
        }
    }

    pub proof fn lemma_order_after_move_slot_shape(
        &self,
        src: SlotPtr,
        dest: SlotPtr,
    )
        requires
            self.summary_wf(),
            self.live_slots@.contains(src),
            !self.live_slots@.contains(dest),
        ensures
            self.order_after_move_slot(src, dest).len() == self.order@.len(),
            self.order_after_move_slot(src, dest)[self.order@.index_of(src)] == dest,
            forall|i: int| 0 <= i < self.order@.len() && i != self.order@.index_of(src) ==> #[trigger]
                self.order_after_move_slot(src, dest)[i] == self.order@[i],
    {
        let idx = self.order@.index_of(src);
        self.order@.index_of_first_ensures(src);
        assert(self.order@.contains(src));
        assert(0 <= idx < self.order@.len());
        assert(self.order_after_move_slot(src, dest).len() == self.order@.len());
        assert(self.order_after_move_slot(src, dest)[idx] == dest);
        assert forall|i: int| (0 <= i < self.order@.len() && i != idx) implies #[trigger]
            self.order_after_move_slot(src, dest)[i] == self.order@[i] by {
            if 0 <= i < self.order@.len() && i != idx {
            }
        }
    }

    pub proof fn lemma_order_after_move_slot_summary(
        &self,
        src: SlotPtr,
        dest: SlotPtr,
    )
        requires
            self.summary_wf(),
            self.live_slots@.contains(src),
            !self.live_slots@.contains(dest),
        ensures
            self.order_after_move_slot(src, dest).no_duplicates(),
            self.order_after_move_slot(src, dest).to_set()
                =~= self.live_slots@.remove(src).insert(dest),
    {
        let idx = self.order@.index_of(src);
        self.lemma_order_after_move_slot_shape(src, dest);
        assert(self.order_after_move_slot(src, dest).to_set() =~= self.order@.to_set().remove(src).insert(dest)) by {
            broadcast use vstd::set::group_set_axioms;
            assert forall|x: SlotPtr| #[trigger] self.order_after_move_slot(src, dest).to_set().contains(x)
                <==> self.order@.to_set().remove(src).insert(dest).contains(x) by {
                if self.order@.to_set().remove(src).insert(dest).contains(x) {
                    if x != dest {
                        assert(self.order@.index_of(x) != idx);
                        assert(self.order_after_move_slot(src, dest)[self.order@.index_of(x)] == x);
                    }
                }
            }
        }
        assert(self.order_after_move_slot(src, dest).to_set() =~= self.live_slots@.remove(src).insert(dest));
        self.order_after_move_slot(src, dest).unique_seq_to_set();
        assert(self.live_slots@.remove(src).len() == self.live_slots@.len() - 1) by {
            broadcast use vstd::set::axiom_set_remove_len;
        }
        assert(self.live_slots@.len() == self.order@.len()) by {
            self.order@.unique_seq_to_set();
        }
        assert(self.order_after_move_slot(src, dest).no_duplicates()) by {
            self.order_after_move_slot(src, dest).lemma_no_dup_set_cardinality();
        }
    }

    pub proof fn lemma_order_after_swap_slots_shape(
        &self,
        slot1: SlotPtr,
        slot2: SlotPtr,
    )
        requires
            self.summary_wf(),
            self.live_slots@.contains(slot1),
            self.live_slots@.contains(slot2),
            slot1 != slot2,
        ensures
            self.order_after_swap_slots(slot1, slot2).len() == self.order@.len(),
            self.order_after_swap_slots(slot1, slot2)[self.order@.index_of(slot1)] == slot2,
            self.order_after_swap_slots(slot1, slot2)[self.order@.index_of(slot2)] == slot1,
            forall|i: int|
                0 <= i < self.order@.len()
                    && i != self.order@.index_of(slot1)
                    && i != self.order@.index_of(slot2) ==> #[trigger]
                    self.order_after_swap_slots(slot1, slot2)[i] == self.order@[i],
    {
        let idx1 = self.order@.index_of(slot1);
        let idx2 = self.order@.index_of(slot2);
        self.order@.index_of_first_ensures(slot1);
        self.order@.index_of_first_ensures(slot2);
        assert(self.order@.contains(slot1));
        assert(self.order@.contains(slot2));
        assert(0 <= idx1 < self.order@.len());
        assert(0 <= idx2 < self.order@.len());
        assert(idx1 != idx2);
        assert(self.order_after_swap_slots(slot1, slot2).len() == self.order@.len());
        assert(self.order_after_swap_slots(slot1, slot2)[idx1] == slot2);
        assert(self.order_after_swap_slots(slot1, slot2)[idx2] == slot1);
        assert forall|i: int|
            (0 <= i < self.order@.len() && i != idx1 && i != idx2) implies #[trigger]
                self.order_after_swap_slots(slot1, slot2)[i] == self.order@[i] by {
            if 0 <= i < self.order@.len() && i != idx1 && i != idx2 {
            }
        }
    }

    pub proof fn lemma_order_after_swap_slots_summary(
        &self,
        slot1: SlotPtr,
        slot2: SlotPtr,
    )
        requires
            self.summary_wf(),
            self.live_slots@.contains(slot1),
            self.live_slots@.contains(slot2),
            slot1 != slot2,
        ensures
            self.order_after_swap_slots(slot1, slot2).no_duplicates(),
            self.order_after_swap_slots(slot1, slot2).to_set() =~= self.live_slots@,
    {
        self.lemma_order_after_swap_slots_shape(slot1, slot2);
        assert(self.order_after_swap_slots(slot1, slot2).to_set() =~= self.order@.to_set()) by {
            broadcast use vstd::set::group_set_axioms;
            assert forall|x: SlotPtr| #[trigger] self.order_after_swap_slots(slot1, slot2).to_set().contains(x)
                <==> self.order@.to_set().contains(x) by {
                if self.order@.to_set().contains(x) && x != slot1 && x != slot2 {
                    assert(self.order_after_swap_slots(slot1, slot2)[self.order@.index_of(x)] == x);
                }
            }
        }
        assert(self.order_after_swap_slots(slot1, slot2).to_set() =~= self.live_slots@);
        self.order_after_swap_slots(slot1, slot2).unique_seq_to_set();
        assert(self.live_slots@.len() == self.order@.len()) by {
            self.order@.unique_seq_to_set();
        }
        assert(self.order_after_swap_slots(slot1, slot2).no_duplicates()) by {
            self.order_after_swap_slots(slot1, slot2).lemma_no_dup_set_cardinality();
        }
    }


    pub fn new() -> (ret: Self)
        ensures
            ret.structural_wf(),
            ret.dom() =~= Set::empty(),
            ret.live_slots@ =~= Set::empty(),
            ret.order@ =~= Seq::empty(),
    {
        let ret = Self::from_entries(
            Tracked(Map::tracked_empty()),
            Ghost(Seq::empty()),
            Ghost(Set::empty()),
        );
        proof {
            assert(ret.forward_chain_wf());
            assert(ret.local_symmetry_wf());
            assert(ret.detached_nonlive_wf());
        }
        ret
    }

    pub fn from_entries(
        Tracked(entries): Tracked<Map<SlotPtr, simple_pptr::PointsTo<cte_t>>>,
        Ghost(order): Ghost<Seq<SlotPtr>>,
        Ghost(live_slots): Ghost<Set<SlotPtr>>,
    ) -> (ret: Self)
        requires
            forall|slot: SlotPtr| #![auto] entries.dom().contains(slot) ==> {
                &&& slot != 0
                &&& entries[slot].is_init()
                &&& entries[slot].addr() == slot
            },
            order.no_duplicates(),
            live_slots =~= order.to_set(),
            live_slots.subset_of(entries.dom()),
            forall|i: int| #![auto]
                0 <= i < order.len() ==> entries.dom().contains(order[i]),
        ensures
            ret.entries_wf(),
            ret.summary_wf(),
            ret.dom() =~= entries.dom(),
            ret.order@ =~= order,
            ret.live_slots@ =~= live_slots,
    {
        MdbTable {
            entries: Tracked(entries),
            order: Ghost(order),
            live_slots: Ghost(live_slots),
        }
    }

    #[verifier(inline)]
    pub open spec fn dom(&self) -> Set<SlotPtr> {
        self.entries@.dom()
    }

    pub open spec fn entry_view(&self, slot: SlotPtr) -> SlotEntrySpec
        recommends
            self.dom().contains(slot),
    {
        crate::cspace::cte::raw::trusted_slot_perm_view(self.entries@[slot])
    }

    pub open spec fn prev_of(&self, slot: SlotPtr) -> Option<SlotPtr>
        recommends
            self.dom().contains(slot),
    {
        self.entry_view(slot).mdb_prev
    }

    pub open spec fn next_of(&self, slot: SlotPtr) -> Option<SlotPtr>
        recommends
            self.dom().contains(slot),
    {
        self.entry_view(slot).mdb_next
    }

    pub open spec fn revocable_of(&self, slot: SlotPtr) -> bool
        recommends
            self.dom().contains(slot),
    {
        self.entry_view(slot).mdb_revocable
    }

    pub open spec fn first_badged_of(&self, slot: SlotPtr) -> bool
        recommends
            self.dom().contains(slot),
    {
        self.entry_view(slot).mdb_first_badged
    }

    pub open spec fn get_cap(&self, slot: SlotPtr) -> crate::capability::spec::CapSpec
        recommends
            self.dom().contains(slot),
    {
        self.entry_view(slot).cap
    }

    pub open spec fn slot_is_empty(&self, slot: SlotPtr) -> bool
        recommends
            self.dom().contains(slot),
    {
        self.get_cap(slot).kind == crate::capability::spec::CapKind::NullCap
    }

    pub open spec fn incoming_parent_edge_ok(&self, slot: SlotPtr) -> bool
        recommends
            self.dom().contains(slot),
    {
        let entry = self.entry_view(slot);
        let parent_cap = if entry.mdb_prev is Some {
            Some(self.get_cap(entry.mdb_prev.unwrap()))
        } else {
            None
        };
        spec_incoming_parent_edge_ok(parent_cap, entry)
    }

    pub open spec fn incoming_badge_edge_ok(&self, slot: SlotPtr) -> bool
        recommends
            self.dom().contains(slot),
    {
        let entry = self.entry_view(slot);
        let parent_cap = if entry.mdb_prev is Some {
            Some(self.get_cap(entry.mdb_prev.unwrap()))
        } else {
            None
        };
        spec_incoming_badge_edge_ok(parent_cap, entry)
    }

    pub open spec fn incoming_untyped_edge_ok(&self, slot: SlotPtr) -> bool
        recommends
            self.dom().contains(slot),
    {
        let entry = self.entry_view(slot);
        let parent_cap = if entry.mdb_prev is Some {
            Some(self.get_cap(entry.mdb_prev.unwrap()))
        } else {
            None
        };
        spec_incoming_untyped_edge_ok(parent_cap, entry)
    }

    pub open spec fn links(&self, left: SlotPtr, right: SlotPtr) -> bool
        recommends
            self.dom().contains(left),
            self.dom().contains(right),
    {
        &&& self.next_of(left) == Some(right)
        &&& self.prev_of(right) == Some(left)
    }

    pub open spec fn order_after_insert_between(&self, src: SlotPtr, dest: SlotPtr) -> Seq<SlotPtr> {
        crate::cspace::mdb::table::order_after_insert_between(self.order@, src, dest)
    }

    pub open spec fn order_after_move_slot(&self, src: SlotPtr, dest: SlotPtr) -> Seq<SlotPtr> {
        crate::cspace::mdb::table::order_after_move_slot(self.order@, src, dest)
    }

    pub open spec fn order_after_swap_slots(&self, slot1: SlotPtr, slot2: SlotPtr) -> Seq<SlotPtr> {
        crate::cspace::mdb::table::order_after_swap_slots(self.order@, slot1, slot2)
    }

    pub open spec fn order_after_remove_slot(&self, slot: SlotPtr) -> Seq<SlotPtr> {
        crate::cspace::mdb::table::order_after_remove_slot(self.order@, slot)
    }

    pub open spec fn same_mdb_links(&self, new_table: &Self) -> bool {
        &&& new_table.dom() =~= self.dom()
        &&& new_table.order@ =~= self.order@
        &&& new_table.live_slots@ =~= self.live_slots@
        &&& forall|slot: SlotPtr| #![auto]
            self.dom().contains(slot) ==> {
                &&& new_table.prev_of(slot) == self.prev_of(slot)
                &&& new_table.next_of(slot) == self.next_of(slot)
                &&& new_table.revocable_of(slot) == self.revocable_of(slot)
                &&& new_table.first_badged_of(slot) == self.first_badged_of(slot)
            }
    }

    pub open spec fn touched_set_with_optionals(
        base: Set<SlotPtr>,
        first: Option<SlotPtr>,
        second: Option<SlotPtr>,
    ) -> Set<SlotPtr> {
        let with_first = if first is Some {
            base.insert(first.unwrap())
        } else {
            base
        };
        if second is Some {
            with_first.insert(second.unwrap())
        } else {
            with_first
        }
    }

    pub open spec fn entries_unchanged_except(
        &self,
        new_table: &Self,
        changed: Set<SlotPtr>,
    ) -> bool {
        &&& new_table.dom() =~= self.dom()
        &&& forall|slot: SlotPtr| #![auto]
            self.dom().contains(slot) && !changed.contains(slot)
                ==> new_table.entry_view(slot) == self.entry_view(slot)
    }

    pub proof fn lemma_entries_unchanged_except_transitive(
        &self,
        mid_table: &Self,
        new_table: &Self,
        changed1: Set<SlotPtr>,
        changed2: Set<SlotPtr>,
    )
        requires
            self.entries_unchanged_except(mid_table, changed1),
            mid_table.entries_unchanged_except(new_table, changed2),
        ensures
            self.entries_unchanged_except(new_table, changed1.union(changed2)),
    {
        assert(self.dom() =~= mid_table.dom());
        assert(mid_table.dom() =~= new_table.dom());
        assert(self.dom() =~= new_table.dom());
        assert forall|slot: SlotPtr| #![auto]
            self.dom().contains(slot) && !changed1.union(changed2).contains(slot)
                ==> new_table.entry_view(slot) == self.entry_view(slot) by {
            if self.dom().contains(slot) && !changed1.union(changed2).contains(slot) {
                assert(!changed1.contains(slot));
                assert(!changed2.contains(slot));
                assert(mid_table.entry_view(slot) == self.entry_view(slot));
                assert(new_table.entry_view(slot) == mid_table.entry_view(slot));
            }
        }
    }

    pub proof fn lemma_caps_unchanged_on_dom_from_touched(
        &self,
        new_table: &Self,
        touched: Set<SlotPtr>,
    )
        requires
            forall|slot: SlotPtr| #![auto]
                self.dom().contains(slot) && touched.contains(slot)
                    ==> new_table.entry_view(slot).cap == self.entry_view(slot).cap,
            self.entries_unchanged_except(new_table, touched),
        ensures
            forall|slot: SlotPtr| #![auto]
                self.dom().contains(slot) ==> new_table.entry_view(slot).cap == self.entry_view(slot).cap,
    {
        assert forall|slot: SlotPtr| #![auto]
            self.dom().contains(slot) ==> new_table.entry_view(slot).cap == self.entry_view(slot).cap by {}
    }

    pub proof fn lemma_one_modified_slot_same_mdb_fields_preserve_structural_wf(
        &self,
        new_table: &Self,
        slot: SlotPtr,
    )
        requires
            self.structural_wf(),
            new_table.entries_wf(),
            new_table.summary_wf(),
            new_table.dom() =~= self.dom(),
            new_table.order@ =~= self.order@,
            new_table.live_slots@ =~= self.live_slots@,
            self.dom().contains(slot),
            same_mdb_fields(self.entry_view(slot), new_table.entry_view(slot)),
            forall|other: SlotPtr| #![auto]
                self.dom().contains(other) && other != slot
                    ==> new_table.entry_view(other) == self.entry_view(other),
        ensures
            self.same_mdb_links(new_table),
            new_table.structural_wf(),
    {
        assert forall|other: SlotPtr| #[trigger] self.dom().contains(other)
            implies {
                &&& new_table.prev_of(other) == self.prev_of(other)
                &&& new_table.next_of(other) == self.next_of(other)
                &&& new_table.revocable_of(other) == self.revocable_of(other)
                &&& new_table.first_badged_of(other) == self.first_badged_of(other)
            } by {
            if self.dom().contains(other) {
                if other == slot {
                    self.lemma_same_mdb_fields_entry_fields(new_table, slot);
                }
            }
        }
        self.lemma_same_mdb_links_preserves_forward_chain(new_table);
        self.lemma_same_mdb_links_preserves_local_symmetry(new_table);
        self.lemma_same_mdb_links_preserves_detached_nonlive(new_table);
    }

    pub proof fn lemma_two_modified_slots_same_mdb_fields_preserve_structural_wf(
        &self,
        new_table: &Self,
        slot1: SlotPtr,
        slot2: SlotPtr,
    )
        requires
            self.structural_wf(),
            new_table.entries_wf(),
            new_table.summary_wf(),
            new_table.dom() =~= self.dom(),
            new_table.order@ =~= self.order@,
            new_table.live_slots@ =~= self.live_slots@,
            self.dom().contains(slot1),
            self.dom().contains(slot2),
            slot1 != slot2,
            same_mdb_fields(self.entry_view(slot1), new_table.entry_view(slot1)),
            same_mdb_fields(self.entry_view(slot2), new_table.entry_view(slot2)),
            forall|slot: SlotPtr| #![auto]
                self.dom().contains(slot) && slot != slot1 && slot != slot2
                    ==> new_table.entry_view(slot) == self.entry_view(slot),
        ensures
            self.same_mdb_links(new_table),
            new_table.structural_wf(),
    {
        assert forall|slot: SlotPtr| #[trigger] self.dom().contains(slot)
            implies {
                &&& new_table.prev_of(slot) == self.prev_of(slot)
                &&& new_table.next_of(slot) == self.next_of(slot)
                &&& new_table.revocable_of(slot) == self.revocable_of(slot)
                &&& new_table.first_badged_of(slot) == self.first_badged_of(slot)
            } by {
            if self.dom().contains(slot) {
                if slot == slot1 {
                    self.lemma_same_mdb_fields_entry_fields(new_table, slot1);
                } else if slot == slot2 {
                    self.lemma_same_mdb_fields_entry_fields(new_table, slot2);
                }
            }
        }
        self.lemma_same_mdb_links_preserves_forward_chain(new_table);
        self.lemma_same_mdb_links_preserves_local_symmetry(new_table);
        self.lemma_same_mdb_links_preserves_detached_nonlive(new_table);
    }

    pub proof fn lemma_same_mdb_links_entry_fields(
        &self,
        new_table: &Self,
        slot: SlotPtr,
    )
        requires
            self.same_mdb_links(new_table),
            self.dom().contains(slot),
        ensures
            new_table.prev_of(slot) == self.prev_of(slot),
            new_table.next_of(slot) == self.next_of(slot),
            new_table.revocable_of(slot) == self.revocable_of(slot),
            new_table.first_badged_of(slot) == self.first_badged_of(slot),
    {}

    pub proof fn lemma_same_mdb_fields_entry_fields(
        &self,
        new_table: &Self,
        slot: SlotPtr,
    )
        requires
            self.dom().contains(slot),
            same_mdb_fields(self.entry_view(slot), new_table.entry_view(slot)),
        ensures
            new_table.prev_of(slot) == self.prev_of(slot),
            new_table.next_of(slot) == self.next_of(slot),
            new_table.revocable_of(slot) == self.revocable_of(slot),
            new_table.first_badged_of(slot) == self.first_badged_of(slot),
    {}

    pub proof fn lemma_same_mdb_fields_preserves_detached_slot(
        &self,
        new_table: &Self,
        slot: SlotPtr,
    )
        requires
            self.dom().contains(slot),
            self.slot_is_detached(slot),
            same_mdb_fields(self.entry_view(slot), new_table.entry_view(slot)),
        ensures
            new_table.slot_is_detached(slot),
    {}

    pub open spec fn slot_is_detached(&self, slot: SlotPtr) -> bool
        recommends
            self.dom().contains(slot),
    {
        &&& self.prev_of(slot) is None
        &&& self.next_of(slot) is None
        &&& !self.revocable_of(slot)
        &&& !self.first_badged_of(slot)
    }

    pub open spec fn nonempty_slots(&self) -> Set<SlotPtr> {
        Set::new(|slot: SlotPtr| self.dom().contains(slot) && !self.slot_is_empty(slot))
    }

    pub open spec fn empty_slots_wf(&self) -> bool {
        forall|slot: SlotPtr| #![auto]
            self.dom().contains(slot) && self.slot_is_empty(slot)
                ==> self.entry_view(slot) == spec_empty_slot_entry()
    }

    pub proof fn lemma_detached_nonlive_and_live_match_implies_empty_slots_wf(&self)
        requires
            self.structural_wf(),
            forall|slot: SlotPtr| #![auto] self.dom().contains(slot)
                ==> (self.live_slots@.contains(slot) <==> !self.slot_is_empty(slot)),
            forall|slot: SlotPtr| #![auto]
                self.dom().contains(slot) && self.slot_is_empty(slot)
                    ==> self.entry_view(slot).cap == spec_empty_slot_entry().cap,
        ensures self.empty_slots_wf(),
    {
        assert(self.empty_slots_wf()) by {
            assert forall|slot: SlotPtr| #[trigger]
                self.dom().contains(slot) && self.slot_is_empty(slot)
                    implies self.entry_view(slot) == spec_empty_slot_entry() by {
                if self.dom().contains(slot) && self.slot_is_empty(slot) {
                    assert(!self.live_slots@.contains(slot));
                    assert(self.slot_is_detached(slot));
                    assert(self.entry_view(slot).cap == spec_empty_slot_entry().cap);
                }
            }
        }
    }

    pub open spec fn live_slots_match_nonempty_wf(&self) -> bool {
        forall|slot: SlotPtr| #![auto]
            self.dom().contains(slot) ==> self.live_slots@.contains(slot) <==> !self.slot_is_empty(slot)
    }

    pub proof fn lemma_live_slots_match_nonempty_wf_implies_nonempty_slots_eq_live_slots(&self)
        requires
            self.summary_wf(),
            forall|slot: SlotPtr| #![auto]
                if self.dom().contains(slot) {
                    self.live_slots@.contains(slot) <==> !self.slot_is_empty(slot)
                } else {
                    true
                },
        ensures
            self.nonempty_slots() =~= self.live_slots@,
    {
        assert(self.nonempty_slots() =~= self.live_slots@) by {
            assert forall|slot: SlotPtr| #[trigger]
                self.nonempty_slots().contains(slot) <==> self.live_slots@.contains(slot) by {
                if self.nonempty_slots().contains(slot) {
                    assert(self.dom().contains(slot));
                    assert(!self.slot_is_empty(slot));
                }
                if self.live_slots@.contains(slot) {
                    assert(self.summary_wf());
                    assert(self.dom().contains(slot));
                    assert(!self.slot_is_empty(slot));
                }
            }
        }
    }

    pub open spec fn mdb_cte_at_wf(&self) -> bool {
        forall|slot: SlotPtr| #![auto]
            self.dom().contains(slot) ==> {
                &&& (self.prev_of(slot) is Some ==> {
                    let prev = self.prev_of(slot).unwrap();
                    &&& self.dom().contains(prev)
                    &&& !self.slot_is_empty(prev)
                })
                &&& (self.next_of(slot) is Some ==> {
                    let next = self.next_of(slot).unwrap();
                    &&& self.dom().contains(next)
                    &&& !self.slot_is_empty(next)
                })
            }
    }

    pub open spec fn all_incoming_edges_wf(&self) -> bool {
        forall|slot: SlotPtr| #![auto]
            self.dom().contains(slot) ==> {
                &&& self.incoming_parent_edge_ok(slot)
                &&& self.incoming_badge_edge_ok(slot)
                &&& self.incoming_untyped_edge_ok(slot)
            }
    }

    pub open spec fn entries_wf(&self) -> bool {
        forall|slot: SlotPtr| #![auto]
            self.dom().contains(slot) ==> {
                &&& slot != 0
                &&& self.entries@[slot].is_init()
                &&& self.entries@[slot].addr() == slot
            }
    }

    pub open spec fn summary_wf(&self) -> bool {
        &&& self.order@.no_duplicates()
        &&& self.live_slots@ =~= self.order@.to_set()
        &&& self.live_slots@.subset_of(self.dom())
        &&& forall|i: int| #![auto]
            0 <= i < self.order@.len() ==> self.dom().contains(self.order@[i])
    }

    pub open spec fn order_prev_of_index(&self, i: int) -> Option<SlotPtr>
        recommends
            0 <= i < self.order@.len(),
    {
        if i == 0 {
            None
        } else {
            Some(self.order@[i - 1])
        }
    }

    pub open spec fn order_next_of_index(&self, i: int) -> Option<SlotPtr>
        recommends
            0 <= i < self.order@.len(),
    {
        if i + 1 == self.order@.len() {
            None
        } else {
            Some(self.order@[i + 1])
        }
    }

    pub open spec fn forward_chain_wf(&self) -> bool {
        &&& forall|i: int|
            #![trigger self.order@[i], self.order@[i + 1]]
            0 <= i && i + 1 < self.order@.len() ==> {
                let slot = self.order@[i];
                &&& self.dom().contains(slot)
                &&& self.next_of(slot) == Some(self.order@[i + 1])
            }
        &&& self.order@.len() > 0 ==> {
            let slot = self.order@[self.order@.len() - 1];
            &&& self.dom().contains(slot)
            &&& self.next_of(slot) is None
        }
    }

    pub open spec fn local_symmetry_wf(&self) -> bool {
        forall|slot: SlotPtr| #![auto]
            self.dom().contains(slot) ==> {
                let prev = self.prev_of(slot);
                let next = self.next_of(slot);
                &&& prev is Some ==> self.dom().contains(prev.unwrap())
                &&& prev is Some ==> self.next_of(prev.unwrap()) == Some(slot)
                &&& next is Some ==> self.dom().contains(next.unwrap())
                &&& next is Some ==> self.prev_of(next.unwrap()) == Some(slot)
            }
    }

    pub open spec fn detached_nonlive_wf(&self) -> bool {
        forall|slot: SlotPtr| #![auto]
            self.dom().contains(slot) && !self.live_slots@.contains(slot)
                ==> self.slot_is_detached(slot)
    }

    pub open spec fn order_links_wf(&self) -> bool {
        &&& self.forward_chain_wf()
        &&& self.local_symmetry_wf()
        &&& self.detached_nonlive_wf()
    }

    pub open spec fn structural_wf(&self) -> bool {
        &&& self.entries_wf()
        &&& self.summary_wf()
        &&& self.forward_chain_wf()
        &&& self.local_symmetry_wf()
        &&& self.detached_nonlive_wf()
    }

    pub proof fn lemma_live_slots_match_nonempty_implies_mdb_cte_at(&self)
        requires
            self.structural_wf(),
            forall|slot: SlotPtr| #![auto]
                if self.dom().contains(slot) {
                    self.live_slots@.contains(slot) <==> !self.slot_is_empty(slot)
                } else {
                    true
                },
        ensures
            self.mdb_cte_at_wf(),
    {
        assert(self.mdb_cte_at_wf()) by {
            assert forall|slot: SlotPtr| #[trigger] self.dom().contains(slot) implies {
                &&& (self.prev_of(slot) is Some ==> {
                    let prev = self.prev_of(slot).unwrap();
                    &&& self.dom().contains(prev)
                    &&& !self.slot_is_empty(prev)
                })
                &&& (self.next_of(slot) is Some ==> {
                    let next = self.next_of(slot).unwrap();
                    &&& self.dom().contains(next)
                    &&& !self.slot_is_empty(next)
                })
            } by {
                if self.dom().contains(slot) {
                    if self.prev_of(slot) is Some {
                        let prev = self.prev_of(slot).unwrap();
                        assert(self.dom().contains(prev));
                        assert(self.next_of(prev) == Some(slot));
                        if !self.live_slots@.contains(prev) {
                            assert(self.slot_is_detached(prev));
                            assert(self.next_of(prev) is None);
                            assert(false);
                        }
                    }
                    if self.next_of(slot) is Some {
                        let next = self.next_of(slot).unwrap();
                        assert(self.dom().contains(next));
                        assert(self.prev_of(next) == Some(slot));
                        if !self.live_slots@.contains(next) {
                            assert(self.slot_is_detached(next));
                            assert(self.prev_of(next) is None);
                            assert(false);
                        }
                    }
                }
            }
        }
    }

    pub proof fn lemma_forward_chain_next_matches_order(
        &self,
        i: int,
    )
        requires
            self.structural_wf(),
            0 <= i < self.order@.len(),
        ensures
            self.next_of(self.order@[i]) == self.order_next_of_index(i),
    {
        if i + 1 < self.order@.len() {
            assert(self.next_of(self.order@[i]) == Some(self.order@[i + 1]));
            assert(self.order_next_of_index(i) == Some(self.order@[i + 1]));
        } else {
            assert(i + 1 == self.order@.len());
            assert(self.next_of(self.order@[i]) is None);
            assert(self.order_next_of_index(i) is None);
        }
    }

    pub proof fn lemma_live_slot_prev_matches_order(
        &self,
        i: int,
    )
        requires
            self.structural_wf(),
            0 <= i < self.order@.len(),
        ensures
            self.prev_of(self.order@[i]) == self.order_prev_of_index(i),
    {
        let slot = self.order@[i];
        if i == 0 {
            if self.prev_of(slot) is Some {
                let prev = self.prev_of(slot).unwrap();
                assert(self.dom().contains(prev));
                assert(self.next_of(prev) == Some(slot));
                if self.live_slots@.contains(prev) {
                    self.order@.index_of_first_ensures(prev);
                    let j = self.order@.index_of(prev);
                    assert(0 <= j < self.order@.len());
                    assert(self.order@[j] == prev);
                    self.lemma_forward_chain_next_matches_order(j);
                    assert(self.next_of(prev) == self.order_next_of_index(j));
                    if j + 1 == self.order@.len() {
                        assert(self.order_next_of_index(j) is None);
                        assert(false);
                    }
                    assert(self.order_next_of_index(j) == Some(self.order@[j + 1]));
                    assert(self.order@[j + 1] == slot);
                    assert(j + 1 != 0);
                    assert(self.order@[0] != self.order@[j + 1]);
                    assert(false);
                } else {
                    assert(self.slot_is_detached(prev));
                    assert(self.next_of(prev) is None);
                    assert(false);
                }
            }
        } else {
            let prev = self.order@[i - 1];
            self.lemma_forward_chain_next_matches_order(i - 1);
            assert(self.next_of(prev) == self.order_next_of_index(i - 1));
            assert(self.order_next_of_index(i - 1) == Some(slot));
            assert(self.next_of(prev) == Some(slot));
            assert(self.prev_of(slot) == Some(prev));
            assert(self.order_prev_of_index(i) == Some(prev));
        }
    }

    pub proof fn lemma_same_mdb_links_preserves_forward_chain(
        &self,
        payload_mdb: &Self,
    )
        requires
            self.structural_wf(),
            self.same_mdb_links(payload_mdb),
        ensures
            payload_mdb.forward_chain_wf(),
    {
        assert(payload_mdb.forward_chain_wf()) by {
            assert forall|i: int|
                #![trigger payload_mdb.order@[i], payload_mdb.order@[i + 1]]
                0 <= i && i + 1 < payload_mdb.order@.len()
                    ==> {
                        let slot = payload_mdb.order@[i];
                        &&& payload_mdb.dom().contains(slot)
                        &&& payload_mdb.next_of(slot) == Some(payload_mdb.order@[i + 1])
                    } by {
                if 0 <= i && i + 1 < payload_mdb.order@.len() {
                    let slot = payload_mdb.order@[i];
                    self.lemma_same_mdb_links_entry_fields(payload_mdb, slot);
                    self.lemma_forward_chain_next_matches_order(i);
                    assert(payload_mdb.next_of(slot) == Some(payload_mdb.order@[i + 1]));
                }
            }
            assert(payload_mdb.order@.len() > 0 ==> {
                let slot = payload_mdb.order@[payload_mdb.order@.len() - 1];
                &&& payload_mdb.dom().contains(slot)
                &&& payload_mdb.next_of(slot) is None
            }) by {
                if payload_mdb.order@.len() > 0 {
                    let i = payload_mdb.order@.len() - 1;
                    let slot = payload_mdb.order@[i];
                    self.lemma_same_mdb_links_entry_fields(payload_mdb, slot);
                    assert(self.next_of(slot) is None);
                    assert(payload_mdb.next_of(slot) is None);
                }
            }
        }
    }

    pub proof fn lemma_same_mdb_links_preserves_local_symmetry(
        &self,
        payload_mdb: &Self,
    )
        requires
            self.structural_wf(),
            self.same_mdb_links(payload_mdb),
        ensures
            payload_mdb.local_symmetry_wf(),
    {
        assert forall|slot: SlotPtr| #![auto]
            payload_mdb.dom().contains(slot) ==> {
                let prev = payload_mdb.prev_of(slot);
                let next = payload_mdb.next_of(slot);
                &&& prev is Some ==> payload_mdb.dom().contains(prev.unwrap())
                &&& prev is Some ==> payload_mdb.next_of(prev.unwrap()) == Some(slot)
                &&& next is Some ==> payload_mdb.dom().contains(next.unwrap())
                &&& next is Some ==> payload_mdb.prev_of(next.unwrap()) == Some(slot)
            } by {
            if payload_mdb.dom().contains(slot) {
                self.lemma_same_mdb_links_entry_fields(payload_mdb, slot);
                if self.prev_of(slot) is Some {
                    let prev = self.prev_of(slot).unwrap();
                    self.lemma_same_mdb_links_entry_fields(payload_mdb, prev);
                }
                if self.next_of(slot) is Some {
                    let next = self.next_of(slot).unwrap();
                    self.lemma_same_mdb_links_entry_fields(payload_mdb, next);
                }
            }
        }
    }

    pub proof fn lemma_same_mdb_links_preserves_detached_nonlive(
        &self,
        payload_mdb: &Self,
    )
        requires
            self.structural_wf(),
            self.same_mdb_links(payload_mdb),
        ensures
            payload_mdb.detached_nonlive_wf(),
    {
        assert forall|slot: SlotPtr| #![auto]
            payload_mdb.dom().contains(slot) && !payload_mdb.live_slots@.contains(slot)
                ==> payload_mdb.slot_is_detached(slot) by {
            if payload_mdb.dom().contains(slot) && !payload_mdb.live_slots@.contains(slot) {
                self.lemma_same_mdb_links_entry_fields(payload_mdb, slot);
                assert(self.slot_is_detached(slot));
            }
        }
    }

    #[verifier(inline)]
    pub open spec fn spec_get_slot(&self, slot: SlotPtr) -> &cte_t
        recommends
            self.entries_wf(),
            self.dom().contains(slot),
    {
        &self.entries@[slot].value()
    }

    #[verifier(when_used_as_spec(spec_get_slot))]
    pub fn get_slot(&self, slot: SlotPtr) -> (ret: &cte_t)
        requires
            self.entries_wf(),
            self.dom().contains(slot),
        ensures
            ret == self.spec_get_slot(slot),
            crate::cspace::cte::raw::trusted_slot_perm_view(self.entries@[slot])
                == self.entry_view(slot),
            crate::cspace::cte::raw::trusted_view_cte(ret) == self.entry_view(slot),
    {
        let tracked slot_perm = self.entries.borrow().tracked_borrow(slot);
        let slot_ref: &cte_t = PPtr::<cte_t>::from_usize(slot).borrow(Tracked(slot_perm));
        proof {
            crate::cspace::cte::raw::lemma_trusted_view_cte_matches_slot_perm_view(
                slot_ref,
                *slot_perm,
            );
        }
        slot_ref
    }

    pub fn runtime_next_of_ref(raw_slot: &cte_t) -> (ret: usize)
        ensures
            crate::cspace::cte::raw::trusted_view_cte(raw_slot).mdb_next is Some
                ==> ret == crate::cspace::cte::raw::trusted_view_cte(raw_slot).mdb_next.unwrap(),
            crate::cspace::cte::raw::trusted_view_cte(raw_slot).mdb_next is None ==> ret == 0,
            ret == 0 ==> crate::cspace::cte::raw::trusted_view_cte(raw_slot).mdb_next is None,
            ret != 0 ==> crate::cspace::cte::raw::trusted_view_cte(raw_slot).mdb_next == Some(ret),
    {
        raw::runtime_slot_mdb_next(raw_slot)
    }

    pub fn runtime_prev_of_ref(raw_slot: &cte_t) -> (ret: usize)
        ensures
            crate::cspace::cte::raw::trusted_view_cte(raw_slot).mdb_prev is Some
                ==> ret == crate::cspace::cte::raw::trusted_view_cte(raw_slot).mdb_prev.unwrap(),
            crate::cspace::cte::raw::trusted_view_cte(raw_slot).mdb_prev is None ==> ret == 0,
            ret == 0 ==> crate::cspace::cte::raw::trusted_view_cte(raw_slot).mdb_prev is None,
            ret != 0 ==> crate::cspace::cte::raw::trusted_view_cte(raw_slot).mdb_prev == Some(ret),
    {
        raw::runtime_slot_mdb_prev(raw_slot)
    }

    pub fn runtime_revocable_of_ref(raw_slot: &cte_t) -> (ret: bool)
        ensures
            ret == crate::cspace::cte::raw::trusted_view_cte(raw_slot).mdb_revocable,
    {
        raw::runtime_slot_mdb_revocable(raw_slot)
    }

    pub fn runtime_first_badged_of_ref(raw_slot: &cte_t) -> (ret: bool)
        ensures
            ret == crate::cspace::cte::raw::trusted_view_cte(raw_slot).mdb_first_badged,
    {
        raw::runtime_slot_mdb_first_badged(raw_slot)
    }

    pub fn runtime_next(&self, slot: SlotPtr) -> (ret: usize)
        requires
            self.entries_wf(),
            self.dom().contains(slot),
        ensures
            self.next_of(slot) is Some ==> ret == self.next_of(slot).unwrap(),
            self.next_of(slot) is None ==> ret == 0,
            ret == 0 ==> self.next_of(slot) is None,
            ret != 0 ==> self.next_of(slot) == Some(ret),
    {
        Self::runtime_next_of_ref(self.get_slot(slot))
    }

    pub fn runtime_prev(&self, slot: SlotPtr) -> (ret: usize)
        requires
            self.entries_wf(),
            self.dom().contains(slot),
        ensures
            self.prev_of(slot) is Some ==> ret == self.prev_of(slot).unwrap(),
            self.prev_of(slot) is None ==> ret == 0,
            ret == 0 ==> self.prev_of(slot) is None,
            ret != 0 ==> self.prev_of(slot) == Some(ret),
    {
        Self::runtime_prev_of_ref(self.get_slot(slot))
    }

    pub fn runtime_revocable(&self, slot: SlotPtr) -> (ret: bool)
        requires
            self.entries_wf(),
            self.dom().contains(slot),
        ensures
            ret == self.revocable_of(slot),
    {
        Self::runtime_revocable_of_ref(self.get_slot(slot))
    }

    pub fn runtime_first_badged(&self, slot: SlotPtr) -> (ret: bool)
        requires
            self.entries_wf(),
            self.dom().contains(slot),
        ensures
            ret == self.first_badged_of(slot),
    {
        Self::runtime_first_badged_of_ref(self.get_slot(slot))
    }

    pub fn take_entry_perm(
        &mut self,
        slot: SlotPtr,
    ) -> (ret: Tracked<simple_pptr::PointsTo<cte_t>>)
        requires
            old(self).entries_wf(),
            old(self).dom().contains(slot),
        ensures
            ret@.is_init(),
            ret@.addr() == slot,
            crate::cspace::cte::raw::trusted_slot_perm_view(ret@) == old(self).entry_view(slot),
            self.entries_wf(),
            self.entries@ == old(self).entries@.remove(slot),
            self.order@ =~= old(self).order@,
            self.live_slots@ =~= old(self).live_slots@,
    {
        let tracked entry_perm = self.entries.borrow_mut().tracked_remove(slot);
        proof {
            broadcast use vstd::map::group_map_axioms;
            broadcast use vstd::set::group_set_axioms;

            assert(self.entries@ == old(self).entries@.remove(slot));
            assert forall|other: SlotPtr| #![auto]
                self.dom().contains(other) ==> {
                &&& other != 0
                &&& self.entries@[other].is_init()
                &&& self.entries@[other].addr() == other
            } by {}
        }
        Tracked(entry_perm)
    }

    pub fn put_entry_perm(
        &mut self,
        slot: SlotPtr,
        Tracked(entry_perm): Tracked<simple_pptr::PointsTo<cte_t>>,
    )
        requires
            old(self).entries_wf(),
            !old(self).dom().contains(slot),
            slot != 0,
            entry_perm.is_init(),
            entry_perm.addr() == slot,
        ensures
            self.entries_wf(),
            self.entries@ == old(self).entries@.insert(slot, entry_perm),
            self.entry_view(slot) == crate::cspace::cte::raw::trusted_slot_perm_view(entry_perm),
            self.order@ =~= old(self).order@,
            self.live_slots@ =~= old(self).live_slots@,
    {
        proof {
            broadcast use vstd::map::group_map_axioms;
            broadcast use vstd::set::group_set_axioms;

            self.entries.borrow_mut().tracked_insert(slot, entry_perm);

            assert(self.entries@ == old(self).entries@.insert(slot, entry_perm));
            assert forall|other: SlotPtr| #![auto]
                self.dom().contains(other) ==> {
                &&& other != 0
                &&& self.entries@[other].is_init()
                &&& self.entries@[other].addr() == other
            } by {}
        }
    }


    pub fn borrow_entry_with_perm<'a>(
        slot: SlotPtr,
        Tracked(entry_perm): Tracked<&'a simple_pptr::PointsTo<cte_t>>,
    ) -> (ret: &'a cte_t)
        requires
            entry_perm.is_init(),
            entry_perm.addr() == slot,
        ensures
            ret == &entry_perm.value(),
    {
        PPtr::<cte_t>::from_usize(slot).borrow(Tracked(entry_perm))
    }

}

}
