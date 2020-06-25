
use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;

use crate::sync::Mutex;

const HEAP_LEN: usize = 128 * 1024;
static mut EARLY_HEAP: [u8; HEAP_LEN] = [0u8; HEAP_LEN];

struct EarlyHeap {}

impl EarlyHeap {
    const fn new() -> Self {
        EarlyHeap {}
    }
}

unsafe impl GlobalAlloc for Mutex<EarlyHeap> {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        let _alloc = self.lock();
        null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        let _alloc = self.lock();
    }
}

#[global_allocator]
static ALLOCATOR: Mutex<EarlyHeap> = Mutex::new(EarlyHeap::new());

#[alloc_error_handler]
fn alloc_error(l: Layout) -> ! {
    panic!("Allocation error allocating {:?}", l);
}
