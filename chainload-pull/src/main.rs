#![no_main]
#![no_std]

use core::ffi;
use core::fmt::Write as _;
use core::mem;

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
use elf::endian::AnyEndian;
use kernel_core::device::bcm2837b0::gpio;
use kernel_core::device::bcm2837b0::mini;
use tock_registers::interfaces::Readable as _;
use tock_registers::interfaces::Writeable as _;

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
    b.ne .L_spin

    ldr x4, =__TEXT
    ldr x5, =__TEXT_LO
    ldr x6, =__TEXT_HI
.L_copy:
    ldp x7, x8, [x4], 16
    stp x7, x8, [x5], 16
    cmp x5, x6
    b.lo .L_copy
.L_start:
    ldr x4, =__STACK_HI
    mov sp, x4
    ldr x4, =_start_hypervisor
    br x4
.L_spin:
    wfe
    b .L_spin

.size _start, . - _start
.type _start, %function
.global _start
.popsection
",
}

unsafe extern "C" {
    static __TEXT: ffi::c_void;
    static __ELF: ffi::c_void;
}

#[unsafe(link_section = ".text.start")]
#[unsafe(no_mangle)]
pub extern "C" fn _start_hypervisor(
    device_tree: u64,
    reserved_1: u64,
    reserved_2: u64,
    reserved_3: u64,
    stack: u64,
) -> ! {
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
        1 => _start_kernel(device_tree, reserved_1, reserved_2, reserved_3),
        level => unreachable!("Unexpected exception level: {}", level),
    }

    SP_EL1.set(stack);

    unsafe {
        core::arch::asm! {
            "mov x0, {:x}",
            "mov x1, {:x}",
            "mov x2, {:x}",
            "mov x3, {:x}",
            in(reg) device_tree,
            in(reg) reserved_1,
            in(reg) reserved_2,
            in(reg) reserved_3,
        }
    }

    asm::eret()
}

#[unsafe(no_mangle)]
fn _start_kernel(device_tree: u64, reserved_1: u64, reserved_2: u64, reserved_3: u64) -> ! {
    unsafe { gpio::Gpio::new(0x3F20_0000).init() }
    let mut uart = unsafe { mini::Uart::new(0x3F21_5000) };
    uart.init();

    // Synchronize transmitter
    for byte in [0xff; 8] {
        uart.write_byte(byte);
    }

    // Synchronize receiver
    let mut len = 0;
    while len < 8 {
        len += (uart.read_byte() == 0xff) as usize;
    }

    let mut buffer = [0u8; 8];
    buffer.iter_mut().for_each(|byte| *byte = uart.read_byte());

    let len = u64::from_le_bytes(buffer) as usize;
    let base = unsafe { &__ELF as *const ffi::c_void as *const u8 as *mut u8 };

    for i in 0..len {
        let byte = uart.read_byte();
        unsafe {
            base.add(i).write_volatile(byte);
        }
    }

    writeln!(&mut uart, "[PULL] Received ELF file ({}B)", len).unwrap();

    let slice = unsafe { core::slice::from_raw_parts(base, len) };

    let elf = elf::ElfBytes::<AnyEndian>::minimal_parse(slice).unwrap();

    uart.flush();

    (unsafe { mem::transmute::<*mut u8, fn(u64, u64, u64, u64) -> !>(base) })(
        device_tree,
        reserved_1,
        reserved_2,
        reserved_3,
    )
}

#[panic_handler]
fn handle_panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
