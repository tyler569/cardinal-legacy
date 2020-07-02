
use crate::x86::{JmpBuf};
use core::pin::Pin;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use spin::{Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};

pub static GLOBAL_THREAD_SET: RwLock<ThreadSet> = RwLock::new(ThreadSet::new());

pub struct ThreadError;

#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct ThreadId(usize);

type Result = ::core::result::Result<ThreadId, ThreadError>;

pub struct ThreadSet {
    pub threads: BTreeMap<ThreadId,
        Pin<Arc<RwLock<Thread>>>
    >,
    next_id: ThreadId,
}

impl ThreadSet {
    pub const fn new() -> ThreadSet {
        ThreadSet {
            threads: BTreeMap::new(),
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

        let mut new_thread = Arc::pin(RwLock::new(Thread {
            id,
            context,
            start_fn,
            stack,
            state: ThreadState::Running,
        }));

        self.threads.insert(self.next_id, new_thread);

        self.next_id = ThreadId(self.next_id.0 + 1);
        Ok(id)
    }
}

fn thread_entry() {
    // thread.start_fn();
    exit(); // or panic! -- not sure which is more correct.
}

fn exit() {
    panic!();
}

pub struct ThreadGroup {
    pub threads: Vec<Weak<RwLock<Thread>>>,
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

