use x86_64::structures::paging::PageTable;
use crate::{core::ProcessorData, page::get_current_pml4, proc::{Process, Thread}};
/*impl ProcessorData {
    pub fn switch<'a>(self: &mut Self, process: &'a mut Process, thread: &'a mut Thread) {
        unsafe {&mut *get_current_pml4()}
    }
}*/