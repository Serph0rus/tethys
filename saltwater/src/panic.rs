use crate::{hcf::hcf, println, qemu};
use core::panic::PanicInfo;
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("system panicking...\n{}", info);
    qemu::exit(qemu::ExitCode::Failure);
    hcf()
}
