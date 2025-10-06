use crate::pfa::{PageFrame};
use crate::mapping::{self, PAGE_SIZE, SYSTEM_STACK_SIZE};
#[repr(align(4096))]
struct DoubleFaultStack {
    memory: [u8; SYSTEM_STACK_SIZE as usize],
}
pub static DOUBLE_FAULT_STACK: DoubleFaultStack = DoubleFaultStack {
    memory: [0; SYSTEM_STACK_SIZE as usize],
};
pub fn initialise(_boot_info: &mut bootloader_api::BootInfo) {
    for processor_index in 1..crate::acpi::PROCESSOR_COUNT.lock().expect("processors not counted before allocation of system stacks!") {
        for frame_index in 0..(SYSTEM_STACK_SIZE / PAGE_SIZE) {
            mapping::system_stack_virtual_address(processor_index)
        }
    }
}