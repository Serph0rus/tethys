#![no_std]
#![no_main]
use core::{arch::asm, panic::PanicInfo};
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        unsafe {
            asm!("hlt")
        }
    }
}
static DEBUGCON_PORT: u16 = 0xe9;
pub extern "C" fn _start() -> ! {
    for byte in "hello from kickstart!".as_bytes() {
            unsafe {
                asm!("out dx, al", in("dx") DEBUGCON_PORT, in("al") *byte, options(nomem, nostack, preserves_flags));
            }
        };
    panic!()
}
