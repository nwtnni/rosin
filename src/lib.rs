#![cfg_attr(test, no_main)]
#![no_std]

#![feature(abi_x86_interrupt)]
#![feature(untagged_unions)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

#[macro_use]
pub mod serial;

pub mod idt;
pub mod gdt;
pub mod qemu;
pub mod vga;

use core::any;
use core::panic;

pub fn init() {
    gdt::init();
    idt::init();
}

pub trait Test {
    fn run(&self);
}

impl<T: Fn()> Test for T {
    fn run(&self) {
        sprint!("{}...\t", any::type_name::<T>());
        self();
        sprintln!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Test]) {
    sprintln!("Running {} tests...", tests.len());
    for test in tests {
        test.run();
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
    init();
    test_main();
    loop {}
}

#[test_case]
fn smoke_lib() {
    assert_eq!(1, 1);
}

#[test_case]
fn breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}
