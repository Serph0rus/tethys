use crate::gdt::{self, Selectors};
use alloc::format;
use x86_64::structures::gdt::GlobalDescriptorTable;
pub struct ProcessorData {
    pub gdt: &'static GlobalDescriptorTable,
    pub selectors: Selectors,
}
pub unsafe extern "C" fn initialise(processor: usize) -> () {
    let gdt_selectors = gdt::GLOBAL_DESCRIPTOR_TABLES.lock().pop().expect(&format!(
        "processor {} could not find global descriptor table!",
        processor
    ));
    let data = ProcessorData {
        gdt: gdt_selectors.0,
        selectors: gdt_selectors.1,
    };
    unsafe { gdt::load(data.gdt, data.selectors) };
}
