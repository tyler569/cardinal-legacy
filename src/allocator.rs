use alloc::alloc::{GlobalAlloc, Layout};

use crate::sync::{Mutex, MutexGuard};

const HEAP_LEN: usize = 128 * 1024;
static mut EARLY_HEAP: [u8; HEAP_LEN] = [0u8; HEAP_LEN];

pub struct Locked<A>(Mutex<A>);

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked(Mutex::new(inner))
    }

    pub fn lock(&self) -> MutexGuard<A> {
        self.0.lock()
    }
}

struct EarlyHeap {
    index: usize,
}

impl EarlyHeap {
    const fn new() -> Self {
        EarlyHeap { index: 0 }
    }
}

unsafe impl GlobalAlloc for Locked<EarlyHeap> {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        let _allocator = self.lock();
        core::ptr::null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        let _allocator = self.lock();
    }
}

#[global_allocator]
static ALLOCATOR: Locked<EarlyHeap> = Locked::new(EarlyHeap::new());

#[alloc_error_handler]
fn alloc_error(l: Layout) -> ! {
    panic!("Allocation error allocating {:?}", l);
}
