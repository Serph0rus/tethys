use x86_64::structures::paging::{FrameAllocator, Mapper};

use crate::{
    mapping::{self, PAGE_SIZE, SYSTEM_STACK_SIZE},
    page::get_offset_table,
    pfa::{PAGE_FRAME_ALLOCATOR, allocate_frame},
    println,
};
#[repr(align(4096))]
struct DoubleFaultStack {
    memory: [u8; SYSTEM_STACK_SIZE as usize],
}
pub static DOUBLE_FAULT_STACK: DoubleFaultStack = DoubleFaultStack {
    memory: [0; SYSTEM_STACK_SIZE as usize],
};
pub fn initialise(_boot_info: &mut bootloader_api::BootInfo) {
    let mut offset_table = get_offset_table();
    println!("constructed offset page table mapper...");
    let mut page_frame_allocator_guard = PAGE_FRAME_ALLOCATOR.lock();
    println!("acquired page frame allocator guard...");
    let page_frame_allocator = page_frame_allocator_guard
        .as_mut()
        .expect("page frame allocator was not initialised before allocating system stacks!");
    for processor_index in 1..crate::acpi::PROCESSOR_COUNT
        .lock()
        .expect("processors not counted before allocation of system stacks!")
    {
        for frame_index in 0..(SYSTEM_STACK_SIZE / PAGE_SIZE) {
            unsafe {
                offset_table.map_to(
                    x86_64::structures::paging::Page::containing_address(x86_64::VirtAddr::new(
                        mapping::system_stack_virtual_address(processor_index)
                            + frame_index * PAGE_SIZE,
                    )),
                    page_frame_allocator
                        .allocate_frame()
                        .expect("exhausted page frames while allocating system stacks!"),
                    x86_64::structures::paging::PageTableFlags::PRESENT
                        | x86_64::structures::paging::PageTableFlags::WRITABLE,
                    page_frame_allocator,
                );
            }
        }
    }
    println!("allocated and mapped system stacks...");
}
