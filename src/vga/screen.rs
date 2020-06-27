use core::fmt;
use core::ops;

use lazy_static::lazy_static;
use volatile::Volatile;
use spin::Mutex;

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
        buffer: unsafe { &mut *(0x000B_8000 as *mut Buffer) },
    });
}

const HEIGHT: usize = 25;
const WIDTH: usize = 80;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Char {
    pub ascii: u8,
    pub color: color::Code,
}

#[repr(transparent)]
struct Buffer([[Volatile<Char>; WIDTH]; HEIGHT]);

impl ops::Index<usize> for Buffer {
    type Output = [Volatile<Char>];
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl ops::IndexMut<usize> for Buffer {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

pub struct Writer {
    column: usize,
    color: color::Code,
    buffer: &'static mut Buffer,
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
        | ascii @ 0x20..=0x7E => ascii,
        | _ => 0xFE,
        };

        let row = HEIGHT - 1;
        let col = self.column;

        self.buffer[row][col].write(Char {
            ascii,
            color: self.color,
        });

        self.column += 1;
    }

    fn new_line(&mut self) {
        for row in 1..HEIGHT {
            for col in 0..WIDTH {
                let char = self.buffer[row][col].read();
                self.buffer[row - 1][col].write(char);
            }
        }
        self.clear_row(HEIGHT - 1);
        self.column = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = Char {
            ascii: b' ',
            color: color::Code::default(),
        };

        for col in 0..WIDTH {
            self.buffer[row][col].write(blank);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, string: &str) -> fmt::Result {
        self.write_string(string);
        Ok(())
    }
}
