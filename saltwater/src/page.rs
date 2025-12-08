use crate::{
    frame::PAGE_FRAME_ALLOCATOR,
    mapping::{self, physical_to_virtual_address},
};
use lazy_static::lazy_static;
use x86_64::{
    PhysAddr,
    registers::control::Cr3,
    structures::paging::{
        FrameAllocator, FrameDeallocator, OffsetPageTable, PageTable, PageTableFlags,
        PhysFrame, Size4KiB,
    },
};
lazy_static! {
    pub static ref KERNEL_PAGE_FLAGS: PageTableFlags = PageTableFlags::GLOBAL
        | PageTableFlags::ACCESSED
        | PageTableFlags::WRITE_THROUGH
        | PageTableFlags::WRITABLE
        | PageTableFlags::PRESENT;
    pub static ref USER_PAGE_FLAGS: PageTableFlags = PageTableFlags::ACCESSED
        | PageTableFlags::USER_ACCESSIBLE
        | PageTableFlags::WRITABLE
        | PageTableFlags::PRESENT;
}
pub fn get_current_pml4<'a>() -> *mut PageTable {
    physical_to_virtual_address(Cr3::read().0.start_address().as_u64()) as *mut PageTable
}
pub fn get_offset_table<'a>(table: &'a mut PageTable) -> OffsetPageTable<'a> {
    unsafe { OffsetPageTable::new(table, x86_64::VirtAddr::new(mapping::DIRECT_PHYSICAL)) }
}
pub struct ManagedPageTable(*mut PageTable);
impl ManagedPageTable {
    pub fn new() -> ManagedPageTable {
        let mut pfa = PAGE_FRAME_ALLOCATOR.lock();
        ManagedPageTable(physical_to_virtual_address(pfa.as_mut().expect("page frame allocator not initialised before managed page table initialisation!").allocate_frame().expect("failed to allocate frame during managed page table initialisation!").start_address().as_u64()) as *mut PageTable)
    }
}
impl Drop for ManagedPageTable {
    fn drop(&mut self) {
        let mut pfa_guard = PAGE_FRAME_ALLOCATOR.lock();
        let pfa = pfa_guard
            .as_mut()
            .expect("page frame allocator not initialised before managed page table dropping!");
        fn free_page_table_level(
            table_frame: PhysFrame<Size4KiB>,
            level: u8,
            pfa: &mut impl FrameDeallocator<Size4KiB>,
        ) {
            let table_virt = physical_to_virtual_address(table_frame.start_address().as_u64());
            let table = unsafe { &mut *(table_virt as *mut PageTable) };
            for entry in table.iter_mut() {
                if !entry.flags().contains(PageTableFlags::PRESENT) {
                    continue;
                }
                match level {
                    3 => unreachable!(
                        "attempted to call free_page_table_level helper function on to-level pml4 during page table dropping!"
                    ),
                    2 => {
                        let pml2_frame = PhysFrame::containing_address(entry.addr());
                        free_page_table_level(pml2_frame, 1, pfa);
                        entry.set_unused();
                    }
                    1 => {
                        let pml1_frame = PhysFrame::containing_address(entry.addr());
                        free_page_table_level(pml1_frame, 0, pfa);
                        entry.set_unused();
                    }
                    0 => {
                        entry.set_unused();
                    }
                    _ => unreachable!(
                        "attempted to call free_page_table_level helper function on invalid page table level!"
                    ),
                }
            }
            unsafe { pfa.deallocate_frame(table_frame) };
        }
        let pml4_frame =
            PhysFrame::containing_address(PhysAddr::new(self.0 as u64 - mapping::DIRECT_PHYSICAL));
        for i in 0..256 {
            let entry = &mut unsafe { &mut *self.0 }[i];
            if entry.flags().contains(PageTableFlags::PRESENT) {
                let pdpt_frame = PhysFrame::containing_address(entry.addr());
                free_page_table_level(pdpt_frame, 2, pfa);
                entry.set_unused();
            }
        }
        unsafe { pfa.deallocate_frame(pml4_frame) };
    }
}
unsafe impl Send for ManagedPageTable {}
unsafe impl Sync for ManagedPageTable {}
