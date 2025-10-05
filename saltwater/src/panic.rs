use core::panic::PanicInfo;
use crate::hcf::hcf;
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    hcf()
}