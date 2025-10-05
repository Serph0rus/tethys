use crate::mapping::physical_to_virtual_address;
use alloc::{boxed::Box, vec::Vec};
use core::slice;
use spinning_top::Spinlock;
pub const PAGE_SIZE: usize = 4096;
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
    let bitmap_address = x86_64::PhysAddr::new(
        boot_info
            .memory_regions
            .iter()
            .find(|x| {
                (x.kind == bootloader_api::info::MemoryRegionKind::Usable)
                    & (x.end as usize - x.start as usize > bitmap_size)
            })
            .expect("no memory regions were large enough to store page frame bitmap!")
            .start,
    );
    PAGE_FRAME_BITMAP.lock().insert(Box::new(unsafe {
        slice::from_raw_parts_mut(
            physical_to_virtual_address(bitmap_address).as_mut_ptr::<u8>(),
            bitmap_size,
        )
    }));
    let mut bitmap_guard = PAGE_FRAME_BITMAP.lock();
    let bitmap = bitmap_guard.as_mut().expect("bitmap guard could not be acquired for page frame allocator initialisation!");
    bitmap.fill(0);
    for memory_region in boot_info
        .memory_regions
        .iter()
        .filter(|x| x.kind == bootloader_api::info::MemoryRegionKind::Usable)
        .collect::<Vec<&bootloader_api::info::MemoryRegion>>()
    {
        for frame in (memory_region.start as usize / PAGE_SIZE)..((memory_region.end - 1) as usize / PAGE_SIZE) + 1 {
            bitmap[frame / 8] &= !(1 << (frame % 8))
        }
    }
    for frame in (bitmap_address.as_u64() as usize / PAGE_SIZE)..(bitmap_address.as_u64() as usize + bitmap_size) + 1 {
        bitmap[frame / 8] |= 1 << (frame % 8);
    }
}