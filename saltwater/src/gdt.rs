use lazy_static::lazy_static;
use spinning_top::Spinlock;
use alloc::{boxed::Box, vec::Vec};
use x86_64::structures::tss::TaskStateSegment;
struct BootstrapSelectors {
    bootstrap_code_selector: x86_64::structures::gdt::SegmentSelector,
    bootstrap_data_selector: x86_64::structures::gdt::SegmentSelector,
    bootstrap_tss_selector: x86_64::structures::gdt::SegmentSelector,
}
lazy_static! {
    static ref BOOTSTRAP_TSS: x86_64::structures::tss::TaskStateSegment = {
        let bootstrap_tss = x86_64::structures::tss::TaskStateSegment::new();
        bootstrap_tss
    };
    static ref BOOTSTRAP_GDT: (x86_64::structures::gdt::GlobalDescriptorTable, BootstrapSelectors) = {
        let mut bootstrap_gdt = x86_64::structures::gdt::GlobalDescriptorTable::new();
        let bootstrap_selectors: BootstrapSelectors = BootstrapSelectors {
            bootstrap_code_selector: bootstrap_gdt.append(x86_64::structures::gdt::Descriptor::kernel_code_segment()),
            bootstrap_data_selector: bootstrap_gdt.append(x86_64::structures::gdt::Descriptor::kernel_data_segment()),
            bootstrap_tss_selector: bootstrap_gdt.append(x86_64::structures::gdt::Descriptor::tss_segment(&BOOTSTRAP_TSS)),
        };
        (bootstrap_gdt, bootstrap_selectors)
    };
}
static TSS: Spinlock<Option<Vec<x86_64::structures::tss::TaskStateSegment>>> = Spinlock::new(None);
static GDT: Spinlock<Option<Box<x86_64::structures::gdt::GlobalDescriptorTable>>> = Spinlock::new(None);
pub fn bootstrap_initialise(_boot_info: &mut bootloader_api::BootInfo) {
    BOOTSTRAP_GDT.0.load();
}
pub fn initialise(_boot_info: &mut bootloader_api::BootInfo) {
    let tss = TSS.lock();
    tss.insert((0..crate::acpi::PROCESSOR_COUNT.lock().expect("processors were not counted before gdt was initialised!")).map(|..| TaskStateSegment::new()))
    for i in 0..crate::acpi::PROCESSOR_COUNT.lock().expect("processors were not counted before gdt was initialised!") {
        
    }
}