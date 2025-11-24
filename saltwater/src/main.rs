#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
pub extern crate alloc;
mod acpi;
mod allocator;
mod config;
mod core;
mod debugcon;
mod frame;
mod gdt;
mod hcf;
mod idt;
mod istacks;
mod kickstart;
mod mapping;
mod page;
mod panic;
mod port;
mod proc;
mod qemu;
mod sstacks;
use crate::hcf::hcf;
const INITIALISERS: [fn(&mut bootloader_api::BootInfo); 7] = [
    mapping::initialise,
    allocator::bootstrap_initialise,
    acpi::bootstrap_initialise,
    frame::initialise,
    istacks::initialise,
    gdt::initialise,
    idt::initialise,
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
        "successfully initialised saltwater tethys kernel! exiting initialisation procedure into kickstart process..."
    );
    //elf::file::parse_ident(data)
    let gdt_selectors = gdt::GLOBAL_DESCRIPTOR_TABLES.lock().pop().unwrap();
    unsafe { gdt::load(gdt_selectors.0, gdt_selectors.1) };
    println!("loaded gdt for bootstrap processor...");

    println!("successfully executed kickstart process!");
    qemu::exit(qemu::ExitCode::Success);
    match &mut boot_info.framebuffer {
        bootloader_api::info::Optional::Some(framebuffer) => framebuffer
            .buffer_mut()
            .iter_mut()
            .skip(1)
            .step_by(4)
            .for_each(|x| *x = 255),
        bootloader_api::info::Optional::None => panic!("no framebuffer!"),
    }
    hcf();
}
