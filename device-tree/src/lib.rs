#![no_std]

use core::fmt::Debug;

pub mod blob;
pub use blob::Blob;

#[derive(Copy, Clone, Debug)]
pub struct Prop<'dtb> {
    name: &'dtb str,
    value: &'dtb [u8],
}

#[repr(C, align(8))]
#[derive(Copy, Clone, Debug)]
pub struct Reservation {
    address: Be64,
    size: Be64,
}

#[derive(Clone)]
pub struct RangeIter<'dtb> {
    address_bytes: u64,
    size_bytes: u64,
    data: &'dtb [u8],
}

impl<'dtb> RangeIter<'dtb> {
    pub(crate) fn new(address_bytes: u64, size_bytes: u64, data: &'dtb [u8]) -> Self {
        Self {
            address_bytes,
            size_bytes,
            data,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = Range> {
        self.data
            .chunks_exact((self.address_bytes * 2 + self.size_bytes) as usize)
            .map(move |chunk| {
                let (child, chunk) = chunk.split_at(self.address_bytes as usize);
                let (parent, len) = chunk.split_at(self.address_bytes as usize);
                let parent = variable_int(self.address_bytes, parent);
                let child = variable_int(self.address_bytes, child);
                let len = variable_int(self.size_bytes, len);
                Range { child, parent, len }
            })
    }
}

impl Debug for RangeIter<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

#[derive(Copy, Clone)]
pub struct Range {
    pub child: u64,
    pub parent: u64,
    pub len: u64,
}

impl Debug for Range {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{:#x} = {:#x} ({:#x})",
            self.child, self.parent, self.len,
        )
    }
}

#[derive(Clone)]
pub struct RegIter<'dtb> {
    address_bytes: u64,
    size_bytes: u64,
    data: &'dtb [u8],
}

impl<'dtb> RegIter<'dtb> {
    pub(crate) fn new(address_bytes: u64, size_bytes: u64, data: &'dtb [u8]) -> Self {
        Self {
            address_bytes,
            size_bytes,
            data,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = Reg> {
        self.data
            .chunks_exact((self.address_bytes + self.size_bytes) as usize)
            .map(move |chunk| {
                let (address, len) = chunk.split_at(self.address_bytes as usize);
                let address = variable_int(self.address_bytes, address);
                let len = variable_int(self.size_bytes, len);
                Reg { address, len }
            })
    }
}

impl Debug for RegIter<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

#[derive(Copy, Clone)]
pub struct Reg {
    pub address: u64,
    pub len: u64,
}

impl Debug for Reg {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{:#x} - {:#x} ({:#x})",
            self.address,
            self.address + self.len,
            self.len,
        )
    }
}

#[derive(Clone)]
pub struct StrIter<'dtb>(&'dtb [u8]);

impl<'dtb> StrIter<'dtb> {
    pub(crate) fn new(data: &'dtb [u8]) -> Self {
        Self(data)
    }

    pub fn iter(&self) -> impl Iterator<Item = &'dtb str> {
        self.0
            .split(|byte| *byte == 0)
            .filter(|str| !str.is_empty())
            .map(str::from_utf8)
            .map(Result::unwrap)
    }
}

impl Debug for StrIter<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Be32(u32);

impl From<Be32> for u32 {
    fn from(Be32(value): Be32) -> Self {
        if cfg!(target_endian = "little") {
            value.swap_bytes()
        } else {
            value
        }
    }
}

impl Debug for Be32 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        u32::from(*self).fmt(f)
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Be64(u64);

impl From<Be64> for u64 {
    fn from(Be64(value): Be64) -> Self {
        if cfg!(target_endian = "little") {
            value.swap_bytes()
        } else {
            value
        }
    }
}

impl Debug for Be64 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        u64::from(*self).fmt(f)
    }
}

fn variable_int(len: u64, data: &[u8]) -> u64 {
    match len {
        0 => 0,
        4 => u32::from_be_bytes(data.try_into().unwrap()) as u64,
        8 => u64::from_be_bytes(data.try_into().unwrap()),
        _ => unimplemented!(),
    }
}
