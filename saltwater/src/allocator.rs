use crate::println;
use core::{alloc::GlobalAlloc, mem::MaybeUninit};
use spinning_top::Spinlock;
const BOOTSTRAP_HEAP_SIZE: usize = 0x100_000;
static mut BOOTSTRAP_HEAP: [MaybeUninit<u8>; BOOTSTRAP_HEAP_SIZE] =
    [MaybeUninit::uninit(); BOOTSTRAP_HEAP_SIZE];
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
        if (&raw const BOOTSTRAP_HEAP as *const u8 <= ptr)
            & ((&raw const BOOTSTRAP_HEAP as *const u8).wrapping_add(BOOTSTRAP_HEAP_SIZE)
                as *const u8
                > ptr)
        {
            unsafe { BOOTSTRAP_ALLOCATOR.dealloc(ptr, layout) };
        } else {
            match ALLOCATOR.lock().as_ref() {
                Some(allocator) => unsafe { allocator.dealloc(ptr, layout) },
                None => panic!(
                    "memory deallocation attempted on region not within bootstrap heap, before initialising system allocator!"
                ),
            }
        }
    }
}
#[global_allocator]
static STUB_GLOBAL_ALLOCATOR: StubGlobalAllocator = StubGlobalAllocator;
pub fn bootstrap_initialise(_boot_info: &mut bootloader_api::BootInfo) {
    unsafe {
        BOOTSTRAP_ALLOCATOR
            .lock()
            .init(&raw mut BOOTSTRAP_HEAP as *mut u8, BOOTSTRAP_HEAP_SIZE)
    };
    println!("initialised bootstrap kernel global allocator...");
}
