#![no_std]
#![no_main]
pub extern crate alloc;
mod acpi;
mod allocator;
mod config;
mod debugcon;
mod frame;
mod gdt;
mod hcf;
mod mapping;
mod page;
mod panic;
mod port;
mod qemu;
use crate::hcf::hcf;
const INITIALISERS: [fn(&mut bootloader_api::BootInfo); 5] = [
    mapping::initialise,
    gdt::bootstrap_initialise,
    allocator::bootstrap_initialise,
    acpi::bootstrap_initialise,
    frame::initialise,
    // stacks::initialise,
    // gdt::initialise
];
bootloader_api::entry_point!(main, config = &config::BOOTLOADER_CONFIG);
pub fn main(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    println!(
        "exited from rust bootloader {} version {}.{}.{}, entering saltwater tethys kernel at address 0x{:x}...",
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
    qemu::exit(qemu::ExitCode::Success);
    println!("successfully initialised saltwater tethys kernel!");
    hcf();
}
