use lazy_static::lazy_static;
use pc_keyboard::DecodedKey;
use pc_keyboard::HandleControl;
use pc_keyboard::Keyboard;
use pc_keyboard::layouts;
use pc_keyboard::ScancodeSet1;
use pic8259_simple::ChainedPics;
use x86_64::instructions::port;
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

        idt[Index::Timer.as_usize()].set_handler_fn(timer);
        idt[Index::Keyboard.as_usize()].set_handler_fn(keyboard);

        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault)
                .set_stack_index(gdt::DOUBLE_FAULT);
        }

        idt
    };

    static ref KEYBOARD: spin::Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = {
        spin::Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore))
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
    Keyboard = PIC_1_OFFSET + 1,
}

impl Index {
    pub fn as_usize(self) -> usize {
        self as u8 as usize
    }
}

extern "x86-interrupt" fn timer(_stack_frame: &mut idt::InterruptStackFrame) {
    print!(".");
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(Index::Timer as u8);
    }
}

extern "x86-interrupt" fn keyboard(_stack_frame: &mut idt::InterruptStackFrame) {
    let scancode: u8 = unsafe {
        port::Port::new(0x60).read()
    };

    let mut keyboard = KEYBOARD.lock();
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
            | DecodedKey::Unicode(character) => print!("{}", character),
            | DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

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
