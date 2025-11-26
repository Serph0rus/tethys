use core::{sync::atomic::AtomicU64, u64};
use alloc::{collections::vec_deque::VecDeque, sync::Arc, vec::{self, Vec}};
use elf::{ElfBytes, endian::AnyEndian};
use spinning_top::RwSpinlock;
use x86_64::structures::paging::PageTable;
use crate::{println, proc::Process};
const KICKSTART_BYTES: &[u8] = if cfg!(debug_assertions) {
    include_bytes!("../../target/x86_64-unknown-none/debug/kickstart")
} else {
    include_bytes!("../../target/x86_64-unknown-none/release/kickstart")
};
pub static mut KICKSTART_PAGE_TABLE: PageTable = PageTable::new();
pub static mut KICKSTART_ARC: RwSpinlock<Option<Arc<Process>>> = RwSpinlock::new(None);
pub fn initialise(_boot_info: &mut bootloader_api::BootInfo) {
    println!("loading kickstart process from embedded elf...");
    let elf_bytes = ElfBytes::<AnyEndian>::minimal_parse(KICKSTART_BYTES)
        .unwrap_or_else(|error| panic!("failed to parse kickstart elf:\n{}", error));
    if elf_bytes.ehdr.abiversion != 1 {
        println!("incorrect kickstart elf abi version! expected 0x1, recieved: 0x{:x}!", elf_bytes.ehdr.abiversion);
    }
    if elf_bytes.ehdr.class != elf::file::Class::ELF64 {
        println!("incorrect kickstart elf class! expected ELF64, received: ELF32!");
    }
    let kickstart_proc = Arc::new(Process {
            set_priority: AtomicU64::new(u64::MAX),
            propagated_priority: u64::MAX,
            parent: None,
            pages: RwSpinlock::new(unsafe {&mut KICKSTART_PAGE_TABLE as *mut PageTable}),
            threads: RwSpinlock::new(Vec::new()),
            children: RwSpinlock::new(Vec::new()),
            requests: RwSpinlock::new(Vec::new()),
            responses: RwSpinlock::new(VecDeque::new()),
            servers: RwSpinlock::new(Vec::new()),
            descriptors: RwSpinlock::new(Vec::new()),
    });
    let thread_arc = kickstart_proc.add_thread();
}
