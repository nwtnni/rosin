use core::fmt::Write;
use core::ops::Deref;
use core::ops::DerefMut;

use aarch64_cpu::registers::Readable as _;
use aarch64_cpu::registers::Writeable as _;
use tock_registers::register_bitfields;
use tock_registers::register_structs;
use tock_registers::registers::ReadOnly;
use tock_registers::registers::ReadWrite;
use tock_registers::registers::WriteOnly;

pub struct Uart {
    address: usize,
}

impl Uart {
    pub const unsafe fn new(address: usize) -> Self {
        Self { address }
    }

    pub fn initialize(&mut self) {
        self.flush();
        self.control
            .write(Control::UARTEN::Disabled + Control::TXE::Disabled + Control::RXE::Disabled);
        self.interrupt_clear.write(InterruptClear::ALL::CLEAR);

        self.integer_baud_rate
            .write(IntegerBaudRate::BAUD_DIVINT.val(3));
        self.fractional_baud_rate
            .write(FractionalBaudRate::BAUD_DIVFRAC.val(16));
        self.line_control
            .write(LineControl::WLEN::EightBit + LineControl::FEN::FifosEnabled);

        self.control
            .write(Control::UARTEN::Enabled + Control::TXE::Enabled + Control::RXE::Enabled);
    }

    pub fn read(&mut self, buffer: &mut [u8]) -> crate::Result<usize> {
        while self.flag.is_set(Flag::RXFE) {
            crate::pause();
        }

        buffer[0] = self.data.get() as u8;
        Ok(1)
    }

    pub fn write_byte(&mut self, byte: u8) {
        while self.flag.is_set(Flag::TXFF) {
            crate::pause();
        }

        self.data.set(byte as u32);
    }

    pub fn flush(&mut self) {
        while self.flag.is_set(Flag::BUSY) {
            crate::pause()
        }
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
    #[allow(non_snake_case)]
    pub Mmio {
        (0x00 => data: ReadWrite<u32>),
        (0x04 => _reserved1),
        (0x18 => flag: ReadOnly<u32, Flag::Register>),
        (0x1c => _reserved2),
        (0x24 => integer_baud_rate: WriteOnly<u32, IntegerBaudRate::Register>),
        (0x28 => fractional_baud_rate: WriteOnly<u32, FractionalBaudRate::Register>),
        (0x2c => line_control: WriteOnly<u32, LineControl::Register>),
        (0x30 => control: WriteOnly<u32, Control::Register>),
        (0x34 => _reserved3),
        (0x44 => interrupt_clear: WriteOnly<u32, InterruptClear::Register>),
        (0x48 => @END),
    }
}

// https://github.com/rust-embedded/rust-raspberrypi-OS-tutorials/blob/644474cc09f755249f9c55d99a5d1e07a2562fc7/05_drivers_gpio_uart/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
register_bitfields! {
    u32,

    /// Flag Register.
    Flag [
        /// Transmit FIFO empty. The meaning of this bit depends on the state of the FEN bit in the
        /// Line Control Register, LCR_H.
        ///
        /// - If the FIFO is disabled, this bit is set when the transmit holding register is empty.
        /// - If the FIFO is enabled, the TXFE bit is set when the transmit FIFO is empty.
        /// - This bit does not indicate if there is data in the transmit shift register.
        TXFE OFFSET(7) NUMBITS(1) [],

        /// Transmit FIFO full. The meaning of this bit depends on the state of the FEN bit in the
        /// LCR_H Register.
        ///
        /// - If the FIFO is disabled, this bit is set when the transmit holding register is full.
        /// - If the FIFO is enabled, the TXFF bit is set when the transmit FIFO is full.
        TXFF OFFSET(5) NUMBITS(1) [],

        /// Receive FIFO empty. The meaning of this bit depends on the state of the FEN bit in the
        /// LCR_H Register.
        ///
        /// - If the FIFO is disabled, this bit is set when the receive holding register is empty.
        /// - If the FIFO is enabled, the RXFE bit is set when the receive FIFO is empty.
        RXFE OFFSET(4) NUMBITS(1) [],

        /// UART busy. If this bit is set to 1, the UART is busy transmitting data. This bit remains
        /// set until the complete byte, including all the stop bits, has been sent from the shift
        /// register.
        ///
        /// This bit is set as soon as the transmit FIFO becomes non-empty, regardless of whether
        /// the UART is enabled or not.
        BUSY OFFSET(3) NUMBITS(1) []
    ],

    /// Integer Baud Rate Divisor.
    IntegerBaudRate [
        /// The integer baud rate divisor.
        BAUD_DIVINT OFFSET(0) NUMBITS(16) []
    ],

    /// Fractional Baud Rate Divisor.
    FractionalBaudRate [
        ///  The fractional baud rate divisor.
        BAUD_DIVFRAC OFFSET(0) NUMBITS(6) []
    ],

    /// Line Control Register.
    LineControl [
        /// Word length. These bits indicate the number of data bits transmitted or received in a
        /// frame.
        #[allow(clippy::enum_variant_names)]
        WLEN OFFSET(5) NUMBITS(2) [
            FiveBit = 0b00,
            SixBit = 0b01,
            SevenBit = 0b10,
            EightBit = 0b11
        ],

        /// Enable FIFOs:
        ///
        /// 0 = FIFOs are disabled (character mode) that is, the FIFOs become 1-byte-deep holding
        /// registers.
        ///
        /// 1 = Transmit and receive FIFO buffers are enabled (FIFO mode).
        FEN  OFFSET(4) NUMBITS(1) [
            FifosDisabled = 0,
            FifosEnabled = 1
        ]
    ],

    /// Control Register.
    Control [
        /// Receive enable. If this bit is set to 1, the receive section of the UART is enabled.
        /// Data reception occurs for either UART signals or SIR signals depending on the setting of
        /// the SIREN bit. When the UART is disabled in the middle of reception, it completes the
        /// current character before stopping.
        RXE OFFSET(9) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ],

        /// Transmit enable. If this bit is set to 1, the transmit section of the UART is enabled.
        /// Data transmission occurs for either UART signals, or SIR signals depending on the
        /// setting of the SIREN bit. When the UART is disabled in the middle of transmission, it
        /// completes the current character before stopping.
        TXE OFFSET(8) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ],

        /// UART enable:
        ///
        /// 0 = UART is disabled. If the UART is disabled in the middle of transmission or
        /// reception, it completes the current character before stopping.
        ///
        /// 1 = The UART is enabled. Data transmission and reception occurs for either UART signals
        /// or SIR signals depending on the setting of the SIREN bit
        UARTEN OFFSET(0) NUMBITS(1) [
            /// If the UART is disabled in the middle of transmission or reception, it completes the
            /// current character before stopping.
            Disabled = 0,
            Enabled = 1
        ]
    ],

    /// Interrupt Clear Register.
    InterruptClear [
        /// Meta field for all pending interrupts.
        ALL OFFSET(0) NUMBITS(11) []
    ]
}
