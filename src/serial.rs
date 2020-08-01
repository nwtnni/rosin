use core::fmt;
use core::fmt::Write as _;

use lazy_static::lazy_static;

lazy_static! {
    pub static ref SERIAL_1: spin::Mutex<uart::SerialPort> = {
        let mut port = unsafe { uart::SerialPort::new(0x3F8) };
        port.init();
        spin::Mutex::new(port)
    };
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    SERIAL_1.lock().write_fmt(args).expect("Failed to print to serial interface");
}

#[macro_export]
macro_rules! sprint {
    ($($arg:tt)*) => ($crate::serial::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! sprintln {
    () => ($crate::sprint!("\n"));
    ($($arg:tt)*) => ($crate::sprint!("{}\n", format_args!($($arg)*)));
}
