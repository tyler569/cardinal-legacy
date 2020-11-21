use crate::sync::{Mutex, MutexGuard};
use crate::util::round_up;
use alloc::alloc::{GlobalAlloc, Layout};

const HEAP_LEN: usize = 1024 * 1024;
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
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.lock();

        let new_base = round_up(allocator.index, layout.align());
        let next_index = new_base + layout.size();

        if next_index > HEAP_LEN {
            return core::ptr::null_mut();
        }

        allocator.index = next_index;

        &mut EARLY_HEAP[new_base] as *mut u8
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
