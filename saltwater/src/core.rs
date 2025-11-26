use crate::{
    acpi::PROCESSOR_COUNT,
    gdt::{self, Selectors},
    println,
    proc::{Process, Thread},
};
use alloc::{boxed::Box, collections::vec_deque::VecDeque, sync::{Arc, Weak}, vec::Vec};
use spinning_top::RwSpinlock;
use x86_64::structures::gdt::GlobalDescriptorTable;
pub struct ProcessorData {
    pub gdt: &'static GlobalDescriptorTable,
    pub selectors: Selectors,
    pub ready_queue: VecDeque<Arc<RwSpinlock<Thread>>>,
    pub current_process: Weak<Process>,
    pub current_thread: Arc<Thread>,
}
pub static PROCESSOR_DATA_VEC: RwSpinlock<Vec<&'static mut RwSpinlock<ProcessorData>>> =
    RwSpinlock::new(Vec::new());
pub fn initialise(_boot_info: &mut bootloader_api::BootInfo) {
    let mut processor_data = PROCESSOR_DATA_VEC.write();
    processor_data.append(
        &mut (0..PROCESSOR_COUNT
            .read()
            .expect("processors not counted before per-processor initialisation!"))
            .map(|index| {
                let gdt_selectors = gdt::new(index);
                Box::leak(Box::new(RwSpinlock::new(ProcessorData {
                    gdt: gdt_selectors.0,
                    selectors: gdt_selectors.1,
                    ready_queue: VecDeque::new(),
                })))
            })
            .collect::<Vec<&'static mut RwSpinlock<ProcessorData>>>(),
    );
    println!(
        "initialised processor data for {} processors...",
        processor_data.len()
    );
    let bootstrap_data = processor_data.get(0).expect("bootstrap processor could not find processor data!").read();
    unsafe {gdt::load(&bootstrap_data.gdt, bootstrap_data.selectors.clone())};
}
