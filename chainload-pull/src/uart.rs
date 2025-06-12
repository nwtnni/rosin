use core::fmt::Write;
use core::ops::Deref;
use core::ops::DerefMut;

use aarch64_cpu::registers::Readable as _;
use aarch64_cpu::registers::Writeable as _;
use tock_registers::register_bitfields;
use tock_registers::register_structs;
use tock_registers::registers::ReadOnly;
use tock_registers::registers::ReadWrite;

pub struct Uart {
    address: usize,
}

impl Uart {
    pub const unsafe fn new(address: usize) -> Self {
        Self { address }
    }

    pub fn init(&self) {
        self.enable.write(Device::UART::SET);

        self.control
            .write(Control::RX::Disable + Control::TX::Disable);

        self.line_control.write(LineControl::DATA_SIZE::Bit8);

        self.baud_rate.set(270);

        self.interrupt_status
            .write(InterruptStatus::TX::SET + InterruptStatus::RX::SET);
        self.control
            .write(Control::RX::Enable + Control::TX::Enable);
    }

    pub fn read_byte(&mut self) -> u8 {
        while !self.line_status.is_set(LineStatus::RX_READY) {}
        self.io.get() as u8
    }

    pub fn write_byte(&mut self, byte: u8) {
        while !self.line_status.is_set(LineStatus::TX_EMPTY) {}
        self.io.set(byte as u32);
    }

    pub fn flush(&mut self) {
        while !self.line_status.is_set(LineStatus::TX_IDLE) {}
    }
}

impl Write for Uart {
    fn write_str(&mut self, string: &str) -> core::fmt::Result {
        for byte in string.bytes() {
            self.write_byte(byte);
        }

        Ok(())
    }
}

impl Deref for Uart {
    type Target = Mmio;
    fn deref(&self) -> &Self::Target {
        unsafe { core::ptr::with_exposed_provenance::<Self::Target>(self.address).as_ref() }
            .unwrap()
    }
}

impl DerefMut for Uart {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { core::ptr::with_exposed_provenance_mut::<Self::Target>(self.address).as_mut() }
            .unwrap()
    }
}

register_structs! {
    pub Mmio {
        (0x00 => _reserved0),
        (0x04 => enable: ReadWrite<u32, Device::Register>),
        (0x08 => _reserved1),
        (0x40 => io: ReadWrite<u32>),
        (0x44 => interrupt_control: ReadWrite<u32, InterruptControl::Register>),
        (0x48 => interrupt_status: ReadWrite<u32, InterruptStatus::Register>),
        (0x4C => line_control: ReadWrite<u32, LineControl::Register>),
        (0x50 => modem_control: ReadWrite<u32>),
        (0x54 => line_status: ReadOnly<u32, LineStatus::Register>),
        (0x58 => _reserved2),
        (0x60 => control: ReadWrite<u32, Control::Register>),
        (0x64 => _reserved3),
        (0x68 => baud_rate: ReadWrite<u32>),
        (0x6C => @END),
    }
}

register_bitfields! {
    u32,

    Device [
        UART OFFSET(0) NUMBITS(1) [],
    ],

    InterruptControl [
        RX OFFSET(0) NUMBITS(1) [
            Disable = 0,
            Enable = 1,
        ],

        TX OFFSET(1) NUMBITS(1) [
            Disable = 0,
            Enable = 1,
        ],
    ],

    InterruptStatus [
        PENDING OFFSET(0) NUMBITS(1) [],
        TX OFFSET(1) NUMBITS(1) [],
        RX OFFSET(2) NUMBITS(1) [],
    ],

    LineControl [
        DATA_SIZE OFFSET(0) NUMBITS(2) [
            Bit7 = 0b00,
            Bit8 = 0b11,
        ],
    ],

    LineStatus [
        RX_READY OFFSET(0) NUMBITS(1),
        RX_OVERRUN OFFSET(1) NUMBITS(1),
        TX_EMPTY OFFSET(5) NUMBITS(1),
        TX_IDLE OFFSET(6) NUMBITS(1),
    ],

    Control [
        RX OFFSET(0) NUMBITS(1) [
            Disable = 0,
            Enable = 1,
        ],

        TX OFFSET(1) NUMBITS(1) [
            Disable = 0,
            Enable = 1,
        ],
    ]
}
