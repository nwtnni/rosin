#![no_main]
#![no_std]

use core::panic;

use rosin::sprint;
use rosin::sprintln;
use rosin::qemu;

#[no_mangle]
extern "C" fn _start() -> ! {
    should_fail();
    qemu::exit(qemu::Exit::Failure);
    rosin::hlt_loop()
}

#[panic_handler]
fn panic(info: &panic::PanicInfo) -> ! {
    sprintln!("[ok]");
    qemu::exit(qemu::Exit::Success);
    rosin::hlt_loop()
}

fn should_fail() {
    sprint!("should_panic::should_fail...\t");
    assert_eq!(0, 1);
    sprintln!("[failed]");
}
