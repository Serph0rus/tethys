use crate::{
    mapping::{PAGE_SIZE, physical_to_virtual_address},
    println,
};
use bootloader_api::info::MemoryRegionKind;
use core::{slice, u8};
use spinning_top::Spinlock;
use x86_64::{
    PhysAddr,
    structures::paging::{FrameAllocator, FrameDeallocator, PhysFrame, Size4KiB},
};
pub struct BitmapPageFrameAllocator {
    bitmap: &'static mut [u8],
    last_allocated_frame_index: usize,
    total_frames: usize,
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
    let total_memory = boot_info
        .memory_regions
        .iter()
        .filter(|x| x.kind == MemoryRegionKind::Usable)
        .map(|x| x.end)
        .max()
        .expect("bootloader did not provide any memory regions!");
    let total_frames = (total_memory / PAGE_SIZE) as usize;
    let bitmap_size = (total_frames + 7) / 8;
    println!(
        "calculated necessary page frame allocator bitmap size as 0x{:x} bytes for 0x{:x} total frames...",
        bitmap_size, total_frames
    );
    let bitmap_address = boot_info
        .memory_regions
        .iter()
        .find(|x| {
            (x.kind == bootloader_api::info::MemoryRegionKind::Usable)
                & ((x.end - x.start) as usize > bitmap_size)
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
                bitmap_size,
            )
        },
        last_allocated_frame_index: 0,
        total_frames,
    });
    println!("constructed page frame allocator bitmap...");
    allocator.bitmap.fill(u8::MAX);
    println!("filled page frame allocator bitmap...");
    for memory_region in boot_info
        .memory_regions
        .iter()
        .filter(|x| x.kind == bootloader_api::info::MemoryRegionKind::Usable)
    {
        for frame_index in (memory_region.start / PAGE_SIZE)..((memory_region.end - 1) / PAGE_SIZE)
        {
            allocator.bitmap[(frame_index / 8) as usize] &= !(1 << (frame_index % 8));
        }
    }
    println!("marked usable page frames as free in page frame allocator bitmap...");
    for frame_index in
        (bitmap_address / PAGE_SIZE)..(bitmap_address + bitmap_size as u64) / PAGE_SIZE + 1
    {
        allocator.bitmap[(frame_index / 8) as usize] |= 1 << (frame_index % 8);
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
unsafe impl FrameAllocator<Size4KiB> for BitmapPageFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let mut found_index: Option<usize> = None;
        for frame_index in self.last_allocated_frame_index..self.total_frames {
            if self.bitmap[frame_index / 8] & (1 << (frame_index % 8)) == 0 {
                found_index.insert(frame_index);
            }
        }
        for frame_index in 0..self.last_allocated_frame_index {
            if self.bitmap[frame_index / 8] & (1 << (frame_index % 8)) == 0 {
                found_index.insert(frame_index);
            }
        }
        let frame_index = found_index?;
        self.bitmap[frame_index / 8] |= 1 << (frame_index % 8);
        self.last_allocated_frame_index = frame_index;
        let frame_address = frame_index as u64 * PAGE_SIZE;
        unsafe { (frame_address as *mut u8).write_bytes(0, PAGE_SIZE as usize) };
        Some(PhysFrame::containing_address(PhysAddr::new(frame_address)))
    }
}
impl FrameDeallocator<Size4KiB> for BitmapPageFrameAllocator {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame) {
        let frame_index = frame.start_address().as_u64() / PAGE_SIZE;
        self.bitmap[frame_index as usize / 8] &= !(1 << (frame_index % 8));
    }
}
