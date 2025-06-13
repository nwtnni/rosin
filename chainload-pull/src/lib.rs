#![no_std]

use core::ffi;
use core::fmt::Write as _;
use core::mem;

use elf::endian::AnyEndian;

mod gpio;
mod uart;

unsafe extern "C" {
    static __TEXT: ffi::c_void;
    static __ELF: ffi::c_void;
}

pub fn main(device_tree: u64, reserved_1: u64, reserved_2: u64, reserved_3: u64) -> ! {
    unsafe { gpio::Gpio::new(0x3F20_0000).init() }
    let mut uart = unsafe { uart::Uart::new(0x3F21_5000) };
    uart.init();

    // Synchronize transmitter
    for byte in [0xff; 8] {
        uart.write_byte(byte);
    }

    // Synchronize receiver
    let mut len = 0;
    while len < 8 {
        len += (uart.read_byte() == 0xff) as usize;
    }

    let mut buffer = [0u8; 8];
    buffer.iter_mut().for_each(|byte| *byte = uart.read_byte());

    let len = u64::from_le_bytes(buffer) as usize;
    let base = unsafe { &__ELF as *const ffi::c_void as *const u8 as *mut u8 };

    for i in 0..len {
        let byte = uart.read_byte();
        unsafe {
            base.add(i).write_volatile(byte);
        }
    }

    writeln!(&mut uart, "[PULL] Received ELF file ({}B)", len).unwrap();

    let slice = unsafe { core::slice::from_raw_parts(base, len) };

    let elf = elf::ElfBytes::<AnyEndian>::minimal_parse(slice).unwrap();

    uart.flush();

    (unsafe { mem::transmute::<*mut u8, fn(u64, u64, u64, u64) -> !>(base) })(
        device_tree,
        reserved_1,
        reserved_2,
        reserved_3,
    )
}
