#![no_std]

pub mod device;

use core::fmt::Write;

use aarch64_cpu::asm;

pub fn spin() -> ! {
    loop {
        asm::wfe()
    }
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    Console.write_fmt(args).unwrap()
}

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

struct Console;

impl Write for Console {
    fn write_str(&mut self, string: &str) -> core::fmt::Result {
        const ADDRESS: *mut u8 = 0x3F20_1000 as _;

        for byte in string.as_bytes() {
            unsafe {
                ADDRESS.write_volatile(*byte);
            }
        }

        Ok(())
    }
}
