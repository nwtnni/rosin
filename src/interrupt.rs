use lazy_static::lazy_static;
use pic8259_simple::ChainedPics;
use x86_64::structures::idt;

use crate::gdt;
use crate::print;
use crate::println;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> = {
    spin::Mutex::new(unsafe {
        ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET)
    })
};

lazy_static! {
    static ref IDT: idt::InterruptDescriptorTable = {
        let mut idt = idt::InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint);

        idt[Index::Timer.as_usize()].set_handler_fn(timer_interrupt);

        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault)
                .set_stack_index(gdt::DOUBLE_FAULT);
        }

        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

pub fn init_pics() {
    unsafe {
        PICS.lock().initialize();
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Index {
    Timer = PIC_1_OFFSET,
}

impl Index {
    pub fn as_usize(self) -> usize {
        self as u8 as usize
    }
}

extern "x86-interrupt" fn timer_interrupt(_stack_frame: &mut idt::InterruptStackFrame) {
    print!(".");
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(Index::Timer as u8);
    }
}

extern "x86-interrupt" fn breakpoint(frame: &mut idt::InterruptStackFrame) {
    println!("Breakpoint Exception");
    println!("{:#?}", frame);
}

extern "x86-interrupt" fn double_fault(frame: &mut idt::InterruptStackFrame, _: u64) -> ! {
    panic!("Double Fault Exception\n{:#?}", frame)
}
