use core::ops::Deref;
use core::ops::DerefMut;
use core::sync::atomic::AtomicU64;
use core::sync::atomic::Ordering;

use aarch64_cpu::registers::Readable as _;
use aarch64_cpu::registers::Writeable as _;
use tock_registers::register_bitfields;
use tock_registers::register_structs;
use tock_registers::registers::ReadWrite;

use crate::time;
use crate::time::Cycle;

pub struct Gpio {
    address: usize,
}

impl Gpio {
    pub const unsafe fn new(address: usize) -> Self {
        Self { address }
    }

    pub fn init(&self) {
        let mut selector = self.function_select[1].get();

        selector &= !(0b111 << 12);
        selector |= (Select::Alt5 as u32) << 12;

        selector &= !(0b111 << 15);
        selector |= (Select::Alt5 as u32) << 15;

        self.function_select[1].set(selector);

        self.pull_enable.write(PullEnable::PUD::Off);
        // let spin = Cycle::new(150);
        // time::spin_cycle(spin);

        macro_rules! delay {
            ($cycles:expr) => {
                unsafe {
                    core::arch::asm! {
                        "0:",
                        "subs {spin:x}, {spin:x}, 1",
                        "bne 0b",
                        spin = inout(reg) $cycles => _,
                    }
                }
            };
        }

        // let spin = AtomicU64::new(0);
        // for _ in 0..150 {
        //     let old = spin.load(Ordering::Relaxed);
        //     spin.store(old + 1, Ordering::Relaxed);
        // }
        //
        delay!(150);

        self.pull_clock[0].set((1 << 14) | (1 << 15));
        // for _ in 0..150 {
        //     let old = spin.load(Ordering::Relaxed);
        //     spin.store(old + 1, Ordering::Relaxed);
        // }
        // time::spin_cycle(spin);

        delay!(150);
        self.pull_clock[0].set(0);
    }
}

enum Select {
    Input = 0b000,
    Output = 0b001,
    Alt0 = 0b100,
    Alt1 = 0b101,
    Alt2 = 0b110,
    Alt3 = 0b111,
    Alt4 = 0b011,
    Alt5 = 0b010,
}

impl Deref for Gpio {
    type Target = Mmio;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { core::ptr::with_exposed_provenance::<Self::Target>(self.address).as_ref() }
            .unwrap()
    }
}

impl DerefMut for Gpio {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { core::ptr::with_exposed_provenance_mut::<Self::Target>(self.address).as_mut() }
            .unwrap()
    }
}

register_structs! {
    #[allow(non_snake_case)]
    pub Mmio {
        (0x00 => function_select: [ReadWrite<u32>; 6]),
        (0x18 => _reserved2),
        (0x94 => pull_enable: ReadWrite<u32, PullEnable::Register>),
        (0x98 => pull_clock: [ReadWrite<u32>; 2]),
        (0xA0 => _reserved3),
        (0xE8 => @END),
    }
}

// https://github.com/rust-embedded/rust-raspberrypi-OS-tutorials/blob/644474cc09f755249f9c55d99a5d1e07a2562fc7/05_drivers_gpio_uart/src/bsp/device_driver/bcm/bcm2xxx_gpio.rs
register_bitfields! {
    u32,

    /// GPIO Pull-up/down Register
    PullEnable [
        /// Controls the actuation of the internal pull-up/down control line to ALL the GPIO pins.
        PUD OFFSET(0) NUMBITS(2) [
            Off = 0b00,
            PullDown = 0b01,
            PullUp = 0b10
        ]
    ],
}
