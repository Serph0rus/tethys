use alloc::boxed::Box;
use spinning_top::Spinlock;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::{hcf::hcf, println};
pub const SYSCALL_IST_INDEX: usize = 0;
pub const INTERRUPT_IST_INDEX: usize = 1;
pub const DOUBLE_FAULT_IST_INDEX: usize = 2;
pub const CRITICAL_IST_INDEX: usize = 3;
static IDT_OPTION: Spinlock<Option<InterruptDescriptorTable>> = Spinlock::new(Some(InterruptDescriptorTable::new()));
static IDT_STATIC: Spinlock<Option<&'static InterruptDescriptorTable>> = Spinlock::new(None);
fn general_handler(_stack_frame: InterruptStackFrame, index: u8, _error_code: Option<u64>) {
    println!("interrupt 0x{:x} triggered!", index);
    hcf();
}
pub fn initialise(_boot_info: &mut bootloader_api::BootInfo) {
    let mut idt = IDT_OPTION.lock().take().expect("interrupt descriptor table not allocated before initialisation!");
    x86_64::set_general_handler!(&mut idt, general_handler);
    println!("set general handler in interrupt descriptor table...");
    let idt_static = Box::leak(Box::new(idt));
    idt_static.load();
    let _ = IDT_STATIC.lock().insert(idt_static);
    println!("loaded interrupt descriptor table...");
}