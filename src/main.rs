#![no_main]
#![no_std]

#![feature(custom_test_frameworks)]
#![test_runner(rosin::test_runner)]
#![reexport_test_harness_main = "test_main"]

// Explicitly link libc implementations
extern crate rlibc;

use core::panic;

use rosin::println;

#[no_mangle]
extern "C" fn _start() -> ! {
    println!("Starting...");

    rosin::init();

    #[cfg(test)]
    test_main();

    println!("Did not crash!");

    loop {}
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &panic::PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &panic::PanicInfo) -> ! {
    rosin::panic(info)
}

#[test_case]
fn smoke_bin() {
    assert_eq!(1, 1);
}
