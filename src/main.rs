#![no_main]
#![no_std]

#![feature(custom_test_frameworks)]
#![test_runner(rosin::test::runner)]
#![reexport_test_harness_main = "test_main"]

// Explicitly link libc implementations
extern crate rlibc;

use core::panic;

use x86_64::structures::paging;

use rosin::println;
use rosin::mem;
use rosin::util::Tap as _;

bootloader::entry_point!(kernel_main);

fn kernel_main(boot_info: &'static bootloader::BootInfo) -> ! {
    println!("Starting...");

    rosin::init();

    #[cfg(test)] {
        test_main();
    }

    #[cfg(not(test))] {
        let phys_mem_offset = boot_info
            .physical_memory_offset
            .tap(x86_64::VirtAddr::new);

        let mut page_table = unsafe { mem::init(phys_mem_offset) };
        let mut frame_allocator = unsafe { mem::BootInfoFrameAllocator::init(&boot_info.memory_map) };
        let page = 0
            .tap(x86_64::VirtAddr::new)
            .tap(paging::Page::containing_address);

        mem::create_example_mapping(page, &mut page_table, &mut frame_allocator);

        let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
        unsafe {
            page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e);
        }
    }

    rosin::hlt_loop()
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &panic::PanicInfo) -> ! {
    println!("{}", info);
    rosin::hlt_loop()
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &panic::PanicInfo) -> ! {
    rosin::test::panic(info)
}

#[test_case]
fn smoke_bin() {
    assert_eq!(1, 1);
}
