#![no_std]

#[macro_use]
pub mod print;

pub mod dev;
pub mod irq;
pub mod mmu;
mod sync;
pub mod time;
mod unit;

use core::fmt::Debug;
use core::fmt::Write;
use core::panic::PanicInfo;
use core::ptr::NonNull;
use core::time::Duration;

use aarch64_cpu::asm;
use aarch64_cpu::registers::CNTHCTL_EL2;
use aarch64_cpu::registers::CurrentEL;
use aarch64_cpu::registers::ELR_EL2;
use aarch64_cpu::registers::ELR_EL3;
use aarch64_cpu::registers::HCR_EL2;
use aarch64_cpu::registers::SCR_EL3;
use aarch64_cpu::registers::SP_EL1;
use aarch64_cpu::registers::SPSR_EL2;
use aarch64_cpu::registers::SPSR_EL3;
use sync::SpinLock;
use tock_registers::interfaces::Readable as _;
use tock_registers::interfaces::Writeable as _;

#[unsafe(no_mangle)]
pub extern "C" fn _start_hypervisor(
    device_tree: u64,
    reserved_1: u64,
    reserved_2: u64,
    reserved_3: u64,
    stack: u64,
) -> ! {
    let level = CurrentEL.read(CurrentEL::EL);

    if level >= 3 {
        SCR_EL3.write(
            SCR_EL3::RW::NextELIsAarch64
                + SCR_EL3::EA::NotTaken
                + SCR_EL3::FIQ::NotTaken
                + SCR_EL3::IRQ::NotTaken,
        );
    }

    if level >= 2 {
        CNTHCTL_EL2.write(CNTHCTL_EL2::EL1PCEN::SET + CNTHCTL_EL2::EL1PCTEN::SET);
        HCR_EL2.write(
            HCR_EL2::RW::EL1IsAarch64
                + HCR_EL2::E2H::DisableOsAtEl2
                + HCR_EL2::AMO::CLEAR
                + HCR_EL2::IMO::DisableVirtualIRQ
                + HCR_EL2::FMO::DisableVirtualFIQ
                + HCR_EL2::TGE::DisableTrapGeneralExceptionsToEl2
                + HCR_EL2::E2H::DisableOsAtEl2,
        );
    }

    #[allow(clippy::fn_to_numeric_cast)]
    match level {
        3 => {
            ELR_EL3.set(_start_kernel as u64);
            SPSR_EL3.write(
                SPSR_EL3::D::Masked
                    + SPSR_EL3::A::Masked
                    + SPSR_EL3::I::Masked
                    + SPSR_EL3::F::Masked
                    + SPSR_EL3::M::EL1h,
            );
        }
        2 => {
            ELR_EL2.set(_start_kernel as u64);
            SPSR_EL2.write(
                SPSR_EL2::D::Masked
                    + SPSR_EL2::A::Masked
                    + SPSR_EL2::I::Masked
                    + SPSR_EL2::F::Masked
                    + SPSR_EL2::M::EL1h,
            );
        }
        1 => _start_kernel(device_tree),
        level => unreachable!("Unexpected exception level: {}", level),
    }

    SP_EL1.set(stack);

    unsafe {
        core::arch::asm! {
            "mov x0, {:x}",
            "mov x1, {:x}",
            "mov x2, {:x}",
            "mov x3, {:x}",
            in(reg) device_tree,
            in(reg) reserved_1,
            in(reg) reserved_2,
            in(reg) reserved_3,
        }
    }

    asm::eret()
}

#[unsafe(no_mangle)]
pub fn _start_kernel(device_tree: u64) -> ! {
    init();

    info!(
        "Hello, world! EL: {}, DTB: {:#x}",
        CurrentEL.read(CurrentEL::EL),
        device_tree,
    );

    info!(
        "Resolution: {}ns, frequency: {}hz",
        Duration::from(time::Cycle::ONE).as_nanos(),
        time::frequency(),
    );

    // kernel::info!(
    //     "Interrupt in 1 second... EL: {}",
    //     CurrentEL.read(CurrentEL::EL)
    // );
    //
    // kernel::irq::enable();
    // kernel::irq::enable_timer(Duration::from_secs(1));

    // let device_tree = &dev::bcm2837b0::DTB;

    let device_tree =
        unsafe { device_tree::Blob::from_ptr(NonNull::new(device_tree as *mut u8).unwrap()) };

    info!("Device tree header: {:#x?}", device_tree.header());

    let mut indent = 0;
    for token in device_tree.iter() {
        match token {
            device_tree::blob::Token::Begin { name } => {
                info!("{:|<width$}{}", "", name, width = indent * 2);
                indent += 1;
            }
            device_tree::blob::Token::Prop(prop) => {
                info!("{:|<width$}-{:?}", "", prop, width = indent * 2)
            }
            device_tree::blob::Token::End => indent -= 1,
        }
    }

    for _ in 0..2 {
        info!("Sleeping for 1s...",);
        time::spin(Duration::from_secs(1));
    }

    println!("Echo:");
    let mut buffer = [0u8];
    loop {
        unsafe { dev::bcm2837b0::mini::Uart::new(0x3F21_5000) }
            .read(&mut buffer)
            .unwrap();

        print!(
            "{}",
            match buffer[0] {
                b'\r' => '\n',
                byte => byte as char,
            }
        );
    }
}

#[inline]
pub fn pause() {
    asm::nop()
}

#[inline]
pub fn spin() -> ! {
    loop {
        asm::wfe()
    }
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    // UART.lock().write_fmt(args).unwrap();
    // UART_MINI.lock().write_fmt(args).unwrap();
    unsafe { dev::bcm2837b0::mini::Uart::new(0x3F21_5000) }
        .write_fmt(args)
        .unwrap();
}

pub fn init() {
    // unsafe {
    //     bcm2837b0::gpio::Gpio::new(0x3F20_0000).init();
    //     dev::bcm2837b0::mini::Uart::new(0x3F21_5000).init();
    // }

    // UART.lock().initialize();
    // UART_MINI.lock().init();

    // unsafe {
    //     // bcm2837b0::clock::Clock::new(0x4000_0000).init();
    //
    //     irq::init();
    // }
}

pub static UART: SpinLock<dev::bcm2837b0::uart::Uart> =
    SpinLock::new(unsafe { dev::bcm2837b0::uart::Uart::new(0x3F20_1000) });

pub static UART_MINI: SpinLock<dev::bcm2837b0::mini::Uart> =
    SpinLock::new(unsafe { dev::bcm2837b0::mini::Uart::new(0x3F21_5000) });

pub type Result<T> = core::result::Result<T, Error>;

pub enum Error {
    Todo,
}

impl Debug for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::Todo => "TODO".fmt(fmt),
        }
    }
}

#[allow(dead_code)]
pub struct Console;

impl Write for Console {
    fn write_str(&mut self, string: &str) -> core::fmt::Result {
        const ADDRESS: *mut u8 = 0x3F20_1000 as _;

        for byte in string.bytes() {
            unsafe {
                ADDRESS.write_volatile(byte);
            }
        }

        Ok(())
    }
}

pub fn handle_panic(info: &PanicInfo) -> ! {
    let _ = writeln!(
        Console,
        "[PANIC][{}:{}] {}",
        info.location()
            .map(|location| location.file())
            .unwrap_or(""),
        info.location().map(|location| location.line()).unwrap_or(0),
        info.message(),
    );

    spin()
}
