#![no_main]
#![no_std]

use core::panic;

#[panic_handler]
fn panic(_: &panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() {
    loop {}
}
