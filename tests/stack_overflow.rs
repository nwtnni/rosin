#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic;

use lazy_static::lazy_static;
use rosin::sprint;
use rosin::sprintln;
use rosin::qemu;
use x86_64::structures::idt;

lazy_static! {
    static ref IDT: idt::InterruptDescriptorTable = {
        let mut idt = idt::InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault)
                .set_stack_index(rosin::gdt::DOUBLE_FAULT);
        }
        idt
    };
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    sprint!("stack_overflow::stack_overflow...\t");

    rosin::gdt::init();
    IDT.load();

    stack_overflow();

    sprintln!("[failed]");
    panic!("Execution continued after stack overflow");
}

#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow();
    // Prevent tail-call optimizations
    volatile::Volatile::new(0).read();
}

#[panic_handler]
fn panic(info: &panic::PanicInfo) -> ! {
    rosin::test::panic(info)
}

extern "x86-interrupt" fn double_fault(
    _: &mut idt::InterruptStackFrame,
    _: u64,
) -> ! {
    sprintln!("[ok]");
    qemu::exit(qemu::Exit::Success);
    rosin::hlt_loop()
}
