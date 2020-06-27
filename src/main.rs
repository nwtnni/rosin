#![no_main]
#![no_std]

use core::panic;

use rosin::vga;

#[panic_handler]
fn panic(_: &panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() {
    let mut writer = vga::screen::Writer {
        column: 0,
        color: vga::color::Code::new(
            vga::color::Fore {
                bright: false,
                color: vga::color::T::Blue,
            },
            vga::color::Back {
                blink: true,
                color: vga::color::T::Red,
            },
        ),
        buffer: unsafe {
            &mut *(vga::ADDRESS as *mut vga::screen::Buffer)
        },
    };

    writer.write_byte(b'H');
    writer.write_string("ello! ");
    writer.write_string("Wörld!");
}
