#![no_std]
#![no_main]
pub extern crate alloc;
mod config;
mod qemu;
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
const INITIALISERS: [fn(&mut bootloader_api::BootInfo); 7] = [
    mapping::initialise,
    gdt::bootstrap_initialise,
    allocator::bootstrap_initialise,
    acpi::bootstrap_initialise,
    pfa::initialise,
    gdt::initialise,
    sysstacks::initialise,
];
bootloader_api::entry_point!(main, config = &config::BOOTLOADER_CONFIG);
pub fn main(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    println!(
        "exited from rust bootloader {} version {}.{}.{}, entering saltwater tethys system at address 0x{:x}...",
        if boot_info.api_version.pre_release() {
            "pre-release"
        } else {
            "release"
        },
        boot_info.api_version.version_major(),
        boot_info.api_version.version_minor(),
        boot_info.api_version.version_patch(),
        x86_64::instructions::read_rip(),
    );
    for initialiser in INITIALISERS {
        initialiser(boot_info);
    }
    println!("successfully initialised saltwater tethys system!");
    hcf();
}
