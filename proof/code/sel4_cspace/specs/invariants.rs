use vstd::prelude::*;

use crate::model::*;

verus! {

pub open spec fn wf_mdb_links(s: AbsState) -> bool {
    forall|slot: SlotId|
        s.ctes.contains_key(slot) ==> {
            let mdb = s.ctes[slot].mdb_node;
            &&& (mdb.prev == null_slot() || s.ctes.contains_key(mdb.prev))
            &&& (mdb.next == null_slot() || s.ctes.contains_key(mdb.next))
            &&& (mdb.prev != null_slot() ==> s.ctes[mdb.prev].mdb_node.next == slot)
            &&& (mdb.next != null_slot() ==> s.ctes[mdb.next].mdb_node.prev == slot)
        }
}

pub open spec fn wf_empty_slots(s: AbsState) -> bool {
    forall|slot: SlotId|
        s.ctes.contains_key(slot) && s.ctes[slot].capability == Capability::NullCap ==> {
            let mdb = s.ctes[slot].mdb_node;
            mdb.next == null_slot() && mdb.prev == null_slot()
        }
}

pub open spec fn wf_mdb_order(s: AbsState) -> bool {
    forall|slot: SlotId|
        s.ctes.contains_key(slot) && s.ctes[slot].mdb_node.next != null_slot() ==> {
            let next = s.ctes[slot].mdb_node.next;
            s.ctes.contains_key(next) ==> same_region_as(s.ctes[slot].capability, s.ctes[next].capability)
                || same_region_as(s.ctes[next].capability, s.ctes[slot].capability)
                || !is_mdb_parent_of(s.ctes[slot], s.ctes[next])
        }
}

pub open spec fn wf_cspace(s: AbsState) -> bool {
    &&& wf_mdb_links(s)
    &&& wf_empty_slots(s)
    &&& wf_mdb_order(s)
}

}
