use core::ops::Deref;
use core::ops::DerefMut;

use aarch64_cpu::registers::ReadWriteable as _;
use aarch64_cpu::registers::Writeable as _;
use tock_registers::register_bitfields;
use tock_registers::register_structs;
use tock_registers::registers::ReadOnly;
use tock_registers::registers::ReadWrite;
use tock_registers::registers::WriteOnly;

pub struct Clock {
    address: usize,
}

impl Clock {
    pub const unsafe fn new(address: usize) -> Self {
        Self { address }
    }

    pub fn init(&self) {
        self.source
            .write(Source::SOURCE::Crystal + Source::INCREMENT::One);
        self.prescale.set(1 << 31);
    }

    pub fn enable(&self) {
        self.control
            .modify(Control::ENABLE::SET + Control::IRQ_ENABLE::SET + Control::RELOAD.val(2000));
    }
}

impl Deref for Clock {
    type Target = Mmio;
    fn deref(&self) -> &Self::Target {
        unsafe { core::ptr::with_exposed_provenance::<Self::Target>(self.address).as_ref() }
            .unwrap()
    }
}

impl DerefMut for Clock {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { core::ptr::with_exposed_provenance_mut::<Self::Target>(self.address).as_mut() }
            .unwrap()
    }
}

// register_structs! {
//     pub Mmio {
//         (0x00 => status: WriteOnly<u32, CS::Register>),
//         (0x04 => lo: ReadOnly<u32>),
//         (0x08 => hi: ReadOnly<u32>),
//         (0x0c => pub compare: [WriteOnly<u32>; 4]),
//         (0x1c => @END),
//     }
// }
//
// register_bitfields! {
//     u32,
//
//     CS [
//         M0 OFFSET(0) NUMBITS(1) [],
//         M1 OFFSET(0) NUMBITS(1) [],
//         M2 OFFSET(0) NUMBITS(1) [],
//         M3 OFFSET(0) NUMBITS(1) [],
//     ],
// }

register_structs! {
    pub Mmio {
        (0x00 => source: WriteOnly<u32, Source::Register>),
        (0x04 => _reserved0),
        (0x08 => prescale: WriteOnly<u32>),
        (0x0c => _reserved1),
        (0x34 => control: ReadWrite<u32, Control::Register>),
        (0x38 => clear: WriteOnly<u32, Clear::Register>),
        (0x3c => @END),
    }
}

register_bitfields! {
    u32,

    Source [
        SOURCE OFFSET(8) NUMBITS(1) [
            Crystal = 0,
            Apb = 1,
        ],

        INCREMENT OFFSET(9) NUMBITS(1) [
            One = 0,
            Two = 1,
        ],
    ],

    Control [
        RELOAD OFFSET(0) NUMBITS(28) [],
        ENABLE OFFSET(28) NUMBITS(1) [],
        IRQ_ENABLE OFFSET(29) NUMBITS(1) [],
        IRQ_FLAG OFFSET(31) NUMBITS(1) [],
    ],

    Clear [
        IRQ_FLAG OFFSET(31) NUMBITS(1) [],
        RELOAD OFFSET(30) NUMBITS(1) [],
    ],
}
