#![no_std]
#![no_main]

// https://devblogs.microsoft.com/oldnewthing/20220726-00/?p=106898
// https://dinfuehr.github.io/blog/encoding-of-immediate-values-on-aarch64/
// https://developer.arm.com/documentation/dui0774/i/armclang-Integrated-Assembler-Directives
// https://stackoverflow.com/questions/38570495/aarch64-relocation-prefixes/38608738#38608738
core::arch::global_asm! {
r"
.pushsection .text._start

# Explicit :pg_hi21: doesn't seem to be supported
# https://reviews.llvm.org/D64455
.macro ADR_REL register, symbol
    adrp \register, \symbol
    add \register, \register, :lo12:\symbol
.endmacro

_start:
    mrs x0, MPIDR_EL1
    and x0, x0, 0b11
    cmp x0, xzr
    b.ne .L_loop

    ADR_REL x0, __BSS_LO
    ADR_REL x1, __BSS_HI
.L_bss:
    cmp x0, x1
    b.eq .L_rust
    stp xzr, xzr, [x0], 16
    b .L_bss
.L_rust:
    ADR_REL x0, __STACK_HI
    mov sp, x0
    b _start_kernel
.L_loop:
    wfe
    b .L_loop

.size _start, . - _start
.type _start, %function
.global _start
.popsection
"
}

#[unsafe(no_mangle)]
fn _start_kernel() -> ! {
    rosin::println!("Hello, world!");
    rosin::spin()
}

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    rosin::spin()
}
