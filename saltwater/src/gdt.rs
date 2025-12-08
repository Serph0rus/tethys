use crate::{idt::{CRITICAL_IST_INDEX, DOUBLE_FAULT_IST_INDEX, INTERRUPT_IST_INDEX, SYSCALL_IST_INDEX}, mapping};
use alloc::boxed::Box;
use x86_64::{
    instructions::tables::load_tss,
    registers::segmentation::{self, Segment},
    structures::{
        gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
        tss::TaskStateSegment,
    },
};
#[derive(Clone)]
pub struct Selectors {
    pub kernel_code: SegmentSelector,
    pub kernel_data: SegmentSelector,
    pub user_code: SegmentSelector,
    pub user_data: SegmentSelector,
    pub task_state: SegmentSelector,
}
pub fn new(processor: usize) -> (&'static GlobalDescriptorTable, Selectors) {
    let mut gdt = GlobalDescriptorTable::new();
    let mut tss = Box::new(TaskStateSegment::new());
    tss.privilege_stack_table[0] = x86_64::VirtAddr::new(mapping::syscall_stack_address(processor));
    tss.interrupt_stack_table[SYSCALL_IST_INDEX] =
        x86_64::VirtAddr::new(mapping::syscall_stack_address(processor));
    tss.interrupt_stack_table[INTERRUPT_IST_INDEX] =
        x86_64::VirtAddr::new(mapping::interrupt_stack_address(processor));
    tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX] =
        x86_64::VirtAddr::new(mapping::double_fault_stack_address(processor));
    tss.interrupt_stack_table[CRITICAL_IST_INDEX] =
        x86_64::VirtAddr::new(mapping::critical_stack_address(processor));
    let selectors = Selectors {
        kernel_code: gdt.append(Descriptor::kernel_code_segment()),
        kernel_data: gdt.append(Descriptor::kernel_data_segment()),
        user_code: gdt.append(Descriptor::user_code_segment()),
        user_data: gdt.append(Descriptor::user_data_segment()),
        task_state: gdt.append(Descriptor::tss_segment(Box::leak(tss))),
    };
    (Box::leak(Box::new(gdt)), selectors)
}
pub unsafe fn load(gdt_selectors: &(&'static GlobalDescriptorTable, Selectors)) {
    unsafe {
        gdt_selectors.0.load();
        segmentation::CS::set_reg(gdt_selectors.1.kernel_code);
        segmentation::DS::set_reg(gdt_selectors.1.kernel_data);
        segmentation::ES::set_reg(gdt_selectors.1.kernel_data);
        segmentation::FS::set_reg(gdt_selectors.1.kernel_data);
        segmentation::GS::set_reg(gdt_selectors.1.kernel_data);
        segmentation::SS::set_reg(gdt_selectors.1.kernel_data);
        load_tss(gdt_selectors.1.task_state);
    };
}
