#![no_std]
#![no_main]

// https://devblogs.microsoft.com/oldnewthing/20220726-00/?p=106898
// https://dinfuehr.github.io/blog/encoding-of-immediate-values-on-aarch64/
// https://developer.arm.com/documentation/dui0774/i/armclang-Integrated-Assembler-Directives
// https://stackoverflow.com/questions/38570495/aarch64-relocation-prefixes/38608738#38608738
core::arch::global_asm! {
"
.pushsection .text._start

_start:
    mrs x0, MPIDR_EL1
    and x0, x0, 0b11
    cmp x0, xzr
    b.ne .L_loop

    # Explicit :pg_hi21: doesn't seem to be supported
    # https://reviews.llvm.org/D64455
    adrp x0, __BSS_LO
    add x0, x0, :lo12:__BSS_LO

    adrp x1, __BSS_HI
    add x1, x1, :lo12:__BSS_HI
.L_bss:
    cmp x0, x1
    b.eq .L_rust
    stp xzr, xzr, [x0], 16
    b .L_bss
.L_rust:
    adr x0, __STACK_HI
    mov sp, x0

.L_loop:
    wfe
    b .L_loop

.size _start, . - _start
.type _start, %function
.global _start
.popsection
"
}

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        aarch64_cpu::asm::wfe();
    }
}
