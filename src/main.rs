#![no_main]
#![no_std]

#![feature(custom_test_frameworks)]
#![test_runner(rosin::test::runner)]
#![reexport_test_harness_main = "test_main"]

// Explicitly link libc implementations
extern crate rlibc;

use core::panic;

use x86_64::structures::paging::MapperAllSizes as _;

use rosin::println;
use rosin::mem;

bootloader::entry_point!(kernel_main);

fn kernel_main(boot_info: &'static bootloader::BootInfo) -> ! {
    println!("Starting...");

    rosin::init();

    #[cfg(test)] {
        test_main();
    }

    #[cfg(not(test))] {
        let mem_map = unsafe {
            mem::init(x86_64::VirtAddr::new(
                boot_info.physical_memory_offset
            ))
        };

        let addresses = [
            // Identitiy mapped (VGA buffer)
            0xb8000,

            // Code
            0x201008,

            // Stack
            0x0100_0020_1A10,

            boot_info.physical_memory_offset,
        ];

        for &address in &addresses {
            let virt = x86_64::VirtAddr::new(address);
            let phys = mem_map.translate_addr(virt);
            println!("{:?} -> {:?}", virt, phys);
        }

    }

    rosin::hlt_loop()
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &panic::PanicInfo) -> ! {
    println!("{}", info);
    rosin::hlt_loop()
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &panic::PanicInfo) -> ! {
    rosin::test::panic(info)
}

#[test_case]
fn smoke_bin() {
    assert_eq!(1, 1);
}
