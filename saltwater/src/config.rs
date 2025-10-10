use crate::mapping;
pub const BOOTLOADER_CONFIG: bootloader_api::BootloaderConfig = {
    let mut bootloader_config = bootloader_api::BootloaderConfig::new_default();
    bootloader_config.mappings.kernel_base =
        bootloader_api::config::Mapping::FixedAddress(mapping::KERNEL_CODE);
    bootloader_config.mappings.framebuffer =
        bootloader_api::config::Mapping::FixedAddress(mapping::FRAMEBUFFER);
    bootloader_config.mappings.kernel_stack = bootloader_api::config::Mapping::FixedAddress(
        mapping::BOOTSTRAP_STACK,
    );
    bootloader_config.kernel_stack_size = mapping::KERNEL_STACK_SIZE;
    bootloader_config.mappings.physical_memory = Some(
        bootloader_api::config::Mapping::FixedAddress(mapping::DIRECT_PHYSICAL),
    );
    bootloader_config
};
