#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::ptr::NonNull;
use core::time::Duration;

use aarch64_cpu::registers::CurrentEL;
use kernel_core::device;
use kernel_core::info;
use kernel_core::print;
use kernel_core::println;
use kernel_core::time;
use tock_registers::interfaces::Readable as _;

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
    ldr x4, =__VECTOR_TABLE
    msr VBAR_EL1, x4

    mrs x4, SCTLR_EL1

    # M: MMU enable
    orr x4, x4, (1 << 0)
    # C: Cacheability for data accesses
    orr x4, x4, (1 << 2)
    # I: Cacheability for instruction accesses
    orr x4, x4, (1 << 12)
    # WXN: Disable write XOR execute
    and x4, x4, ~(1 << 19)

    isb sy
    msr SCTLR_EL1, x4
    isb sy

    ldr x4, =__STACK_HI
    mov sp, x4
    ldr x4, =_start_kernel
    br x4

.size _start, . - _start
.type _start, %function
.global _start
.popsection
"
}

#[unsafe(link_section = ".text.start")]
#[unsafe(no_mangle)]
unsafe extern "C" fn _start_kernel(
    device_tree: u64,
    _reserved_1: u64,
    _reserved_2: u64,
    _reserved_3: u64,
) -> ! {
    kernel_core::init();

    info!(
        "Hello, world! EL: {}, DTB: {:#x}",
        CurrentEL.read(CurrentEL::EL),
        device_tree,
    );

    info!(
        "Resolution: {}ns, frequency: {}hz",
        Duration::from(time::Cycle::ONE).as_nanos(),
        time::frequency(),
    );

    // kernel::info!(
    //     "Interrupt in 1 second... EL: {}",
    //     CurrentEL.read(CurrentEL::EL)
    // );
    //
    // kernel::irq::enable();
    // kernel::irq::enable_timer(Duration::from_secs(1));

    // let device_tree = &dev::bcm2837b0::DTB;

    let device_tree =
        unsafe { device_tree::Blob::from_ptr(NonNull::new(device_tree as *mut u8).unwrap()) };

    info!("Device tree header: {:#x?}", device_tree.header());

    let mut indent = 0;
    for token in device_tree.iter() {
        match token {
            device_tree::blob::Token::Begin { name } => {
                info!("{:|<width$}{}", "", name, width = indent * 2);
                indent += 1;
            }
            device_tree::blob::Token::Prop(prop) => {
                info!("{:|<width$}-{:?}", "", prop, width = indent * 2)
            }
            device_tree::blob::Token::End => indent -= 1,
        }
    }

    for _ in 0..2 {
        info!("Sleeping for 1s...",);
        time::spin(Duration::from_secs(1));
    }

    println!("Echo:");
    loop {
        let byte = unsafe { device::bcm2837b0::mini::Uart::new(0x3F21_5000) }.read_byte();
        print!("{}", byte as char);
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kernel_core::handle_panic(info)
}
