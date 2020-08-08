#![cfg_attr(test, no_main)]
#![no_std]

#![feature(abi_x86_interrupt)]
#![feature(untagged_unions)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test::runner)]
#![reexport_test_harness_main = "test_main"]

// Explicitly link libc implementations
extern crate rlibc;

#[macro_use]
pub mod serial;

pub mod interrupt;
pub mod gdt;
pub mod mem;
pub mod qemu;
pub mod test;
pub mod vga;

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
