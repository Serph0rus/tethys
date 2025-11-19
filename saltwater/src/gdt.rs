use crate::{acpi::PROCESSOR_COUNT, mapping, println};
use alloc::{boxed::Box, vec::Vec};
use core::arch::asm;
use spinning_top::RwSpinlock;
use x86_64::{
    registers::segmentation::Segment,
    structures::{
        gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
        tss::TaskStateSegment,
    },
};
static GLOBAL_DESCRIPTOR_TABLES: RwSpinlock<Vec<GlobalDescriptorTable>> =
    RwSpinlock::new(Vec::new());
static KERNEL_CODE_SELECTOR: SegmentSelector =
    SegmentSelector::new(1, x86_64::PrivilegeLevel::Ring0);
static KERNEL_DATA_SELECTOR: SegmentSelector =
    SegmentSelector::new(2, x86_64::PrivilegeLevel::Ring0);
static USER_CODE_SELECTOR: SegmentSelector = SegmentSelector::new(3, x86_64::PrivilegeLevel::Ring3);
static USER_DATA_SELECTOR: SegmentSelector = SegmentSelector::new(4, x86_64::PrivilegeLevel::Ring3);
pub fn initialise(_boot_info: &mut bootloader_api::BootInfo) {
    let mut global_descriptor_tables = GLOBAL_DESCRIPTOR_TABLES.write();
    global_descriptor_tables.append(
        &mut (0..PROCESSOR_COUNT
            .read()
            .expect("cores not counted before initialisation of global descriptor table!"))
            .map(|x| {
                let mut gdt = GlobalDescriptorTable::new();
                gdt.append(Descriptor::kernel_code_segment());
                gdt.append(Descriptor::kernel_data_segment());
                gdt.append(Descriptor::user_code_segment());
                gdt.append(Descriptor::user_data_segment());
                let mut tss = Box::new(TaskStateSegment::new());
                tss.privilege_stack_table[0] =
                    x86_64::VirtAddr::new(mapping::kernel_stack_address(x));
                gdt.append(Descriptor::tss_segment(Box::leak(tss)));
                gdt
            })
            .collect(),
    );
    println!("constructed bootstrap global descriptor table...");
}
pub unsafe fn load(index: usize) {
    unsafe {
        let global_descriptor_tables = GLOBAL_DESCRIPTOR_TABLES.read();
        asm!("lgdt [{}]", in(reg) global_descriptor_tables.get(index).unwrap_or_else(|| panic!("core {} could not find global descriptor table!", index)), options(readonly, nostack, preserves_flags));
        drop(global_descriptor_tables);
        println!("loaded bootstrap global descriptor table...");
        x86_64::registers::segmentation::CS::set_reg(
            x86_64::registers::segmentation::SegmentSelector(KERNEL_CODE_SELECTOR.0),
        );
        println!("far-returned into new code segment...");
        x86_64::registers::segmentation::DS::set_reg(
            x86_64::registers::segmentation::SegmentSelector(KERNEL_DATA_SELECTOR.0),
        );
        x86_64::registers::segmentation::ES::set_reg(
            x86_64::registers::segmentation::SegmentSelector(KERNEL_DATA_SELECTOR.0),
        );
        x86_64::registers::segmentation::FS::set_reg(
            x86_64::registers::segmentation::SegmentSelector(KERNEL_DATA_SELECTOR.0),
        );
        x86_64::registers::segmentation::GS::set_reg(
            x86_64::registers::segmentation::SegmentSelector(KERNEL_DATA_SELECTOR.0),
        );
        x86_64::registers::segmentation::SS::set_reg(
            x86_64::registers::segmentation::SegmentSelector(KERNEL_DATA_SELECTOR.0),
        );
        println!("reloaded data segment registers...");
    };
}
