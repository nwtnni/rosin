use core::ops::Deref;
use core::ops::DerefMut;

use tock_registers::register_bitfields;
use tock_registers::register_structs;
use tock_registers::registers::ReadOnly;
use tock_registers::registers::WriteOnly;

pub struct Local {
    address: usize,
}

impl Local {
    pub const unsafe fn new(address: usize) -> Self {
        Self { address }
    }
}

impl Deref for Local {
    type Target = Mmio;
    fn deref(&self) -> &Self::Target {
        unsafe { core::ptr::with_exposed_provenance::<Self::Target>(self.address).as_ref() }
            .unwrap()
    }
}

impl DerefMut for Local {
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

    Timer [
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

    Source [
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

    TimerControl [
        RELOAD OFFSET(0) NUMBITS(28) [],
        ENABLE OFFSET(28) NUMBITS(1) [],
        IRQ_ENABLE OFFSET(29) NUMBITS(1) [],
        IRQ_FLAG OFFSET(31) NUMBITS(1) [],
    ],

    TimerClear [
        IRQ_FLAG OFFSET(31) NUMBITS(1) [],
        RELOAD OFFSET(30) NUMBITS(1) [],
    ],
}
