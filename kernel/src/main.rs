#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::ptr::NonNull;
use core::time::Duration;

use kernel_core::device;
use kernel_core::info;
use kernel_core::print;
use kernel_core::println;
use kernel_core::time;

core::arch::global_asm! {
r"
.pushsection .text.boot

_start:
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
    page_table: *mut kernel_core::mmu::PageTable<kernel_core::mem::Kernel>,
) -> ! {
    kernel_core::init();

    info!("Hello, world!");

    info!(
        "device_tree={:#x}, page_table={:#x?}",
        device_tree, page_table
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

    recurse(0, device_tree.root());

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

fn recurse(depth: usize, node: device_tree::blob::Node) {
    info!("{:|<depth$}{:?}", "", node, depth = depth);
    for child in node.children() {
        recurse(depth + 1, child);
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kernel_core::handle_panic(info)
}
