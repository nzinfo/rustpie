use buddy_system_allocator::LockedHeap;

use crate::constants::PAGE_SIZE;
use crate::microcall::mem_alloc;
use crate::arch::page_table::Entry;

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init() {
  const HEAP_SIZE: usize = 16;
  for i in 0..HEAP_SIZE {
    mem_alloc(0, common::CONFIG_HEAP_BTM + i * PAGE_SIZE, Entry::default());
  }
  unsafe {
    HEAP_ALLOCATOR.lock().init(common::CONFIG_HEAP_BTM, HEAP_SIZE * PAGE_SIZE)
  }
}

#[alloc_error_handler]
fn alloc_error_handler(_: core::alloc::Layout) -> ! {
  panic!("alloc_error_handler: heap panic");
}
