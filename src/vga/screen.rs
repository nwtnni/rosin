use core::fmt;

use lazy_static::lazy_static;
use spin::Mutex;

use crate::vga::buffer;
use crate::vga::color;

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column: 0,
        color: color::Code::new(
            color::Fore {
                bright: false,
                color: color::T::Green,
            },
            color::Back::default(),
        ),
        buffer: unsafe { buffer::Buffer::new() },
    });
}

pub struct Writer {
    column: usize,
    color: color::Code,
    pub(super) buffer: buffer::Buffer,
}

impl Writer {
    fn write_string(&mut self, string: &str) {
        for byte in string.bytes() {
            self.write_byte(byte);
        }
    }

    fn write_byte(&mut self, byte: u8) {
        let ascii = match byte {
        | b'\n' => return self.new_line(),
        | 0x00..=0x1F => 0xFE,
        | 0x20..=0x7E => byte,
        | 0x7F..=0xFF => 0xFE,
        };

        if self.column >= buffer::WIDTH {
            self.new_line();
        }

        self.buffer[(buffer::HEIGHT - 1, self.column)]
            .write(buffer::Char {
                ascii,
                color: self.color,
            });

        self.column += 1;
    }

    fn new_line(&mut self) {
        for row in 1..self.buffer.len() {
            let prev = self.buffer[row].read();
            let next = &mut self.buffer[row - 1];
            next.write(prev);
        }
        self.buffer[buffer::HEIGHT - 1].write(buffer::BLANK);
        self.column = 0;
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, string: &str) -> fmt::Result {
        self.write_string(string);
        Ok(())
    }
}
