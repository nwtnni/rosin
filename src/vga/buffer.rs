use core::ops;

use volatile::Volatile;

use crate::vga;

const ADDRESS: *mut u8 = 0x000B_8000 as *mut u8;

pub const HEIGHT: usize = 25;
pub const WIDTH: usize = 80;

pub static BLANK: [Char; WIDTH] = [Char { ascii: b' ', color: vga::color::Code::DEFAULT }; WIDTH];

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Char {
    pub ascii: u8,
    pub color: vga::color::Code,
}

#[repr(C)]
pub union Buffer {
    /// Access to individual columns
    cols: &'static mut [[Volatile<Char>; WIDTH]; HEIGHT],

    /// Access to rows
    rows: &'static mut [Volatile<[Char; WIDTH]>; HEIGHT],
}

impl Buffer {
    pub unsafe fn new() -> Self {
        Buffer { cols: &mut *(ADDRESS as *mut _) }
    }

    pub fn len(&self) -> usize {
        unsafe {
            self.rows.len()
        }
    }
}

impl ops::Index<usize> for Buffer {
    type Output = Volatile<[Char; WIDTH]>;
    fn index(&self, index: usize) -> &Self::Output {
        unsafe {
            &self.rows[index]
        }
    }
}

impl ops::IndexMut<usize> for Buffer {
    fn index_mut(&mut self, row: usize) -> &mut Self::Output {
        unsafe {
            &mut self.rows[row]
        }
    }
}

impl ops::Index<(usize, usize)> for Buffer {
    type Output = Volatile<Char>;
    fn index(&self, (row, col): (usize, usize)) -> &Self::Output {
        unsafe {
            &self.cols[row][col]
        }
    }
}

impl ops::IndexMut<(usize, usize)> for Buffer {
    fn index_mut(&mut self, (row, col): (usize, usize)) -> &mut Self::Output {
        unsafe {
            &mut self.cols[row][col]
        }
    }
}
