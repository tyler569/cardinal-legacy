
use crate::sync::Mutex;
use crate::x86::{JmpBuf};
use alloc::boxed::Box;
use alloc::vec::Vec;


// TODO!!
// This should be behind an RwLock or similar, and the Threads should have
// interior mutability! - You should be able to Read the vec and Write the
// Threads at the same time, as long as noone is mutating the same threads.
// This is just a bad starting point.
static THREADS: Mutex<Vec<Option<Thread>>> = Mutex::new(Vec::new());
static ID: Mutex<i32> = Mutex::new(1);
static RUNNING_THREAD_ID: Mutex<usize> = Mutex::new(1);

pub struct Thread {
    id: i32,
    should_run: bool,
    // stack_pointer: *const u8,
    context: JmpBuf,
}

pub struct JoinHandle<T> {
    idk: T,
}

fn new_stack_pointer() -> usize {
    const STACK_SIZE: usize = 512;

    let b = Box::new([0u8; STACK_SIZE]);
    let ptr = Box::leak(b).as_ptr();
    unsafe { ptr.add(STACK_SIZE) as usize }
}

// TODO! JoinHandle?
pub fn spawn(f: fn()) {
    let mut threads = THREADS.lock();
    let new_id;
    {
        let mut id = ID.lock();
        new_id = *id;
        *id += 1;
    }
    threads.push(Some(Thread {
        id: new_id,
        should_run: true,
        context: JmpBuf {
            sp: new_stack_pointer(),
            ip: f as usize,
            ..Default::default()
        },
    }));
}

pub fn exit() {
    let mut threads = THREADS.lock();
    let running = RUNNING_THREAD_ID.lock();
    threads[*running] = None;
    // free(stack_pointer)
}

/*
pub fn switch() {
    
}

pub fn scheduler() {

}
*/
