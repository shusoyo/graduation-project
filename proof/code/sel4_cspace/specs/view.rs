use vstd::prelude::*;

use crate::model::*;

verus! {

pub struct ConcreteCTEDraft {
    pub raw_addr: nat,
}

pub struct ConcreteStateDraft {
    pub raw_slots: Map<nat, ConcreteCTEDraft>,
}

pub open spec fn slot_view_draft(slot: ConcreteCTEDraft) -> SlotId {
    slot.raw_addr
}

pub open spec fn cte_view_draft(_slot: ConcreteCTEDraft) -> CTE {
    null_cte()
}

pub open spec fn state_view_draft(_state: ConcreteStateDraft) -> AbsState {
    empty_abs_state()
}

}
