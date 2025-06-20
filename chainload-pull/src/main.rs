#![no_main]
#![no_std]

use core::ffi;
use core::fmt::Write as _;
use core::mem;
use core::ptr::NonNull;

use aarch64_cpu::registers::CNTHCTL_EL2;
use aarch64_cpu::registers::CurrentEL;
use aarch64_cpu::registers::ELR_EL2;
use aarch64_cpu::registers::ELR_EL3;
use aarch64_cpu::registers::HCR_EL2;
use aarch64_cpu::registers::SCR_EL3;
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

#[unsafe(link_section = ".text.start")]
#[unsafe(no_mangle)]
pub extern "C" fn _start_hypervisor(
    _device_tree: u64,
    _reserved_1: u64,
    _reserved_2: u64,
    _reserved_3: u64,
    stack: u64,
) -> ! {
    let level = CurrentEL.read(CurrentEL::EL);

    #[cfg(feature = "qemu")]
    let _device_tree = include_bytes!("bcm2710-rpi-3-b-plus.dtb").as_ptr() as u64;

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
        1 => _start_kernel(_device_tree),
        level => unreachable!("Unexpected exception level: {}", level),
    }

    unsafe {
        core::arch::asm! {
            "mov x0, {:x}",
            "msr SP_EL1, {:x}",
            "eret",
            in(reg) _device_tree,
            in(reg) stack,
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
    let base = (1 << 29) as *mut u8;

    for i in 0..len {
        let byte = uart.read_byte();
        unsafe {
            base.add(i).write_volatile(byte);
        }
    }

    writeln!(
        &mut uart,
        "[PULL] Wrote ELF file ({:#x?}) at {:#x?}",
        kernel_core::unit::Byte::new(len),
        base,
    )
    .unwrap();

    let elf = elf::ElfBytes::<AnyEndian>::minimal_parse(unsafe {
        core::slice::from_raw_parts(base, len)
    })
    .unwrap();

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

    let page_table_len =
        mem::size_of::<kernel_core::mmu::PageTable<kernel_core::mem::Kernel>>() as u64;
    let page_table_kernel = unsafe {
        (heap as *mut kernel_core::mmu::PageTable<kernel_core::mem::Kernel>)
            .as_mut()
            .unwrap()
    };

    let allocator_len = mem::size_of::<kernel_core::mem::page::Allocator>() as u64;
    let allocator = unsafe {
        ((heap + page_table_len) as *mut kernel_core::mem::page::Allocator)
            .as_mut()
            .unwrap()
    };

    let device_tree_len =
        unsafe { device_tree::Blob::from_ptr(NonNull::new(device_tree as *mut u8).unwrap()) }
            .header()
            .len();
    let device_tree_src = device_tree;
    let device_tree_dst = (heap + page_table_len + allocator_len).next_multiple_of(1 << 16);

    let page_table_identity = unsafe {
        ((device_tree_dst + device_tree_len as u64).next_multiple_of(1 << 16)
            as *mut kernel_core::mmu::PageTable<kernel_core::mem::User>)
            .as_mut()
            .unwrap()
    };

    writeln!(
        &mut uart,
        "[PULL] Relocating device tree blob ({:#x?}) from {:#x} to {:#x}",
        kernel_core::unit::Byte::new(device_tree_len),
        device_tree_src,
        device_tree_dst,
    )
    .unwrap();

    unsafe {
        core::ptr::copy(
            device_tree_src as *const u8,
            device_tree_dst as *mut u8,
            device_tree_len,
        )
    };

    writeln!(
        &mut uart,
        "[PULL] Initializing page allocator at {:#x?}",
        allocator as *mut _,
    )
    .unwrap();

    allocator.clear_mut();
    reserve(allocator, allocator as *const _);
    reserve_unsized(allocator, device_tree_dst, device_tree_len as u64);

    kernel_core::mmu::init();

    writeln!(
        &mut uart,
        "[PULL] Initializing kernel page table at {:#x?} (offset {:#x})",
        page_table_kernel as *mut _, offset
    )
    .unwrap();

    page_table_kernel.init(offset);
    reserve(allocator, page_table_kernel);

    writeln!(
        &mut uart,
        "[PULL] Initializing identity page table at {:#x?} (offset {:#x})",
        page_table_identity as *mut _, 0
    )
    .unwrap();

    page_table_identity.init(0);
    // NOTE: identity table is used only for bootstrapping and can be clobbered
    // Do not need to reserve in page allocator

    // Map device tree
    page_table_kernel.map(
        Virt::new(device_tree_dst + offset),
        Phys::new(device_tree_dst),
        kernel_core::mmu::Attr::Normal {
            read: true,
            write: false,
            execute: false,
        },
    );

    // Load kernel binary
    for segment in segments {
        if segment.p_type != elf::abi::PT_LOAD {
            continue;
        }

        let read = segment.p_flags & elf::abi::PF_R > 0;
        let write = segment.p_flags & elf::abi::PF_W > 0;
        let execute = segment.p_flags & elf::abi::PF_X > 0;
        let data = elf.segment_data(&segment).unwrap().as_ptr();
        let attr = kernel_core::mmu::Attr::Normal {
            read,
            write,
            execute,
        };

        writeln!(
            &mut uart,
            "[PULL] Copying {}{}{} segment ({:#x?} file / {:#x?} mem) from {:#?} to {:#?}",
            if read { "R" } else { "" },
            if write { "W" } else { "" },
            if execute { "X" } else { "" },
            kernel_core::unit::Byte::new(segment.p_filesz as usize),
            kernel_core::unit::Byte::new(segment.p_memsz as usize),
            data as *mut u8,
            segment.p_paddr as *mut u8,
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

        reserve_unsized(allocator, segment.p_paddr, segment.p_memsz);

        // Only need to identity map first page of executable to bootstrap
        if execute {
            map(
                page_table_identity,
                0,
                segment.p_paddr,
                segment.p_memsz.min(1 << 16),
                attr,
            );
        }

        map(
            page_table_kernel,
            offset,
            segment.p_paddr,
            segment.p_memsz,
            attr,
        );
    }

    writeln!(
        &mut uart,
        "[PULL] Calling kernel at {:#x} with device_tree={:#x}, page_table_kernel={:#x}, heap={:#x}",
        elf.ehdr.e_entry - offset,
        device_tree_dst + offset,
        page_table_kernel as *mut _ as u64 + offset,
        page_table_identity as *mut _ as u64 + offset,
    )
    .unwrap();

    uart.flush();

    unsafe {
        core::arch::asm! {
            "mov x0, {arg_0:x}",
            "mov x1, {arg_1:x}",
            "mov x2, {arg_2:x}",
            "br {entry:x}",
            arg_0 = in(reg) device_tree_dst + offset,
            arg_1 = in(reg) page_table_kernel as *mut _ as u64 + offset,
            arg_2 = in(reg) allocator as *mut _ as u64 + offset,
            entry = in(reg) elf.ehdr.e_entry - offset,
            options(nomem, noreturn)
        }
    }
}

fn map<S: kernel_core::mem::AddressSpace>(
    page_table: &mut kernel_core::mmu::PageTable<S>,
    offset: u64,
    phys: u64,
    len: u64,
    attr: kernel_core::mmu::Attr,
) {
    (0..len)
        .step_by(1 << 16)
        .map(|page| phys + page)
        .map(|phys| (phys + offset, phys))
        .map(|(virt, phys)| (Virt::new(virt), Phys::new(phys)))
        .for_each(|(virt, phys)| page_table.map(virt, phys, attr))
}

fn reserve<T>(allocator: &mut kernel_core::mem::page::Allocator, phys: *const T) {
    reserve_unsized(allocator, phys as u64, mem::size_of::<T>() as u64)
}

fn reserve_unsized(allocator: &mut kernel_core::mem::page::Allocator, phys: u64, len: u64) {
    (0..len)
        .step_by(1 << 16)
        .map(|offset| phys + offset)
        .map(Phys::new)
        .map(kernel_core::mem::page::Id::from)
        .for_each(|page| allocator.reserve_mut(page))
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
