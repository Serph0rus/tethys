use crate::{
    core::ProcessorData, frame::PAGE_FRAME_ALLOCATOR, sstacks::SyscallStack,
    mapping::physical_to_virtual_address,
};
use alloc::{
    boxed::Box,
    collections::vec_deque::VecDeque,
    sync::{Arc, Weak},
    vec::Vec,
};
use core::sync::atomic::AtomicU64;
use spinning_top::RwSpinlock;
use x86_64::{
    PrivilegeLevel, VirtAddr,
    registers::rflags::RFlags,
    structures::{
        gdt::SegmentSelector,
        idt::InterruptStackFrameValue,
        paging::{FrameAllocator, PageTable, PhysFrame},
    },
};
enum MessageStatus {
    Sent(Vec<PhysFrame>),
    Received,
    Responded(Vec<PhysFrame>),
}
struct Message {
    tag: u64,
    status: RwSpinlock<MessageStatus>,
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
    bindings: RwSpinlock<Vec<Box<[u8]>>>,
    kind: ServerKind,
}
enum ServerKind {
    User(UserServer),
    Kernel(KernelServer),
}
struct UserServer {
    requests: RwSpinlock<VecDeque<Arc<Message>>>,
    working: RwSpinlock<Vec<Arc<Message>>>,
    priority_sum: Option<RwSpinlock<usize>>,
}
struct KernelServer {
}
enum Status {
    Ready,
    Aborted,
    Executing(Weak<ProcessorData>),
    AwaitingRequest(Weak<Server>),
    AwaitingResponse(Weak<Message>),
}
struct PageMapping {
    physical_start: usize,
    virtual_start: usize,
    count: usize,
}
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
#[repr(C, align(16))]
pub struct Thread {
    pub general_registers: [u64; 16],
    pub stack_pointer: usize,
    pub interrupt_frame: InterruptStackFrameValue,
    pub status: Status,
    pub set_priority: u64,
    pub propagated_priority: u64,
    pub inherited_priority: u64,
    pub kernel_stack: SyscallStack,
    pub panic_vectors: PanicVectors,
    pub fs_base: u64,
    pub gs_base: u64,
}
pub struct Process {
    pub set_priority: AtomicU64,
    pub propagated_priority: AtomicU64,
    pub parent: Option<Weak<Process>>,
    pub pages: RwSpinlock<*mut PageTable>,
    pub threads: RwSpinlock<Vec<Arc<RwSpinlock<Thread>>>>,
    pub children: RwSpinlock<Vec<Arc<Process>>>,
    pub requests: RwSpinlock<Vec<Weak<Message>>>,
    pub responses: RwSpinlock<VecDeque<Arc<Message>>>,
    pub servers: RwSpinlock<Vec<Arc<Server>>>,
    pub descriptors: RwSpinlock<Vec<Descriptor>>,
}
impl Process {
    pub fn add_child(self_arc: Arc<Self>) -> Arc<Self> {
        let mut children_write = self_arc.children.write();
        let mut pfa_lock = PAGE_FRAME_ALLOCATOR.lock();
        let new_process = Arc::new(Self {
            set_priority: AtomicU64::new(0),
            propagated_priority: AtomicU64::new(0),
            parent: Some(Arc::downgrade(&self_arc)),
            pages: RwSpinlock::new(physical_to_virtual_address(
                pfa_lock
                    .as_mut()
                    .expect("page frame allocator not initialised before process creation!")
                    .allocate_frame()
                    .expect("no page frame could be allocated for new process's page table!")
                    .start_address()
                    .as_u64(),
            ) as *mut PageTable),
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
    pub fn add_thread(self_arc: Arc<Self>) -> Arc<RwSpinlock<Thread>> {
        let mut threads_write = self_arc.threads.write();
        let new_thread = Arc::new(RwSpinlock::new(Thread {
            general_registers: [0; 16],
            stack_pointer: 0,
            interrupt_frame: InterruptStackFrameValue::new(
                VirtAddr::new(0),
                SegmentSelector::new(3, PrivilegeLevel::Ring3),
                RFlags::empty(),
                VirtAddr::new(0),
                SegmentSelector::new(4, PrivilegeLevel::Ring3),
            ),
            status: Status::Ready,
            set_priority: 0,
            propagated_priority: 0,
            inherited_priority: 0,
            kernel_stack: SyscallStack::new().expect("failed to allocate kernel stack during thread creation!"),
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
            fs_base: 0,
            gs_base: 0,
        }));
        threads_write.push(new_thread.clone());
        new_thread
    }
}
