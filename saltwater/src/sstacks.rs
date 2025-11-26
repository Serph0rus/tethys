use crate::{
    frame::PAGE_FRAME_ALLOCATOR, istacks, mapping::{PAGE_SIZE, SYSCALL_STACK_SIZE, syscall_stack_address}, page::{KERNEL_PAGE_FLAGS, get_current_pml4, get_offset_table}
};
use alloc::vec::Vec;
use spinning_top::RwSpinlock;
use x86_64::{VirtAddr, structures::paging::{
    FrameAllocator, Mapper, Page, Size4KiB
}};
static SYSCALL_STACK_BOOLMAP: RwSpinlock<Vec<bool>> = RwSpinlock::new(Vec::new());
#[derive(Debug)]
pub struct SyscallStack(usize);
impl SyscallStack {
    pub fn new() -> Option<SyscallStack> {
        let mut boolmap = SYSCALL_STACK_BOOLMAP.write();
        let stack_index = boolmap
            .iter_mut()
            .enumerate()
            .find(|index_bool| *index_bool.1)
            .map(|index_bool| {
                *index_bool.1 = false;
                index_bool.0
            })
            .unwrap_or_else(|| {
                boolmap.push(false);
                boolmap.len() - 1
            });
        let mut table = get_offset_table(unsafe {&mut *get_current_pml4()});
        for page in (0..(SYSCALL_STACK_SIZE / PAGE_SIZE)).map(|page_index| Page::<Size4KiB>::containing_address(VirtAddr::new(syscall_stack_address(stack_index) + page_index))) {
            let mut pfa_lock = PAGE_FRAME_ALLOCATOR.lock();
            let pfa = pfa_lock.as_mut().expect("page frame allocator not initialised before allocation of syscall stack!");
            unsafe {table.map_to(page, pfa.allocate_frame()?, *KERNEL_PAGE_FLAGS, pfa)}.ok()?.flush();
        }
        Some(SyscallStack(stack_index))
    }
    pub fn bottom(self: &Self) -> u64 {
        syscall_stack_address(self.0)
    }
    pub fn top(self: &Self) -> u64 {
        self.bottom() + SYSCALL_STACK_SIZE - size_of::<usize>() as u64
    }
}
impl Drop for SyscallStack {
    fn drop(&mut self) {
        todo!()
    }
}