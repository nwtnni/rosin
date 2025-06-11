#![no_std]
#![no_main]

use core::panic::PanicInfo;

// Avoid clobbering DTB and next three reserved arguments
// - https://github.com/raspberrypi/tools/blob/439b6198a9b340de5998dd14a26a0d9d38a6bcac/armstubs/armstub8.S#L163-L171
//
// https://devblogs.microsoft.com/oldnewthing/20220726-00/?p=106898
// https://dinfuehr.github.io/blog/encoding-of-immediate-values-on-aarch64/
// https://developer.arm.com/documentation/dui0774/i/armclang-Integrated-Assembler-Directives
// https://stackoverflow.com/questions/38570495/aarch64-relocation-prefixes/38608738#38608738
core::arch::global_asm! {
r"
.pushsection .text.boot

# Explicit :pg_hi21: doesn't seem to be supported
# https://reviews.llvm.org/D64455
.macro ADR_REL register, symbol
    adrp \register, \symbol
    add \register, \register, :lo12:\symbol
.endmacro

_start:
    mrs x4, MPIDR_EL1
    and x4, x4, 0b11
    cmp x4, xzr
    b.ne .L_loop

    ADR_REL x4, __BSS_LO
    ADR_REL x5, __BSS_HI
.L_bss:
    cmp x4, x5
    b.eq .L_rust
    stp xzr, xzr, [x4], 16
    b .L_bss
.L_rust:
    ADR_REL x4, __STACK_HI
    mov SP, x4
    b _start_hypervisor
.L_loop:
    wfe
    b .L_loop

.size _start, . - _start
.type _start, %function
.global _start
.popsection
"
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kernel::handle_panic(info)
}
