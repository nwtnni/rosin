#![no_main]
#![no_std]

use core::panic;

#[panic_handler]
fn panic(_: &panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() {
    let vga = 0x000B_8000 as *mut u8;
    for (index, &byte) in b"Hello, world!".iter().enumerate() {
        unsafe {
            *vga.offset(index as isize * 2) = byte;
            *vga.offset(index as isize * 2 + 1) = 0xB;
        }
    }
}
