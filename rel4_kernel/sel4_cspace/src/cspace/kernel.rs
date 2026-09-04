//! Kernel-facing compatibility functions for CSpace operations.
//!
//! `CSpaceManager` is currently an exec method host: its proof/tracked fields
//! erase away in ordinary Rust builds. Kernel callers may still install a
//! long-lived object for future runtime state, but operation dispatch does not
//! require boot-time manager population.

use crate::cspace::cte::cte_t;
use crate::cspace::manager::CSpaceManager;
use crate::cspace::types::SlotPtr;
use sel4_common::structures::exception_t;
use sel4_common::structures_gen::cap;
use spin::Mutex;

pub struct CSpaceKernel {
    pub manager: CSpaceManager,
}

unsafe impl Send for CSpaceKernel {}

impl CSpaceKernel {
    pub fn new(manager: CSpaceManager) -> Self {
        Self { manager }
    }
}

static CSPACE: Mutex<Option<CSpaceKernel>> = Mutex::new(None);

#[inline]
pub fn init_cspace_kernel(cspace: CSpaceKernel) {
    *CSPACE.lock() = Some(cspace);
}

#[inline]
pub fn init_empty_cspace_kernel() {
    init_cspace_kernel(CSpaceKernel::new(CSpaceManager::new()));
}

#[inline]
pub fn clear_cspace_kernel_for_tests() {
    *CSPACE.lock() = None;
}

#[inline]
pub fn cspace_kernel_is_initialized() -> bool {
    CSPACE.lock().is_some()
}

#[inline]
pub fn is_cspace_kernel_initialized() -> bool {
    cspace_kernel_is_initialized()
}

#[inline]
fn slot_ptr(slot: &mut cte_t) -> SlotPtr {
    slot as *mut cte_t as usize
}

#[inline]
fn with_cspace_manager<R>(f: impl FnOnce(&mut CSpaceManager) -> R) -> R {
    let mut guard = CSPACE.lock();
    if let Some(cspace) = guard.as_mut() {
        return f(&mut cspace.manager);
    }
    drop(guard);

    let mut manager = CSpaceManager::new();
    f(&mut manager)
}

#[inline]
pub fn cte_insert(new_cap: &cap, src_slot: &mut cte_t, dest_slot: &mut cte_t) {
    let src = slot_ptr(src_slot);
    let dest = slot_ptr(dest_slot);
    with_cspace_manager(|manager| manager.cte_insert(new_cap, src, dest));
}

#[inline]
pub fn insert_new_cap(parent: &mut cte_t, slot: &mut cte_t, capability: &cap) {
    let parent_ptr = slot_ptr(parent);
    let slot_ptr = slot_ptr(slot);
    with_cspace_manager(|manager| manager.insert_new_cap(parent_ptr, slot_ptr, capability));
}

#[inline]
pub fn cte_move(new_cap: &cap, src_slot: &mut cte_t, dest_slot: &mut cte_t) {
    let src = slot_ptr(src_slot);
    let dest = slot_ptr(dest_slot);
    with_cspace_manager(|manager| manager.cte_move(new_cap, src, dest));
}

#[inline]
pub fn cte_swap(cap1: &cap, slot1: &mut cte_t, cap2: &cap, slot2: &mut cte_t) {
    let slot1_ptr = slot_ptr(slot1);
    let slot2_ptr = slot_ptr(slot2);
    with_cspace_manager(|manager| manager.cte_swap(cap1, slot1_ptr, cap2, slot2_ptr));
}

#[inline]
pub fn delete_all(slot: &mut cte_t, exposed: bool) -> exception_t {
    let slot = slot_ptr(slot);
    with_cspace_manager(|manager| manager.delete_all(slot, exposed))
}

#[inline]
pub fn delete_one(slot: &mut cte_t) {
    let slot = slot_ptr(slot);
    with_cspace_manager(|manager| manager.delete_one(slot));
}

#[inline]
pub fn revoke(slot: &mut cte_t) -> exception_t {
    let slot = slot_ptr(slot);
    with_cspace_manager(|manager| manager.revoke(slot))
}
