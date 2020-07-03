
use crate::x86::{JmpBuf, set_jump, long_jump};
// use core::pin::Pin;
use alloc::boxed::Box;
use alloc::collections::{BTreeMap, VecDeque};
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use spin::{Mutex, RwLock};

lazy_static! {
    pub static ref GLOBAL_THREAD_SET: RwLock<ThreadSet> = RwLock::new(ThreadSet::new());
    pub static ref RUNNING_THREAD: Mutex<ThreadHandle> = Mutex::new(Weak::new());
}

pub struct ThreadError;

#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct ThreadId(usize);

type Result = ::core::result::Result<ThreadId, ThreadError>;
type ThreadHandle = Weak<RwLock<Thread>>;

pub struct ThreadSet {
    pub threads: BTreeMap<ThreadId,
        Arc<RwLock<Thread>>
    >,
    runnable: VecDeque<ThreadHandle>,
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
            state: ThreadState::Running,
        }));
        let new_weak = Arc::downgrade(&new_thread);

        self.threads.insert(self.next_id, new_thread);

        self.next_id = ThreadId(self.next_id.0 + 1);
        self.runnable.push_back(new_weak);
        Ok(id)
    }

    pub fn next(&mut self) -> ThreadHandle {
        self.runnable.pop_front().unwrap_or(Weak::new())
    }
}



fn running() -> ThreadHandle {
    RUNNING_THREAD.lock().clone() 
}

fn thread_entry() {
    let start_fn_opt = running().upgrade().map(|th| {
        th.read().start_fn
    });

    match start_fn_opt {
        Some(start_fn) => start_fn(),
        None => panic!("can't start a thread that does not exist"),
    }
}

fn schedule() {
    let to;
    let from;
    {
        // all locks must be inside this inner block
        from = running()
            .upgrade()
            .map(|th| { &mut th.write().context as *mut JmpBuf })
            .unwrap_or(core::ptr::null_mut());
        to = GLOBAL_THREAD_SET.write().next()
            .upgrade()
            .map(|th| { &th.read().context as *const JmpBuf })
            .unwrap();
    }
    unsafe { switch(to, from) };
}

unsafe fn switch(to: *const JmpBuf, from: *mut JmpBuf) {
    if set_jump(from) == 1 {
        return;
    }
    long_jump(to, 1);
}

fn exit() {
    panic!();
}

pub struct ThreadGroup {
    pub threads: Vec<ThreadHandle>,
}

pub enum ThreadState {
    Running,
    Stopped,
}

pub struct Thread {
    pub id: ThreadId,
    pub context: JmpBuf,
    pub start_fn: fn(),
    pub stack: Box<[u8]>,
    pub state: ThreadState,
}

