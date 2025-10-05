#![no_std]
#![no_main]
pub extern crate alloc;
mod acpi;
mod allocator;
mod debugcon;
mod gdt;
mod hcf;
mod mapping;
mod panic;
mod pfa;
mod sysstacks;
use crate::hcf::hcf;
pub static BOOTLOADER_CONFIG: bootloader_api::BootloaderConfig = {
    let mut bootloader_config = bootloader_api::BootloaderConfig::new_default();
    bootloader_config.mappings.kernel_base =
        bootloader_api::config::Mapping::FixedAddress(mapping::SYSTEM_CODE);
    bootloader_config.mappings.framebuffer =
        bootloader_api::config::Mapping::FixedAddress(mapping::FRAMEBUFFER);
    bootloader_config.mappings.kernel_stack = bootloader_api::config::Mapping::FixedAddress(
        mapping::system_stack_virtual_address(0) + mapping::SYSTEM_STACK_SIZE,
    );
    bootloader_config.kernel_stack_size = mapping::SYSTEM_STACK_SIZE;
    bootloader_config.mappings.physical_memory = Some(
        bootloader_api::config::Mapping::FixedAddress(mapping::DIRECT_PHYSICAL),
    );
    bootloader_config
};
const INITIALISERS: [fn(&mut bootloader_api::BootInfo); 6] = [
    gdt::bootstrap_initialise,
    allocator::bootstrap_initialise,
    acpi::bootstrap_initialise,
    pfa::initialise,
    gdt::initialise,
    sysstacks::initialise,
];
bootloader_api::entry_point!(main);
pub fn main(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    println!(
        "exited from rust bootloader {} version {}.{}.{}...",
        if boot_info.api_version.pre_release() {
            "pre-release"
        } else {
            "release"
        },
        boot_info.api_version.version_major(),
        boot_info.api_version.version_minor(),
        boot_info.api_version.version_patch()
    );
    println!("entering saltwater tethys kernel...");
    boot_info
        .physical_memory_offset
        .into_option()
        .expect("bootloader did not provide higher-half direct memory map!");
    for initialiser in INITIALISERS {
        initialiser(boot_info);
    }
    println!("exiting to halt-and-catch-fire loop...");
    hcf();
}
