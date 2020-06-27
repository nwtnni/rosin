use core::ops;

use volatile::Volatile;

use crate::vga;

const HEIGHT: usize = 25;
const WIDTH: usize = 80;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Char {
    pub ascii: u8,
    pub color: vga::color::Code,
}

#[repr(transparent)]
pub struct Buffer([[Volatile<Char>; WIDTH]; HEIGHT]);

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
    pub column: usize,
    pub color: vga::color::Code,
    pub buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_string(&mut self, string: &str) {
        for byte in string.bytes() {
            self.write_byte(byte);
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
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
        todo!()
    }
}
