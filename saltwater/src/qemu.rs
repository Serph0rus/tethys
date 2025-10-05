use crate::println;

const EXIT_CODE_PORT: u16 = 0xf4;
#[derive(Clone)]
pub enum ExitCode {
    Success = 0x10,
    Failure = 0x11,
}
pub fn exit(exit_code: ExitCode) {
    println!("exiting qemu with code {:x} ({})...", exit_code.clone() as usize, match exit_code {
        ExitCode::Success => "success",
        ExitCode::Failure => "failure",
    });
    unsafe {x86_64::instructions::port::PortWrite::write_to_port(EXIT_CODE_PORT, exit_code as u32)};
}