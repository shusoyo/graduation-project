use crate::cspace::types::SlotPtr;
use vstd::prelude::*;

verus! {

pub ghost struct CdtState {
    pub dom: Set<SlotPtr>,
    pub parent_of: Map<SlotPtr, Option<SlotPtr>>,
    pub is_original: Map<SlotPtr, bool>,
}

pub ghost struct CdtDepthWitness {
    // Reserved witness for acyclicity lemmas; not wired into manager-level proof yet.
    pub depth: Map<SlotPtr, int>,
}

impl CdtState {
    pub open spec fn swap_slot(slot: SlotPtr, slot1: SlotPtr, slot2: SlotPtr) -> SlotPtr {
        if slot == slot1 {
            slot2
        } else if slot == slot2 {
            slot1
        } else {
            slot
        }
    }

    pub open spec fn swap_ref(
        parent: Option<SlotPtr>,
        slot1: SlotPtr,
        slot2: SlotPtr,
    ) -> Option<SlotPtr> {
        if parent is Some {
            Some(Self::swap_slot(parent.unwrap(), slot1, slot2))
        } else {
            None
        }
    }

    pub open spec fn dom(&self) -> Set<SlotPtr> {
        self.dom
    }

    pub open spec fn parent(&self, slot: SlotPtr) -> Option<SlotPtr>
        recommends
            self.dom().contains(slot),
    {
        self.parent_of[slot]
    }

    pub open spec fn original(&self, slot: SlotPtr) -> bool
        recommends
            self.dom().contains(slot),
    {
        self.is_original[slot]
    }

    pub open spec fn empty() -> Self {
        Self {
            dom: Set::empty(),
            parent_of: Map::empty(),
            is_original: Map::empty(),
        }
    }

    pub open spec fn maps_cover_dom(self) -> bool {
        &&& self.parent_of.dom() =~= self.dom()
        &&& self.is_original.dom() =~= self.dom()
    }

    pub open spec fn with_parent(self, slot: SlotPtr, parent: Option<SlotPtr>) -> Self {
        Self {
            dom: self.dom,
            parent_of: self.parent_of.insert(slot, parent),
            is_original: self.is_original,
        }
    }

    pub open spec fn with_original(self, slot: SlotPtr, original: bool) -> Self {
        Self {
            dom: self.dom,
            parent_of: self.parent_of,
            is_original: self.is_original.insert(slot, original),
        }
    }

    pub open spec fn state_after_cap_insert(
        self,
        src: SlotPtr,
        dest: SlotPtr,
        src_parent: bool,
        dest_original: bool,
    ) -> Self
        recommends
            self.dom().contains(src),
            self.dom().contains(dest),
    {
        let dest_parent = if src_parent { Some(src) } else { self.parent(src) };
        self.with_parent(dest, dest_parent).with_original(dest, dest_original)
    }

    pub open spec fn state_after_insert_new_cap(self, parent: SlotPtr, slot: SlotPtr) -> Self
        recommends
            self.dom().contains(parent),
            self.dom().contains(slot),
    {
        self.state_after_cap_insert(parent, slot, true, true)
    }

    pub open spec fn moved_parent_of(self, slot: SlotPtr, src: SlotPtr, dest: SlotPtr) -> Option<SlotPtr> {
        if slot == src {
            None
        } else if slot == dest {
            self.parent(src)
        } else if self.parent(slot) == Some(src) {
            Some(dest)
        } else {
            self.parent(slot)
        }
    }

    pub open spec fn state_after_move(self, src: SlotPtr, dest: SlotPtr) -> Self {
        let old_src_original = self.original(src);
        Self {
            dom: self.dom,
            parent_of: Map::new(
                |slot: SlotPtr| self.dom().contains(slot),
                |slot: SlotPtr| self.moved_parent_of(slot, src, dest),
            ),
            is_original: self.is_original.insert(src, false).insert(dest, old_src_original),
        }
    }

    pub open spec fn swapped_parent_of(self, slot: SlotPtr, slot1: SlotPtr, slot2: SlotPtr) -> Option<SlotPtr> {
        Self::swap_ref(self.parent(Self::swap_slot(slot, slot1, slot2)), slot1, slot2)
    }

    pub open spec fn state_after_swap(self, slot1: SlotPtr, slot2: SlotPtr) -> Self {
        let old_original1 = self.original(slot1);
        let old_original2 = self.original(slot2);
        Self {
            dom: self.dom,
            parent_of: Map::new(
                |slot: SlotPtr| self.dom().contains(slot),
                |slot: SlotPtr| self.swapped_parent_of(slot, slot1, slot2),
            ),
            is_original: self.is_original.insert(slot1, old_original2).insert(slot2, old_original1),
        }
    }

    pub open spec fn deleted_parent_of(self, slot: SlotPtr, deleted: SlotPtr) -> Option<SlotPtr> {
        if slot == deleted {
            None
        } else if self.parent(slot) == Some(deleted) {
            None
        } else {
            self.parent(slot)
        }
    }

    pub open spec fn state_after_delete(self, deleted: SlotPtr) -> Self {
        Self {
            dom: self.dom,
            parent_of: Map::new(
                |slot: SlotPtr| self.dom().contains(slot),
                |slot: SlotPtr| self.deleted_parent_of(slot, deleted),
            ),
            is_original: self.is_original.insert(deleted, false),
        }
    }

}

impl CdtDepthWitness {
    pub open spec fn dom(&self) -> Set<SlotPtr> {
        self.depth.dom()
    }

    pub open spec fn depth_of(&self, slot: SlotPtr) -> int
        recommends
            self.dom().contains(slot),
    {
        self.depth[slot]
    }
}

pub open spec fn depth_witness_valid_for(state: CdtState, witness: CdtDepthWitness) -> bool {
    &&& state.maps_cover_dom()
    &&& witness.dom() =~= state.dom()
    &&& forall|slot: SlotPtr| #![auto]
        state.dom().contains(slot) && state.parent(slot) is Some ==> {
            let parent = state.parent(slot).unwrap();
            &&& state.dom().contains(parent)
            &&& witness.depth_of(parent) < witness.depth_of(slot)
        }
}

}
