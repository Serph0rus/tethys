use core::arch::asm;
use crate::println;
pub fn hcf() -> ! {
    println!("exiting to halt-and-catch-fire loop...");
    loop {
        unsafe {
            asm!("hlt")
        }
    }
}