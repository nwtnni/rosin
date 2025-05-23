#![no_std]
#![no_main]

core::arch::global_asm! {
    "
        .pushsection .text._start

        _start:
        wfe
        b _start

        .global _start
        .size _start, . - _start
        .type _start, %function
        .popsection
    "
}

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unimplemented!()
}
