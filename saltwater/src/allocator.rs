use core::{alloc::GlobalAlloc, mem::MaybeUninit};
use spinning_top::Spinlock;
const BOOTSTRAP_HEAP_SIZE: usize = 0x1_000_000;
static mut BOOTSTRAP_HEAP: [MaybeUninit<u8>; BOOTSTRAP_HEAP_SIZE] =
    [MaybeUninit::new(0); BOOTSTRAP_HEAP_SIZE];
static BOOTSTRAP_ALLOCATOR: linked_list_allocator::LockedHeap =
    linked_list_allocator::LockedHeap::empty();
static ALLOCATOR: Spinlock<Option<linked_list_allocator::LockedHeap>> = Spinlock::new(None);
struct StubGlobalAllocator;
unsafe impl GlobalAlloc for StubGlobalAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        match ALLOCATOR.lock().as_ref() {
            Some(allocator) => unsafe { allocator.alloc(layout) },
            None => unsafe { BOOTSTRAP_ALLOCATOR.alloc(layout) },
        }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        if unsafe {
            (BOOTSTRAP_HEAP.as_ptr() as *const u8 <= ptr)
                & (BOOTSTRAP_HEAP.as_ptr().wrapping_add(BOOTSTRAP_HEAP_SIZE) as *const u8 > ptr)
        } {
            unsafe { BOOTSTRAP_ALLOCATOR.dealloc(ptr, layout) };
        } else {
            match ALLOCATOR.lock().as_ref() {
                Some(allocator) => unsafe { allocator.dealloc(ptr, layout) },
                None => panic!(
                    "Memory deallocation attempted on region not within bootstrap heap, before initialising system allocator!"
                ),
            }
        }
    }
}
pub fn bootstrap_initialise(_boot_info: &mut bootloader_api::BootInfo) {
    BOOTSTRAP_ALLOCATOR
        .lock()
        .init_from_slice(unsafe { &mut BOOTSTRAP_HEAP });
}
