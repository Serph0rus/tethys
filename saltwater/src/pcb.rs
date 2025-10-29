use alloc::vec::Vec;
struct PageMapping {
    physical_start: usize,
    virtual_start: usize,
    count: usize,
}
pub struct ProcessControlBlock {
    mappings: Vec<PageMapping>,
    interrupt_frame: x86_64::structures::idt::InterruptStackFrame,
}