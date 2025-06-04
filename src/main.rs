#![no_std]
#![no_main]

use core::fmt::Write as _;

use aarch64_cpu::asm;
use aarch64_cpu::registers::CNTHCTL_EL2;
use aarch64_cpu::registers::CNTVOFF_EL2;
use aarch64_cpu::registers::CurrentEL;
use aarch64_cpu::registers::ELR_EL2;
use aarch64_cpu::registers::ELR_EL3;
use aarch64_cpu::registers::HCR_EL2;
use aarch64_cpu::registers::SCR_EL3;
use aarch64_cpu::registers::SP_EL1;
use aarch64_cpu::registers::SPSR_EL2;
use aarch64_cpu::registers::SPSR_EL3;
use tock_registers::interfaces::Readable as _;
use tock_registers::interfaces::Writeable as _;

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
    mov SP, x0
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

#[unsafe(no_mangle)]
extern "C" fn _start_hypervisor(stack: u64) -> ! {
    let level = CurrentEL.get();

    if level >= 3 {
        SCR_EL3.write(SCR_EL3::RW::NextELIsAarch64);
    }

    if level >= 2 {
        CNTHCTL_EL2.write(CNTHCTL_EL2::EL1PCEN::SET + CNTHCTL_EL2::EL1PCTEN::SET);
        CNTVOFF_EL2.set(0);
        HCR_EL2.write(HCR_EL2::RW::EL1IsAarch64);
    }

    #[allow(clippy::fn_to_numeric_cast)]
    match level {
        2 => {
            ELR_EL2.set(_start_kernel as u64);
            SPSR_EL2.write(
                SPSR_EL2::D::Masked
                    + SPSR_EL2::A::Masked
                    + SPSR_EL2::I::Masked
                    + SPSR_EL2::F::Masked
                    + SPSR_EL2::M::EL1h,
            );
        }
        3 => {
            ELR_EL3.set(_start_kernel as u64);
            SPSR_EL3.write(
                SPSR_EL3::D::Masked
                    + SPSR_EL3::A::Masked
                    + SPSR_EL3::I::Masked
                    + SPSR_EL3::F::Masked
                    + SPSR_EL3::M::EL1h,
            );
        }
        _ => _start_kernel(),
    }

    SP_EL1.set(stack);
    asm::eret()
}

#[unsafe(no_mangle)]
fn _start_kernel() -> ! {
    rosin::initialize();
    rosin::info!("Hello, world!");

    rosin::info!("Resolution: {}ns", rosin::time::resolution().as_nanos());

    for _ in 0..2 {
        rosin::info!("Sleeping for 1s...");
        rosin::time::spin(Duration::from_secs(1));
    }

    rosin::info!("Echo:");
    let mut buffer = [0u8];
    loop {
        rosin::UART.lock().read(&mut buffer).unwrap();
        rosin::print!(
            "{}",
            match buffer[0] {
                b'\r' => '\n',
                byte => byte as char,
            }
        );
    }
}

use core::panic::PanicInfo;
use core::time::Duration;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    let _ = writeln!(
        rosin::Console,
        "[PANIC][{}:{}] {}",
        _info
            .location()
            .map(|location| location.file())
            .unwrap_or(""),
        _info
            .location()
            .map(|location| location.line())
            .unwrap_or(0),
        _info.message(),
    );

    rosin::spin()
}
