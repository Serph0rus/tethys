use alloc::vec::Vec;
use spinning_top::RwSpinlock;
static KERNEL_STACK_BOOLMAP: RwSpinlock<Vec<u8>> = RwSpinlock::new(Vec::new());
pub struct KernelStack(usize);
impl KernelStack {
    pub fn new() -> Option<KernelStack> {
        
    }
}