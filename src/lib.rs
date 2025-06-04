#![no_std]

pub mod device;
pub mod mmu;
pub mod print;
mod sync;
pub mod time;

use core::fmt::Debug;
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

pub type Result<T> = core::result::Result<T, Error>;

pub enum Error {
    Todo,
}

impl Debug for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::Todo => "TODO".fmt(fmt),
        }
    }
}

#[allow(dead_code)]
pub struct Console;

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
