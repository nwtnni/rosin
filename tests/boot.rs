#![no_main]
#![no_std]
#![feature(custom_test_frameworks)]
#![test_runner(rosin::test::runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic;

use rosin::println;

#[no_mangle]
extern "C" fn _start() -> ! {
    test_main();
    rosin::hlt_loop()
}

#[panic_handler]
fn panic(info: &panic::PanicInfo) -> ! {
    rosin::test::panic(info)
}

#[test_case]
fn smoke() {
    println!("println output");
}
