use crate::{
    core::ProcessorData, kickstart::KICKSTART_ARC, proc::{Process, Status, Thread}
};
use alloc::{
    collections::linked_list::LinkedList,
    sync::{Arc, Weak},
};
use spinning_top::RwSpinlock;
pub fn initialise(_boot_info: &mut bootloader_api::BootInfo) {}
impl Process {
    fn branches(self: &Self) -> LinkedList<(Weak<RwSpinlock<Thread>>, usize)> {
        let mut threads = unsafe { self.threads.make_read_guard_unchecked() }
            .iter()
            .filter(
                |thread| match unsafe { thread.make_read_guard_unchecked() }.status {
                    Status::UserReady => true,
                    Status::KernelReady => true,
                    _ => false,
                },
            )
            .map(|arc| (Arc::downgrade(arc), 100))
            .collect::<LinkedList<(Weak<RwSpinlock<Thread>>, usize)>>();
        for child in unsafe { self.children.make_read_guard_unchecked() }.iter() {
            threads.append(&mut child.branches());
        }
        threads
    }
}
pub fn new_queue(processor: &ProcessorData) -> LinkedList<(Weak<RwSpinlock<Thread>>, usize)> {
    let kickstart_guard = unsafe { KICKSTART_ARC.make_read_guard_unchecked() };
    kickstart_guard
        .as_ref()
        .expect("kickstart process not initialised before long-term scheduler built ready queue!")
        .branches()
}
