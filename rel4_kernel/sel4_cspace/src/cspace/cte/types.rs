use sel4_common::structures::exception_t;
use sel4_common::structures_gen::{cap, mdb_node};
use vstd::prelude::*;

verus! {

#[repr(C)]
#[derive(Debug)]
pub struct deriveCap_ret {
    pub status: exception_t,
    pub capability: cap,
}

#[repr(C)]
#[cfg_attr(not(verus_keep_ghost), derive(Clone))]
#[derive(Debug)]
pub struct cte_t {
    pub capability: cap,
    pub cteMDBNode: mdb_node,
}

}
