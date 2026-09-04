use crate::cspace::cte::cte_t;
use crate::capability::raw::runtime_null_cap;
use crate::kernel_api::raw::runtime_exception_none;
use sel4_common::{
    structures::exception_t,
    structures_gen::cap,
};
use vstd::prelude::*;

verus! {

/// This struct is used when finaliseSlot return a value,
///
/// Arguments:
///
/// Status: exit value
///
/// success: Whether the finalising process is successfully
///
/// cleanupInfo: When finalise tcb_cap or cnode_cap, cleanupInfo is zombie_cap, otherwise cleanupInfo is null_cap
#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct finaliseSlot_ret {
    pub status: exception_t,
    pub success: bool,
    pub cleanupInfo: cap,
}

impl Default for finaliseSlot_ret {
    fn default() -> Self {
        finaliseSlot_ret {
            status: runtime_exception_none(),
            success: true,
            cleanupInfo: runtime_null_cap(),
        }
    }
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct FinaliseCapRet {
    pub remainder: cap,
    pub cleanupInfo: cap,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct resolveAddressBits_ret_t {
    pub status: exception_t,
    pub slot: *mut cte_t,
    pub bitsRemaining: usize,
}

impl Default for resolveAddressBits_ret_t {
    #[inline]
    fn default() -> Self {
        resolveAddressBits_ret_t {
            status: runtime_exception_none(),
            slot: core::ptr::null_mut::<cte_t>(),
            bitsRemaining: 0,
        }
    }
}

}
