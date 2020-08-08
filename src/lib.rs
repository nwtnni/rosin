#![cfg_attr(test, no_main)]
#![no_std]

#![feature(abi_x86_interrupt)]
#![feature(untagged_unions)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

// Explicitly link libc implementations
extern crate rlibc;

#[macro_use]
pub mod serial;

pub mod interrupt;
pub mod gdt;
pub mod qemu;
pub mod vga;

use core::any;
use core::panic;

#[cfg(test)]
bootloader::entry_point!(test_kernel_main);

#[cfg(test)]
fn test_kernel_main(_boot_info: &'static bootloader::BootInfo) -> ! {
    init();
    test_main();
    hlt_loop()
}

pub fn init() {
    gdt::init();
    interrupt::init_idt();
    interrupt::init_pics();
    x86_64::instructions::interrupts::enable();
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
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
    hlt_loop()
}

#[test_case]
fn smoke_lib() {
    assert_eq!(1, 1);
}

#[test_case]
fn breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}
