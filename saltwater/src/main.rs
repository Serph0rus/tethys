#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
pub extern crate alloc;
pub mod acpi;
pub mod allocator;
pub mod config;
pub mod core;
pub mod debugcon;
pub mod frame;
pub mod gdt;
pub mod hcf;
pub mod idt;
pub mod istacks;
pub mod kickstart;
pub mod mapping;
pub mod page;
pub mod panic;
pub mod port;
pub mod proc;
pub mod qemu;
pub mod sstacks;
pub mod scheduler;
pub mod switch;
use crate::hcf::hcf;
const INITIALISERS: [fn(&mut bootloader_api::BootInfo); 8] = [
    mapping::initialise,
    allocator::bootstrap_initialise,
    acpi::bootstrap_initialise,
    frame::initialise,
    istacks::initialise,
    core::initialise,
    idt::initialise,
    kickstart::initialise,
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
    println!(
        "successfully initialised saltwater tethys kernel! exiting initialisation procedure to scheduler & kickstart process..."
    );
    scheduler::enter();
    println!("successfully executed tethys operating system!");
    qemu::exit(qemu::ExitCode::Success);
    println!("exited qemu with success code...");
    match &mut boot_info.framebuffer {
        bootloader_api::info::Optional::Some(framebuffer) => framebuffer
            .buffer_mut()
            .iter_mut()
            .skip(1)
            .step_by(4)
            .for_each(|x| *x = 255),
        bootloader_api::info::Optional::None => panic!("no framebuffer!"),
    }
    println!("wrote success graphic to framebuffer...");
    hcf();
}
