#![no_main]
#![no_std]

#![feature(custom_test_frameworks)]
#![test_runner(rosin::test::runner)]
#![reexport_test_harness_main = "test_main"]

// Explicitly link alloc and libc implementations
extern crate alloc;
extern crate rlibc;

use alloc::string::String;
use core::panic;

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

        let mut page_table_mapper = unsafe { mem::init(phys_mem_offset) };
        let mut frame_allocator = unsafe { mem::BootInfoFrameAllocator::init(&boot_info.memory_map) };

        rosin::heap::init(&mut page_table_mapper, &mut frame_allocator)
            .expect("Failed to initialize heap");

        println!("{}", String::from("Did not crash!"));
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
