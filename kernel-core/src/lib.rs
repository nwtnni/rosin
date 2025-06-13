#![no_std]

#[macro_use]
pub mod print;

pub mod device;
pub mod irq;
pub mod mmu;
mod sync;
pub mod time;
mod unit;

use core::fmt::Debug;
use core::fmt::Write;
use core::panic::PanicInfo;

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
    // UART.lock().write_fmt(args).unwrap();
    // UART_MINI.lock().write_fmt(args).unwrap();
    unsafe { device::bcm2837b0::mini::Uart::new(0x3F21_5000) }
        .write_fmt(args)
        .unwrap();
}

pub fn init() {
    // unsafe {
    //     bcm2837b0::gpio::Gpio::new(0x3F20_0000).init();
    //     dev::bcm2837b0::mini::Uart::new(0x3F21_5000).init();
    // }

    // UART.lock().initialize();
    // UART_MINI.lock().init();

    // unsafe {
    //     // bcm2837b0::clock::Clock::new(0x4000_0000).init();
    //
    //     irq::init();
    // }
}

pub static UART: SpinLock<device::bcm2837b0::uart::Uart> =
    SpinLock::new(unsafe { device::bcm2837b0::uart::Uart::new(0x3F20_1000) });

pub static UART_MINI: SpinLock<device::bcm2837b0::mini::Uart> =
    SpinLock::new(unsafe { device::bcm2837b0::mini::Uart::new(0x3F21_5000) });

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

pub fn handle_panic(info: &PanicInfo) -> ! {
    let _ = writeln!(
        Console,
        "[PANIC][{}:{}] {}",
        info.location()
            .map(|location| location.file())
            .unwrap_or(""),
        info.location().map(|location| location.line()).unwrap_or(0),
        info.message(),
    );

    spin()
}
