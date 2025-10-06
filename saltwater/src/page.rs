use lazy_static::lazy_static;
use spinning_top::Spinlock;
use x86_64::structures::paging::{OffsetPageTable, PageTable};
use crate::mapping;
lazy_static! {
    pub static ref GLOBAL_PAGE_TABLE: Spinlock<PageTable> = Spinlock::new(unsafe {PageTable::new()});
}
pub fn initialise(boot_info: &mut bootloader_api::BootInfo) {
    x86_64::registers::control::Cr3::
    let mut offset_table = unsafe {x86_64::structures::paging::OffsetPageTable::new(&mut PageTable::new(), x86_64::VirtAddr::new(mapping::DIRECT_PHYSICAL))});
    offset_table.
}