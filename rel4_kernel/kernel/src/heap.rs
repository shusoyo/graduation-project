use buddy_system_allocator::LockedHeap;
use core::alloc::Layout;

#[global_allocator]
static GLOBAL_ALLOCATOR: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
fn alloc_error(layout: Layout) -> ! {
    panic!("kernel allocation failed: size={} align={}", layout.size(), layout.align());
}
