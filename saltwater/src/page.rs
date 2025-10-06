use crate::mapping::{self, physical_to_virtual_address};
unsafe fn frame_as_page_table<'a>(
    frame: x86_64::structures::paging::PhysFrame,
) -> &'a mut x86_64::structures::paging::PageTable {
    unsafe {
        &mut *(physical_to_virtual_address(frame.start_address().as_u64())
            as *mut x86_64::structures::paging::PageTable)
    }
}
pub fn get_offset_table<'a>() -> x86_64::structures::paging::OffsetPageTable<'a> {
    unsafe {
        x86_64::structures::paging::OffsetPageTable::new(
            frame_as_page_table(x86_64::registers::control::Cr3::read().0),
            x86_64::VirtAddr::new(mapping::DIRECT_PHYSICAL),
        )
    }
}
