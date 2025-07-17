#![no_std]
#![no_main]

use core::ffi;
use core::panic::PanicInfo;
use core::ptr::NonNull;
use core::time::Duration;

use kernel_core::device;
use kernel_core::info;
use kernel_core::mem::Phys;
use kernel_core::mem::Virt;
use kernel_core::mem::page;
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

unsafe extern "C" {
    static __KERNEL_LO: ffi::c_void;
    static __KERNEL_HI: ffi::c_void;
    static __KERNEL_OFFSET: ffi::c_void;
}

#[unsafe(link_section = ".text.start")]
#[unsafe(no_mangle)]
unsafe extern "C" fn _start_kernel(
    device_tree: NonNull<device_tree::blob::Header>,
    page_table: &mut kernel_core::mmu::PageTable<kernel_core::mem::Kernel>,
    heap: *mut u8,
) -> ! {
    kernel_core::init();

    info!("Hello, world!");

    let device_tree = unsafe { device_tree::Blob::from_ptr(device_tree) };

    info!("Device tree header: {:#x?}", device_tree.header());

    let root = device_tree.root();

    const PAGE_SIZE: usize = 1 << 16;

    let ranges = [
        (
            device_tree.as_ptr().as_ptr() as usize,
            device_tree.header().len(),
        ),
        (
            page_table as *mut _ as usize,
            core::mem::size_of_val(page_table),
        ),
        unsafe {
            (
                (&__KERNEL_OFFSET as *const _ as usize),
                (&__KERNEL_HI as *const _ as usize) - (&__KERNEL_OFFSET as *const _ as usize),
            )
        },
    ];

    let total = ((1 << 30)
        - ranges
            .into_iter()
            .map(|(_, len)| len.next_multiple_of(PAGE_SIZE))
            .sum::<usize>())
        / PAGE_SIZE
        / 64;

    let allocator =
        unsafe { kernel_core::mem::alloc::Page::from_raw_parts_mut(heap.cast::<u64>(), total) };

    let offset = unsafe { &__KERNEL_OFFSET } as *const _ as usize;

    for page in (0..kernel_core::mem::alloc::Page::size_of(total))
        .step_by(PAGE_SIZE)
        .map(|page| heap as u64 + page as u64)
    {
        page_table.map(
            Virt::new(page),
            Phys::new(page - offset as u64),
            kernel_core::mmu::Attr::Normal {
                read: true,
                write: true,
                execute: false,
            },
        );
    }

    allocator.fill();
    info!("Total pages: {:#x?}", allocator.len());
    for (base, len) in ranges.into_iter().chain(core::iter::once((
        heap as usize,
        kernel_core::mem::alloc::Page::size_of(total),
    ))) {
        info!("Reserving region {:#x?}...", len);
        let base = base - offset;

        for offset in (0..len).step_by(PAGE_SIZE) {
            let phys = Phys::new(base as u64 + offset as u64);
            info!("Reserve {:#x?}", phys);
            allocator.reserve(page::Id::from(phys));
        }
    }

    info!("Available pages: {:#x?}", allocator.len());

    // info!(
    //     "Resolution: {}ns, frequency: {}hz",
    //     Duration::from(time::Cycle::ONE).as_nanos(),
    //     time::frequency(),
    // );
    // kernel::info!(
    //     "Interrupt in 1 second... EL: {}",
    //     CurrentEL.read(CurrentEL::EL)
    // );
    //
    // kernel::irq::enable();
    // kernel::irq::enable_timer(Duration::from_secs(1));

    // let device_tree = &dev::bcm2837b0::DTB;

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
    info!("{:|<depth$}-{:?}", "", node, depth = depth);
    for child in node.children() {
        recurse(depth + 1, child);
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kernel_core::handle_panic(info)
}
