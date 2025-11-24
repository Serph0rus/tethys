use crate::{
    frame::PAGE_FRAME_ALLOCATOR,
    mapping::{
        INTERRUPT_STACK_SIZE, PAGE_SIZE, critical_stack_address, double_fault_stack_address,
        interrupt_stack_address,
    },
    page::{get_current_pml4, get_offset_table},
    println,
};
use lazy_static::lazy_static;
use x86_64::{
    VirtAddr,
    structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB},
};
lazy_static! {
    pub static ref KERNEL_PAGE_FLAGS: PageTableFlags = PageTableFlags::ACCESSED
                                | PageTableFlags::WRITABLE
                                | PageTableFlags::PRESENT;
}
pub fn initialise(_boot_info: &mut bootloader_api::BootInfo) {
    let mut table = get_offset_table(unsafe { &mut *get_current_pml4() });
    println!("constructed offset page table mapper...");
    let mut allocator_guard = PAGE_FRAME_ALLOCATOR.lock();
    let allocator = allocator_guard
        .as_mut()
        .expect("page frame allocator not initialised before allocating interrupt stacks!");
    for processor in 0..crate::acpi::PROCESSOR_COUNT
        .read()
        .expect("processors not counted before allocation of system stacks!")
    {
        println!("initialising stacks for processor no. {}...", processor);
        for (name, stack_address) in [
            ("interrupt", interrupt_stack_address(processor)),
            ("double fault", double_fault_stack_address(processor)),
            ("critical", critical_stack_address(processor)),
        ] {
            for page in (0..(INTERRUPT_STACK_SIZE / PAGE_SIZE)).map(|x| {
                Page::<Size4KiB>::containing_address(VirtAddr::new(stack_address + x * PAGE_SIZE))
            }) {
                unsafe {
                    let _ = table
                        .map_to(
                            page,
                            allocator
                                .allocate_frame()
                                .expect("failed to allocate frame during interrupt stack mapping!"),
                            *KERNEL_PAGE_FLAGS,
                            allocator,
                        )
                        .expect("failed to map page during interrupt stack mapping!");
                }
            }
            println!(
                "allocated and mapped 0x{:x}-byte {} stack at address 0x{:x}",
                INTERRUPT_STACK_SIZE, name, stack_address
            );
        }
    }
    println!("initialised system stacks...");
    x86_64::instructions::tlb::flush_all();
}
