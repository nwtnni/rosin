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
use rosin::device;
use rosin::fdt;
use tock_registers::interfaces::Readable as _;
use tock_registers::interfaces::Writeable as _;

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

#[unsafe(no_mangle)]
extern "C" fn _start_hypervisor(device_tree: u32, _x1: u64, _x2: u64, _x3: u64, stack: u64) -> ! {
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
        _ => _start_kernel(device_tree),
    }

    SP_EL1.set(stack);

    unsafe {
        core::arch::asm! {
            "mov w0, {:w}",
            in(reg) device_tree,
        }
    }

    asm::eret()
}

#[unsafe(no_mangle)]
fn _start_kernel(_device_tree: u32) -> ! {
    rosin::initialize();
    rosin::info!("Hello, world!");

    let device_tree = &device::bcm2837b0::DTB;

    rosin::info!("Device tree header: {:#x?}", device_tree.header());

    let mut indent = 0;
    for token in device_tree.iter() {
        match token {
            fdt::Token::Begin { name } => {
                rosin::info!("{:|<width$}{}", "", name, width = indent * 2);
                indent += 1;
            }
            fdt::Token::Prop(prop) => {
                rosin::info!("{:|<width$}-{:?}", "", prop, width = indent * 2)
            }
            fdt::Token::End => indent -= 1,
        }
    }

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
