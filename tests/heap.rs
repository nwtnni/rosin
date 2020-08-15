#![no_main]
#![no_std]
#![feature(custom_test_frameworks)]
#![test_runner(rosin::test::runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::panic;

use rosin::heap;
use rosin::mem;
use rosin::util::Tap as _;

bootloader::entry_point!(main);

fn main(boot_info: &'static bootloader::BootInfo) -> ! {
    rosin::init();

    let phys_mem_offset = boot_info
        .physical_memory_offset
        .tap(x86_64::VirtAddr::new);

    let mut page_table_mapper = unsafe { mem::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { mem::BootInfoFrameAllocator::init(&boot_info.memory_map) };

    heap::init(&mut page_table_mapper, &mut frame_allocator)
        .expect("Failed to initialize heap");

    test_main();

    rosin::hlt_loop()
}

#[panic_handler]
fn panic(info: &panic::PanicInfo) -> ! {
    rosin::test::panic(info)
}

#[test_case]
fn smoke() {
    let x = Box::new(12);
    let y = Box::new('b');
    assert_eq!(*x, 12);
    assert_eq!(*y, 'b');
}

#[test_case]
fn large_vec() {
    let total = 1000;
    assert_eq!(
        (0..total)
            .collect::<Vec<_>>()
            .into_iter()
            .sum::<u64>(),
        ((total - 1) * total) / 2,
    );
}

#[test_case]
fn many_boxes() {
    for i in 0..heap::SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
}
