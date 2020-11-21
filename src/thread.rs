use crate::x86::{self, long_jump, set_jump, JmpBuf};
use alloc::boxed::Box;
use alloc::collections::{BTreeMap, VecDeque};
use alloc::sync::Arc;
use core::fmt;
use core::mem;
use core::ptr;
use spin::RwLock;

#[repr(C, align(32))]
struct Stack([u8; Stack::SIZE]);

impl Stack {
    const SIZE: usize = 4096;

    fn new_boxed() -> Box<Stack> {
        Box::new(Stack([0; Self::SIZE]))
    }

    fn stack_ptr(&self) -> usize {
        (&self.0[0] as *const u8).wrapping_add(Self::SIZE - 2048) as usize
    }
}

impl fmt::Debug for Stack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Stack").finish()
    }
}

struct StartFn(Box<dyn Fn() + Send + Sync>);

impl fmt::Debug for StartFn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "StartFn(<>)")
    }
}

#[derive(Debug)]
pub struct Thread {
    id: usize,
    pub context: JmpBuf,
    start_fn: Option<StartFn>,
    stack: Box<Stack>,
    state: State,
}

impl Thread {
    fn new_raw(id: usize, ip: fn()) -> Self {
        let stack = Stack::new_boxed();
        let mut context = JmpBuf::new();
        context.sp = stack.stack_ptr();
        context.bp = context.sp;
        context.ip = ip as usize;

        Self {
            id,
            start_fn: None,
            stack,
            context,
            state: State::Running,
        }
    }

    fn new(id: usize) -> Self {
        Self::new_raw(id, thread_entry)
    }

    fn new_idle() -> Self {
        Self::new_raw(0, thread_idle)
    }

    fn is_running(&self) -> bool {
        self.state == State::Running
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum State {
    Running,
    Stopped,
    Dead,
}

lazy_static! {
    static ref THREADS: RwLock<ThreadSet> = RwLock::new(ThreadSet::new());
}

type ThreadArc = Arc<RwLock<Thread>>;

#[derive(Debug)]
struct ThreadSet {
    threads: BTreeMap<usize, ThreadArc>,
    running: Option<ThreadArc>,
    runnable: VecDeque<ThreadArc>,
    idle: ThreadArc,
}

impl ThreadSet {
    fn new() -> Self {
        let idle = Arc::new(RwLock::new(Thread::new_idle()));
        ThreadSet {
            threads: BTreeMap::new(),
            runnable: VecDeque::new(),
            running: None,
            idle,
        }
    }

    fn next_id(&self) -> usize {
        self.threads.keys().nth_back(0).unwrap_or(&0) + 1
    }

    fn get(&self, id: usize) -> Option<ThreadArc> {
        self.threads.get(&id).cloned()
    }

    fn idle(&self) -> ThreadArc {
        self.idle.clone()
    }

    fn spawn(&mut self, func: Box<dyn Fn() + Send + Sync>) -> ThreadArc {
        let id = self.next_id();
        let mut th = Thread::new(id);
        th.start_fn = Some(StartFn(func));
        let arc = Arc::new(RwLock::new(th));
        self.threads.insert(id, arc.clone());
        self.set_runnable(arc.clone());
        arc
    }

    fn next_runnable(&mut self) -> Option<ThreadArc> {
        self.runnable.pop_front()
    }

    fn set_runnable(&mut self, th: ThreadArc) {
        self.runnable.push_back(th);
    }
}

fn running() -> Option<ThreadArc> {
    THREADS.read().running.clone()
}

pub fn spawn<F>(func: F) -> ThreadArc
where
    F: Fn() + Send + Sync + 'static,
{
    THREADS.write().spawn(Box::new(func))
}

pub fn exit() -> ! {
    if let Some(thread) = running() {
        let mut th = thread.write();
        let id = th.id;
        th.state = State::Dead;
        THREADS.write().threads.remove(&id);
    }
    schedule();
    panic!("thread continued after exitting");
}

// Definitely panics if the thread either does not exist or does not have
// a start_fn. I don't know what else could be done in those cases.
fn thread_entry() {
    x86::enable_irqs();
    let start_fn: Option<StartFn>;
    {
        let thread =
            running().expect("Attempt to enter a thread that does not exist");
        start_fn = mem::replace(&mut thread.write().start_fn, None);
    }
    (start_fn.unwrap().0)();
    exit();
}

fn thread_idle() {
    dprintln!(" --> in thread_idle");
    loop {
        x86::enable_irqs();
        x86::pause();
    }
}

use crate::interrupt::InterruptDisabler;

pub fn schedule() {
    let _int = InterruptDisabler::new();
    let from_buf: *mut JmpBuf;
    let to_buf: *const JmpBuf;
    let to_stack: usize;

    {
        let to: ThreadArc;
        let from: Option<ThreadArc>;

        let mut threads = match THREADS.try_write() {
            Some(guard) => guard,
            None => return,
        };

        let to_opt = threads.next_runnable();
        from = threads.running.clone();

        if let Some(from_arc) = from.as_ref() {
            // If _from_ is write locked, we have to try again later
            let from_guard = match from_arc.try_read() {
                Some(guard) => guard,
                None => return,
            };
            // If the current thread is still running, let it run again
            if from_guard.is_running() {
                // Unless there's no one to switch to, in which case
                // just return and keep doing work.
                if to_opt.is_none() {
                    return;
                }
                threads.set_runnable(from_arc.clone());
            }
        }
        to = to_opt.unwrap_or_else(|| threads.idle());

        threads.running = Some(to.clone());

        // dprintln!(" --> SWAP {:x?} -> {:x?}", from, to);

        to_buf = &to.read().context as *const JmpBuf;
        to_stack = to.read().stack.stack_ptr();
        from_buf = from
            .map(|th| &mut th.write().context as *mut JmpBuf)
            .unwrap_or(ptr::null_mut());
    }

    unsafe {
        x86::set_kernel_stack(to_stack);
        switch(to_buf, from_buf);
    }
}

unsafe fn switch(to: *const JmpBuf, from: *mut JmpBuf) {
    if !from.is_null() && set_jump(from) == 1 {
        return;
    }
    long_jump(to, 1);
}
