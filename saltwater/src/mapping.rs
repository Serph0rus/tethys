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
