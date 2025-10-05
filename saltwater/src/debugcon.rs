use core::fmt;
use spinning_top::Spinlock;
static DEBUGCON_LOCK: Spinlock<()> = Spinlock::new(());
static DEBUGCON_PORT: u16 = 0xe9;
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::debugcon::_print(format_args!($($arg)*)));
}
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
#[doc(hidden)]
pub fn _print(format: fmt::Arguments) {
    let _debugcon_port_guard = DEBUGCON_LOCK.lock();
    match format.as_str() {
        Some(as_str) => for byte in as_str.as_bytes() {
            unsafe {x86_64::instructions::port::PortWrite::write_to_port(DEBUGCON_PORT, *byte)};
        },
        None => (),
    }
}
