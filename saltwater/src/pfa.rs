use core::slice;
use spinning_top::Spinlock;
use crate::mapping::physical_to_virtual_address;
use alloc::boxed::Box;
static PAGE_FRAME_BITMAP: Spinlock<Option<Box<&mut [u8]>>> = Spinlock::new(None);
pub fn initialise(boot_info: &mut bootloader_api::BootInfo) {
    let bitmap_size: usize = boot_info
        .memory_regions
        .iter()
        .map(|x| x.end)
        .max()
        .expect("bootloader did not provide any memory regions!")
        as usize
        / (8 * 4096)
        + 1;
    let bitmap_address = x86_64::PhysAddr::new(boot_info
        .memory_regions
        .iter()
        .find(|x| (x.kind == bootloader_api::info::MemoryRegionKind::Usable) & (x.end - x.start > bitmap_size))
        .expect("no memory regions were large enough to store page frame bitmap!").start);
    PAGE_FRAME_BITMAP.lock().insert(Box::new(unsafe {slice::from_raw_parts_mut(physical_to_virtual_address(bitmap_address).as_mut_ptr::<u8>(), bitmap_size)})); // the slice needs to be static
}