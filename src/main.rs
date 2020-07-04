#![no_main]
#![no_std]

#![feature(custom_test_frameworks)]
#![test_runner(rosin::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic;

use rosin::print;
use rosin::println;
use rosin::sprint;
use rosin::sprintln;

#[no_mangle]
extern "C" fn _start() -> ! {
    println!("Starting...");

    #[cfg(test)]
    test_main();

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
    sprint!("smoke_bin... ");
    assert_eq!(1, 1);
    sprintln!("[ok]");
}
