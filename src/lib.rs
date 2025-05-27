#![no_std]

use aarch64_cpu::asm;

pub fn spin() -> ! {
    loop {
        asm::wfe()
    }
}
