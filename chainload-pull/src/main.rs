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
use kernel_core::mem::Phys;
use kernel_core::mem::Virt;
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
    _reserved_1: u64,
    _reserved_2: u64,
    _reserved_3: u64,
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
                + HCR_EL2::E2H::DisableOsAtEl2
                + HCR_EL2::VM::Disable,
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

    unsafe {
        core::arch::asm! {
            "mov x0, {:x}",
            "msr SP_EL1, {:x}",
            "eret",
            in(reg) stack,
            in(reg) device_tree,
            options(noreturn, nomem)
        }
    }
}

#[unsafe(no_mangle)]
fn _start_kernel(device_tree: u64) -> ! {
    unsafe { gpio::Gpio::new(0x3F20_0000).init() }
    let mut uart = unsafe { mini::Uart::new(0x3F21_5000) };
    uart.init();

    // Synchronize receiver
    let mut len = 0;
    while len < 8 {
        match uart.read_byte() {
            0xff => len += 1,
            _ => len = 0,
        }
    }

    // Synchronize transmitter
    for byte in [0xff; 8] {
        uart.write_byte(byte);
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

    writeln!(
        &mut uart,
        "[PULL] Copied kernel to {:#x?} ({:?})",
        base,
        kernel_core::unit::Byte::new(len)
    )
    .unwrap();

    let slice = unsafe { core::slice::from_raw_parts(base, len) };

    let elf = elf::ElfBytes::<AnyEndian>::minimal_parse(slice).unwrap();
    let segments = elf.segments().unwrap();

    let heap = segments
        .iter()
        .filter(|segment| segment.p_type == elf::abi::PT_LOAD)
        .map(|segment| segment.p_paddr + segment.p_memsz)
        .map(|address| address.next_multiple_of(1 << 16))
        .max()
        .unwrap();

    let offset = segments
        .iter()
        .find(|segment| segment.p_type == elf::abi::PT_LOAD)
        .map(|segment| segment.p_vaddr - segment.p_paddr)
        .unwrap();

    let page_table_hi = unsafe {
        (heap as *mut kernel_core::mmu::PageTable<kernel_core::mem::Kernel>)
            .as_mut()
            .unwrap()
    };

    let page_table_lo = unsafe {
        (heap as *mut kernel_core::mmu::PageTable<kernel_core::mem::User>)
            .byte_add(
                mem::size_of::<kernel_core::mmu::PageTable<kernel_core::mem::Kernel>>()
                    .next_multiple_of(1 << 16),
            )
            .as_mut()
            .unwrap()
    };

    kernel_core::mmu::init();

    writeln!(
        &mut uart,
        "[PULL] Initializing page tables at {:#x?} for kernel (offset {:#x}) and {:#x?} for identity",
        page_table_hi as *mut _, offset, page_table_lo as *mut _
    )
    .unwrap();

    page_table_hi.init(offset);
    page_table_lo.init(0);

    for segment in segments {
        if segment.p_type != elf::abi::PT_LOAD {
            continue;
        }

        let read = segment.p_flags & elf::abi::PF_R > 0;
        let write = segment.p_flags & elf::abi::PF_W > 0;
        let execute = segment.p_flags & elf::abi::PF_X > 0;
        let data = elf.segment_data(&segment).unwrap().as_ptr();

        writeln!(
            &mut uart,
            "[PULL] Copying {:?} to {:?} ({}{}{}, {:#x} file / {:#x} mem)",
            data as *mut u8,
            segment.p_paddr as *mut u8,
            if read { "R" } else { "" },
            if write { "W" } else { "" },
            if execute { "X" } else { "" },
            kernel_core::unit::Byte::new(segment.p_filesz as usize),
            kernel_core::unit::Byte::new(segment.p_memsz as usize),
        )
        .unwrap();

        unsafe {
            core::ptr::copy_nonoverlapping(
                data,
                segment.p_paddr as *mut u8,
                segment.p_filesz as usize,
            );

            core::ptr::write_bytes(
                (segment.p_paddr as *mut u8).byte_add(segment.p_filesz as usize),
                0,
                (segment.p_memsz - segment.p_filesz) as usize,
            )
        }

        for offset in (0..segment.p_memsz.next_multiple_of(1 << 16)).step_by(1 << 16) {
            let virt_hi = Virt::<kernel_core::mem::Kernel>::new(segment.p_vaddr + offset);
            let virt_lo = Virt::<kernel_core::mem::User>::new(segment.p_paddr + offset);
            let phys = Phys::new(segment.p_paddr + offset);

            page_table_hi.map(
                virt_hi,
                phys,
                kernel_core::mmu::Attr::Normal {
                    read,
                    write,
                    execute,
                },
            );

            page_table_lo.map(
                virt_lo,
                phys,
                kernel_core::mmu::Attr::Normal {
                    read,
                    write,
                    execute,
                },
            );
        }
    }

    writeln!(
        &mut uart,
        "[PULL] Jumping to entry {:#x}",
        elf.ehdr.e_entry - offset,
    )
    .unwrap();

    uart.flush();

    unsafe {
        core::arch::asm! {
            "mov x0, {arg_0:x}",
            "mov x1, {arg_1:x}",
            "br {entry:x}",
            arg_0 = in(reg) device_tree,
            arg_1 = in(reg) page_table_hi as *mut _ as u64 + offset,
            entry = in(reg) elf.ehdr.e_entry - offset,
            options(nomem, noreturn)
        }
    }
}

#[panic_handler]
fn handle_panic(info: &core::panic::PanicInfo) -> ! {
    writeln!(
        &mut kernel_core::Console,
        "[PANIC][{}] {}",
        info.location().unwrap(),
        info.message()
    )
    .unwrap();

    loop {
        aarch64_cpu::asm::wfe();
    }
}
