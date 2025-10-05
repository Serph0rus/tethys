use crate::pfa::PageFrame;
pub fn initialise(_boot_info: &mut bootloader_api::BootInfo) {
    for i in 1..crate::acpi::PROCESSOR_COUNT.lock().expect("processors not counted before allocation of system stacks!") {
        // PageFrame::new()
    }
}