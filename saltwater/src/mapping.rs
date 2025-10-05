use crate::println;
const SIXTEEN_TERABYTES: u64 = 0x0000_1000_0000_0000;
pub const HIGHER_HALF: u64 = 0xffff_8000_0000_0000;
pub const SYSTEM_CODE: u64 = HIGHER_HALF;
pub const SYSTEM_HEAP: u64 = SYSTEM_CODE + SIXTEEN_TERABYTES;
pub const FRAMEBUFFER: u64 = SYSTEM_HEAP + SIXTEEN_TERABYTES;
pub const SYSTEM_STACKS: u64 = FRAMEBUFFER + SIXTEEN_TERABYTES;
pub const DIRECT_PHYSICAL: u64 = SYSTEM_STACKS + SIXTEEN_TERABYTES;
pub const SYSTEM_STACK_SIZE: u64 = 0x0000_0000_1000_0000;
pub const fn system_stack_virtual_address(index: usize) -> u64 {
    SYSTEM_STACKS + index as u64 * SYSTEM_STACK_SIZE
}
pub const fn physical_to_virtual_address(physical: x86_64::PhysAddr) -> x86_64::VirtAddr {
    x86_64::VirtAddr::new(DIRECT_PHYSICAL + physical.as_u64())
}
pub fn initialise(boot_info: &mut bootloader_api::BootInfo) {
    println!("mapped system code at address 0x{:x}", boot_info.kernel_addr);
    println!("mapped system heap at address 0x{:x}", SYSTEM_HEAP);
    println!("mapped framebuffer at address 0x{:x}", match &boot_info.framebuffer {
        bootloader_api::info::Optional::Some(framebuffer) => framebuffer.buffer().as_ptr() as usize,
        bootloader_api::info::Optional::None => panic!("bootloader did not establish framebuffer!")
    });
    println!("mapped system stacks of size 0x{:x} at address 0x{:x}", SYSTEM_STACK_SIZE, SYSTEM_STACKS);
    println!("mapped physical memory at address 0x{:x}", boot_info.physical_memory_offset.into_option().expect("bootloader did not map physical memory!"));
}