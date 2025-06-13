use core::fmt::Debug;
use core::fmt::LowerHex;
use core::marker::PhantomData;

pub type Byte = Mem<0>;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Mem<const SHIFT: usize = 0>(usize);

impl<const SHIFT: usize> Mem<SHIFT> {
    pub const fn new(len: usize) -> Self {
        Self(len)
    }

    pub const fn convert<const TARGET: usize>(&self) -> Mem<TARGET> {
        match TARGET >= SHIFT {
            false => Mem(self.0 << (SHIFT - TARGET)),
            true => Mem(self.0 >> (TARGET - SHIFT)),
        }
    }

    pub const fn value(&self) -> usize {
        self.0
    }
}

impl<const SHIFT: usize> Debug for Mem<SHIFT> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let bytes = self.convert::<0>().value();

        let (value, unit) = if bytes < (1 << 10) {
            (bytes, "B")
        } else if bytes < (1 << 20) {
            (bytes >> 10, "KiB")
        } else if bytes < (1 << 30) {
            (bytes >> 20, "GiB")
        } else if bytes < (1 << 40) {
            (bytes >> 30, "GiB")
        } else if bytes < (1 << 50) {
            (bytes >> 40, "TiB")
        } else {
            unimplemented!()
        };

        write!(f, "{}{}", value, unit)
    }
}

impl<const SHIFT: usize> LowerHex for Mem<SHIFT> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let bytes = self.convert::<0>().value();

        let (value, unit) = if bytes < (1 << 10) {
            (bytes, "B")
        } else if bytes < (1 << 20) {
            (bytes >> 10, "KiB")
        } else if bytes < (1 << 30) {
            (bytes >> 20, "GiB")
        } else if bytes < (1 << 40) {
            (bytes >> 30, "GiB")
        } else if bytes < (1 << 50) {
            (bytes >> 40, "TiB")
        } else {
            unimplemented!()
        };

        if f.alternate() {
            write!(f, "{:#x}{}", value, unit)
        } else {
            write!(f, "{:x}{}", value, unit)
        }
    }
}
