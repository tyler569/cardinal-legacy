
use crate::x86::{JmpBuf};
use alloc::boxed::Box;
use alloc::vec::Vec;
use spin::{Mutex, RwLock};

static THREADS: RwLock<Vec<RwLock<Option<Thread>>>> = RwLock::new(Vec::new());
static ID: Mutex<i32> = Mutex::new(1);
static RUNNING_THREAD_ID: Mutex<usize> = Mutex::new(1);

pub struct Thread {
    id: i32,
    should_run: bool,
    start: Box<dyn Fn() + Send + Sync>,
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

fn start_thread() {
    // bluh
    let threads = THREADS.read();
    let running_id = *RUNNING_THREAD_ID.lock();
    let running_rw = &threads[running_id];
    let running_guard = running_rw.read();
    let running_opt = &*running_guard;
    if let Some(running) = running_opt.as_ref() {
        (running.start)();
    } else {
        panic!("running thread does not exist");
    }
}

// TODO! JoinHandle?
pub fn spawn<F: Fn() + Send + Sync + 'static>(f: F) {
    let mut threads = THREADS.write();
    let new_id;
    {
        let mut id = ID.lock();
        new_id = *id;
        *id += 1;
    }
    let new_thread = RwLock::new(Some(Thread {
        id: new_id,
        should_run: true,
        start: Box::new(f),
        context: JmpBuf {
            sp: new_stack_pointer(),
            ip: start_thread as usize,
            ..Default::default()
        },
    }));
    threads.push(new_thread)
}

pub fn exit() {
    let threads = THREADS.read();
    let running_id = RUNNING_THREAD_ID.lock();
    let mut running = threads[*running_id].write();
    *running = None;
}


pub fn switch() {
//     let threads = THREADS.read();
//     for thread_rw in threads {
//         let thread_guard = thread_rw.read();
//         let thread_opt = &*thread_guard;
//         if let Some(thread) = thread_opt.as_ref() {
//             // problem number one:
//             // this is going to long jump out of borrows.
//         }
//     }
}

/*
pub fn scheduler() {

}
*/
