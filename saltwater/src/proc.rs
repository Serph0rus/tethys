use crate::{core::ProcessorData, frame::PAGE_FRAME_ALLOCATOR, mapping::physical_to_virtual_address};
use alloc::{
    boxed::Box,
    collections::vec_deque::VecDeque,
    sync::{Arc, Weak},
    vec::Vec,
};
use core::sync::atomic::{AtomicU64, Ordering};
use spinning_top::RwSpinlock;
use x86_64::{PrivilegeLevel, VirtAddr, registers::rflags::RFlags, structures::{
    gdt::SegmentSelector, idt::{InterruptStackFrame, InterruptStackFrameValue}, paging::{FrameAllocator, PageTable, PhysFrame}
}};
struct Message {
    tag: u64,
    frames: Vec<PhysFrame>,
    sender: Weak<Thread>,
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
    tag: u64,
    messages: RwSpinlock<VecDeque<Arc<Message>>>,
    bindings: RwSpinlock<Vec<Box<[u8]>>>,
    priority_sum: RwSpinlock<usize>,
}
enum Status {
    Ready,
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
    double_fault: u64,
    stack: u64,
    protection: u64,
    page: u64,
    floating_point: u64,
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
    pub kernel_stack: usize,
    pub panic_vectors: PanicVectors,
}
impl Thread {
    pub fn new(
        general_registers: [u64; 16],
        stack_pointer: usize,
        interrupt_frame: x86_64::structures::idt::InterruptStackFrameValue,
        set_priority: u64,
        panic_vectors: PanicVectors,
    ) -> Thread {
        Thread {
            general_registers,
            stack_pointer,
            interrupt_frame,
            status: Status::Ready,
            set_priority: set_priority,
            propagated_priority: set_priority,
            inherited_priority: set_priority,
            kernel_stack: 0,
            panic_vectors,
        }
    }
}
pub struct Process {
    pub set_priority: AtomicU64,
    pub propagated_priority: AtomicU64,
    pub parent: Option<Weak<Process>>,
    pub pages: RwSpinlock<*mut PageTable>,
    pub threads: RwSpinlock<Vec<Arc<RwSpinlock<Thread>>>>,
    pub children: RwSpinlock<Vec<Arc<Process>>>,
    pub messages: RwSpinlock<Vec<Message>>,
    pub servers: RwSpinlock<Vec<Arc<Server>>>,
    pub descriptors: RwSpinlock<Vec<Descriptor>>,
}
impl Process {
    pub fn add_child(self_arc: Arc<Process>) -> Arc<Process> {
        let mut children_write = self_arc.children.write();
        let mut pfa_lock = PAGE_FRAME_ALLOCATOR.lock();
        let new_process = Arc::new(Process {
            set_priority: AtomicU64::new(0),
            propagated_priority: AtomicU64::new(0),
            parent: Some(Arc::downgrade(&self_arc)),
            pages: RwSpinlock::new(physical_to_virtual_address(pfa_lock.as_mut().expect("page frame allocator not initialised before process creation!").allocate_frame().expect("no page frame could be allocated for new process's page table!").start_address().as_u64()) as *mut PageTable),
            threads: RwSpinlock::new(Vec::new()),
            children: RwSpinlock::new(Vec::new()),
            messages: RwSpinlock::new(Vec::new()),
            servers: RwSpinlock::new(Vec::new()),
            descriptors: RwSpinlock::new(Vec::new()),
        });
        children_write.push(new_process.clone());
        new_process
    }
    pub fn add_thread(self_arc: Arc<Process>) -> Arc<RwSpinlock<Thread>> {
        let mut threads_write = self_arc.threads.write();
        let new_thread = Arc::new(RwSpinlock::new(Thread {
            general_registers: [0; 16],
            stack_pointer: 0,
            interrupt_frame: InterruptStackFrameValue::new(VirtAddr::new(0), SegmentSelector::new(3, PrivilegeLevel::Ring3), RFlags::empty(), VirtAddr::new(0), SegmentSelector::new(4, PrivilegeLevel::Ring3)),
            status: Status::Ready,
            set_priority: 0,
            propagated_priority: 0,
            inherited_priority: 0,
            kernel_stack: todo!(),
            panic_vectors: todo!(),
        }));
        threads_write.push(new_thread.clone());
        new_thread
    }
}