use crate::{
    acpi::PROCESSOR_COUNT,
    idt::{CRITICAL_IST_INDEX, DOUBLE_FAULT_IST_INDEX, INTERRUPT_IST_INDEX, SYSCALL_IST_INDEX},
    mapping, println,
};
use alloc::{boxed::Box, vec::Vec};
use spinning_top::Spinlock;
use x86_64::{
    instructions::tables::load_tss,
    registers::segmentation::{self, Segment},
    structures::{
        gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
        tss::TaskStateSegment,
    },
};
pub struct Selectors {
    pub kernel_code: SegmentSelector,
    pub kernel_data: SegmentSelector,
    pub user_code: SegmentSelector,
    pub user_data: SegmentSelector,
    pub task_state: SegmentSelector,
}
pub static GLOBAL_DESCRIPTOR_TABLES: Spinlock<
    Vec<(&'static mut GlobalDescriptorTable, Selectors)>,
> = Spinlock::new(Vec::new());
static KERNEL_CODE_SELECTOR: SegmentSelector =
    SegmentSelector::new(1, x86_64::PrivilegeLevel::Ring0);
static KERNEL_DATA_SELECTOR: SegmentSelector =
    SegmentSelector::new(2, x86_64::PrivilegeLevel::Ring0);
static USER_CODE_SELECTOR: SegmentSelector = SegmentSelector::new(3, x86_64::PrivilegeLevel::Ring3);
static USER_DATA_SELECTOR: SegmentSelector = SegmentSelector::new(4, x86_64::PrivilegeLevel::Ring3);
static TSS_SELECTOR: SegmentSelector = SegmentSelector::new(5, x86_64::PrivilegeLevel::Ring0);
pub fn initialise(_boot_info: &mut bootloader_api::BootInfo) {
    let mut global_descriptor_tables = GLOBAL_DESCRIPTOR_TABLES.lock();
    global_descriptor_tables.append(
        &mut (0..PROCESSOR_COUNT
            .read()
            .expect("cores not counted before initialisation of global descriptor table!"))
            .map(|processor| {
                let mut gdt = Box::new(GlobalDescriptorTable::new());
                let mut tss = Box::new(TaskStateSegment::new());
                tss.privilege_stack_table[0] =
                    x86_64::VirtAddr::new(mapping::syscall_stack_address(processor));
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
                (Box::leak(gdt), selectors)
            })
            .collect(),
    );
    println!("constructed bootstrap global descriptor table...");
}
pub unsafe fn load(gdt: &'static GlobalDescriptorTable, selectors: Selectors) {
    unsafe {
        gdt.load();
        segmentation::CS::set_reg(selectors.kernel_code);
        segmentation::DS::set_reg(selectors.kernel_data);
        segmentation::ES::set_reg(selectors.kernel_data);
        segmentation::FS::set_reg(selectors.kernel_data);
        segmentation::GS::set_reg(selectors.kernel_data);
        segmentation::SS::set_reg(selectors.kernel_data);
        load_tss(selectors.task_state);
    };
}
