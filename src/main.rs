#![no_main]
#![no_std]

use core::fmt::Write as _;
use core::panic;

use rosin::vga;

#[panic_handler]
fn panic(_: &panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() {
    let mut writer = vga::screen::WRITER.lock();
    writeln!(writer, "Hello, wörld!").unwrap();
    writeln!(writer, "The numbers are {} and {}", 42, 1.0/3.0).unwrap();
}
