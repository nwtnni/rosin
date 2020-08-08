use core::any;
use core::panic;

use crate::qemu;

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

pub fn runner(tests: &[&dyn Test]) {
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
    crate::hlt_loop()
}

#[test_case]
fn smoke_lib() {
    assert_eq!(1, 1);
}

#[test_case]
fn breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}
