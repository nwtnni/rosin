use lazy_static::lazy_static;
use x86_64::structures::idt;

use crate::println;

lazy_static! {
    static ref IDT: idt::InterruptDescriptorTable = {
        let mut idt = idt::InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint(frame: &mut idt::InterruptStackFrame) {
    println!("Breakpoint Exception");    
    println!("{:#?}", frame);    
}
