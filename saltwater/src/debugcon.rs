use core::fmt::Write;
use alloc::fmt;
use spinning_top::Spinlock;
use crate::port;
struct DebugconWriter;
static DEBUGCON_PORT: u16 = 0xe9;
static DEBUGCON_WRITER: Spinlock<DebugconWriter> = Spinlock::new(DebugconWriter);
impl fmt::Write for DebugconWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.as_bytes() {
            unsafe {port::write_u8(DEBUGCON_PORT, *byte) };
        };
        Ok(())
    }
}
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
    let _ = DebugconWriter::write_fmt(&mut DEBUGCON_WRITER.lock(), format);
}
