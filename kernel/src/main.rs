#![no_std]
#![no_main]

use core::fmt::Write as _;

use aarch64_cpu::asm;
use aarch64_cpu::registers::CNTHCTL_EL2;
use aarch64_cpu::registers::CurrentEL;
use aarch64_cpu::registers::ELR_EL2;
use aarch64_cpu::registers::ELR_EL3;
use aarch64_cpu::registers::HCR_EL2;
use aarch64_cpu::registers::SCR_EL3;
use aarch64_cpu::registers::SP_EL1;
use aarch64_cpu::registers::SPSR_EL2;
use aarch64_cpu::registers::SPSR_EL3;
use kernel::dev;
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
    let level = CurrentEL.read(CurrentEL::EL);

    if level >= 3 {
        SCR_EL3.write(
            SCR_EL3::RW::NextELIsAarch64
                + SCR_EL3::EA::NotTaken
                + SCR_EL3::FIQ::NotTaken
                + SCR_EL3::IRQ::NotTaken,
        );
    }

    if level >= 2 {
        CNTHCTL_EL2.write(CNTHCTL_EL2::EL1PCEN::SET + CNTHCTL_EL2::EL1PCTEN::SET);
        HCR_EL2.write(
            HCR_EL2::RW::EL1IsAarch64
                + HCR_EL2::E2H::DisableOsAtEl2
                + HCR_EL2::AMO::CLEAR
                + HCR_EL2::IMO::DisableVirtualIRQ
                + HCR_EL2::FMO::DisableVirtualFIQ
                + HCR_EL2::TGE::DisableTrapGeneralExceptionsToEl2
                + HCR_EL2::E2H::DisableOsAtEl2,
        );
    }

    #[allow(clippy::fn_to_numeric_cast)]
    match level {
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
        1 => _start_kernel(device_tree),
        level => unreachable!("Unexpected exception level: {}", level),
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
    kernel::initialize();
    kernel::info!("Hello, world!");
    kernel::info!(
        "Resolution: {}ns, frequency: {}hz",
        Duration::from(kernel::time::Cycle::ONE).as_nanos(),
        kernel::time::frequency(),
    );

    kernel::info!(
        "Interrupt in 1 second... EL: {}",
        CurrentEL.read(CurrentEL::EL)
    );

    kernel::irq::enable();
    kernel::irq::enable_timer(Duration::from_secs(1));

    let device_tree = &dev::bcm2837b0::DTB;

    kernel::info!("Device tree header: {:#x?}", device_tree.header());

    let mut indent = 0;
    for token in device_tree.iter() {
        match token {
            device_tree::blob::Token::Begin { name } => {
                kernel::info!("{:|<width$}{}", "", name, width = indent * 2);
                indent += 1;
            }
            device_tree::blob::Token::Prop(prop) => {
                kernel::info!("{:|<width$}-{:?}", "", prop, width = indent * 2)
            }
            device_tree::blob::Token::End => indent -= 1,
        }
    }

    for _ in 0..2 {
        kernel::info!("Sleeping for 1s...");
        kernel::time::spin(Duration::from_secs(1));
    }

    kernel::info!("Echo:");
    let mut buffer = [0u8];
    loop {
        kernel::UART.lock().read(&mut buffer).unwrap();
        kernel::print!(
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
        kernel::Console,
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

    kernel::spin()
}
