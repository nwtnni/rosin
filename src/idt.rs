use lazy_static::lazy_static;
use x86_64::structures::idt;

use crate::println;

lazy_static! {
    static ref IDT: idt::InterruptDescriptorTable = {
        let mut idt = idt::InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint);
        idt.double_fault.set_handler_fn(double_fault);
        idt
    };
}

pub fn init() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint(frame: &mut idt::InterruptStackFrame) {
    println!("Breakpoint Exception");    
    println!("{:#?}", frame);    
}

extern "x86-interrupt" fn double_fault(frame: &mut idt::InterruptStackFrame, _: u64) -> ! {
    panic!("Double Fault Exception\n{:#?}", frame)
}
