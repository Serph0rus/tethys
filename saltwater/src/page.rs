use crate::{
    mapping::{self, physical_to_virtual_address},
    println,
};
use alloc::string::String;
use x86_64::structures::paging::{Mapper, PageTableFlags};
unsafe fn frame_as_page_table<'a>(
    frame: x86_64::structures::paging::PhysFrame,
) -> &'a mut x86_64::structures::paging::PageTable {
    unsafe {
        &mut *(physical_to_virtual_address(
            x86_64::registers::control::Cr3::read()
                .0
                .start_address()
                .as_u64(),
        ) as *mut x86_64::structures::paging::PageTable)
    }
}
fn get_offset_table<'a>() -> x86_64::structures::paging::OffsetPageTable<'a> {
    unsafe {
        x86_64::structures::paging::OffsetPageTable::new(
            frame_as_page_table(x86_64::registers::control::Cr3::read().0),
            x86_64::VirtAddr::new(mapping::DIRECT_PHYSICAL),
        )
    }
}
fn stringify_page_table_flags(flags: x86_64::structures::paging::PageTableFlags) -> String {
    String::new()
        + if flags.contains(PageTableFlags::PRESENT) {
            "present, "
        } else {
            ""
        }
        + if flags.contains(PageTableFlags::WRITABLE) {
            "writable, "
        } else {
            ""
        }
        + if flags.contains(PageTableFlags::USER_ACCESSIBLE) {
            "user-accessible, "
        } else {
            ""
        }
        + if flags.contains(PageTableFlags::WRITE_THROUGH) {
            "write-through, "
        } else {
            ""
        }
        + if flags.contains(PageTableFlags::NO_CACHE) {
            "no-cache, "
        } else {
            ""
        }
        + if flags.contains(PageTableFlags::ACCESSED) {
            "accessed, "
        } else {
            ""
        }
        + if flags.contains(PageTableFlags::DIRTY) {
            "dirty, "
        } else {
            ""
        }
        + if flags.contains(PageTableFlags::HUGE_PAGE) {
            "huge, "
        } else {
            ""
        }
        + if flags.contains(PageTableFlags::GLOBAL) {
            "global, "
        } else {
            ""
        }
        + if flags.contains(PageTableFlags::NO_EXECUTE) {
            "no-execute, "
        } else {
            ""
        }
}
pub fn initialise(boot_info: &mut bootloader_api::BootInfo) {
    let mut offset_table = get_offset_table();
    println!("acquired bootstrap page table with entries:");
    
    for i in 0..3 {

    }
    for (pml4_index, pml4_entry) in offset_table.level_4_table().iter().enumerate() {
        if !pml4_entry.flags().is_empty() {
            println!(
                "no. {} (0x{:x}): {}",
                pml4_index,
                pml4_entry.addr().as_u64(),
                stringify_page_table_flags(pml4_entry.flags())
            );
            match pml4_entry.frame() {
                Ok(pml4_frame) => {
                    for (pml3_index, pml3_entry) in (unsafe { frame_as_page_table(pml4_frame) })
                        .iter()
                        .enumerate()
                    {
                        if !pml3_entry.flags().is_empty() {
                            println!(
                                "   no. {} (0x{:x}): {}",
                                pml3_index,
                                pml3_entry.addr().as_u64(),
                                stringify_page_table_flags(pml3_entry.flags())
                            );
                            match pml3_entry.frame() {
                                Ok(pml3_frame) => {
                                    for (pml2_index, pml2_entry) in
                                        (unsafe { frame_as_page_table(pml3_frame) })
                                            .iter()
                                            .enumerate()
                                    {
                                        if !pml2_entry.flags().is_empty() {
                                            println!(
                                                "       no. {} (0x{:x}): {}",
                                                pml2_index,
                                                pml2_entry.addr().as_u64(),
                                                stringify_page_table_flags(pml2_entry.flags())
                                            );
                                            match pml2_entry.frame() {
                                                Ok(pml2_frame) => {
                                                    for (pml1_index, pml1_entry) in
                                                        (unsafe { frame_as_page_table(pml2_frame) })
                                                            .iter()
                                                            .enumerate()
                                                    {
                                                        if !pml1_entry.flags().is_empty() {
                                                            println!(
                                                                "           no. {} (0x{:x}): {}",
                                                                pml1_index,
                                                                pml1_entry.addr().as_u64(),
                                                                stringify_page_table_flags(
                                                                    pml1_entry.flags()
                                                                )
                                                            );
                                                        }
                                                    }
                                                }
                                                Err(..) => (),
                                            }
                                        }
                                    }
                                }
                                Err(..) => (),
                            }
                        }
                    }
                }
                Err(..) => (),
            }
        }
    }
}
