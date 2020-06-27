#![no_main]
#![no_std]

use core::panic;

use rosin::println;

#[panic_handler]
fn panic(info: &panic::PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() {
    println!("Hello, wörld!");
    println!("The numbers are {} and {}", 42, 1.0/3.0);
    panic!("Here");
}
