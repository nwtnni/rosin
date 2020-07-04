#![cfg_attr(test, no_main)]
#![no_std]

#![feature(untagged_unions)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

pub mod vga;

use core::panic;

pub fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests...", tests.len());
    for test in tests {
        test();
    }
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &panic::PanicInfo) -> ! {
    println!("{}", info);
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
    print!("smoke_lib... ");
    assert_eq!(1, 1);
    println!("[ok]");
}
