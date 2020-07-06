use crate::x86::{long_jump, set_jump, JmpBuf};
// use core::pin::Pin;
use alloc::boxed::Box;
use alloc::collections::{BTreeMap, VecDeque};
use alloc::sync::{Arc, Weak};
use spin::{Mutex, RwLock};

lazy_static! {
    pub static ref GLOBAL_THREAD_SET: RwLock<ThreadSet> =
        RwLock::new(ThreadSet::new_with_idle_thread());
    pub static ref RUNNING_THREAD: Mutex<Handle> = Mutex::new(Weak::new());
}

#[derive(Debug)]
pub struct ThreadError;

#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct ThreadId(usize);

type Result = ::core::result::Result<Handle, ThreadError>;
type Handle = Weak<RwLock<Thread>>;

#[derive(Debug)]
pub struct Thread {
    id: ThreadId,
    context: JmpBuf,
    start_fn: fn(),
    stack: Box<[u8]>,
    state: State,
    //    children: Vec<Handle>,
    //    parent: Handle,
    //    waiters: Vec<Handle>,
    //    tracer: Handle,
}

#[derive(Debug)]
pub enum State {
    Running,
    Stopped,
}

pub struct ThreadSet {
    threads: BTreeMap<ThreadId, Arc<RwLock<Thread>>>,
    runnable: VecDeque<Handle>,
    next_id: ThreadId,
}

impl ThreadSet {
    pub fn new() -> ThreadSet {
        ThreadSet {
            threads: BTreeMap::new(),
            runnable: VecDeque::new(),
            next_id: ThreadId(1),
        }
    }

    pub fn new_with_idle_thread() -> ThreadSet {
        let mut ts = Self::new();
        ts.make_thread_zero();
        ts
    }

    fn make_thread_zero(&mut self) {
        let thread_zero = Arc::new(RwLock::new(Thread {
            id: ThreadId(0),
            context: JmpBuf::new(),
            start_fn: || {
                panic!();
            },
            stack: Box::new([0; 0]),
            state: State::Running,
        }));

        self.threads.insert(ThreadId(0), thread_zero);
    }

    pub fn spawn(&mut self, start_fn: fn()) -> Result {
        let stack = Box::new([0; 2048]);
        let mut context = JmpBuf::new();
        context.sp = stack.as_ptr() as usize + 2048;
        context.bp = context.sp;
        context.ip = thread_entry as usize;
        let id = self.next_id;

        let new_thread = Arc::new(RwLock::new(Thread {
            id,
            context,
            start_fn,
            stack,
            state: State::Running,
        }));
        let new_weak = Arc::downgrade(&new_thread);

        self.threads.insert(self.next_id, new_thread);

        self.next_id = ThreadId(self.next_id.0 + 1);
        self.runnable.push_back(new_weak.clone());
        Ok(new_weak.clone())
    }

    pub fn next(&mut self) -> Handle {
        self.runnable.pop_front().unwrap_or(Weak::new())
    }
}

pub fn spawn(f: fn()) {
    GLOBAL_THREAD_SET.write().spawn(f).unwrap();
}

fn running() -> Handle {
    RUNNING_THREAD.lock().clone()
}

fn thread_entry() {
    let start_fn_opt = running().upgrade().map(|th| th.read().start_fn);

    match start_fn_opt {
        Some(start_fn) => start_fn(),
        None => panic!("can't start a thread that does not exist"),
    }
    exit();
}

pub fn schedule() {
    let to;
    let from;
    print!("in runnable: [ ");
    for th in &GLOBAL_THREAD_SET.read().runnable {
        let ThreadId(thread) = th.upgrade().unwrap().read().id;
        print!("{:?} ", thread);
    }
    println!("]");
    {
        // all locks must be inside this inner block
        let running_thread = running().upgrade();
        let to_thread = GLOBAL_THREAD_SET.write().next();

        from = running()
            .upgrade()
            .map(|th| &mut th.write().context as *mut JmpBuf)
            .unwrap_or(core::ptr::null_mut());
        to = to_thread
            .upgrade()
            .map(|th| &th.read().context as *const JmpBuf)
            .unwrap();

        if running_thread.is_some() {
            GLOBAL_THREAD_SET.write().runnable.push_back(running());
        }
        *RUNNING_THREAD.lock() = to_thread;
    }
    println!("schedule");
    unsafe { switch(to, from) };
}

unsafe fn switch(to: *const JmpBuf, from: *mut JmpBuf) {
    if !from.is_null() && set_jump(from) == 1 {
        return;
    }
    long_jump(to, 1);
}

fn exit() -> ! {
    println!("exit");
    {
        let id = RUNNING_THREAD.lock().upgrade().unwrap().read().id;
        GLOBAL_THREAD_SET.write().threads.remove(&id);
    }
    println!("calling scheudle");
    schedule();
    panic!();
}
