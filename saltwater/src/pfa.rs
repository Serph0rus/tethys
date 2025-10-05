use crate::mapping::physical_to_virtual_address;
use alloc::vec::Vec;
use core::{slice, u8};
use spinning_top::Spinlock;
pub const PAGE_SIZE: usize = 4096;
struct BitmapPageFrameAllocator {
    bitmap: &'static mut [u8],
    last_allocated_frame_index: usize,
}
static PAGE_FRAME_ALLOCATOR: Spinlock<Option<BitmapPageFrameAllocator>> = Spinlock::new(None);
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
    let mut allocator_guard = PAGE_FRAME_ALLOCATOR.lock();
    let allocator = allocator_guard.insert(BitmapPageFrameAllocator {
        bitmap: unsafe {
            slice::from_raw_parts_mut(
                physical_to_virtual_address(bitmap_address).as_mut_ptr::<u8>(),
                bitmap_size,
            )
        },
        last_allocated_frame_index: 0,
    });
    allocator.bitmap.fill(u8::MAX);
    for memory_region in boot_info
        .memory_regions
        .iter()
        .filter(|x| x.kind == bootloader_api::info::MemoryRegionKind::Usable)
        .collect::<Vec<&bootloader_api::info::MemoryRegion>>()
    {
        for frame in (memory_region.start as usize / PAGE_SIZE)
            ..((memory_region.end - 1) as usize / PAGE_SIZE) + 1
        {
            allocator.bitmap[frame / 8] &= !(1 << (frame % 8))
        }
    }
    for frame in (bitmap_address.as_u64() as usize / PAGE_SIZE)
        ..(bitmap_address.as_u64() as usize + bitmap_size) + 1
    {
        allocator.bitmap[frame / 8] |= 1 << (frame % 8);
    }
}
pub struct PageFrame {
    index: usize,
}
impl PageFrame {
    pub fn new() -> PageFrame {
        let mut allocator_guard = PAGE_FRAME_ALLOCATOR.lock();
        let allocator = allocator_guard
            .as_mut()
            .expect("page frame allocator not initialised before page frame allocation attempted!");
        for frame in allocator.last_allocated_frame_index..allocator.bitmap.len() * 8 {
            if allocator.bitmap[frame / 8] & (1 << (frame % 8)) == 0 {
                allocator.bitmap[frame / 8] |= 1 << (frame % 8);
                allocator.last_allocated_frame_index = frame;
                return PageFrame { index: frame };
            }
        }
        for frame in 0..allocator.last_allocated_frame_index {
            if allocator.bitmap[frame / 8] & (1 << (frame % 8)) == 0 {
                allocator.bitmap[frame / 8] |= 1 << (frame % 8);
                allocator.last_allocated_frame_index = frame;
                return PageFrame { index: frame };
            }
        }
        panic!("page frame allocator could not find any free page frames!")
    }
    pub fn drop(self: Self) {
        let mut allocator_guard = PAGE_FRAME_ALLOCATOR.lock();
        let allocator = allocator_guard.as_mut().expect(
            "page frame allocator not initialised before page frame deallocation attempted!",
        );
        allocator.bitmap[self.index / 8] &= !(1 << (self.index % 8));
    }
}
unsafe impl x86_64::structures::paging::FrameAllocator<x86_64::structures::paging::Size4KiB> for BitmapPageFrameAllocator {
    fn allocate_frame(&mut self) -> Option<x86_64::structures::paging::PhysFrame<x86_64::structures::paging::Size4KiB>> {
        Some(x86_64::structures::paging::PhysFrame::containing_address(x86_64::PhysAddr::new((PageFrame::new().index * PAGE_SIZE) as u64)))
    }
}
impl x86_64::structures::paging::FrameDeallocator<x86_64::structures::paging::Size4KiB> for BitmapPageFrameAllocator {
    unsafe fn deallocate_frame(&mut self, frame: x86_64::structures::paging::PhysFrame<x86_64::structures::paging::Size4KiB>) {
        PageFrame {index: (frame.start_address().as_u64() / 4096) as usize}.drop();
    }
}