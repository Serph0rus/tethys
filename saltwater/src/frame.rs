use crate::{
    mapping::{PAGE_SIZE, physical_to_virtual_address},
    println,
};
use core::{slice, u8};
use spinning_top::Spinlock;
pub struct BitmapPageFrameAllocator {
    bitmap: &'static mut [u8],
    last_allocated_frame_index: usize,
}
pub static PAGE_FRAME_ALLOCATOR: Spinlock<Option<BitmapPageFrameAllocator>> = Spinlock::new(None);
pub fn initialise(boot_info: &mut bootloader_api::BootInfo) {
    println!("bootloader describes memory regions:");
    for (index, region) in boot_info.memory_regions.iter().enumerate() {
        println!(
            "no.{} from 0x{:x} to 0x{:x} is {}.",
            index,
            region.start,
            region.end,
            match region.kind {
                bootloader_api::info::MemoryRegionKind::Usable => "usable",
                bootloader_api::info::MemoryRegionKind::Bootloader => "bootloader",
                bootloader_api::info::MemoryRegionKind::UnknownUefi(_) => "uefi",
                bootloader_api::info::MemoryRegionKind::UnknownBios(_) => "bios",
                _ => "unknown",
            }
        )
    }
    let bitmap_size: u64 = (boot_info
        .memory_regions
        .iter()
        .filter(|x| x.kind == bootloader_api::info::MemoryRegionKind::Usable)
        .map(|x| x.end)
        .max()
        .expect("bootloader did not provide any usable memory regions!")
        / PAGE_SIZE
        + 7)
        / 8;
    println!(
        "calculated necessary page frame allocator bitmap size as 0x{:x} bytes...",
        bitmap_size
    );
    let bitmap_address = boot_info
        .memory_regions
        .iter()
        .find(|x| {
            (x.kind == bootloader_api::info::MemoryRegionKind::Usable)
                & (x.end - x.start > bitmap_size)
        })
        .expect("no memory regions were large enough to store page frame bitmap!")
        .start;
    println!(
        "determined page frame allocator bitmap physical address of 0x{:x}...",
        bitmap_address
    );
    let mut allocator_guard = PAGE_FRAME_ALLOCATOR.lock();
    println!("acquired bitmap page frame allocator guard...");
    let allocator = allocator_guard.insert(BitmapPageFrameAllocator {
        bitmap: unsafe {
            slice::from_raw_parts_mut(
                physical_to_virtual_address(bitmap_address) as *mut u8,
                bitmap_size as usize,
            )
        },
        last_allocated_frame_index: 0,
    });
    println!("constructed page frame allocator bitmap...");
    allocator.bitmap.fill(u8::MAX);
    println!("filled page frame allocator bitmap...");
    for memory_region in boot_info
        .memory_regions
        .iter()
        .filter(|x| x.kind == bootloader_api::info::MemoryRegionKind::Usable)
    {
        for frame in (memory_region.start / PAGE_SIZE)..((memory_region.end - 1) / PAGE_SIZE) {
            allocator.bitmap[(frame / 8) as usize] &= !(1 << (frame % 8))
        }
    }
    println!("marked usable page frames as free in page frame allocator bitmap...");
    for frame in (bitmap_address / PAGE_SIZE)..(bitmap_address + bitmap_size) / PAGE_SIZE + 1 {
        allocator.bitmap[(frame / 8) as usize] |= 1 << (frame % 8);
    }
    println!("marked page frame allocator bitmap as unusable within itself...");
    let free_page_count = allocator
        .bitmap
        .iter()
        .map(|x| x.count_zeros())
        .sum::<u32>() as u64;
    println!(
        "initialised bitmap page frame allocator, counted 0x{:x} free page frames, {} bytes free...",
        free_page_count,
        free_page_count * PAGE_SIZE
    );
}
pub fn new_frame() -> Option<usize> {
    let mut page_frame_allocator_guard = PAGE_FRAME_ALLOCATOR.lock();
    let page_frame_allocator = page_frame_allocator_guard.as_mut().expect("page frame allocator not initialised before page frame allocation attempted!");
    for frame_index in page_frame_allocator.last_allocated_frame_index..page_frame_allocator.bitmap.len() * 8 {
        if page_frame_allocator.bitmap[frame_index / 8] & (1 << (frame_index % 8)) == 0 {
            page_frame_allocator.bitmap[frame_index / 8] |= 1 << (frame_index % 8);
            page_frame_allocator.last_allocated_frame_index = frame_index;
            return Some(frame_index);
        }
    }
    for frame_index in 0..page_frame_allocator.last_allocated_frame_index {
        if page_frame_allocator.bitmap[frame_index / 8] & (1 << (frame_index % 8)) == 0 {
            page_frame_allocator.bitmap[frame_index / 8] |= 1 << (frame_index % 8);
            page_frame_allocator.last_allocated_frame_index = frame_index;
            return Some(frame_index);
        }
    }
    None
}
pub fn drop_frame(frame_index: usize) {
    let mut page_frame_allocator_guard = PAGE_FRAME_ALLOCATOR.lock();
    let page_frame_allocator = page_frame_allocator_guard.as_mut().expect("page frame allocator not initialised before page frame deallocation attempted!");
    page_frame_allocator.bitmap[frame_index / 8] &= !(1 << (frame_index % 8))
}