use alloc::alloc::{GlobalAlloc, Layout};

use crate::sync::{Mutex, MutexGuard};

const HEAP_LEN: usize = 128 * 1024;
static mut EARLY_HEAP: [u8; HEAP_LEN] = [0u8; HEAP_LEN];

pub struct Locked<A> {
    inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> MutexGuard<A> {
        self.inner.lock()
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
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut alloc = self.lock();
        let ptr = EARLY_HEAP.as_mut_ptr();
        let new_offset = alloc.index + layout.size();
        let ret = ptr.add(alloc.index);
        alloc.index = new_offset;
        ret
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        let _alloc = self.lock();
    }
}

#[global_allocator]
static ALLOCATOR: Locked<EarlyHeap> = Locked::new(EarlyHeap::new());

#[alloc_error_handler]
fn alloc_error(l: Layout) -> ! {
    panic!("Allocation error allocating {:?}", l);
}
