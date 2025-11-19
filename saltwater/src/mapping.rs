use crate::println;
const ONE_MEGABYTE: u64 = 0x0000_0000_0010_0000;
const SIXTEEN_MEGABYTES: u64 = 0x0000_0000_0100_0000;
const ONE_TERABYTE: u64 = 0x0000_0100_0000_0000;
const TWELVE_TERABYTES: u64 = 0x0000_0c00_0000_0000;
const SIXTEEN_TERABYTES: u64 = 0x0000_1000_0000_0000;
pub const PAGE_SIZE: u64 = 4096;
pub const HIGHER_HALF: u64 = 0xffff_8000_0000_0000;
pub const KERNEL_CODE: u64 = HIGHER_HALF;
pub const KERNEL_HEAP: u64 = KERNEL_CODE + SIXTEEN_TERABYTES;
pub const BOOTSTRAP_STACK: u64 = KERNEL_HEAP + SIXTEEN_TERABYTES;
pub const FRAMEBUFFER: u64 = BOOTSTRAP_STACK + SIXTEEN_TERABYTES;
pub const SYSCALL_STACKS: u64 = FRAMEBUFFER + SIXTEEN_TERABYTES;
pub const INTERRUPT_STACKS: u64 = SYSCALL_STACKS + TWELVE_TERABYTES;
pub const DOUBLE_FAULT_STACKS: u64 = INTERRUPT_STACKS + ONE_TERABYTE;
pub const CRITICAL_STACKS: u64 = DOUBLE_FAULT_STACKS + ONE_TERABYTE;
pub const DIRECT_PHYSICAL: u64 = CRITICAL_STACKS + ONE_TERABYTE;
pub const SYSCALL_STACK_SIZE: u64 = SIXTEEN_MEGABYTES;
pub const INTERRUPT_STACK_SIZE: u64 = ONE_MEGABYTE;
pub const fn syscall_stack_address(index: usize) -> u64 {
    SYSCALL_STACKS + SYSCALL_STACK_SIZE * index as u64
}
pub const fn interrupt_stack_address(index: usize) -> u64 {
    INTERRUPT_STACKS + INTERRUPT_STACK_SIZE * index as u64
}
pub const fn double_fault_stack_address(index: usize) -> u64 {
    DOUBLE_FAULT_STACKS + INTERRUPT_STACK_SIZE * index as u64
}
pub const fn critical_stack_address(index: usize) -> u64 {
    CRITICAL_STACKS + INTERRUPT_STACK_SIZE * index as u64
}
pub const fn physical_to_virtual_address(physical: u64) -> u64 {
    DIRECT_PHYSICAL + physical
}
pub fn initialise(boot_info: &mut bootloader_api::BootInfo) {
    println!("mapped kernel ELF file at address 0x{:x}...", boot_info.kernel_addr);
    println!("mapped framebuffer at address 0x{:x}...", match &boot_info.framebuffer {
        bootloader_api::info::Optional::Some(framebuffer) => framebuffer.buffer().as_ptr() as usize,
        bootloader_api::info::Optional::None => panic!("bootloader did not establish framebuffer!")
    });
    println!("mapped physical memory at address 0x{:x}...", boot_info.physical_memory_offset.into_option().expect("bootloader did not map physical memory!"));
}
