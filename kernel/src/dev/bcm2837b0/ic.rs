pub use core::Core;
pub use peripheral::Peripheral;

mod peripheral {
    use core::ops::Deref;
    use core::ops::DerefMut;

    use aarch64_cpu::registers::Writeable as _;
    use tock_registers::register_bitfields;
    use tock_registers::register_structs;
    use tock_registers::registers::ReadOnly;
    use tock_registers::registers::ReadWrite;

    pub struct Peripheral {
        address: usize,
    }

    impl Peripheral {
        pub const unsafe fn new(address: usize) -> Self {
            Self { address }
        }

        pub fn init(&self) {
            self.enable_basic.write(Basic::TIMER::SET);
        }
    }

    impl Deref for Peripheral {
        type Target = Mmio;
        fn deref(&self) -> &Self::Target {
            unsafe { core::ptr::with_exposed_provenance::<Self::Target>(self.address).as_ref() }
                .unwrap()
        }
    }

    impl DerefMut for Peripheral {
        fn deref_mut(&mut self) -> &mut Self::Target {
            unsafe { core::ptr::with_exposed_provenance_mut::<Self::Target>(self.address).as_mut() }
                .unwrap()
        }
    }

    register_structs! {
        pub Mmio {
            (0x000 => _reserved0),
            (0x200 => pending_basic: ReadOnly<u32, Pending::Register>),
            (0x204 => pending: [ReadOnly<u32>; 2]),
            (0x20C => fiq: ReadWrite<u32>),
            (0x210 => enable: [ReadWrite<u32>; 2]),
            (0x218 => enable_basic: ReadWrite<u32, Basic::Register>),
            (0x21C => disable: [ReadWrite<u32>; 2]),
            (0x224 => disable_basic: ReadWrite<u32, Basic::Register>),
            (0x228 => @END),
        }
    }

    register_bitfields! {
        u32,

        Pending [
            TIMER OFFSET(0) NUMBITS(1) [],
            MAILBOX OFFSET(1) NUMBITS(1) [],
            DOORBELL OFFSET(2) NUMBITS(2) [],

            PENDING OFFSET(8) NUMBITS(2) [],
        ],

        Basic [
            TIMER OFFSET(0) NUMBITS(1) [],
            MAILBOX OFFSET(1) NUMBITS(1) [],
            DOORBELL OFFSET(2) NUMBITS(2) [],
            GPU_HALTED OFFSET(4) NUMBITS(2) [],
            ACCESS_ERROR OFFSET(6) NUMBITS(2) [],
        ],
    }
}

mod core {
    use core::ops::Deref;
    use core::ops::DerefMut;

    use aarch64_cpu::registers::Writeable as _;
    use tock_registers::register_bitfields;
    use tock_registers::register_structs;
    use tock_registers::registers::ReadOnly;
    use tock_registers::registers::WriteOnly;

    pub struct Core {
        address: usize,
    }

    impl Core {
        pub const unsafe fn new(address: usize) -> Self {
            Self { address }
        }

        pub fn init(&self) {
            self.timer[0].write(Timer::PNS_IRQ::SET);
        }
    }

    impl Deref for Core {
        type Target = Mmio;
        fn deref(&self) -> &Self::Target {
            unsafe { core::ptr::with_exposed_provenance::<Self::Target>(self.address).as_ref() }
                .unwrap()
        }
    }

    impl DerefMut for Core {
        fn deref_mut(&mut self) -> &mut Self::Target {
            unsafe { core::ptr::with_exposed_provenance_mut::<Self::Target>(self.address).as_mut() }
                .unwrap()
        }
    }

    register_structs! {
        pub Mmio {
            (0x00 => _reserved0),
            (0x40 => timer: [WriteOnly<u32, Timer::Register>; 4]),
            (0x50 => mailbox: [WriteOnly<u32, Mailbox::Register>; 4]),
            (0x60 => source_irq: [ReadOnly<u32, Source::Register>; 4]),
            (0x70 => source_fiq: [ReadOnly<u32, Source::Register>; 4]),
            (0x80 => @END),
        }
    }

    register_bitfields! {
        u32,

        pub Timer [
            PS_IRQ OFFSET(0) NUMBITS(1) [],
            PNS_IRQ OFFSET(1) NUMBITS(1) [],
            HP_IRQ OFFSET(2) NUMBITS(1) [],
            V_IRQ OFFSET(3) NUMBITS(1) [],

            PS_FIQ OFFSET(4) NUMBITS(1) [],
            PNS_FIQ OFFSET(5) NUMBITS(1) [],
            HP_FIQ OFFSET(6) NUMBITS(1) [],
            V_FIQ OFFSET(7) NUMBITS(1) [],
        ],

        Mailbox [
            M0_IRQ OFFSET(0) NUMBITS(1) [],
            M1_IRQ OFFSET(1) NUMBITS(1) [],
            M2_IRQ OFFSET(2) NUMBITS(1) [],
            M3_IRQ OFFSET(3) NUMBITS(1) [],

            M0_FIQ OFFSET(4) NUMBITS(1) [],
            M1_FIQ OFFSET(5) NUMBITS(1) [],
            M2_FIQ OFFSET(6) NUMBITS(1) [],
            M3_FIQ OFFSET(7) NUMBITS(1) [],
        ],

        pub Source [
            CNTPS OFFSET(0) NUMBITS(1) [],
            CNTPNS OFFSET(1) NUMBITS(1) [],
            CNTHP OFFSET(2) NUMBITS(1) [],
            CNTV OFFSET(3) NUMBITS(1) [],
            M0 OFFSET(4) NUMBITS(1) [],
            M1 OFFSET(5) NUMBITS(1) [],
            M2 OFFSET(6) NUMBITS(1) [],
            M3 OFFSET(7) NUMBITS(1) [],
            GPU OFFSET(8) NUMBITS(1) [],
            PMU OFFSET(9) NUMBITS(1) [],
        ],
    }
}
