use core::arch::global_asm;
use core::time::Duration;

use aarch64_cpu::registers::CNTP_CTL_EL0;
use aarch64_cpu::registers::CNTP_TVAL_EL0;
use aarch64_cpu::registers::DAIF;
use aarch64_cpu::registers::ReadWriteable as _;
use aarch64_cpu::registers::VBAR_EL1;
use tock_registers::interfaces::Writeable as _;

use crate::time;

global_asm! {
r#"
.pushsection .text

.macro PUSH x, y
    stp \x, \y, [sp, -16]!
.endmacro

.macro IRQ_ENTER
    PUSH x0, x1
    PUSH x2, x3
    PUSH x4, x5
    PUSH x6, x7
    PUSH x8, x9
    PUSH x10, x11
    PUSH x12, x13
    PUSH x14, x15
    PUSH x16, x17
    PUSH x18, x19
    PUSH x20, x21
    PUSH x22, x23
    PUSH x24, x25
    PUSH x26, x27
    PUSH x28, x29
    str x30, [sp, -16]!
.endmacro

.macro POP x, y
    ldp \x, \y, [sp], 16
.endmacro

.macro IRQ_LEAVE
    ldr x30, [sp], 16
    POP x28, x29
    POP x26, x27
    POP x24, x25
    POP x22, x23
    POP x20, x21
    POP x18, x19
    POP x16, x17
    POP x14, x15
    POP x12, x13
    POP x10, x11
    POP x8, x9
    POP x6, x7
    POP x4, x5
    POP x2, x3
    POP x0, x1
    eret
.endmacro

.macro VECTOR label
    .align 7
    b \label
.endmacro

__VECTOR_TABLE:
    VECTOR irq_invalid
    VECTOR irq_el1t
    VECTOR irq_invalid
    VECTOR irq_invalid

    VECTOR irq_invalid
    VECTOR irq_invalid
    VECTOR irq_invalid
    VECTOR irq_invalid

    VECTOR irq_invalid
    VECTOR irq_invalid
    VECTOR irq_invalid
    VECTOR irq_invalid

    VECTOR irq_invalid
    VECTOR irq_invalid
    VECTOR irq_invalid
    VECTOR irq_invalid

irq_el1t:
    IRQ_ENTER
    bl handle_irq_el1t
    IRQ_LEAVE

irq_invalid:

irq_hang:
    wfe
    b irq_invalid

.global __VECTOR_TABLE
.size __VECTOR_TABLE, . - __VECTOR_TABLE

.popsection
"#
}

unsafe extern "C" {
    static __VECTOR_TABLE: u64;
}

pub unsafe fn init() {
    VBAR_EL1.set(unsafe { __VECTOR_TABLE });
}

pub fn enable() {
    DAIF.write(DAIF::D::Unmasked + DAIF::A::Unmasked + DAIF::I::Unmasked + DAIF::F::Unmasked)
}

pub fn enable_timer(duration: Duration) {
    CNTP_TVAL_EL0.set(time::Cycle::from(duration).value());
    CNTP_CTL_EL0.modify(CNTP_CTL_EL0::ENABLE::SET + CNTP_CTL_EL0::IMASK::CLEAR);
}

#[unsafe(no_mangle)]
pub extern "C" fn handle_irq_el1t() {
    info!("here",);
}
