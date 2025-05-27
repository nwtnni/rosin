#![no_std]

pub mod device;
mod sync;

use core::fmt::Write;

use aarch64_cpu::asm;
use sync::SpinLock;

#[inline]
pub fn pause() {
    asm::nop()
}

#[inline]
pub fn spin() -> ! {
    loop {
        asm::wfe()
    }
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    UART.lock().write_fmt(args).unwrap();
}

pub fn initialize() {
    UART.lock().initialize();
}

pub static UART: SpinLock<device::bcm2837b0::uart::Uart> =
    SpinLock::new(unsafe { device::bcm2837b0::uart::Uart::new(0x3F20_1000) });

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::_print(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => {
        {
            $crate::_print(format_args!($($arg)*));
            $crate::print!("\n");
        }
    };
}

#[allow(dead_code)]
struct Console;

impl Write for Console {
    fn write_str(&mut self, string: &str) -> core::fmt::Result {
        const ADDRESS: *mut u8 = 0x3F20_1000 as _;

        for byte in string.bytes() {
            unsafe {
                ADDRESS.write_volatile(byte);
            }
        }

        Ok(())
    }
}
