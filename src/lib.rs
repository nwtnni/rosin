#![cfg_attr(test, no_main)]
#![no_std]

#![feature(untagged_unions)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

#[macro_use]
pub mod serial;

pub mod qemu;
pub mod vga;

use core::panic;

pub fn test_runner(tests: &[&dyn Fn()]) {
    sprintln!("Running {} tests...", tests.len());
    for test in tests {
        test();
    }
    qemu::exit(qemu::Exit::Success);
}

#[cfg_attr(test, panic_handler)]
pub fn panic(info: &panic::PanicInfo) -> ! {
    sprintln!("[failed]");
    sprintln!("Error: {}", info);
    qemu::exit(qemu::Exit::Failure);
    loop {}
}

#[cfg(test)]
#[no_mangle]
extern "C" fn _start() -> ! {
    test_main();
    loop {}
}

#[test_case]
fn smoke_lib() {
    sprint!("smoke_lib... ");
    assert_eq!(1, 1);
    sprintln!("[ok]");
}
