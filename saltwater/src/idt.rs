#![feature(abi_x86_interrupt)]
static INTERRUPT_DESCRIPTOR_TABLE: x86_64::structures::idt::InterruptDescriptorTable = {
    let mut idt = x86_64::structures::idt::InterruptDescriptorTable::new();
    x86_64::set_general_handler!(&mut idt, interrupt_handler)
};
extern "x86-interrupt" fn interrupt_handler(stack_frame: x86_64::structures::idt::InterruptStackFrame) {

}