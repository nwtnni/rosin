#![no_std]

use core::ffi;
use core::fmt::Write as _;
use core::mem;

mod gpio;
mod uart;

unsafe extern "C" {
    static __CHAINLOADER_INITIAL_LO: ffi::c_void;
}

pub fn main(device_tree: u32, reserved_1: u64, reserved_2: u64, reserved_3: u64) -> ! {
    unsafe { gpio::Gpio::new(0x3F20_0000).init() }
    let mut uart = unsafe { uart::Uart::new(0x3F21_5000) };
    uart.init();

    for byte in [0xff; 8] {
        uart.write_byte(byte);
    }

    let mut buffer = [0u8; 8];
    let mut len = 0;

    while len < 8 {
        len += (uart.read_byte() == 0xff) as usize;
    }

    buffer.iter_mut().for_each(|byte| *byte = uart.read_byte());

    let len = u64::from_le_bytes(buffer) as usize;
    let base = unsafe { &__CHAINLOADER_INITIAL_LO as *const ffi::c_void as *const u8 as *mut u8 };

    for i in 0..len {
        let byte = uart.read_byte();
        unsafe {
            base.add(i).write_volatile(byte);
        }
    }

    writeln!(
        &mut uart,
        "[PULL]: copied {} bytes, transferring control...",
        len
    )
    .unwrap();

    uart.flush();

    (unsafe { mem::transmute::<*mut u8, fn(u32, u64, u64, u64) -> !>(base) })(
        device_tree,
        reserved_1,
        reserved_2,
        reserved_3,
    )
}
