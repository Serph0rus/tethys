use crate::{hcf::hcf, println};
use core::panic::PanicInfo;
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("system panicking...\n{}", info.message());
    hcf()
}
