use crate::{
    frame::PAGE_FRAME_ALLOCATOR, mapping::physical_to_virtual_address, page::ManagedPageTable, sstacks::SyscallStack
};
use alloc::{
    boxed::Box,
    collections::vec_deque::VecDeque,
    sync::{Arc, Weak},
    vec::Vec,
};
use core::sync::atomic::AtomicU64;
use spinning_top::RwSpinlock;
use x86_64::
    structures::
        paging::{FrameAllocator, PageTable, PhysFrame}
    
;
enum MessageStatus {
    Sent(Vec<PhysFrame>),
    Received,
    Responded(Vec<PhysFrame>),
}
struct Message {
    tag: u64,
    status: RwSpinlock<MessageStatus>,
    waiting: RwSpinlock<Vec<Arc<RwSpinlock<Thread>>>>,
}
struct State {
    walk: bool,
    rename: bool,
    make: bool,
    remove: bool,
    read: bool,
    insert: bool,
    overwrite: bool,
    truncate: bool,
    seek_forward: bool,
    seek_backward: bool,
    seek_start: bool,
    seek_end: bool,
    tell: bool,
    lock: bool,
}
struct Binding {
    from_server: Weak<Server>,
    from_path: Box<[u8]>,
    to_path: Box<[u8]>,
    state_mask: State,
}
struct Server {
    bindings: RwSpinlock<Vec<Binding>>,
    kind: ServerKind,
}
enum ServerKind {
    User(Arc<RwSpinlock<UserServer>>),
    Kernel(KernelServer),
}
struct UserServer {
    priority_sum: RwSpinlock<u64>,
    requests: RwSpinlock<VecDeque<Arc<Message>>>,
    working: RwSpinlock<Vec<Arc<Message>>>,
    waiting: RwSpinlock<Vec<Arc<RwSpinlock<Thread>>>>,
}
struct KernelServer {}
pub struct Descriptor {
    server: Weak<Server>,
    path: Box<[u8]>,
    state_mask: State,
}
pub struct PanicVectors {
    emergency: u64,
    divide: u64,
    debug: u64,
    breakpoint: u64,
    overflow: u64,
    bound: u64,
    opcode: u64,
    device: u64,
    double: u64,
    stack: u64,
    protection: u64,
    page: u64,
    floating: u64,
    simd: u64,
    control: u64,
    security: u64,
}
pub struct ExecutionContext {
    pub registers: [u64; 16],
    pub instruction_pointer: u64,
    pub rflags: u64,
    pub debug: [u64; 8],
    pub segment_base: u64,
}
pub struct Thread {
    pub user_context: ExecutionContext,
    pub handler_context: ExecutionContext,
    pub aborted: bool,
    pub set_priority: u64,
    pub propagated_priority: u64,
    pub inherited_priority: u64,
    pub kernel_stack: SyscallStack,
    pub panic_vectors: PanicVectors,
}
pub struct Process {
    pub set_priority: AtomicU64,
    pub propagated_priority: u64,
    pub parent: Option<Weak<Process>>,
    pub pages: RwSpinlock<ManagedPageTable>,
    pub threads: RwSpinlock<Vec<Arc<RwSpinlock<Thread>>>>,
    pub children: RwSpinlock<Vec<Arc<Process>>>,
    pub requests: RwSpinlock<Vec<(u64, Weak<Message>)>>,
    pub responses: RwSpinlock<VecDeque<Arc<Message>>>,
    pub servers: RwSpinlock<Vec<Arc<Server>>>,
    pub descriptors: RwSpinlock<Vec<Descriptor>>,
}
impl Process {
    pub fn add_child(self_arc: Arc<Self>) -> Arc<Self> {
        let mut children_write = self_arc.children.write();
        let new_process = Arc::new(Self {
            set_priority: AtomicU64::new(0),
            propagated_priority: 0,
            parent: Some(Arc::downgrade(&self_arc)),
            pages: RwSpinlock::new(ManagedPageTable::new()),
            threads: RwSpinlock::new(Vec::new()),
            children: RwSpinlock::new(Vec::new()),
            requests: RwSpinlock::new(Vec::new()),
            responses: RwSpinlock::new(VecDeque::new()),
            servers: RwSpinlock::new(Vec::new()),
            descriptors: RwSpinlock::new(Vec::new()),
        });
        children_write.push(new_process.clone());
        new_process
    }
    pub fn add_thread(self: &Self) -> Arc<RwSpinlock<Thread>> {
        let mut threads_write = self.threads.write();
        let new_thread = Arc::new(RwSpinlock::new(Thread {
            user_context: ExecutionContext {
                registers: [0; 16],
                instruction_pointer: 0,
                rflags: 0,
                debug: [0; 8],
                segment_base: 0,
            },
            handler_context: ExecutionContext {
                registers: [0; 16],
                instruction_pointer: 0,
                rflags: 0,
                debug: [0; 8],
                segment_base: 0,
            },
            aborted: true,
            set_priority: 0,
            propagated_priority: 0,
            inherited_priority: 0,
            kernel_stack: SyscallStack::new()
                .expect("failed to allocate kernel stack during thread creation!"),
            panic_vectors: PanicVectors {
                emergency: 0,
                divide: 0,
                debug: 0,
                breakpoint: 0,
                overflow: 0,
                bound: 0,
                opcode: 0,
                device: 0,
                double: 0,
                stack: 0,
                protection: 0,
                page: 0,
                floating: 0,
                simd: 0,
                control: 0,
                security: 0,
            },
        }));
        threads_write.push(new_thread.clone());
        new_thread
    }
    /*fn propagate_priorities(self: &Self) {
        let set_priority_sum = unsafe { self.threads.read() }
            .iter()
            .map(|thread| unsafe { thread.read() }.set_priority as u128)
            .chain(
                unsafe { self.children.read() }
                    .iter()
                    .map(|child| child.set_priority.load(Ordering::Relaxed) as u128),
            )
            .sum::<u128>();
        for (set_priority, propagated_priority) in
            unsafe { self.threads.read() }
                .iter()
                .map(|thread| {
                    (
                        unsafe { thread.read() }.set_priority,
                        &mut unsafe { thread.make_write_guard_unchecked() }.propagated_priority,
                    )
                })
                .chain(
                    unsafe { self.children.make_write_guard_unchecked() }
                        .iter()
                        .map(|child| {
                            (
                                child.set_priority.load(Ordering::Relaxed),
                                &mut child.propagated_priority,
                            )
                        }),
                )
        {
            *propagated_priority = ((set_priority as u128 / set_priority_sum)
                * self.propagated_priority as u128) as u64
        }
    }*/
}
