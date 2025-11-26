use bootloader_api::BootInfo;
use x86_64::{PhysAddr, VirtAddr, registers::control::Cr3, structures::paging::{OffsetPageTable, PageTable, PageTableFlags, PhysFrame}};
use crate::{mapping::{self, physical_to_virtual_address}};
use lazy_static::lazy_static;
#[repr(C, packed)]
struct Entry(u64);
const ENTRY_PRESENT: u64 = 0;
const ENTRY_WRITABLE: u64 = 1;
const ENTRY_USER_ACCESSIBLE: u64 = 2;
const ENTRY_WRITE_THROUGH: u64 = 3;
const ENTRY_NO_CACHE: u64 = 4;
const ENTRY_ACCESSED: u64 = 5;
const ENTRY_DIRTY: u64 = 6;
const ENTRY_HUGE_PAGE: u64 = 7;
const ENTRY_GLOBAL: u64 = 8;
const ENTRY_NO_EXECUTE: u64 = 63;
lazy_static! {
    pub static ref KERNEL_PAGE_FLAGS: PageTableFlags =
          PageTableFlags::ACCESSED
        | PageTableFlags::WRITABLE
        | PageTableFlags::PRESENT;
}
impl Entry {
    pub fn new(
        present: bool,
        writable: bool,
        user_accessible: bool,
        write_through: bool,
        no_cache: bool,
        accessed: bool,
        dirty: bool,
        huge_page: bool,
        global: bool,
        no_execute: bool,
        frame_address: PhysAddr,
    ) -> Entry {
        Self(
            ((present as u64) << ENTRY_PRESENT)
                | ((writable as u64) << ENTRY_WRITABLE)
                | ((user_accessible as u64) << ENTRY_USER_ACCESSIBLE)
                | ((write_through as u64) << ENTRY_WRITE_THROUGH)
                | ((no_cache as u64) << ENTRY_NO_CACHE)
                | ((accessed as u64) << ENTRY_ACCESSED)
                | ((dirty as u64) << ENTRY_DIRTY)
                | ((huge_page as u64) << ENTRY_HUGE_PAGE)
                | ((global as u64) << ENTRY_GLOBAL)
                | ((no_execute as u64) << ENTRY_NO_EXECUTE)
                | (frame_address.as_u64() & 0x000f_ffff_ffff_f000) as u64,
        )
    }
    fn empty() -> Entry {
        Entry(0)
    }
    fn default_kernel(frame_address: PhysAddr) -> Entry {
        Entry::new(
            true,
            true,
            false,
            true,
            false,
            false,
            false,
            false,
            true,
            false,
            frame_address,
        )
    }
    fn default_user(frame_address: PhysAddr) -> Entry {
        Entry::new(
            true,
            true,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            frame_address,
        )
    }
}
pub fn get_current_pml4<'a>() -> *mut PageTable {
    physical_to_virtual_address(Cr3::read().0.start_address().as_u64()) as *mut PageTable
}
pub fn get_offset_table<'a>(table: &'a mut PageTable) -> OffsetPageTable<'a> {
    unsafe {OffsetPageTable::new(table, x86_64::VirtAddr::new(mapping::DIRECT_PHYSICAL))}
}
struct ManagedPageTable {
}
impl ManagedPageTable {
    fn map_to(self: &mut Self, from: PhysFrame, to: VirtAddr) {
    }
    fn flush(self: &mut Self) {

    }
}
pub fn initialise(_boot_info: &mut BootInfo) {
    
}