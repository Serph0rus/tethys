use lazy_static::lazy_static;
use spinning_top::Spinlock;
use alloc::{boxed::Box, vec::Vec};
use x86_64::structures::tss::TaskStateSegment;

use crate::println;
struct BootstrapSelectors {
    bootstrap_code_selector: x86_64::structures::gdt::SegmentSelector,
    bootstrap_data_selector: x86_64::structures::gdt::SegmentSelector,
    bootstrap_tss_selector: x86_64::structures::gdt::SegmentSelector,
}
lazy_static! {
    static ref BOOTSTRAP_TSS: x86_64::structures::tss::TaskStateSegment = {
        let bootstrap_tss = x86_64::structures::tss::TaskStateSegment::new();
        println!("constructed bootstrap task state segment...");
        bootstrap_tss
    };
    static ref BOOTSTRAP_GDT: (x86_64::structures::gdt::GlobalDescriptorTable, BootstrapSelectors) = {
        let mut bootstrap_gdt = x86_64::structures::gdt::GlobalDescriptorTable::new();
        let bootstrap_selectors: BootstrapSelectors = BootstrapSelectors {
            bootstrap_code_selector: bootstrap_gdt.append(x86_64::structures::gdt::Descriptor::kernel_code_segment()),
            bootstrap_data_selector: bootstrap_gdt.append(x86_64::structures::gdt::Descriptor::kernel_data_segment()),
            bootstrap_tss_selector: bootstrap_gdt.append(x86_64::structures::gdt::Descriptor::tss_segment(&BOOTSTRAP_TSS)),
        };
        println!("constructed global descriptor table...");
        (bootstrap_gdt, bootstrap_selectors)
    };
}
static TSS: Spinlock<Option<Vec<x86_64::structures::tss::TaskStateSegment>>> = Spinlock::new(None);
static GDT: Spinlock<Option<Box<x86_64::structures::gdt::GlobalDescriptorTable>>> = Spinlock::new(None);
pub fn bootstrap_initialise(_boot_info: &mut bootloader_api::BootInfo) {
    BOOTSTRAP_GDT.0.load();
    println!("loaded bootstrap global descriptor table...");
}
pub fn initialise(_boot_info: &mut bootloader_api::BootInfo) {
    let mut tss = TSS.lock();
    println!("acquired additional-processor task state segment vector guard...");
    tss.insert((0..crate::acpi::PROCESSOR_COUNT.lock().expect("processors were not counted before gdt was initialised!")).map(|_| TaskStateSegment::new()).collect());
    println!("inserted per-processor task state segments into global descriptor table...");
    for i in 0..crate::acpi::PROCESSOR_COUNT.lock().expect("processors were not counted before gdt was initialised!") {
        
    }
    println!("loaded global descriptor table...");
}