#![no_main]
#![no_std]

core::arch::global_asm! {
r"
.pushsection .text.boot

.macro ADR_REL register, symbol
    adrp \register, \symbol
    add \register, \register, :lo12:\symbol
.endmacro

_start:
    mrs x4, MPIDR_EL1
    and x4, x4, 0b11
    cmp x4, xzr
    b.ne .L_hang
    ldr x4, =__TEXT
    ldr x5, =__TEXT_LO
    ldr x6, =__TEXT_HI
.L_copy:
    ldp x7, x8, [x4], 16
    stp x7, x8, [x5], 16
    cmp x5, x6
    b.lo .L_copy
.L_rust:
    ldr x4, =__STACK_HI
    mov sp, x4
    ldr x4, =_start_rust
    br x4
.L_hang:
    wfe
    b .L_hang

.size _start, . - _start
.type _start, %function
.global _start
.popsection
",
}

#[unsafe(link_section = ".text.start")]
#[unsafe(no_mangle)]
pub extern "C" fn _start_rust(
    device_tree: u64,
    reserved_1: u64,
    reserved_2: u64,
    reserved_3: u64,
) -> ! {
    chainload_pull::main(device_tree, reserved_1, reserved_2, reserved_3)
}

#[panic_handler]
fn handle_panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
