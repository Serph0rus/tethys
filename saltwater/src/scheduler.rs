use alloc::{collections::vec_deque::VecDeque, sync::{Arc, Weak}};
use spinning_top::RwSpinlock;
use crate::proc::{Process, Thread};
pub struct ProcessorScheduler {
    pub ready_queue: VecDeque<Arc<RwSpinlock<Thread>>>,
    pub current_process: Option<Weak<Process>>,
    pub current_thread: Option<Arc<RwSpinlock<Thread>>>,
}
impl ProcessorScheduler {
    pub fn enter() -> ! {
        panic!()
    }
    pub fn new() -> ProcessorScheduler {
        ProcessorScheduler { ready_queue: VecDeque::new(), current_process: None, current_thread: None }
    }
}