#![no_main]
#![no_std]

#![feature(custom_test_frameworks)]
#![test_runner(rosin::test::runner)]
#![reexport_test_harness_main = "test_main"]

// Explicitly link libc implementations
extern crate rlibc;

use core::panic;

use rosin::println;

bootloader::entry_point!(kernel_main);

fn kernel_main(_boot_info: &'static bootloader::BootInfo) -> ! {
    println!("Starting...");

    rosin::init();

    #[cfg(test)]
    test_main();

    println!("Did not crash!");
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
