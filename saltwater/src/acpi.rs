use crate::{mapping::physical_to_virtual_address, println};
use acpi::platform::AcpiPlatform;
use core::ptr::NonNull;
use spinning_top::Spinlock;
#[derive(Clone)]
struct SystemAcpiHandler {}
impl acpi::Handler for SystemAcpiHandler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> acpi::PhysicalMapping<Self, T> {
        acpi::PhysicalMapping {
            physical_start: physical_address,
            virtual_start: unsafe {
                NonNull::<T>::new_unchecked(
                    physical_to_virtual_address(x86_64::PhysAddr::new(physical_address as u64))
                        .as_u64() as *mut T,
                )
            },
            region_length: size,
            mapped_length: size,
            handler: self.clone(),
        }
    }

    fn unmap_physical_region<T>(_region: &acpi::PhysicalMapping<Self, T>) {
        // noop
    }

    fn read_u8(&self, address: usize) -> u8 {
        unsafe { *(address as *const u8) }
    }

    fn read_u16(&self, address: usize) -> u16 {
        unsafe { *(address as *const u16) }
    }

    fn read_u32(&self, address: usize) -> u32 {
        unsafe { *(address as *const u32) }
    }

    fn read_u64(&self, address: usize) -> u64 {
        unsafe { *(address as *const u64) }
    }

    fn write_u8(&self, address: usize, value: u8) {
        unsafe { *(address as *mut u8) = value }
    }

    fn write_u16(&self, address: usize, value: u16) {
        unsafe { *(address as *mut u16) = value }
    }

    fn write_u32(&self, address: usize, value: u32) {
        unsafe { *(address as *mut u32) = value }
    }

    fn write_u64(&self, address: usize, value: u64) {
        unsafe { *(address as *mut u64) = value }
    }

    fn read_io_u8(&self, port: u16) -> u8 {
        unsafe { x86_64::structures::port::PortRead::read_from_port(port) }
    }

    fn read_io_u16(&self, port: u16) -> u16 {
        unsafe { x86_64::structures::port::PortRead::read_from_port(port) }
    }

    fn read_io_u32(&self, port: u16) -> u32 {
        unsafe { x86_64::structures::port::PortRead::read_from_port(port) }
    }

    fn write_io_u8(&self, port: u16, value: u8) {
        unsafe { x86_64::structures::port::PortWrite::write_to_port(port, value) }
    }

    fn write_io_u16(&self, port: u16, value: u16) {
        unsafe { x86_64::structures::port::PortWrite::write_to_port(port, value) }
    }

    fn write_io_u32(&self, port: u16, value: u32) {
        unsafe { x86_64::structures::port::PortWrite::write_to_port(port, value) }
    }

    fn read_pci_u8(&self, address: acpi::PciAddress, offset: u16) -> u8 {
        todo!()
    }

    fn read_pci_u16(&self, address: acpi::PciAddress, offset: u16) -> u16 {
        todo!()
    }

    fn read_pci_u32(&self, address: acpi::PciAddress, offset: u16) -> u32 {
        todo!()
    }

    fn write_pci_u8(&self, address: acpi::PciAddress, offset: u16, value: u8) {
        todo!()
    }

    fn write_pci_u16(&self, address: acpi::PciAddress, offset: u16, value: u16) {
        todo!()
    }

    fn write_pci_u32(&self, address: acpi::PciAddress, offset: u16, value: u32) {
        todo!()
    }

    fn nanos_since_boot(&self) -> u64 {
        todo!()
    }

    fn stall(&self, microseconds: u64) {
        todo!()
    }

    fn sleep(&self, milliseconds: u64) {
        todo!()
    }

    fn create_mutex(&self) -> acpi::Handle {
        todo!()
    }

    fn acquire(&self, mutex: acpi::Handle, timeout: u16) -> Result<(), acpi::aml::AmlError> {
        todo!()
    }

    fn release(&self, mutex: acpi::Handle) {
        todo!()
    }
}
static SYSTEM_ACPI_HANDLER: SystemAcpiHandler = SystemAcpiHandler {};
pub static ACPI_PLATFORM: Spinlock<Option<acpi::platform::AcpiPlatform<SystemAcpiHandler>>> =
    Spinlock::new(None);
pub static PROCESSOR_COUNT: Spinlock<Option<usize>> = Spinlock::new(None);
pub fn bootstrap_initialise(boot_info: &mut bootloader_api::BootInfo) {
    println!(
        "found ACPI root system description pointer at address ({})",
        boot_info.rsdp_addr.into_option().unwrap()
    );
    let mut acpi_platform_guard = ACPI_PLATFORM.lock();
    println!("acquired acpi platform guard...");
    match acpi_platform_guard.as_ref() {
        Some(..) => panic!("ACPI platform initialised before ACPI bootstrap initialiser called!"),
        None => {
            acpi_platform_guard.insert(
                acpi::platform::AcpiPlatform::new(
                    unsafe {
                        let acpi_tables = acpi::AcpiTables::from_rsdp(
                            SYSTEM_ACPI_HANDLER.clone(),
                            boot_info
                                .rsdp_addr
                                .into_option()
                                .expect("bootloader did not provide RSDP address!")
                                as usize,
                        )
                        .unwrap();
                        println!("parsed bootstrap ACPI tables...");
                        acpi_tables
                    },
                    SYSTEM_ACPI_HANDLER.clone(),
                )
                .unwrap(),
            );
        }
    }
    PROCESSOR_COUNT.lock().insert(
        ACPI_PLATFORM
            .lock()
            .as_ref()
            .expect("ACPI platform could not be acquired!")
            .processor_info
            .as_ref()
            .expect("ACPI platform does not contain processor info!")
            .application_processors
            .len()
            + 1,
    );
    println!(
        "counted {} logical processors...",
        PROCESSOR_COUNT.lock().unwrap()
    );
}
